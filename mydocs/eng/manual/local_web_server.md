# Local Web Server Manual

---

## [rhwp-studio] Vite Dev Server (Supported)

### Overview

Run the TypeScript-based rhwp-studio with the Vite dev server.
Since `localhost` is a browser Secure Context, the Clipboard API works correctly even over HTTP.

### Prerequisites

- Node.js v24+, npm v11+
- Docker (for WASM builds)

### Execution Steps

#### 1. WASM Build (run after each source change)

```bash
cd ~/vsworks/rhwp
docker compose --env-file .env.docker run --rm wasm
```

Build output: `pkg/rhwp_bg.wasm`, `pkg/rhwp.js`, `pkg/rhwp.d.ts`

#### 2. Start Dev Server

```bash
cd ~/vsworks/rhwp/rhwp-studio
npx vite
```

Access in browser:

```text
http://localhost:7700        # Local
http://<PC-IP>:7700          # Other devices on the same network
```

> `npm run dev` works the same way. (The `dev` script in `package.json` runs `vite`)

### One-Shot Execution (WASM Build + Server Start)

```bash
cd ~/vsworks/rhwp && \
docker compose --env-file .env.docker run --rm wasm && \
cd rhwp-studio && npx vite
```

### Ports

| Service | Port | Config File |
|---------|------|-------------|
| Vite Dev Server | **7700** | `rhwp-studio/vite.config.ts` |
