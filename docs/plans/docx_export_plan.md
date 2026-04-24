# Implementation Plan - DOCX Export Support

Add the ability to export HWP documents as DOCX files in the HOP desktop application.

## User Review Required

> [!IMPORTANT]
> The initial implementation will focus on basic text and formatting. Complex elements like tables and images may be simplified or skipped in the first version.

## Proposed Changes

### [Component] apps/desktop/src-tauri

#### [MODIFY] [Cargo.toml](file:///d:/mywork/hwp/apps/desktop/src-tauri/Cargo.toml)
- Add `docx-rs = "0.4"` to dependencies.

#### [NEW] [docx_export.rs](file:///d:/mywork/hwp/apps/desktop/src-tauri/src/docx_export.rs)
- Implement `export_core_to_docx` function.
- Map `rhwp::Document` (via `DocumentCore`) to `docx_rs::Docx`.
- Handle text runs, paragraph shapes, and character shapes.

#### [MODIFY] [mod.rs](file:///d:/mywork/hwp/apps/desktop/src-tauri/src/lib.rs) or wherever modules are declared.
- Declare `pub mod docx_export;`.

#### [MODIFY] [commands.rs](file:///d:/mywork/hwp/apps/desktop/src-tauri/src/commands.rs)
- Add `export_docx` and `export_docx_from_hwp_path` commands (similar to PDF export).

### [Component] apps/studio-host

#### [MODIFY] [tauri-bridge.ts](file:///d:/mywork/hwp/apps/studio-host/src/core/tauri-bridge.ts)
- Add `exportDocxFromCommand` to `DesktopBridgeApi`.
- Implement `exportDocxFromCommand` in `TauriBridge`.
- Update `saveDocumentAsFromCommand` filters to include `.docx`. (Alternatively, add a separate "Export as DOCX" item).

## Open Questions

- Should we include tables in the first version? `docx-rs` supports them, but mapping HWP table models can be tricky.
- How should we handle images? HWP stores them as binary data which needs to be converted and embedded.

## Verification Plan

### Automated Tests
- Add a unit test in `docx_export.rs` that converts a simple mock `Document` to a DOCX byte array.

### Manual Verification
1. Run `pnpm dev` for both studio-host and desktop.
2. Open an HWP file.
3. Select "File" -> "Save As..." (or "Export as DOCX").
4. Choose `.docx` format.
5. Open the saved file in Microsoft Word.
6. Check if the text and basic formatting (Bold, Align) are correct.
