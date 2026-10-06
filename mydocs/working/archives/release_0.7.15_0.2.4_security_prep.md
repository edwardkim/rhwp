# rhwp 0.7.15 / browser extension 0.2.4 security release prep

작성일: 2026-06-06

## 목적

#1307 보안 hardening 이 `devel`에 반영되었으므로, core `0.7.15` 패치 릴리즈와
Chrome/Edge/Firefox 확장 `0.2.4` 배포 준비를 시작한다.

공개 문서와 스토어 제출 문서에는 취약점 재현 PoC, 내부 URL, 세부 공격 절차를 포함하지 않는다.

## 배포 대상

| 대상 | 현재 버전 | 목표 버전 | 비고 |
|---|---:|---:|---|
| `Cargo.toml` / `@rhwp/core` | `0.7.14` | `0.7.15` | GitHub Release → npm OIDC publish |
| `rhwp-studio/package.json` | `0.7.14` | `0.7.15` | 제품정보/Pages 빌드 표시 |
| `rhwp-vscode/package.json` | `0.7.14` | `0.7.15` | Release workflow 배포 대상 |
| `npm/editor/package.json` | `0.7.14` | `0.7.15` | `@rhwp/editor` |
| `rhwp-chrome` / Edge | `0.2.3` | `0.2.4` | Chrome Web Store + Edge Add-ons 수동 업로드 |
| `rhwp-firefox` | `0.2.3` | `0.2.4` | AMO extension zip + source zip |

## #1307 반영 상태

GitHub Issue: https://github.com/edwardkim/rhwp/issues/1307

반영 커밋:

- `f8d7f091 Merge task #1307: harden extension fetch security`
- `e07b76ba fix(extension): harden document fetch security`

주요 변경 범위:

- Chrome/Firefox service worker fetch sender 검증
- extension-side document fetch URL 정책 추가
- localhost / loopback / private / link-local / 내부 호스트명 차단
- redirect 이후 최종 URL 재검증
- `credentials: "omit"` 적용
- 자동 thumbnail 데이터의 page DOM 직접 노출 방어

## 현재 로컬 검증

2026-06-06 기준 통과:

```bash
node rhwp-chrome/sw/fetch-security.test.mjs
node --check rhwp-chrome/content-script.js
node --check rhwp-firefox/content-script.js
cargo fmt --all -- --check
cargo build
cargo clippy --lib -- -D warnings
cargo test --lib
cargo test
docker compose --env-file .env.docker run --rm wasm wasm-pack build --target web --release
bash scripts/prepare-npm.sh
cd rhwp-chrome && npm run build
cd rhwp-firefox && npm run build
```

결과:

- fetch security policy tests passed
- Chrome content script syntax check 통과
- Firefox content script syntax check 통과
- Rust format/build/clippy/lib test/full test 통과
- WASM release build 통과 (`pkg/rhwp_bg.wasm`: 5.4MB)
- `pkg/package.json`: `@rhwp/core` `0.7.15`
- Chrome extension build 통과
- Firefox extension build 통과
- Chrome/Firefox dist manifest: `0.2.4`
- Chrome/Firefox dist WASM: 5.4MB

## 준비 작업 체크리스트

### 1. 버전 bump

- [x] `Cargo.toml`: `0.7.14` → `0.7.15`
- [x] `Cargo.lock`: rhwp package `0.7.14` → `0.7.15`
- [x] `rhwp-studio/package.json`: `0.7.14` → `0.7.15`
- [x] `rhwp-vscode/package.json`: `0.7.14` → `0.7.15`
- [x] `npm/editor/package.json`: `0.7.14` → `0.7.15`
- [x] `rhwp-chrome/manifest.json`: `0.2.3` → `0.2.4`
- [x] `rhwp-chrome/package.json`: `0.2.3` → `0.2.4`
- [x] `rhwp-chrome/package-lock.json`: `0.2.3` → `0.2.4`
- [x] `rhwp-chrome/content-script.js`: extension exposed version `0.2.3` → `0.2.4`
- [x] `rhwp-chrome/dev-tools-inject.js`: version 상수 `0.2.3` → `0.2.4`
- [x] `rhwp-firefox/manifest.json`: `0.2.3` → `0.2.4`
- [x] `rhwp-firefox/package.json`: `0.2.3` → `0.2.4`
- [x] `rhwp-firefox/package-lock.json`: `0.2.3` → `0.2.4`

### 2. 릴리즈 문서

- [x] `CHANGELOG.md` / `CHANGELOG_EN.md`: `0.7.15` 항목 추가
- [x] `README.md` / `README_EN.md`: 현재 배포 버전 및 v0.7.15 cycle 요약 반영
- [x] 공개 기여자 목록에 보안 제보자 `Dangel` 추가
- [x] `rhwp-chrome/README.md`: `v0.2.4` 변경사항 추가
- [x] `rhwp-firefox/README.md`: `v0.2.4` 변경사항 추가
- [x] `mydocs/feedback/chrome-0.2.4_kor.md`
- [x] `mydocs/feedback/chrome-0.2.4_eng.md`
- [x] `mydocs/feedback/edge-0.2.4_reviewer_notes.md`
- [x] Firefox AMO 제출용 reviewer/source 설명 준비

### 3. 보안 문구 원칙

- [x] PoC URL, 재현 절차, 내부망 스캔 방식 등 악용 가능한 세부 정보 제외
- [x] “service worker document fetch 경로 강화” 수준으로 공개 설명
- [x] 새 권한 없음 / 새 외부 네트워크 endpoint 없음 명시
- [x] 문서 처리는 브라우저 내부 WASM에서 수행됨 명시
- [x] 제보자 `Dangel`에 대한 감사 문구 포함

### 4. 검증

- [x] `cargo build`
- [x] `cargo test`
- [x] `cargo clippy --lib -- -D warnings`
- [x] WASM 빌드
- [x] `node rhwp-chrome/sw/fetch-security.test.mjs`
- [x] Chrome extension build
- [x] Firefox extension build
- [x] Chrome/Firefox dist manifest version 확인
- [x] 확장 dist에 `.env`, token, `node_modules`, `target`, 불필요 테스트 산출물 미포함 확인

### 5. 배포 순서

1. `devel`에서 준비 커밋 작성
2. `origin/devel` push 후 CI 통과 확인
3. `main` merge + push
4. GitHub Release `v0.7.15` 생성
5. npm / VSCode / Open VSX Actions 완료 확인
6. Chrome Web Store `0.2.4` 업로드
7. Edge Add-ons `0.2.4` 업로드
8. Firefox AMO `0.2.4` extension zip + source zip 업로드
