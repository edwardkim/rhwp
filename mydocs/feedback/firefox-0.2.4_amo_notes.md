--- Firefox AMO reviewer notes (v0.2.4) ---

# What it does

rhwp opens HWP/HWPX documents directly in Firefox. The viewer/editor runs locally in WebAssembly. Documents are not uploaded, and the extension does not collect analytics or personal data.

# Security changes in v0.2.4

This release hardens extension-side document fetch paths:

- service worker sender validation for document fetch requests
- localhost, loopback, private-network, link-local, and internal-host URLs are blocked
- redirect final URLs are revalidated with the same policy
- fetch uses `credentials: "omit"`
- automatically extracted thumbnail data is kept out of the page DOM

No new permissions and no new external network endpoints were added.

# How to test

1. Install the extension package.
2. Open a public HWP/HWPX link or drag and drop a local HWP/HWPX file into the viewer.
3. Confirm the document opens in the local viewer.
4. Check hover preview/badge behavior on HWP/HWPX links.
5. Confirm local/internal/private-network HWP/HWPX URLs are not fetched by the extension-side thumbnail path.

# Source package

Firefox AMO source upload should use the trimmed source archive described in `mydocs/manual/publish_guide.md`.
Do not include `node_modules/`, `target/`, `dist/`, `output/`, `samples/`, `pdf-large/`, local `.env`, or private credentials.
