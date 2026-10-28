rhwp is a free and open-source browser extension for opening, editing, and printing HWP/HWPX documents.
Document processing runs locally in WebAssembly without uploading files to an external server.

## Key features

- Open HWP/HWPX downloads and links in the rhwp viewer
- Edit text, tables, and formatting, then save as HWP
- Print preview, PDF output, and printer output
- Open local files by drag and drop after explicit confirmation
- Link badges, preview cards, and an "Open with rhwp" context menu

## Privacy

- Files and passwords are not uploaded to an external server.
- There are no ads, user tracking, sign-up requirements, or personal-data collection.

## Changes in v0.8.6 — 2026-09-02

- Fixed `.xlsx` downloads being incorrectly opened in the HWP viewer when a provisional source URL ended in
  `.hwp`; interception now waits for and honors finalized filename and MIME evidence.
- Added bounded output and exact-length checks to preview-image decompression.
- Bundled the v0.8.6 WebAssembly document engine with improved typesetting, font, save fidelity, and editing.
- Added no permissions and no external network endpoints compared with v0.8.4.

Full history: https://github.com/edwardkim/rhwp/releases

Source code: https://github.com/edwardkim/rhwp

"Hangul", "Hancom", "HWP", and "HWPX" are registered trademarks of Hancom Inc. rhwp is an independent
open-source project with no affiliation, sponsorship, or endorsement by Hancom Inc.
