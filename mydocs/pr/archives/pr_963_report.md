---
PR: #963
제목: fix — line break 직전 inline TAC control 의 line 매핑 정정 (시험지 page 2 cases formula off-by-one 해소, closes #960)
컨트리뷰터: @jangster77 (Taesup Jang) — 24+ 사이클 핵심 컨트리뷰터 (연속 5 PR 4번째)
처리: 옵션 A — 본질 commit cherry-pick + orders 충돌 수동 해결 + 자기 검증 + WASM 재빌드 + no-ff merge
처리일: 2026-05-18
머지 commit: 415b9d8d
---

# PR #963 처리 보고서

## 1. 처리 결과

✅ **머지 완료** — 옵션 A (본질 commit `90fb1684`만 cherry-pick, devel merge commit 2개 제외)

| 항목 | 값 |
|------|-----|
| 머지 commit | `415b9d8d` (--no-ff merge) |
| Cherry-pick commit | `11b6d419` (본질만, orders 1건 충돌 수동 해결) |
| closes | #960 (Issue #952 영역 영역 **Issue 4** — PR #961 page 2 검증 중 발견) |
| 시각 판정 | ✅ 작업지시자 시각 검증 통과 + exam_math p18 한컴 2020 직접 확인 |
| 자기 검증 | cargo test 1288 passed + clippy + sweep **168 same / 1 diff (의도 확정)** + WASM 4.4 MB |
| 연속 5 PR | #956 ✅ → #958 ✅ → #961 ✅ → **#963 (4번째)** → #964 |

## 2. 본질 (Issue #960)

시험지 (3-11월) page 2 문14 (pi=117) cases formula 가 line 0 (y=329) 에 emit
(예상 line 1 ~y=347) → header text 와 시각 overlap.

### Root cause (RHWP_DEBUG_PARA_TAC 추적)
- pi=117 text 에 FFFC 없음 (HWP3 인코딩)
- char_offsets gap 분석 (model/paragraph.rs:817-838): cases (ci=3) → position 30 (= `\n`)
- compose_lines line 1 chars [23, 30) 에서 position 30 (`\n`) 제외
- paragraph_layout filter `is_last_run && pos == end` 만 허용 → line 1 (last line 아님) 누락
- → line 0 (header) emit → overlap

## 3. 정정 본질 — `src/renderer/layout/paragraph_layout.rs:1730`

```rust
let allow_end_tac = is_last_run
    || (comp_line.has_line_break && is_last_run_of_line(run_idx));
let run_tacs = tac_offsets_px.iter()
    .filter(|(pos, _, _)| *pos >= run_char_pos
        && (*pos < run_char_end || (allow_end_tac && *pos == run_char_end)))
    ...
```

- `allow_end_tac` — `is_last_run` (기존) OR `has_line_break line 마지막 run`
- has_line_break line 마지막 run end-position TAC 포함 → line 정확 위치 inline emit
- `RHWP_DEBUG_PARA_TAC` 진단 영구화 (PR #956~#961 진단 패턴 정합)
- 기존 인프라 활용 — `is_last_run_of_line` (:1525), `has_line_break`, `is_last_run` (:1719)

## 4. 영역 좁힘

| 영역 | 영향 |
|------|------|
| has_line_break + end-position control | 정상화 (이전 누락 → line 내 inline emit) |
| has_line_break 없는 line | 영향 없음 |
| 일반 TAC control (line 안쪽) | 영향 없음 |

→ has_line_break + end-position 한정 — `feedback_hancom_compat_specific_over_general` 정합.

## 5. ⚠️ sweep diff 1건 — exam_math page 18 (의도 확정)

광범위 sweep 결과 **168 same / 1 diff** (`exam_math_017.svg`, page 18).

### diff 분석
- **AFTER**: equation 1개 추가 `translate(366.84, 301.65)` (y=301.65 line, 우측 x=366.84)
- **BEFORE (devel)**: 해당 위치 영역 영역 텍스트 `translate(85.52~120.1, 343.83)` (y=343.83 다른 line)

→ 본 PR 정정 (has_line_break end-position TAC) 영역 영역 exam_math page 18 영역 영역
line break 직전 inline equation 영역 영역 올바른 line 영역 영역 inline emit — 시험지
page 2 cases 와 **동일 본질**.

### 판정 — 의도된 정정 확정
PR 본문 영역 영역 "exam_math 회귀 0" 명시였으나 본 환경 영역 영역 diff 1건 검출.
**작업지시자 한컴 2020 에디터 직접 확인 영역 영역 의도된 정정 확정** — 이전 equation
영역 영역 잘못된 line (y=343.8) emit → 정정 후 올바른 line (y=301.7).
회귀 아님, 추가 정합 효과. `feedback_visual_judgment_authority` 정합 —
sweep diff 영역 영역 작업지시자 시각 판정 영역 영역 의도/회귀 최종 결정.

## 6. 본 환경 충돌 수동 해결

| 파일 | 충돌 | 정합 |
|------|------|------|
| `mydocs/orders/20260517.md` | changed in both | `git checkout --ours` (본 환경 PR #956/#958/#961 처리 표 보존) + Task #960/#962 작업 일지 갱신 |
| `src/renderer/layout/paragraph_layout.rs` | auto-merge | devel 변경 부재. PR #956/#958/#961 은 `layout.rs`, 본 PR 은 `layout/paragraph_layout.rs` — 다른 파일, **4 정정 양립** |
| `task_m100_960*` 9 | added in remote | 신규 추가 |
| 시험지 fixture/PDF | 미포함 | PR 본문 명시 — PR #956/#961 영역 영역 추가, 중복 방지 |

devel merge commit (`d690b878`/`8626a8b1`) 영역 영역 cherry-pick 제외 — 본질 `90fb1684` 만.

## 7. 본 환경 검증

| 검증 | 결과 |
|------|------|
| `cherry-pick` 본질 commit + orders 수동 해결 | ✅ |
| PR #956/#958/#961/#963 양립 | ✅ layout.rs 3 + paragraph_layout.rs 1 (다른 파일) |
| `cargo test --release --lib` | ✅ **1288 passed, 0 failed** (PR 본문 정합) |
| `cargo clippy --release --lib -- -D warnings` | ✅ 통과 |
| **광범위 sweep 7 fixture / 169 페이지** | ⚠️ **168 same / 1 diff (exam_math_017, 의도 확정)** |
| WASM 재빌드 | ✅ 4.4 MB |
| 작업지시자 시각 판정 | ✅ **통과** (한컴 2020 에디터 직접 확인 — exam_math p18 의도 확정) |

## 8. 작업지시자 시각 판정 ✅ 통과

- 시험지 (3-11월) page 2 문14 cases formula — y=352 line 1 정합 (한컴 PDF `pdf/3-11월_실전_통합_2022.pdf` 권위, header overlap 해소)
- **exam_math page 18 sweep diff — 한컴 2020 에디터 직접 확인 영역 영역 의도된 정정 확정**
- 시험지 4종 page 2 / exam_kor/eng / sample10~14 회귀 부재
- PR #956/#958/#961 회귀 부재 (4 정정 양립)

## 9. Issue #952 / 분리 결함 추적 (최종)

| Issue | PR | 상태 |
|-------|-----|------|
| #952 Issue 1 (외곽선) | #956 | ✅ merged |
| #952 Issue 2 (sample16 p18) | #958 (#957) | ✅ merged |
| #952 Issue 3 (시험지 p1 문9) | #961 (#959) | ✅ merged (Issue #952 closed) |
| #960 (시험지 p2 cases) | #963 (본 PR) | ✅ merged |
| #962 (보기 textbox scramble) | 별도 task | OPEN (pre-existing) |

## 10. CI 상태

CI Build & Test pending (devel merge commit 재트리거). 본 환경 자기 검증
(cargo test + clippy + sweep + 작업지시자 시각 판정 + 한컴 2020 직접 확인) 영역 영역 보완.

## 11. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @jangster77 **24+ 사이클** (연속 5 PR 4번째) |
| `feedback_image_renderer_paths_separate` | paragraph_layout.rs TAC line 매핑 단일 — layout.rs (#956/#958/#961) 와 다른 파일 |
| `feedback_hancom_compat_specific_over_general` | has_line_break + end-position 한정 가드 — 케이스별 명시 (일반화 없음) |
| `feedback_diagnosis_layer_attribution` 권위 사례 강화 | RHWP_DEBUG_PARA_TAC + char_offsets gap 분석 영역 영역 pi=117 position 30 (`\n`) off-by-one 정확 진단 |
| `feedback_visual_judgment_authority` 권위 사례 강화 | sweep diff 1건 (exam_math p18) 영역 영역 작업지시자 한컴 2020 에디터 직접 확인 영역 영역 의도/회귀 최종 결정 — sweep ≠ 자동 회귀 판정, 작업지시자 시각 판정 권위 |
| `feedback_pr_supersede_chain` 권위 사례 강화 | Issue #952 (Issue 1/2/3) → #956/#958/#961 + Issue #960 (Issue 4, PR #961 검증 발견) → **#963** + #962 (신규 분리) — 검증 중 결함 발견 연쇄 |
| `reference_authoritative_hancom` | 시험지 한컴 PDF (`pdf/3-11월`) page 2 cases y=352 기준 + 작업지시자 한컴 2020 에디터 직접 확인 (exam_math p18) |

## 12. 잔존 후속

- 본 PR 본질 정정 (Issue 4) 의 잔존 결함 부재
- Issue #960 close 완료
- Issue #962 (page 2 보기 textbox scramble) — 별도 task, 연속 PR #964 영역 영역 후속 가능

---

작성: 2026-05-18
