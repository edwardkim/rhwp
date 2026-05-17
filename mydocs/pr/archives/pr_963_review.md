---
PR: #963
제목: fix — line break 직전 inline TAC control 의 line 매핑 정정 (시험지 page 2 cases formula off-by-one 해소, closes #960)
컨트리뷰터: @jangster77 (Taesup Jang) — 24+ 사이클 핵심 컨트리뷰터 (연속 5 PR 4번째)
base / head: devel / local/task960
mergeStateStatus: BLOCKED (CI 진행 중 — devel merge commit 재트리거)
mergeable: MERGEABLE
CI: Build & Test / Analyze pending (Python ✅ / Canvas visual diff ✅)
변경 규모: +704 / -1, 10 files (코드 1 / 문서 9)
커밋: 3 (본질 1 + devel merge 2)
검토일: 2026-05-18
---

# PR #963 검토

## 1. 메타

| 항목 | 값 |
|------|-----|
| PR 번호 | #963 |
| 제목 | fix: line break 직전 inline TAC control 의 line 매핑 정정 (시험지 page 2 cases formula off-by-one) |
| 컨트리뷰터 | @jangster77 — **24+ 사이클** (연속 5 PR **4번째**, #956/#958/#961 직후) |
| base / head | devel / local/task960 |
| mergeable | MERGEABLE (BLOCKED — CI 진행 중, devel merge 2 commit 재트리거) |
| CI | Build & Test/Analyze pending, Python ✅ / Canvas visual diff ✅ |
| 변경 규모 | +704 / -1, 10 files (코드 1 / 문서 9) |
| 커밋 수 | 3 (본질 `90fb1684` + devel merge `d690b878`/`8626a8b1`) |
| closes | #960 (Issue #952 영역 영역 **Issue 4** — 본 PR #961 page 2 검증 중 발견) |
| 연속 5 PR | #956 ✅ → #958 ✅ → #961 ✅ → **#963 (4번째)** → #964 |

## 2. 본질 (Issue #960)

`samples/3-11월_실전_통합_2022.hwp` page 2 문14 (pi=117) cases formula 가
line 0 영역 (y=329) 에 emit (예상 line 1 ~y=347) → header text 와 시각 overlap.

### Root cause (RHWP_DEBUG_PARA_TAC 추적)
- pi=117 text 에 FFFC 없음 (HWP3 인코딩)
- char_offsets gap 분석 (model/paragraph.rs:817-838): cases (ci=3) → codepoint position 30 (= `\n`)
- compose_lines: line 1 chars [23, 30) 에서 position 30 (`\n`) 제외
- paragraph_layout filter: `pos < end` 또는 `is_last_run && pos == end` 만 허용
  → line 1 (last line 아님) 누락 → line 0 (header) 영역 영역 emit → overlap

## 3. 정정 본질 — `src/renderer/layout/paragraph_layout.rs:1721`

```rust
let allow_end_tac = is_last_run
    || (comp_line.has_line_break && is_last_run_of_line(run_idx));
let run_tacs = tac_offsets_px.iter()
    .filter(|(pos, _, _)| *pos >= run_char_pos
        && (*pos < run_char_end || (allow_end_tac && *pos == run_char_end)))
    ...
```

- `allow_end_tac` — `is_last_run` (기존) **OR** `has_line_break line 의 마지막 run`
- has_line_break line 영역 영역 마지막 run 의 end position TAC 포함 → line 정확 위치 inline emit
- `RHWP_DEBUG_PARA_TAC` 진단 영구화 (PR #956~#961 진단 패턴 정합)

기존 인프라 활용 — `is_last_run_of_line` (:1525), `has_line_break`, `is_last_run` (:1719) 모두 devel 존재.

## 4. 영역 좁힘 (PR 본문 명시)

| 영역 | 영향 |
|------|------|
| has_line_break + end-position control | 정상화 (이전 누락 → line 내 inline emit) |
| has_line_break 없는 line | 영향 없음 |
| 일반 TAC control (line 안쪽) | 영향 없음 |

→ has_line_break + end-position 한정 — `feedback_hancom_compat_specific_over_general` 정합.

## 5. 본 환경 충돌 분석

| 파일 | 충돌 | 정합 |
|------|------|------|
| `mydocs/orders/20260517.md` | changed in both | 본 환경 PR #956/#958/#961 처리 표 + PR #963 Task #960 작업 일지 양측 보존 통합 |
| `src/renderer/layout/paragraph_layout.rs` | 충돌 없음 | devel 변경 부재 → auto-merge (PR #956/#958/#961 은 layout.rs, 본 PR 은 layout/paragraph_layout.rs — 다른 파일) |
| `task_m100_960*` 9 | added in remote | 신규 추가 (충돌 없음) |
| 시험지 fixture/PDF | 미포함 | PR 본문 명시 — PR #956/#961 영역 영역 추가, 본 PR 중복 방지 |

## 6. 본 환경 점검

### 6.1 PR #956/#958/#961/#963 양립
- PR #956/#958/#961: `src/renderer/layout.rs` (paper_based/caption_is_empty/saved_y_offset)
- PR #963: `src/renderer/layout/paragraph_layout.rs` (allow_end_tac) — **다른 파일, 무관**
→ 4 정정 양립.

### 6.2 CI 상태
- Build & Test / Analyze pending (devel merge commit `d690b878`/`8626a8b1` 재트리거)
- Python ✅ / Canvas visual diff ✅
- BLOCKED 사유 — CI 미완 (base 정책 아님). cherry-pick 후 본 환경 자기 검증으로 보완.

### 6.3 검증 (PR 본문)
- cargo test --release --lib: 1288 passed, 0 failed
- 시험지 page 2 cases formula y: 329 → 352 ✓ (line 1 정상 위치)
- 한컴 PDF 정합 ✓
- LAYOUT_OVERFLOW count: 41 → 41 (회귀 0)
- exam_kor/math/eng, sample10~14, 시험지 4종: 시각 회귀 0

### 6.4 잔존 (별도 issue #962)
page 2 보기 textbox content scramble (pi=118 InFrontOfText TAC 사각형 + 내부 글상자)
— pre-existing (Fix 적용 전/후 동일). Issue #962 등록 (연속 5 PR #964 영역 영역 후속 가능).

## 7. Issue #952 / 분리 결함 추적

원 Issue #952 영역 영역 4 분리 결함 — Issue #952 (Issue 1/2/3) 영역 영역 PR #961 머지 영역 영역
이미 close. Issue #960 (Issue 4) 영역 영역 PR #961 page 2 검증 중 발견된 별도 결함.

| Issue | PR | 상태 |
|-------|-----|------|
| #952 Issue 1 (외곽선) | #956 | ✅ merged |
| #952 Issue 2 (sample16 p18) | #958 (#957) | ✅ merged |
| #952 Issue 3 (시험지 p1 문9) | #961 (#959) | ✅ merged (Issue #952 closed) |
| **#960 (시험지 p2 cases)** | **#963 (본 PR)** | 처리 중 |
| #962 (보기 textbox scramble) | 별도 task | OPEN (pre-existing) |

## 8. 처리 옵션

### 옵션 A (권장) — 본질 commit cherry-pick + orders 충돌 수동 해결 + 자기 검증 + WASM 재빌드

```bash
git checkout local/devel
git cherry-pick 90fb1684   # 본질 commit만 (devel merge commit 2개 제외)
# orders/20260517.md 충돌 수동 해결 (양측 보존)
# cargo test + 광범위 sweep (paragraph_layout TAC line 매핑 변경 → sweep 필수)
# WASM 재빌드
git checkout devel
git merge local/devel --no-ff
```

- devel merge commit (`d690b878`/`8626a8b1`) 영역 영역 cherry-pick 제외 — 본질 commit `90fb1684` 만

### 옵션 B — squash 3 commits (devel merge 포함 영역 영역 비권장, A 정합)

## 9. 검증 게이트

### 9.1 자기 검증
- [ ] cherry-pick `90fb1684` (본질만) + orders 충돌 수동 해결
- [ ] PR #956/#958/#961/#963 양립 확인 (layout.rs 3 + paragraph_layout.rs 1)
- [ ] cargo test --release --lib ALL GREEN (PR 본문 1288 passed)
- [ ] cargo clippy --release -- -D warnings
- [ ] **광범위 sweep 7 fixture / 169 페이지** — TAC line 매핑 변경 영역 영역 회귀 점검 필수
- [ ] LAYOUT_OVERFLOW count 회귀 0 점검
- [ ] WASM 재빌드 (paragraph_layout.rs 변경)

### 9.2 시각 판정 게이트 — **작업지시자 시각 검증 권장**
- 시험지 (3-11월) page 2 문14 cases formula — y=352 line 1 정합 (한컴 PDF `pdf/3-11월_실전_통합_2022.pdf` 권위, header overlap 해소)
- 시험지 4종 page 2 정상
- exam_kor/math/eng, sample10~14 회귀 부재
- PR #956/#958/#961 (page border / sample16 p18 / 시험지 p1 문9) 회귀 부재 (4 정정 양립)
- 잔존: page 2 보기 textbox scramble (Issue #962) — 본 PR 범위 외

## 10. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @jangster77 **24+ 사이클** (연속 5 PR 4번째) |
| `feedback_image_renderer_paths_separate` | paragraph_layout.rs TAC line 매핑 단일 — layout.rs (#956/#958/#961) 와 다른 파일 |
| `feedback_hancom_compat_specific_over_general` | has_line_break + end-position 한정 가드 — 케이스별 명시 (일반화 없음) |
| `feedback_diagnosis_layer_attribution` 권위 사례 강화 | RHWP_DEBUG_PARA_TAC + char_offsets gap 분석 영역 영역 pi=117 position 30 (`\n`) off-by-one 정확 진단 |
| `feedback_pr_supersede_chain` 권위 사례 강화 | Issue #952 (Issue 1/2/3) → #956/#958/#961 + Issue #960 (Issue 4, PR #961 검증 발견) → **#963** + #962 (신규 분리) — 검증 중 결함 발견 연쇄 |
| `reference_authoritative_hancom` | 시험지 한컴 PDF (`pdf/3-11월`) 권위 page 2 cases formula y=352 기준 |

## 11. 처리 순서 (승인 후)

1. `local/devel` 영역 cherry-pick `90fb1684` (본질만, devel merge commit 제외) + orders 충돌 수동 해결
2. 자기 검증 — PR #956/#958/#961/#963 양립 + cargo test + clippy + 광범위 sweep + LAYOUT_OVERFLOW 회귀 0 + WASM 재빌드
3. 작업지시자 시각 검증 (시험지 page 2 cases formula y=352 한컴 PDF 정합 + 회귀 부재)
4. 검증 통과 → no-ff merge + push + archives + 5/17 orders
5. Issue #960 close (Issue #962 잔존 — 별도 task)
6. PR #963 close + 연속 PR #964 진행 (Issue #962 가능성)

---

작성: 2026-05-18
