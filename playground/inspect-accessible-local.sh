#!/usr/bin/env bash
set +x
set -euo pipefail
umask 077
export -n ZIX_GOOGLE_ACCESS_TOKEN
[[ $# -ge 3 && $# -le 4 ]] || { echo 'Usage: inspect-accessible-local.sh ZIX_PACKAGE GOOGLE_PACKAGE NEW_PRIVATE_OUTPUT [QUOTA_PROJECT]' >&2; exit 2; }
zix_package=$(realpath -- "$1")
google_package=$(realpath -- "$2")
output=$3
quota=${4:-}
command -v jq >/dev/null || { echo 'jq is required for this optional report consumer.' >&2; exit 2; }
(cd "$zix_package" && sha256sum --check --quiet CONTENTS.sha256)
(cd "$google_package" && sha256sum --check --quiet CONTENTS.sha256)
[[ ! -e "$output" && ! -L "$output" ]] || { echo 'Choose a new private output directory.' >&2; exit 2; }
mkdir -m 700 -- "$output"
output=$(realpath -- "$output")
mkdir -m 700 "$output/registry"
export XDG_CONFIG_HOME="$output/registry"
trap 'unset ZIX_GOOGLE_ACCESS_TOKEN; rm -rf -- "$output/registry"' EXIT
zix="$zix_package/zix"
"$zix" extensions add --manifest "$google_package/google.extension.json" \
 --manifest-sha256 "$(sha256sum "$google_package/google.extension.json" | cut -d' ' -f1)" \
 --executable "$google_package/zix-google" \
 --executable-sha256 "$(sha256sum "$google_package/zix-google" | cut -d' ' -f1)" > "$output/registration.json"
quota_args=()
if [[ -n "$quota" ]]; then
 [[ "$quota" =~ ^[a-z][a-z0-9-]{4,28}[a-z0-9]$ || "$quota" =~ ^[0-9]{1,20}$ ]] || { echo 'Invalid quota project.' >&2; exit 2; }
 quota_args=(--quota-project "$quota")
fi
if [[ -z ${ZIX_GOOGLE_ACCESS_TOKEN:-} ]]; then
 [[ -t 0 ]] || { echo 'credential_required: supply ZIX_GOOGLE_ACCESS_TOKEN or run from an interactive terminal.' >&2; exit 2; }
 read -rsp 'Existing short-lived Google access token (not saved): ' ZIX_GOOGLE_ACCESS_TOKEN
 printf '\n'
fi
[[ -n "$ZIX_GOOGLE_ACCESS_TOKEN" ]] || exit 2
run() {
 local label=$1
 shift
 local status=0
 ZIX_GOOGLE_ACCESS_TOKEN="$ZIX_GOOGLE_ACCESS_TOKEN" "$zix" google "$@" "${quota_args[@]}" > "$output/$label.json" 2> "$output/$label.stderr" || status=$?
 local complete=false
 if [[ $status == 0 ]]; then
  complete=$(jq -er 'if .result.coverage then .result.coverage.complete elif .result.selected_scans_complete != null then .result.selected_scans_complete else false end | tostring' "$output/$label.json")
 fi
 jq -nc --arg label "$label" --argjson status "$status" --argjson complete "$complete" \
  '{label:$label,exitCode:$status,selectedEnumerationComplete:$complete}' >> "$output/commands.jsonl"
 printf 'Observed %s (exit=%s, selectedEnumerationComplete=%s)\n' "$label" "$status" "$complete"
}
run projects projects list --max-pages 2
run billing-accounts billing accounts list --max-pages 2
index=0
while IFS= read -r account; do
 index=$((index+1))
 run "billing-projects-$index" billing projects list --billing-account "$account" --max-pages 2
done < <(jq -r '(.result.records // [])[:8][] | .name | sub("^billingAccounts/";"")' "$output/billing-accounts.json")
shopt -s nullglob
billing_reports=("$output"/billing-projects-*.json)
jq -s '[.[]|(.result.records // [])[]|.project_id|select(type=="string")]|unique' \
 "$output/projects.json" "${billing_reports[@]}" > "$output/project-targets.json"
index=0
while IFS= read -r project; do
 index=$((index+1))
 run "project-overview-$index" overview project --project "$project" --max-pages 1
done < <(jq -r '.[:8][]' "$output/project-targets.json")
unset ZIX_GOOGLE_ACCESS_TOKEN
jq -n --slurpfile commands "$output/commands.jsonl" --slurpfile projects "$output/projects.json" \
 --slurpfile targets "$output/project-targets.json" --slurpfile billing "$output/billing-accounts.json" --arg observedAt "$(date -u +%FT%TZ)" \
 '{schema:"zix-google-local-analysis/v1",observedAt:$observedAt,credentialsSaved:false,
   commands:$commands,allSelectedCommandsComplete:all($commands[];.exitCode==0 and .selectedEnumerationComplete),
   projectSelectionLimit:8,billingSelectionLimit:8,
   allDiscoveredTargetsInspected:($projects[0].result.coverage.complete==true and $billing[0].result.coverage.complete==true
     and ($targets[0]|length)<=8 and ($billing[0].result.records|length)<=8
     and all($commands[];.exitCode==0 and .selectedEnumerationComplete)),
   discoveredProjects:($projects[0].result.records|if type=="array" then length else null end),
   discoveredBillingAccounts:($billing[0].result.records|if type=="array" then length else null end),
   discoveredProjectTargets:($targets[0]|length),
   selectedProjectCount:([$commands[]|select(.label|startswith("project-overview-"))]|length),
   selectedBillingAccountCount:([$commands[]|select(.label|startswith("billing-projects-"))]|length),
   costConclusion:"unknown; inventory is not an invoice or proof of zero charges",
   visibility:"caller-visible resources; source paging and selection limits apply"}' > "$output/summary.json"
printf 'Private analysis saved to %s; token discarded; temporary registry removed on exit.\n' "$output"

if ! jq -e '.allDiscoveredTargetsInspected == true' "$output/summary.json" >/dev/null; then
 echo 'Analysis incomplete: inspect the private summary and per-command errors.' >&2
 exit 2
fi
