What this extension does

rhwp is an open-source viewer/editor for HWP/HWPX documents (Korea's dominant document format). All parsing/rendering happens locally via WebAssembly. No external server, no remote code, no data collection.


How to test

1. Install. Visit a page with an HWP/HWPX link. Sample:
   https://github.com/edwardkim/rhwp/raw/main/samples/cell_data_test.hwp
2. Click the link → file downloads and opens in a viewer tab
3. Viewer renders text, tables, images, formatting
4. Try Ctrl+P (print), edit text, Ctrl+S (save HWP)
5. Right-click any HWP link → "Open with rhwp"


Permission justifications

- activeTab: detect HWP links on current page, add icon badges
- downloads: auto-open downloaded HWP/HWPX in viewer (user toggle)
- contextMenus: right-click "Open with rhwp" on HWP links
- clipboardWrite: copy selected text from documents
- storage: user preferences only (toggles). No personal data
- host_permissions <all_urls>: HWP links can appear on any domain.
  Only anchor tags with .hwp/.hwpx extensions are inspected locally.
  Page content is NOT read, sent, or tracked.


Remote code

No. All JS and WebAssembly bundled in the package. Nothing fetched
or executed from external servers at runtime.


Privacy

No personal data, no analytics, no tracking. No external requests
other than the file the user clicks. All local via WebAssembly.
Privacy policy:
https://github.com/edwardkim/rhwp/blob/main/rhwp-chrome/PRIVACY.md


Source code

Open source (MIT): https://github.com/edwardkim/rhwp


v0.2.1 notes

- Bug fix: previous version inadvertently broke "remember last save
  location" for ALL Chrome downloads when active. This release calls
  suggest() only when an HWP file is detected.
- HWPX direct save is intentionally disabled (serializer in beta).
  Viewing/editing work; saving blocked with a clear user notice.
  Full HWPX save coming next release.
- Includes 6 external contributors via merged PRs.


Test account

No account/sign-up required.
