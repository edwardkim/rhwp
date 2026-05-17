---
PR: #961
제목: fix — horz_rel_to=Column picture column 외부 emit 시 cursor advance skip (시험지 page 1 문9 정합, closes #959)
컨트리뷰터: @jangster77 (Taesup Jang) — 24+ 사이클 핵심 컨트리뷰터 (연속 5 PR 3번째)
처리: 옵션 A — 1 commit cherry-pick + orders 충돌 수동 해결 + 자기 검증 + WASM 재빌드 + no-ff merge
처리일: 2026-05-18
머지 commit: 586e3cc0
---

# PR #961 처리 보고서

## 1. 처리 결과

✅ **머지 완료** — 옵션 A. **Issue #952 종결** (Issue 1/2/3 모두 해결).

| 항목 | 값 |
|------|-----|
| 머지 commit | `586e3cc0` (--no-ff merge) |
| Cherry-pick commit | `03f2105e` (orders 1건 충돌 수동 해결, layout.rs auto-merge) |
| closes | #959 (Issue #952 영역 영역 **Issue 3**) |
| Issue #952 | **CLOSED** (Issue 1/2/3 모두 해결) |
| 시각 판정 | ✅ 작업지시자 시각 검증 통과 |
| 자기 검증 | cargo test 1288 passed + clippy + sweep 169/169 same + WASM 4.4 MB |
| 연속 5 PR | #956 ✅ → #958 ✅ → **#961 (3번째)** → #963 → #964 |

## 2. 본질 (Issue #959 = Issue #952 Issue 3)

시험지 (3-11월) page 1 우측 단 문9 ~250px 처짐.

### Root cause (RHWP_DEBUG_TAC_CURSOR 추적)
```
Shape pi=69 ci=0 y_in=709.4 y_out=983.4 dy=274.0 ⚠️
```
pi=69 picture (horz_rel_to=Column, h_offset=79.5mm, Center):
pic_emit_x = col.x + (col.w-pic.w)/2 + h_offset = 399+68+300 = 767
> col_area right 759.7 → picture 좌측 edge column 외부.
한컴 PDF: 우측 단 picture 미표시. 그럼에도 cursor 274px advance → 문9 처짐.

## 3. 정정 본질 — `src/renderer/layout.rs:3537`

```rust
let saved_y_offset = y_offset;
result_y = self.layout_body_picture(...);
// horz_rel_to=Column picture pic_emit_x 가 col_area 우측 초과 시
result_y = saved_y_offset;  // (:3566) advance skip
```

영역 좁힘: horz_rel_to=Column + col 외부 한정 — col 내부/Paper/Page/Para/TAC 영향 없음.

## 4. 본 환경 충돌 수동 해결

| 파일 | 충돌 | 정합 |
|------|------|------|
| `mydocs/orders/20260517.md` | changed in both | 본 환경 PR #956/#958 처리 표 + PR #961 Task #959/#960 작업 일지 양측 보존 통합 |
| `src/renderer/layout.rs` | auto-merge | PR #956 `paper_based=true` :770 + PR #958 `caption_is_empty` :3491 + PR #961 `saved_y_offset` :3537 **3 정정 양립** (RHWP_DEBUG_TAC_CURSOR :2022 PR #958 머지 영역 영역 동일 자동 병합) |
| 시험지 fixture 8 + PDF 4 | already exists | PR #956 머지 영역 영역 동일 content (devel 보존) |
| task_m100_959* 8 | added in remote | 신규 추가 (충돌 없음) |

## 5. 본 환경 검증

| 검증 | 결과 |
|------|------|
| `cherry-pick` 1 commit + orders 수동 해결 | ✅ |
| PR #956/#958/#961 layout.rs 3 정정 양립 | ✅ :770 + :3491 + :3537 공존 확인 |
| `cargo test --release --lib` | ✅ **1288 passed, 0 failed** (PR 본문 정합) |
| `cargo clippy --release --lib -- -D warnings` | ✅ 통과 |
| **광범위 sweep 7 fixture / 169 페이지** | ✅ **169 same / 0 diff** (회귀 부재) |
| WASM 재빌드 | ✅ 4.4 MB |
| 작업지시자 시각 판정 | ✅ **통과** |

## 6. 작업지시자 시각 판정 ✅ 통과

- 시험지 (3-11월) page 1 문9 — y=805 정합 (한컴 PDF `pdf/3-11월_실전_통합_2022.pdf` 권위, ~250px 처짐 해소)
- 시험지 4종 (3-09월/3-09월2023/3-10월/3-11월) page 1 정상
- exam_kor page 18 (Square wrap, Task #722) 정상
- PR #956 page border + PR #958 sample16 page 18 회귀 부재 (3 정정 양립)

## 7. Issue #952 종결

| Issue | PR | 머지 |
|-------|-----|------|
| Issue 1 (페이지 외곽선 paper/body) | #956 | `b31e38ff` |
| Issue 2 (sample16 page 18 본문 밀림) | #958 (#957) | `0b630773` |
| Issue 3 (시험지 page 1 문9 vertical) | #961 (#959) | `586e3cc0` |

→ Issue #952 CLOSED. 초기 진단 영역 영역 3 별개 회귀 분리 → 부분 해결 + 명확한 분리
(archive/task936 "9회 시도 + 5회 revert" 대조 교훈) 정합.

잔존: Issue #960 (page 2 multi-line equation off-by-one, pre-existing, 별도 task).

## 8. CI 통과

✅ Build & Test + CodeQL (js-ts/python/rust) + Canvas visual diff

## 9. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @jangster77 **24+ 사이클** (연속 5 PR 3번째) |
| `feedback_image_renderer_paths_separate` | layout.rs column picture advance 단일 — 다른 렌더 경로 무관 |
| `feedback_hancom_compat_specific_over_general` | horz_rel_to=Column + col 외부 한정 가드 — 케이스별 명시 (일반화 없음) |
| `feedback_diagnosis_layer_attribution` 권위 사례 강화 | RHWP_DEBUG_TAC_CURSOR 추적 영역 영역 pi=69 ci=0 dy=274 정확 진단 + Issue #952 → Issue 1/2/3 + #960 분리 (4-way 진단) |
| `feedback_pr_supersede_chain` 권위 사례 강화 | Issue #952 통합 → #956 (Issue 1) → #958 (Issue 2) → **#961 (Issue 3)** + #960 (신규 분리) — 3 PR 분리 정정 + Issue 종결 |
| `reference_authoritative_hancom` | 시험지 한컴 PDF (`pdf/3-11월`) 권위 page 1 문9 y=805 기준 |
| `feedback_close_issue_verify_merged` | Issue #952 close 전 Issue 1/2/3 3 PR 머지 (`b31e38ff`/`0b630773`/`586e3cc0`) devel 반영 확인 |

## 10. 잔존 후속

- 본 PR 본질 정정 (Issue 3) 의 잔존 결함 부재
- Issue #959 + Issue #952 close 완료
- Issue #960 (page 2 multi-line equation off-by-one) — 별도 task, 연속 PR #963/#964 영역 영역 후속 가능

---

작성: 2026-05-18
