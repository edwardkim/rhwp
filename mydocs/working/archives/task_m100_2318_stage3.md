# #2318 3단계 완료보고 — studio TS 정정 + WASM + CDP e2e

- 계획서: `mydocs/plans/task_m100_2318.md`
- 브랜치: `local/task2318`

## 수정 내용

| 파일 | 내용 |
|------|------|
| `rhwp-studio/src/core/types.ts` | `LayerInfo.masterPage?: boolean` (layer JSON 의 masterPage 소비) |
| `rhwp-studio/src/view/canvaskit/replay-plane.ts` | `capMasterPagePlane` — rust `cap_master_page_plane` 과 동일 계약 (background 제외 behindText 상한). 두 분류 함수 모두 적용 |
| `rhwp-studio/src/view/page-renderer.ts` | 로컬 중복 분류 함수를 공유 모듈(`replay-plane.ts`) 위임으로 통일 — plane 집계(collectLayerPlaneSummary)와 DOM flow image 필터가 cap 을 자동 반영 |
| `rhwp-studio/tests/render-backend.test.ts` | masterPage cap 단위 테스트 추가 (front→cap, 상속 flow→cap, behind 유지, pageBackground 예외, 미표시 layer 기존 분류 유지) |
| `rhwp-studio/e2e/issue-2318-master-page-zorder.test.mjs` | 신규 e2e — overlay 구조 계약 검증 + 시각 기록 |

canvaskit-renderer.ts 는 공유 모듈 사용 중이라 수정 불요 (자동 반영).

## 게이트

- studio 단위 테스트 **307/307** (신규 cap 테스트 포함), `tsc --noEmit` 통과
- WASM 빌드 완료 (Docker, 2단계 rust 정정 포함 pkg/ 갱신)
  - Docker Desktop 다운 상태였음 — 복구 절차(taskkill stale 프로세스 →
    `wsl --terminate docker-desktop` → 재시작) 적용 후 정상 빌드
- CDP e2e (`e2e-cdp.md` 준수, host Chrome 19222 + vite 7700) **4/4 PASS**:
  1. shortcut.hwp 로드 (7페이지)
  2. 바탕쪽 → behind overlay canvas 존재
  3. front overlay canvas 부재
  4. 본문 flow canvas(z=2) > behind overlay(z=1)
- 시각 실측 (`e2e/screenshots/issue2318-02-page1-bottom-overlap.png`):
  본문 "Ctrl+Y"/"Alt+Y" 가 바탕쪽 쪽번호 "1" **위**에 표시 — 한컴 실기와 일치
  (정정 전에는 "1" 이 텍스트를 덮음)

## 비고

- e2e 샘플 경로: vite dev 서버는 루트 `samples/` 를 `/samples/*` 로 직접
  서빙(#741)하므로 `basic/shortcut.hwp` 상대 경로 사용 (public/ 복사 불요)

## 다음 단계

4단계: 전 게이트 (`cargo test --tests` release-test + fmt --check) +
Header/Footer 유사 증상 점검 → 시각 판정 요청.
