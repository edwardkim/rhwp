---
PR: #966
제목: fix — WMF SetTextAlign vertical bits 파싱 정정 + baseline y shift 정합 (WMF 박스 외부 텍스트 표시 해소, closes #965, ports PR #918 Stage 33-A)
컨트리뷰터: @jangster77 (Taesup Jang) — 24+ 사이클 핵심 컨트리뷰터 (연속 5 PR #956~#964 완결 후 추가 2개 중 1번째)
처리: 옵션 A — 본질 commit cherry-pick + orders 충돌 수동 해결 + 자기 검증 + WASM 재빌드 + no-ff merge
처리일: 2026-05-18
머지 commit: 235e049c
---

# PR #966 처리 보고서

## 1. 처리 결과

✅ **머지 완료** — 옵션 A (본질 commit `88291fdb` 만 cherry-pick, devel merge 4 제외)

| 항목 | 값 |
|------|-----|
| 머지 commit | `235e049c` (--no-ff merge) |
| Cherry-pick commit | `1a890422` (본질만, orders 1건 충돌 수동 해결, svg/mod.rs auto-merge) |
| closes | #965 |
| 관련 | **PR #918 (CLOSED, +5082/-74, 5/16) Stage 33-A root cause 단독 포팅** |
| 시각 판정 | ✅ 작업지시자 시각 검증 통과 |
| 자기 검증 | cargo test 1288 passed + clippy + sweep 169/169 same + WASM 4.4 MB |

## 2. 본질 (Issue #965)

sample16 page 18 WMF 다이어그램 (주전산센터 목표시스템 구성안) 박스 내부 한글
텍스트 ("PE6450", "기록서버", "Windows 서버군") 박스 외부 벗어남.

### Root cause (`src/wmf/converter/svg/mod.rs:2208` set_text_align)
`mode & VTA_TOP(=0x0000) == 0x0000` 영역 영역 **항상 true** → BASELINE/BOTTOM mode 도
VTA_TOP 오매핑 → +ascent shift → baseline 박스 하단 걸침.
WMF [MS-WMF] 2.1.2.18: TA_TOP=0x0000 / TA_BOTTOM=0x0008 / TA_BASELINE=0x0018.

## 3. 정정 본질 — `src/wmf/converter/svg/mod.rs` 3 영역 (~60 lines)

| 영역 | 라인 | 정정 |
|------|------|------|
| `set_text_align` | :2208 | `v_bits = mode & 0x0018` 마스킹 + 우선순위 BASELINE→BOTTOM→TOP (root cause) |
| `ext_text_out` | :811 | baseline y shift (VTA_BASELINE=0, BOTTOM=-em*0.2, TOP=+ascent), `font.height < 0` 잘못된 보정 제거 |
| `text_out` | :1545 | META_TEXTOUT 동일 보정 (PR #918 미포함, 본 PR 추가) |

## 4. PR #918 supersede 분석

PR #918 (CLOSED, 2026-05-16, **+5082/-74 거대 PR**) close 사유 — 다양한 부작용
(LibreOffice emfio 포팅 / WASM RasterPlayer / nested SVG inline embed /
woff2 base64 임베드 제거 / DX byte-aware indexing / POLYPOLYGON fill-rule).

본 PR 영역 영역 Stage 33-A root cause fix (~60 lines) 만 단독 포팅 + text_out
보정 (PR #918 미포함) 추가.

→ `feedback_pr_supersede_chain` (a) 패턴 (CLOSED 거대 PR → root cause 작은 PR)
+ `feedback_small_batch_release_strategy` (5082 → 60 lines 분리).

## 5. 본 환경 충돌 수동 해결

| 파일 | 충돌 | 정합 |
|------|------|------|
| `mydocs/orders/20260517.md` | changed in both | `git checkout --ours` (본 환경 PR 처리 표 보존) + Task #965 작업 일지 갱신 |
| `src/wmf/converter/svg/mod.rs` | **auto-merge** | devel PR #860/#864 (5/16, `735d3057`, EMF/WMF image 렌더) 변경 영역 영역 PR #966 정정 (3 영역) 영역 영역 다른 라인 → 자동 병합. devel 함수 (ext_text_out:790/text_out:1534/set_text_align:2185) 보존 + PR #966 정정 양립 확인 |
| `task_m100_965*` 8 | added in remote | 신규 추가 |

devel merge commit (`a02ac9bf`/`66f6158b`/`ba674ee8`/`8620def4`) cherry-pick 제외 — 본질 `88291fdb` 만.

## 6. 본 환경 검증

| 검증 | 결과 |
|------|------|
| `cherry-pick` 본질 commit + 충돌 수동 해결 | ✅ |
| PR #966 정정 적용 | ✅ v_bits :2208 + VTA_BOTTOM/BASELINE 분기 :827/831/:1554/1558 |
| devel PR #860/#864 보존 | ✅ ext_text_out/text_out/set_text_align 함수 보존 |
| `cargo test --release --lib` | ✅ **1288 passed, 0 failed** (PR 본문 정합) |
| `cargo clippy --release --lib -- -D warnings` | ✅ 통과 |
| **광범위 sweep 7 fixture / 169 페이지** | ✅ **169 same / 0 diff** (회귀 부재) |
| WASM 재빌드 | ✅ 4.4 MB |
| 작업지시자 시각 판정 | ✅ **통과** |

sweep fixture 영역 영역 WMF 다이어그램 미포함 → 작업지시자 시각 검증 영역 영역 핵심 게이트.

## 7. 작업지시자 시각 판정 ✅ 통과

- sample16 (HWP3) page 18 WMF 다이어그램 — 박스 내부 한글 텍스트 ("PE6450", "기록서버", "Windows 서버군") 정상 위치 (한컴 viewer 정합, 박스 외부 벗어남 해소)
- WMF sample (hwp3-sample14 page 0~8, sample4 page 1) 회귀 부재
- devel PR #860/#864 EMF/WMF image 렌더 회귀 부재 (svg/mod.rs auto-merge 검증)
- 비-WMF sample (exam_kor/math/eng, sample10~13) 회귀 부재

## 8. 영역 좁힘 (PR 본문 명시)

| 영역 | 영향 |
|------|------|
| WMF BASELINE/BOTTOM 모드 텍스트 | 정상 위치 (회귀 fix) |
| WMF TOP 모드 텍스트 | 기존 동작 (영향 없음) |
| 비-WMF | 영향 없음 (svg/mod.rs WMF converter 만 변경) |
| WASM 환경 | 정합 개선 (Canvas2D 동일 SVG 사용) |

## 9. CI 통과

✅ Build & Test + CodeQL (js-ts/python/rust)

## 10. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @jangster77 **24+ 사이클** (연속 5 PR 완결 후 추가 #966/#968) |
| `feedback_image_renderer_paths_separate` | WMF converter svg/mod.rs 단일 — Canvas2D 동일 SVG (정합 개선) |
| `feedback_hancom_compat_specific_over_general` | WMF [MS-WMF] 2.1.2.18 spec 정합 (v_bits 0x0018 mask) — 추측 아닌 spec 기반 |
| `feedback_diagnosis_layer_attribution` 권위 사례 강화 | `mode & VTA_TOP(=0) == 0` 항상-true 버그 root cause 정확 진단 |
| `feedback_pr_supersede_chain` 권위 사례 강화 | **PR #918 (CLOSED, +5082 거대 PR, 다양한 부작용) → #966 (root cause ~60 lines 단독 포팅 + text_out 추가)** — (a) 패턴 |
| `feedback_small_batch_release_strategy` 권위 사례 강화 | 5082 lines 거대 PR → 60 lines root cause 분리 — 작은 단위 회전 입증 |
| `reference_authoritative_hancom` | sample16 page 18 WMF 박스 한컴 viewer 정합 기준 |

## 11. 잔존 후속

- 본 PR 본질 정정 (Issue #965) 의 잔존 결함 부재
- Issue #965 close 완료
- 추가 PR #968 (HWP3 sample18 빈 paragraph + 쪽나누기) 후속 진행 예정

---

작성: 2026-05-18
