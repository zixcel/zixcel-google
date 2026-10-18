#!/usr/bin/env bash
set +x
set -euo pipefail
umask 077
export -n ZIX_GOOGLE_ACCESS_TOKEN
[[ $# -ge 4 && $# -le 5 ]] || { echo 'Usage: inspect-billing-local.sh ZIX_PACKAGE GOOGLE_PACKAGE PRIOR_REPORT NEW_OUTPUT [QUOTA_PROJECT]' >&2; exit 2; }
zix_package=$(realpath -- "$1")
google_package=$(realpath -- "$2")
prior=$(realpath -- "$3")
output=$4
quota=${5:-}
command -v jq >/dev/null
(cd "$zix_package" && sha256sum --check --quiet CONTENTS.sha256)
(cd "$google_package" && sha256sum --check --quiet CONTENTS.sha256)
jq -e 'type=="array" and length<=512 and all(.[];type=="string")' "$prior/project-targets.json" >/dev/null
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
[[ -z "$quota" ]] || quota_args=(--quota-project "$quota")
if [[ -z ${ZIX_GOOGLE_ACCESS_TOKEN:-} ]]; then
 [[ -t 0 ]] || { echo 'credential_required: run from an interactive terminal.' >&2; exit 2; }
 read -rsp 'Existing short-lived Google access token (not saved): ' ZIX_GOOGLE_ACCESS_TOKEN
 printf '\n'
fi
[[ -n "$ZIX_GOOGLE_ACCESS_TOKEN" ]] || exit 2
partial=false
run() {
 local label=$1 status=0
 shift
 ZIX_GOOGLE_ACCESS_TOKEN="$ZIX_GOOGLE_ACCESS_TOKEN" "$zix" google "$@" "${quota_args[@]}" > "$output/$label.json" 2> "$output/$label.stderr" || status=$?
 jq -nc --arg label "$label" --argjson status "$status" '{label:$label,exitCode:$status}' >> "$output/commands.jsonl"
 if jq -e '.result.selected_scans_complete==false or .result.coverage.complete==false' "$output/$label.json" >/dev/null; then partial=true; fi
 printf 'Observed %s (exit=%s)\n' "$label" "$status"
}
source "$google_package/billing-export-observations.sh"
billing_observe
unset ZIX_GOOGLE_ACCESS_TOKEN
jq -n --slurpfile commands "$output/commands.jsonl" --arg observedAt "$(date -u +%FT%TZ)" \
 --argjson limited "$limited" --argjson partial "$partial" --argjson pages "$read_pages" --argjson tables "$read_tables" \
 '{schema:"zix-google-billing-local-analysis/v1",observedAt:$observedAt,commands:$commands,credentialsSaved:false,
  selectionLimited:$limited,observationsPartial:$partial,billingTablesRead:$tables,billingPagesRead:$pages,
  allCommandsSucceeded:all($commands[];.exitCode==0),invoiceFinalityConfirmed:false,
  interpretation:"no export or denied metadata is not zero charges; export rows are not an invoice"}' > "$output/summary.json"
printf 'Private billing/resource observations saved to %s; token discarded; temporary registry removed on exit.\n' "$output"
if [[ "$limited" == true || "$partial" == true ]] || ! jq -e '.allCommandsSucceeded' "$output/summary.json" >/dev/null; then exit 2; fi
