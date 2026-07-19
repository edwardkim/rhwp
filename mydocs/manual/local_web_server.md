---
kind: guide
status: active
canonical: mydocs/manual/dev_environment_guide.md
last_verified: 2026-07-17
---

# 로컬 웹서버 동작 매뉴얼

---

## [rhwp-studio] Vite 개발 서버 (현재 지원)

### 개요

TypeScript 기반 rhwp-studio를 Vite 개발 서버로 실행한다.
`localhost`는 브라우저 Secure Context이므로 HTTP로도 Clipboard API가 정상 동작한다.

### 사전 조건

- Node.js와 npm
- `wasm-pack`

### 실행 순서

#### 1. WASM 빌드 (소스 변경 시마다 실행)

저장소 루트에서 실행한다.

```bash
wasm-pack build --target web --out-dir pkg
```

빌드 결과물: `pkg/rhwp_bg.wasm`, `pkg/rhwp.js`, `pkg/rhwp.d.ts`

#### 2. 개발 서버 시작

```bash
cd rhwp-studio
npm ci
npx vite --host 0.0.0.0 --port 7700
```

브라우저에서 접속:

```text
http://localhost:7700        # 로컬
http://<PC의 IP>:7700        # 같은 네트워크의 다른 기기
```

> `npm run dev`도 동일하게 동작한다. (`package.json`의 dev 스크립트가 `vite`를 실행)

### 포트

| 서비스 | 포트 | 설정 파일 |
|--------|------|-----------|
| Vite 개발 서버 | **7700** | `rhwp-studio/vite.config.ts` |
