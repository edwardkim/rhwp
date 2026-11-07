# Chrome Web Store description — v0.9.0 preparation

rhwp is a free HWP/HWPX viewer and editor for your browser. Open document downloads, use link
badges and context menus, open local files, edit, print and save. Parsing and rendering run in bundled
JavaScript and WebAssembly.

This version improves nested and split tables, text boxes, cell pictures, headers/footers, equation/font
fallback and round-trip preservation. It adds picture movement, cell picture insertion and continuous
highlighting, plus large-table/JPEG processing improvements.

The shared engine adds PDF import and developer onboarding. Use CLI/Studio to convert PDFs into editable
documents; the extension does not automatically intercept every PDF on the web.
Chrome manifest permissions, host_permissions and CSP are unchanged from v0.8.7.
Final-package open/edit/save checks and store review are required before publication.

PDF guide: https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/pdf_developer_onboarding_EN.md
Privacy: https://edwardkim.github.io/rhwp/privacy.html
Source, changes and credits: https://github.com/edwardkim/rhwp
