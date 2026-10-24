# Existing export bytes to bounded usage monitoring

The Rust `exports::read_usage` entry point reuses `exports::execute`, its actual
BigQuery response byte decoder, scoped request construction, bounded projection,
raw response SHA256 and before/after table revision checks. Google owns field
selection and normalization; `zixcel-cost` owns exact money and bounded revision
monitoring. The CLI manifest and live permission surface are unchanged.

Select an existing standard/detailed export query, invoice reporting month
(YYYYMM), currency and observation time. `usage_scope` binds provider/account,
source project/dataset/table and UTC month boundaries before collection. A source
hash forms part of the monitor account identity: changing the export source
requires a new explicitly selected session even under the same billing account.
The source/table identity is a digest of structural JSON, not concatenated names.

`read_usage` refuses a continuation request, incomplete page, changing table,
wrong account/project and unsupported precision. It reads no more than the
existing three metadata/data/metadata requests and creates no SQL job. It does
not accumulate pages or retry acquisition. Duplicate/continuation pages cannot
be declared a complete snapshot. Only a stable single-page table can be admitted;
larger tables require a separately designed consistent export snapshot method.

The adapter selects rows by invoice reporting month and adds exact decimal cost
and signed credits using Money, without floats, FX or rounding. Other months do
not enter that reporting period. Mixed currencies within the selected month and
precision exceeding nine fractional places are errors, never truncation or a
zero fallback. This preserves the decimal strings in the response; it does not
recover precision already lost in the vendor FLOAT source. Identical individual
line items are not deduplicated because the export does not supply a unique row ID.

A table etag plus last-modified time identifies the report revision; raw data
bytes supply its evidence digest. Repeated current observations have unchanged
content and totals, corrected tables require new revisions, same-revision byte
changes and previously superseded revisions are rejected. Rejections do not
replace the monitor's last complete observation. Opaque etags are not sorted to
invent chronological ordering; source selection and authoritative revision
choice remain caller responsibilities. Unauthorized/forbidden responses retain
the existing Google error and do not imply zero cost.

A complete read means completeness of this chosen export table, not global
billing completeness. Missing export history/backfill, late adjustments, issued
billing documents and payment evidence remain separate. `UsageSnapshot` cannot
be passed to the invoice monitor. No invoice ID or paid status is invented.

Official schema: https://docs.cloud.google.com/billing/docs/how-to/export-data-bigquery-tables/standard-usage
The invoice month can differ from the usage month and credits are separate fields.

Synthetic consumer acceptance is in `crates/google-core/tests/usage_cost_consumer.rs`:
real vendor-shaped bytes -> existing decoder -> Google normalizer -> actual shared
UsageMonitor. It needs no account credentials, paid query, live API, cloud storage,
polling, daemon or notification. Normal source CI acceptance, public cost crate
publication and native distribution remain independently verified steps.
