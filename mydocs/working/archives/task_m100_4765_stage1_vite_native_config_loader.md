---
kind: working
status: active
issue: 4765
stage: 1
last_verified: 2026-08-14
---

# #4765 Stage 1: Vite native config loader 경로 호환성

## 배경

`rhwp-studio`의 Vite 설정은 ESM 구성 파일에서 CommonJS 전역인 `__dirname`을 사용했다. Vite 8은 향후 기본값이 될 native config loader에서 이 패턴을 지원하지 않는다고 경고한다.

## 변경

- 설정 모듈의 디렉터리를 `import.meta.dirname`으로 한 번 계산했다.
- `package.json`, `pkg`, `target/rhwp-subsecond-vite`, `samples`, `npm`의 경로는 모두 기존과 같은 설정 파일 상대 위치를 기준으로 해석한다.
- 애플리케이션 런타임, WASM 산출물 위치, Vite 플러그인 설정은 바꾸지 않는다.

## 검증 계약

- PR CI와 동일한 Node.js 22에서 TypeScript 검사와 Studio 패키지 테스트를 실행한다.
- Node.js 22에서 Vite native config-loader 경고 없이 설정을 읽는지 개발 서버 기동으로 확인한다.
