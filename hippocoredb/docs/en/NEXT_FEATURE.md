# Next Feature

## Feature name

**Ingestion & Recall UX Hardening v0.1**

## Why it matters

The Ingestion & Recall page now works end-to-end, but the experience is still
purely form-based. Users who don't know which collection to use must leave the
page to check. Adding inline collection autocomplete (populated from the loaded
collections list), recall result card rendering (instead of raw JSON dump), and
a token count for build_context results will make the ingest→recall feedback
loop much tighter without any server-side changes.

## Behavior

- **Inline collection autocomplete**: the collection fields in Store and Recall
  sections auto-populate with a `<datalist>` from `state.collections`.
- **Recall result cards**: replace the raw `JsonViewer` for recall results with
  a structured card list showing score, type badge, matched terms, a text
  snippet, and a copy-id button.
- **build_context result**: show token count, items included/dropped badges,
  and the formatted context text in a readable pre block.
- **Radio card type selection**: the Memory/Document/Record/File radio switches
  should show/hide the relevant input fields (e.g., table name only for Record).

## Likely files

- `crates/hippocore-server/src/admin/app.js` (IngestionRecallPage, render cards)
- `crates/hippocore-server/src/admin/styles.css` (recall card styles)
- `docs/en/STATUS.md`
- `docs/pt-br/STATUS.md`
- `CHANGELOG.md`

## Acceptance criteria

- Collection input shows a datalist with known collections for the selected tenant.
- Recall results render as cards, not raw JSON (unless the JSON tab is selected).
- build_context result shows token count + items included/dropped + formatted text.
- Type radio selection dynamically shows/hides the table field.
- All existing functionality preserved; quality gate green.

## Out of scope

- File upload UI (drag-and-drop file import is a separate feature).
- Bulk ingest (batch API is available but UI not in scope here).
- Real-time streaming of ingest progress.
