# Sourced by the checksum-verified report consumer; never reads arbitrary table data.
billing_observe() {
 limited=false read_pages=0 read_tables=0
 local pi=0 di=0 ti=0 project dataset table account token label page
 [[ $(jq 'length' "$prior/project-targets.json") -le 8 ]] || limited=true
 while IFS= read -r project; do
  pi=$((pi+1))
  run "structure-$pi" overview project --project "$project" --max-pages 8
  label="datasets-$pi"
  run "$label" billing exports discover --project "$project"
  if jq -e '.result.coverage.complete==false' "$output/$label.json" >/dev/null; then limited=true; fi
  [[ $(jq '(.result.records // [])|length' "$output/$label.json") -le 8 ]] || limited=true
  while IFS= read -r dataset; do
   di=$((di+1))
   if [[ $di -gt 16 ]]; then limited=true; break; fi
   label="tables-$di"
   run "$label" billing exports discover --project "$project" --dataset "$dataset"
   if jq -e '.result.coverage.complete==false' "$output/$label.json" >/dev/null; then limited=true; fi
   while IFS=$'\t' read -r table account; do
    ti=$((ti+1))
    if [[ $ti -gt 8 ]]; then limited=true; break; fi
    read_tables=$((read_tables+1))
    token=''
    local seen_cursors=() etag='' next_etag seen repeated=false
    for page in 1 2 3 4 5 6 7 8; do
     label="billing-rows-$ti-$page"
     local cursor=()
     [[ -z "$token" ]] || cursor=(--page-token "$token")
     run "$label" billing exports read --project "$project" --dataset "$dataset" --table "$table" --billing-account "$account" "${cursor[@]}"
     read_pages=$((read_pages+1))
     next_etag=$(jq -r '.result.source.table_etag // empty' "$output/$label.json")
     if [[ -n "$etag" && "$etag" != "$next_etag" ]]; then limited=true; break; fi
     etag=$next_etag
     token=$(jq -r '.result.coverage.next_page_token // empty' "$output/$label.json")
     if jq -e '.result.coverage.table_stable_during_page_read==false' "$output/$label.json" >/dev/null; then limited=true; break; fi
     [[ -n "$token" ]] || break
     for seen in "${seen_cursors[@]}"; do
      if [[ "$seen" == "$token" ]]; then repeated=true; break; fi
     done
     if [[ "$repeated" == true ]]; then limited=true; break; fi
     seen_cursors+=("$token")
    done
    [[ -z "$token" ]] || limited=true
   done < <(jq -r '(.result.records // [])[]|[.table,.billing_account]|@tsv' "$output/tables-$di.json")
  done < <(jq -r '(.result.records // [])[:8][]|.dataset' "$output/datasets-$pi.json")
 done < <(jq -r '.[:8][]' "$prior/project-targets.json")
}
