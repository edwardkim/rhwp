# Edge Add-ons certification notes — v0.9.0 preparation

Package the same Chrome archive as `rhwp-edge-0.9.0.zip` and verify byte identity before submission.
Update the existing listing. The source manifest's permissions, host_permissions and CSP are unchanged
from v0.8.7; this does not substitute for inspecting the final ZIP.

## Reviewer journey

1. Open a public HWP/HWPX sample from https://github.com/edwardkim/rhwp/tree/main/samples.
2. Check the viewer, zoom, editing, print and save.
3. Use the document link context menu and open a local HWP/HWPX file.
4. Move a picture, use Undo/Redo and reopen the saved document.
5. Confirm saved downloads do not create duplicate viewer tabs. A non-HWP download must remain ordinary.

The shared engine also powers PDF conversion in CLI/Studio. See the developer guide below; this is not
a request to intercept arbitrary PDF downloads or to add a new permission. Final packaged-browser checks
must be recorded before store submission.

Permissions retained: activeTab, downloads, contextMenus, clipboardWrite and storage; broad host/content
matches support HWP/HWPX links on arbitrary sites. Parsing/rendering use the bundled engine.

PDF guide: https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/pdf_developer_onboarding_EN.md
Source and credits: https://github.com/edwardkim/rhwp
