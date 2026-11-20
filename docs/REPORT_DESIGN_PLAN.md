# Report presentation work plan

Owner requirement recorded 20 November 2026. Linked to REQ-RPT and Phase 5 report-format work (RPT-FORMATS). **Planned only: no report renderer or checkbox has been changed by this documentation update.** Existing export implementation does not imply this design requirement is complete.

## Default: polished reports

Reports produced by Pusthakam should be aesthetically beautiful, professionally composed and suitable to present or share. A plain table dumped onto a page is not the intended default.

Design a consistent report family that fits Pusthakam's visual identity. Use deliberate typography, spacing, hierarchy, alignment, restrained color and readable tables. Include the configured library identity, report title, reporting period/filters and appropriate generation information. Use clear summary figures and meaningful charts where they help the particular report; avoid decorative charts or clutter. Give long tables repeated headings, appropriate page breaks, page numbers and readable totals. Check long titles, empty results and large/multipage reports.

Apply this to the report preview and designed output formats: PDF, printing and styled XLSX exports. XLSX should remain useful for analysis, including readable column widths, correct number/date/currency formats and sensible header treatment. CSV is a data format and retains the same plain data; it has no visual design mode.

## Optional: simple report design

Provide a clearly labeled **Use simple report design** checkbox in the report preview/export flow.

- Unchecked by default: the polished design is shown and used for output.
- Checked: show a conventional, restrained library-management report layout, with straightforward headings and tables, minimal decoration and economical printing.
- The selection must apply consistently to preview and the applicable PDF/print/XLSX outputs.
- Both designs contain the same selected records, filters, counts, calculations, currencies and totals. Choosing simple design changes presentation only.
- Retain readable typography, useful headings, pagination and clear errors in the simple mode; it must remain accessible and usable.

## Implementation approach

Before designing, review the existing report screens, export generators, current design tokens and representative outputs. Consult the relevant installed design, PDF, spreadsheet, document and visualization skills for the formats being changed. Use their layout, rendering and visual-quality workflows as appropriate; do not treat skill availability as a reason to add unnecessary graphics. These are development resources, not application runtime dependencies. Bundle required fonts/assets locally and preserve offline report generation.

Create representative polished and simple templates, then apply the same standards across report types. Keep one underlying report dataset/calculation path for both designs. The current development checkpoint remains BAK-RESTORE; this request is added to the report work plan, without reordering unrelated phases or claiming implementation credit.

## Acceptance checklist

1. Polished mode is the default for a newly opened report, and the simple-design checkbox is visible and keyboard accessible.
2. Toggling the checkbox updates preview and relevant exported/printed presentation consistently.
3. Both modes reconcile exactly on data, row counts, totals, date filters and currency separation.
4. Open PDF and XLSX artifacts in actual readers; visually inspect representative operational, financial, short, empty and multipage reports in both modes. Check typography, spacing, chart usefulness, column widths, repeated headers and page breaks. Automated assertions alone do not establish visual quality.
5. Print checks cover clipping, grayscale legibility and readable page structure. All report types retain their existing permission and export safeguards.

Pending implementation and visual/native acceptance. No new roadmap weight or completion percentage is assigned for recording this plan.
