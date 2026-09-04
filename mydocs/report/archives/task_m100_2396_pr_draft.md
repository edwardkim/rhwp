# PR 초안 — #2396 custom scheme 최상위 legacy 요청 회귀

## 제목

```text
fix(rhwp-studio): custom scheme 최상위 legacy 요청 복구
```

## 본문

```markdown
## Summary

- custom URL scheme으로 로드된 top-level same-window `rhwp-studio`에서 legacy `rhwp-request`를 허용합니다.
- iframe parent의 HTTP(S) origin 검사와 parent source 검사를 유지합니다.
- custom scheme top-level v1 connect, iframe parent와 forged sibling 거부를 회귀 테스트로 고정합니다.

## Background

`v0.7.19`의 embed runtime은 legacy dispatch 전에 모든 message에 HTTP(S) parent origin 검사를 적용합니다.
WKWebView의 custom scheme 최상위 문서는 `parentWindow === hostWindow`이고 source도 동일하지만 origin이
HTTP(S)가 아니어서 `ready`, `pageCount`, `getPageSvg`, `exportHwp` 등이 timeout됐습니다.

## Changes

- top-level same-window legacy 요청 여부를 parent/host window identity와 message type으로 판정합니다.
- unusable origin은 iframe 환경에서 계속 거부합니다.
- source가 parent와 다른 요청은 환경과 무관하게 계속 거부합니다.
- top-level legacy 허용과 v1 connect/iframe/sibling 거부 unit regression tests를 추가합니다.

## Security boundary

- 허용: `event.source === parentWindow`이고 `parentWindow === hostWindow`인 top-level same-window legacy `rhwp-request`
- 거부: custom scheme top-level v1 `rhwp-connect`
- 거부: custom scheme iframe parent
- 거부: parent가 아닌 forged sibling
- 유지: 기존 HTTP(S) iframe MessageChannel/legacy 경로

custom scheme allowlist, protocol version과 public API는 변경하지 않습니다.

## Validation

- `node --test rhwp-studio/tests/embed-protocol.test.ts`: 15/15 PASS
- `npm --prefix rhwp-studio test`: 365/365 PASS
- `wasm-pack build --target web --out-dir pkg --dev`: PASS
- `npm --prefix rhwp-studio run build`: PASS
- `git diff --check`: PASS

실제 macOS WKWebView downstream representative suite는 이 저장소 환경에서 실행하지 않았습니다.

Closes #2396
```

## 생성 조건

- base: `edwardkim/rhwp:devel`
- head: `postmelee:codex/issue-2396-custom-scheme-legacy-request`
- push와 PR 생성은 작업지시자 별도 승인 후 실행한다.
