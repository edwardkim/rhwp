# PR #413 처리 보고서 — rhwp-studio PWA support

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#413](https://github.com/edwardkim/rhwp/pull/413) |
| 작성자 | [@dyjung150605](https://github.com/dyjung150605) (Dayeon Jung) — 신규 컨트리뷰터 (첫 PR) |
| 이슈 | [#383](https://github.com/edwardkim/rhwp/issues/383) (closes #383) |
| 처리 결정 | **옵션 A (cherry-pick 머지)** |
| 처리 일자 | 2026-04-29 |

## 처리 절차

### Stage 1: cherry-pick

`local/pr413` 브랜치에서 PR head 까지 2 commit cherry-pick — 작성자 attribution 보존:

| commit | 작성자 | 내용 |
|--------|--------|------|
| `0e7baad` (cherry-pick) | @dyjung150605 | feat(studio): add PWA support (manifest + service worker) |
| `e832840` (cherry-pick) | @dyjung150605 | refine(studio): address maintainer review on PWA config (#383) |

merge commit (`acc4814`) 은 제외.

### Stage 2: 검증

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | ✅ 1037 passed (동일 — Rust core 영향 없음) |
| `cargo test --test issue_418` | ✅ 1/1 (Task #418 보존) |
| `npx tsc --noEmit` (rhwp-studio) | ✅ 통과 |
| `npm install --legacy-peer-deps` | ✅ 통과 |
| `cargo clippy --lib -- -D warnings` | ✅ warning 0건 |

## 변경 요약

### 본질

`rhwp-studio` 에 PWA 설치 지원 추가:
- Web App Manifest (시작 URL, 아이콘, 테마색)
- Service Worker (vite-plugin-pwa + workbox)
- 사용자 설치형 앱 경험 (데스크톱 / 모바일)

### 변경 파일 (실 코드 +52 / -1, 외 package-lock.json)

| 파일 | 변경 |
|------|------|
| `rhwp-studio/vite.config.ts` | VitePWA 플러그인 + manifest + workbox + runtimeCaching |
| `rhwp-studio/.npmrc` | `legacy-peer-deps=true` (vite-plugin-pwa@1.2.0 의 vite≤7 peer 호환) + 임시성 주석 |
| `rhwp-studio/index.html` | theme-color + apple-touch-icon |
| `rhwp-studio/package.json` | vite-plugin-pwa@^1.2.0 + workbox-window@^7.4.0 |
| `rhwp-studio/public/icons/icon-{128,192,256,512}.png` | PWA 아이콘 4 사이즈 |
| `rhwp-studio/package-lock.json` | dependency 추가 lock 갱신 |

### 메인테이너 안내 7항목 100% 대응

이슈 #383 의 메인테이너 안내 ([comment-4330856482](https://github.com/edwardkim/rhwp/issues/383#issuecomment-4330856482)) 7개 점검 항목 모두 코드/문서로 반영:

| 항목 | 메인테이너 안내 | 작성자 대응 |
|------|----------------|------------|
| 1. manifest scope | `start_url: '/rhwp/'`, `scope: '/rhwp/'` | ✅ 정확 |
| 2. icon 사이즈 | 192 + maskable 권장 | ✅ 128/192/256/512 + 512 maskable |
| 3. registerType 정책 | autoUpdate vs prompt 사유 | ✅ `autoUpdate` 선택 사유 명시 |
| 4. legacy-peer-deps 임시성 | 추적 코멘트 | ✅ `.npmrc` 주석 + PR 본문 |
| 5. WASM precache | 12 MB precache 안전성 | ✅ runtimeCaching CacheFirst 분리 |
| 6. E2E 회귀 | dev 서버 SW 충돌 회피 | ✅ `devOptions.enabled: false` |
| 7. deploy-pages.yml | 무변경 | ✅ `.npmrc` 자동 적용 |

### 작성자 사전 검증

PR 본문에 명시:
- 4 환경 (Windows/Chrome, Windows/Edge, macOS/Safari, Android/Chrome) 사전 검증
- fork 배포 (`https://dyjung150605.github.io/rhwp/`) 로 동작 확인

## 시각 판정 정황

본 PR 은 **PWA 설정 추가** — dev server 동작은 그대로 (`devOptions.enabled: false`). 작성자 fork 배포로 4 환경 사전 검증 완료. 머지 후 본 저장소 배포 (`edwardkim.github.io/rhwp/`) 에서 PWA 설치 동작은 deploy-pages.yml 자동 트리거로 검증.

## 메모리 원칙 부합

| 메모리 | 부합 여부 |
|--------|----------|
| 외부 PR 머지 전 검증 게이트 | ✅ cargo test 1037 + tsc + npm install + clippy 0 |
| PR 댓글 톤 — 과도한 표현 자제 | ✅ |
| 보고서는 타스크 브랜치에서 커밋 | ✅ `local/pr413` 에서 커밋 |
| 이슈 close 시 commit devel 머지 검증 | (closes #383 명시, 머지 후 자동 close) |

## 다음 단계

1. 본 보고서 + 오늘할일 갱신 commit
2. `local/pr413` → `local/devel` → `devel` 머지 + push
3. PR #413 close + 작성자 댓글 (이슈 #383 자동 close)
4. 본 저장소 배포 (`edwardkim.github.io/rhwp/`) PWA 동작 확인 (deploy-pages.yml 자동 트리거)

## 참고

- 검토 문서: `mydocs/pr/pr_413_review.md`
- PR: [#413](https://github.com/edwardkim/rhwp/pull/413)
- 이슈: [#383](https://github.com/edwardkim/rhwp/issues/383)
- 메인테이너 안내 댓글: [comment-4330856482](https://github.com/edwardkim/rhwp/issues/383#issuecomment-4330856482)
- 작성자 fork 배포: https://dyjung150605.github.io/rhwp/
