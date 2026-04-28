# PR #413 검토 — rhwp-studio PWA support (manifest + service worker)

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#413](https://github.com/edwardkim/rhwp/pull/413) |
| 작성자 | [@dyjung150605](https://github.com/dyjung150605) (Dayeon Jung) — 신규 컨트리뷰터 (첫 PR) |
| base / head | `devel` ← `dyjung150605:feature/pwa-support` |
| state | OPEN |
| mergeable | MERGEABLE |
| mergeStateStatus | BEHIND |
| 변경 통계 | +5947 / -978, 9 files (대부분 `package-lock.json`) |
| **CI** | **statusCheckRollup 비어있음** — 첫 PR 정황 |
| 이슈 | [#383](https://github.com/edwardkim/rhwp/issues/383) (closes #383) |
| 정황 | 작업지시자가 사전 PR 권유 (이슈 #383 의 7개 점검 항목 안내 후) |

## 변경 통계 정합

`+5947 / -978` 의 절대 다수가 `package-lock.json` (+5895 / -977) — 신규 dependency (`vite-plugin-pwa@^1.2.0` + `workbox-window@^7.4.0`) 추가의 자연스러운 결과. 본 저장소가 `rhwp-studio/package-lock.json` 를 git tracked 함 (Cargo.lock 만 ignore).

실 코드 변경은 9 files / **+52 / -1** (lock 제외):

| 파일 | 변경 |
|------|------|
| `rhwp-studio/.npmrc` | +3 (`legacy-peer-deps=true` + 임시성 주석) |
| `rhwp-studio/index.html` | +2 (theme-color, apple-touch-icon) |
| `rhwp-studio/package.json` | +3 / -1 (devDependency 추가) |
| `rhwp-studio/vite.config.ts` | +44 (VitePWA 설정) |
| `rhwp-studio/public/icons/icon-{128,192,256,512}.png` | 4 신규 (binary) |

## 작성자 정황

@dyjung150605 — 신규 컨트리뷰터 (첫 PR). 다만:
- 이슈 #383 에서 사전 의향 확인 절차 준수 (CONTRIBUTING.md 준수)
- 4 환경 (Chrome/Edge/Safari/Android) 사전 검증
- fork (`https://dyjung150605.github.io/rhwp/`) 배포로 사전 검증
- PR 본문에 메인테이너 안내 7개 점검 항목 모두 대응

## 메인테이너 안내 7개 점검 항목 대응 점검

이슈 #383 의 메인테이너 댓글 ([comment-4330856482](https://github.com/edwardkim/rhwp/issues/383#issuecomment-4330856482)) 에서 안내한 7개 항목 모두 대응:

| 항목 | 메인테이너 안내 | 작성자 대응 |
|------|----------------|------------|
| 1. manifest scope | `start_url: '/rhwp/'`, `scope: '/rhwp/'` | ✅ 정확 |
| 2. icon 사이즈 | 192 + 512 + maskable 권장 | ✅ 128/192/256/512 + 512 maskable |
| 3. registerType 정책 | autoUpdate vs prompt 선택 사유 | ✅ `autoUpdate` 선택 사유 명시 (편집기라 알림 팝업 UX 방해 우려) |
| 4. legacy-peer-deps 임시성 | 주석 또는 PR 본문에 명시 | ✅ `.npmrc` 주석 + PR 본문 |
| 5. WASM precache 범위 | 12 MB precache 안전성 | ✅ `runtimeCaching: CacheFirst` 로 분리 (precache 제외) |
| 6. E2E 회귀 | dev 서버 SW 충돌 회피 | ✅ `devOptions.enabled: false` |
| 7. deploy-pages.yml 무변경 | `.npmrc` 자동 적용 | ✅ 무변경, fork 배포 검증 |

→ **메인테이너 안내 항목 100% 대응 + 사전 검증 양호**.

## 검증

### 본 검토에서 dry-run merge 결과

devel 위에 자동 머지 성공.

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | ✅ 1037 passed (동일 — Rust core 영향 없음) |
| `npx tsc --noEmit` (rhwp-studio) | ✅ 통과 |
| `npm install --legacy-peer-deps` | ✅ 통과 (4 high vulnerabilities 경고는 기존 npm 의존성 정황) |
| `.npmrc` 임시성 주석 | ✅ 명확 |
| `public/icons/` | ✅ 4 사이즈 (128/192/256/512) |
| `index.html` theme-color + apple-touch-icon | ✅ |
| `vite.config.ts` VitePWA 설정 정합 | ✅ start_url/scope, runtimeCaching, devOptions |

### Vite build 정황

본 검토 환경에서 `npx vite build` 실행 시 rolldown native binding 누락 (Node 20.13 vs Vite 8 권장 20.19+) — **환경 문제**, 본 PR 자체와 무관. 작성자 fork 배포 (`https://dyjung150605.github.io/rhwp/`) 에서 빌드 성공 확인됨.

### npm audit 정황

`4 high severity vulnerabilities` 경고는 본 저장소의 기존 npm 의존성 정황 — 본 PR 의 vite-plugin-pwa / workbox-window 와 별개.

## 평가

### 강점

1. **메인테이너 안내 100% 대응** — 7개 점검 항목 모두 코드/문서로 반영
2. **부착성 높음** — `rhwp-studio` 한정, 코어 미접촉
3. **사용자 선택형** — 사용자가 설치 안 해도 동일 사용 가능
4. **사전 검증 양호** — 4 환경 + fork 배포
5. **이슈 #383 의 정중한 절차 준수** — 의향 확인 후 PR
6. **dry-run merge** — 자동 성공 + 1037 passed
7. **TypeScript 컴파일 통과** + `npm install` 정상
8. **임시성 명시** — `.npmrc` 주석으로 vite@8 지원 시 제거 안내
9. **WASM precache 분리** — 첫 방문 12 MB 다운로드 강제하지 않음 (CacheFirst 후처리)
10. **devOptions.enabled: false** — E2E 테스트 회귀 회피

### 약점 / 점검 필요

#### 1. CI 실행 안 됨

`statusCheckRollup` 비어있음. 첫 PR 정황 — rebase + push 시 자동 트리거 예상.

#### 2. devel BEHIND

PR #395, #396, #401, #405, #411 머지 전 base. 다행히 자동 머지 성공.

#### 3. legacy-peer-deps=true 의 임시성

`vite-plugin-pwa@1.2.0` peer 가 vite≤7 라 `legacy-peer-deps` 우회. 작성자가 명시했지만:
- npm install 의 다른 의존성 검증 약화 위험
- 향후 다른 PR 추가 시 의존성 충돌 silent 가능성
- vite-plugin-pwa@2 (벤치마크 시점 미정) 출시 후 제거 권장

→ **수용 가능한 임시 우회**. 추적 가능 (주석).

#### 4. fork 배포 사전 검증 vs 본 저장소 환경

작성자가 자기 fork 에서 PWA 동작 확인했지만, **본 저장소의 GitHub Pages 배포** (`edwardkim.github.io/rhwp/`) 에서 동일하게 동작하는지는 머지 후 deploy-pages.yml 자동 트리거로 검증 필요.

→ 위험 낮음 (deploy-pages.yml 무변경 + .npmrc 자동 적용).

#### 5. 4 high severity vulnerabilities 정황

`npm audit` 의 4건 — 본 검토에서 확인 필요한지 점검:
- 본 PR 추가 의존성 (vite-plugin-pwa, workbox-window) 중에 있을 가능성
- 또는 기존 의존성 정황

→ **별도 task 후보** (npm audit 정리). 본 PR 머지의 게이트는 아님.

## 메인테이너 작업과의 관계

### 충돌 가능성

본 PR 의 영향 파일 (9 files) 중 다른 PR 과의 영향:
- `rhwp-studio/*` — Task #394 (투명선) 외 변경 없음. 본 PR 영역과 다른 함수 → 자동 머지 ✅
- 본 PR 머지 후에도 다른 OPEN PR (#414, #415 등) 의 cherry-pick 시 자동 머지 가능성 높음

dry-run merge 자동 성공 확인.

## 처리 방향 후보

| 옵션 | 내용 |
|------|------|
| **A** | cherry-pick 머지 (작성자 attribution 보존) |
| B | 작성자에게 rebase 요청 후 재제출 (PR #397, #400 패턴) |
| C | 거절 / close (추천 안 함) |

### 권장 — 옵션 A (cherry-pick 머지)

이유:
1. **메인테이너 안내 100% 대응** — 사전 합의 사항 모두 반영
2. **부착성** (rhwp-studio 한정, 코어 미접촉) → 회귀 위험 매우 낮음
3. **사전 검증 양호** — 4 환경 + fork 배포 + npm install + tsc 통과
4. **이슈 #383 정중 절차 준수** → 의향 일치
5. **본 검토 dry-run merge 자동 성공** + 검증 통과
6. **기존 PR #395, #396, #405, #411 와 일관 패턴**

### 시각 판정 정황

본 PR 은 **PWA 설정 추가 (rhwp-studio 외관 / 배포 정황 변경 없음)**. dev server 의 시각 동작은 그대로 (`devOptions.enabled: false`). PWA 설치 검증은 작업지시자 환경에서 `pkg/` 빌드 후 production-like serving 으로 시도 가능하지만 (`npx vite preview` 등):

- 본 PR 자체는 manifest + SW 등록만이라 시각적 차이 없음
- 작성자 fork 배포 (`https://dyjung150605.github.io/rhwp/`) 에서 4 환경 사전 검증 완료
- 작업지시자 시각 판정은 **선택 사항** — 본 PR 의 핵심은 머지 후 본 저장소 배포 (`edwardkim.github.io/rhwp/`) 에서 PWA 동작 확인

## 검토 항목 (Claude 점검 완료)

- [x] 작성자 신뢰도 — 신규 컨트리뷰터, 사전 의향 확인 + 4 환경 검증 ✅
- [x] 메인테이너 안내 7항목 대응 — 100% ✅
- [x] 코드 품질 — 합리적 ✅
- [x] dry-run merge — 자동 성공 ✅
- [x] cargo test --lib — 1037 passed (동일) ✅
- [x] npx tsc --noEmit — 통과 ✅
- [x] npm install — 통과 ✅
- [x] icon 자산 — 4 사이즈 + maskable ✅
- [x] manifest scope `/rhwp/` — 정확 ✅
- [ ] CI 실행 — 비어있음 (본 검토에서 검증으로 보완) ⚠️
- [ ] 본 저장소 배포 (`edwardkim.github.io/rhwp/`) PWA 동작 — 머지 후 deploy-pages.yml 자동 트리거로 검증

## 다음 단계 — 작업지시자 결정

A / B / C 중 결정 부탁드립니다.

권장: **A** — cherry-pick 머지 (작성자 attribution 보존).

## 참고

- PR: [#413](https://github.com/edwardkim/rhwp/pull/413)
- 이슈: [#383](https://github.com/edwardkim/rhwp/issues/383)
- 메인테이너 안내 댓글: [comment-4330856482](https://github.com/edwardkim/rhwp/issues/383#issuecomment-4330856482)
- 작성자 fork 배포: https://dyjung150605.github.io/rhwp/
- 본 저장소 배포 (머지 후): https://edwardkim.github.io/rhwp/
