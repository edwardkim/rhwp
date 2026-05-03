# PR #531 검토 문서

**PR**: [#531 fix(layout): TAC inline 경로 Top caption 오프셋 누락 정정 (Task #530)](https://github.com/edwardkim/rhwp/pull/531)
**작성자**: @postmelee (Taegyu Lee) — 외부 컨트리뷰터
**Base / Head**: `devel` ← `task530-tac-top-caption`
**Linked Issue**: [#530](https://github.com/edwardkim/rhwp/issues/530) (Refs, not closes)
**상태**: OPEN, MERGEABLE, mergeStateStatus = **BEHIND** (devel 진행으로 뒤처짐 — cherry-pick 시 자동 해소)
**CI**: ALL SUCCESS (Build & Test + CodeQL × 3 + Canvas visual diff; WASM Build SKIPPED)
**작성일**: 2026-05-02
**검토일**: 2026-05-03

---

## 1. 개요

### 1.1 본질

`samples/basic/treatise sample.hwp` 5페이지 우측 단의 TAC (`treat_as_char=true`) 표 (`pi=60 ci=0`, 3x3, `wrap=TopAndBottom`) 의 **Top caption 두 줄이 표 머리행 텍스트와 같은 y 영역에 렌더링되어 겹쳐 보이는 결함** 정정.

### 1.2 근본 원인

`src/renderer/layout/table_layout.rs::layout_table` 의 inline (TAC) 경로:

```rust
// 변경 전
let table_y = if inline_x_override.is_some() {
    y_start  // ← Top caption 의 caption_height + caption_spacing 누락
} else {
    self.compute_table_y_position(/* ... caption_height, caption_spacing 반영 ... */)
};
```

→ `inline_x_override.is_some()` 인 TAC 경로는 외부에서 이미 x/y 를 계산했기 때문에 `y_start` 를 그대로 사용하나, **Top caption 의 높이는 외부에서 반영하지 않아 누락**. 일반 (non-TAC) Top caption 경로의 `caption_height + caption_spacing` 오프셋이 적용 안 됨.

결과: caption 의 y_start 와 표 본문의 y 가 동일 → 머리행 위로 caption 이 겹침.

### 1.3 정정

```rust
// 변경 후
let inline_top_caption_offset = if inline_x_override.is_some() && depth == 0 {
    if let Some(ref caption) = table.caption {
        if matches!(caption.direction, CaptionDirection::Top) {
            caption_height + caption_spacing
        } else {
            0.0
        }
    } else {
        0.0
    }
} else {
    0.0
};

let table_y = if inline_x_override.is_some() {
    y_start + inline_top_caption_offset  // ← Top caption 만 오프셋 추가
} else {
    self.compute_table_y_position(/* 기존 */)
};
```

**핵심 결정 사항:**
- `depth == 0` 조건으로 중첩 표 (cell 내부 표) 회피
- `Top` direction 만 적용 — Bottom/Left/Right caption 경로는 변경 없음
- caption 자체의 y 는 `y_start` 그대로 (caption 위치는 외부에서 이미 정합)

---

## 2. 변경 정합

### 2.1 본질 변경 (PR #531 자체)

| 파일 | 변경 | 비고 |
|------|------|------|
| `src/renderer/layout/table_layout.rs` | +18 / -2 | inline_top_caption_offset 변수 추가 + table_y 보정 |
| `tests/issue_530.rs` (신규) | +99 | 회귀 테스트 1건 (treatise sample.hwp p5 TAC top caption + table_top y 비교) |
| `mydocs/plans/task_m100_530{,_impl}.md` | 신규 (174 + 185) | 수행 / 구현 계획서 |
| `mydocs/working/task_m100_530_stage{1,3,4}.md` | 신규 (175 + 127 + 86) | 단계별 보고서 |
| `mydocs/report/task_m100_530_report.md` | 신규 (110) | 최종 보고서 |
| `mydocs/orders/20260502.md` | +2 / -1 | 컨트리뷰터 일지 |

**소스**: 1 파일 (+18 / -2) — 매우 작은 변경
**테스트**: 1 신규 (page=4, pi=60, ci=0 fixture-based 회귀)

### 2.2 base 차이 (단순 history skew)

PR #531 의 base 가 본 환경의 devel 보다 이전 시점이라 `git diff devel..pr-531-review` 에 PR #507 cherry-pick / Task #516 / 메모리 등 본 PR 본질과 무관한 -3919 lines 차이가 표시됨. cherry-pick 시 본질만 적용되므로 영향 없음.

---

## 3. 검토 항목

### 3.1 코드 품질

- ✅ **변경 본질 정합** — TAC inline 경로의 누락된 caption offset 보정. 다른 결함 도입 위험 매우 작음.
- ✅ **방어적 처리** — `depth == 0` (중첩 표 회피) + `caption.is_some()` + `Top` direction 만 적용. 영향 범위 최소화.
- ✅ **주석** — "Top 캡션은 표 본문 위의 별도 영역이므로 표 본문 y 에 캡션 높이만큼 반영한다" — 본질을 정합하게 설명.
- ✅ **변수 추출** — `inline_top_caption_offset` 변수로 의도 명확화.
- 경미: 변수 정의가 길지만 (10줄), 명확성을 위한 정합한 트레이드오프.

### 3.2 회귀 위험 점검

| 영역 | 영향 |
|------|------|
| TAC + Top caption 표 | 표 본문이 caption 아래로 이동 (정정 의도) |
| TAC + Bottom/Left/Right caption | 변경 없음 (Top 만 분기) |
| TAC + caption 없음 | 변경 없음 (`caption.is_some()` 분기) |
| non-TAC 표 | 변경 없음 (`inline_x_override.is_some()` 분기) |
| 중첩 표 (cell 내부 표) | 변경 없음 (`depth == 0` 분기) |
| partial table caption | `table_partial.rs` 별도 경로 — 변경 없음 |

### 3.3 회귀 테스트 (정합)

`tests/issue_530.rs::issue_530_tac_top_caption_does_not_overlap_header_row`:

- ✅ **fixture 의존**: `samples/basic/treatise sample.hwp` (이미 repo 에 존재)
- ✅ **결정적 검증**: section=0, pi=60, ci=0 표 노드 추출 + caption baseline 추출 + `table.bbox.y > max_caption_baseline + 0.1` 비교
- ✅ **헬퍼 함수**: `collect_issue_530_tables` / `collect_issue_530_caption_baselines` — RenderNode 트리 재귀 정합
- ✅ **에러 메시지**: 결함 시 caption_runs 까지 출력 → 디버깅 용이

본 환경 검증 결과:
- `cargo test --test issue_530` ✅ 1 passed
- `cargo test --test issue_501` ✅ 1 passed (회귀 0)
- `cargo test --test issue_418` ✅ 1 passed (회귀 0)
- `cargo test --test svg_snapshot` ✅ 6/6 passed (table_text_page_0 도 통과 — 표 영역 회귀 0)
- `cargo test --lib` ✅ 1110 passed
- `cargo clippy --lib -- -D warnings` ✅ 0 건
- `cargo clippy --test issue_530 -- -D warnings` ✅ 0 건
- `cargo build --release` ✅ Finished

### 3.4 디버그 도구 활용 (특히 정합)

컨트리뷰터의 Stage 1 진단 보고서가 본 프로젝트의 디버깅 워크플로우 (CLAUDE.md `### 디버깅 워크플로우`) 를 정확히 따른다:

1. `dump-pages -p 4` — 페이지 5 의 표 위치 식별 (`Table pi=60 ci=0 ... wrap=TopAndBottom tac=true vpos=27708`)
2. `dump -s 0 -p 60` — 셀 내부 문단 line segment 점검 (vpos=0 / 1080 정상)
3. `export-svg -p 4 --debug-overlay` — SVG 디버그 오버레이로 좌표 비교 (캡션 baseline y=551.32, 머리행 top y=525.49)

이 절차로 **셀 내부 line segment 의 y advance 가 정상이지만 표 본문의 y 자체가 caption 과 동일** 이라는 본질을 정밀 식별. PR 본문의 좌표 비교 표 (525.49 → 560.83, caption baseline y=551.32 유지) 도 정합.

### 3.5 외부 영역 정합 (PR #506 / #507 / Task #509 / Task #516)

- ✅ **회귀 위험 작음** — 변경은 `src/renderer/layout/table_layout.rs` 내. Task #509 (`paragraph_layout.rs`, PUA) / Task #516 (다층 레이어) / PR #506 (HWP 3.0 파서) / PR #507 (수식 parser) 와 충돌 0.
- ✅ **PR #506 의 Square wrap 정정 영역 (`paragraph_layout.rs`)** 과 무관.
- ✅ **PR #507 의 equation parser 영역 (`equation/parser.rs`)** 과 무관.

### 3.6 PR 본문 비-목표 (별도 이슈 후보)

PR 본문에 별도 비-목표 명시 없음. 본 결함의 본질만 정합하게 정정. 정합.

### 3.7 외부 컨트리뷰터 점검

- ✅ 컨트리뷰터 (`postmelee`) 의 task #530 PR.
- ✅ 내부 워크플로우 정합 — 수행 계획서 / 구현 계획서 / 단계별 보고서 (Stage 1, 3, 4) / 최종 보고서 5종 작성 (`mydocs/...`). 외부 컨트리뷰터로서 절차 준수.
- ✅ **디버그 도구 적극 활용** — `--debug-overlay` + `dump-pages` + `dump` 의 통합 사용으로 시각 결함의 본질을 코드 변경 전에 정합하게 식별.
- ✅ 시각 검증 — 좌표 기반 before/after (525.49 → 560.83) + SVG 스크린샷 첨부.
- 메모리 `feedback_pr_comment_tone` 적용 — 차분하고 사실 중심.

---

## 4. 시각 검증 게이트

### 4.1 fixture 정합

`samples/basic/treatise sample.hwp` 가 이미 repo 에 존재. 추가 fixture 없이 메인테이너 환경에서 직접 시각 판정 가능.

### 4.2 본 환경 시각 자료 준비 완료

```bash
target/release/rhwp export-svg "samples/basic/treatise sample.hwp" -p 4 -o output/svg/pr531/
target/release/rhwp export-svg "samples/basic/treatise sample.hwp" -p 4 --debug-overlay -o output/svg/pr531/debug/
```

| 자료 | 위치 |
|------|------|
| p5 SVG (정정 후) | `output/svg/pr531/treatise sample_005.svg` |
| p5 SVG with debug overlay | `output/svg/pr531/debug/treatise sample_005.svg` |

작업지시자가 한컴 2010/2020 의 동일 페이지와 비교하여 시각 판정.

### 4.3 시각 판정 항목

| 항목 | 결함 (정정 전) | 정정 후 (예상) |
|------|--------------|------------|
| 우측 단의 표 (pi=60 ci=0) | Top caption 두 줄과 머리행 텍스트 겹침 | caption 두 줄 → 머리행 → 본문 (정상 분리) |
| caption baseline y | 551.32 | 551.32 (유지) |
| 머리행 top y | 525.49 (caption 위에 침범) | 560.83 (caption 아래로 이동) |
| Bottom/Left/Right caption 표 | (영향 없음) | (변경 없음) |

---

## 5. 위험 정리

| 위험 | 가능성 | 회피책 |
|------|--------|--------|
| TAC + Top caption 정정으로 다른 fixture 회귀 | 🟢 작음 | svg_snapshot 6/6 (table_text_page_0 포함) + issue_418/501 회귀 0 |
| 중첩 표 (cell 내부 표) 영향 | 🟢 매우 작음 | `depth == 0` 분기로 회피 |
| Bottom/Left/Right caption 영향 | 🟢 매우 작음 | `Top` direction 만 분기 |
| caption 없는 TAC 표 영향 | 🟢 매우 작음 | `caption.is_some()` 분기 |
| Square wrap 표 (PR #506 영역) 영향 | 🟢 매우 작음 | `inline_x_override.is_some()` 만 분기 (Square wrap 은 다른 경로) |
| 시각 판정 (한컴 2010/2020) | 🟧 게이트 | 작업지시자 직접 판정 대기 |
| WASM 영향 | 🟧 중간 | 본 PR 은 native lib 변경. WASM 의 동일 영역 영향 받음 — 머지 전 WASM 빌드 + 작업지시자 시각 판정 2차 (rhwp-studio) 필요 |

---

## 6. 결정

**권장**: ✅ **머지 진행 (cherry-pick)** — 코드/테스트 자체 정합 + 결정적 검증 통과 + 디버그 도구 활용 정합 + fixture 이미 존재.

**근거:**
1. 코드 변경은 매우 작고 본질 정합 (+18 / -2, 단일 함수의 한 분기).
2. 회귀 테스트 1건은 fixture-based + 결정적 좌표 비교.
3. 변경 영역의 모든 분기 (depth=0 / Top direction / caption 존재 / inline_x_override) 가 정합하게 격리됨.
4. svg_snapshot 6/6 + issue_418/501 회귀 0 + cargo test --lib 1110 통과.
5. 디버그 도구 (`dump-pages` + `dump` + `--debug-overlay`) 의 정합한 활용으로 본질 식별.
6. fixture 가 이미 repo 에 존재 → 추가 수정 요청 불필요.

**남은 게이트 (작업지시자):**
1. **시각 판정 1차** (SVG, CLI) — `output/svg/pr531/treatise sample_005.svg` 와 한컴 2010/2020 의 동일 페이지 비교
2. **시각 판정 2차** (rhwp-studio web Canvas) — WASM 재빌드 + studio 동기화 후 동일 fixture 시각 확인 (PR #507 절차 정합)
3. 시각 판정 통과 후:
   - cherry-pick 머지 (3 commits: ac298d4 + 104b4f7 + b3d848b) → `pr_531_report.md` 작성
   - 시각 판정 미통과 시 추가 수정 요청

**머지 시 추가 정합 사항:**
- 이슈 #530 — PR 본문에 `Refs #530` 명시 (closes 아님). 머지 후 작업지시자가 #530 close 결정 (정정 충분 시 close, 다른 결함 잠재 시 open 유지)
- 이슈 #530 milestone 미지정 → v1.0.0 추가 권장
- README 기여자 목록 갱신 (postmelee 의 PR 카운트 누적)
- mergeStateStatus = BEHIND → cherry-pick 시 자동 해소

---

## 7. PR 본문 산출물 점검

PR 본문 보고 산출물:
- 수행 계획서: `mydocs/plans/task_m100_530.md`
- 구현 계획서: `mydocs/plans/task_m100_530_impl.md`
- 단계별 보고서: `mydocs/working/task_m100_530_stage{1,3,4}.md` (3 파일, Stage 2 는 구현 계획서로 통합한 것으로 보임)
- 최종 보고서: `mydocs/report/task_m100_530_report.md`

✅ 외부 컨트리뷰터로서 내부 워크플로우 정합. (Stage 2 가 별도 파일이 아닌 점은 PR #507 도 동일 — 일부 단계가 통합되는 것은 컨트리뷰터 재량 영역).

---

## 8. 메모리 정합

- `feedback_check_open_prs_first` — 본 PR 처리 정합 (이슈 #530 → PR #531 연결 확인)
- `feedback_pr_comment_tone` — close 댓글 차분/사실 중심
- `feedback_release_sync_check` — cherry-pick 시점 main 동기화 점검 필수
- `feedback_v076_regression_origin` — 컨트리뷰터의 좌표 기반 시각 판정은 정합한 보조 자료. 게이트는 작업지시자 직접 판정.
- `feedback_visual_regression_grows` — 시각 판정 게이트 (1차 + 2차) 진행
- `feedback_pdf_not_authoritative` / `reference_authoritative_hancom` — 작업지시자 한컴 2010/2020 직접 판정으로 정답지 확정
- `feedback_hancom_compat_specific_over_general` — 본 PR 의 정정은 case-specific (TAC + Top caption + depth=0 한정). 정합.

---

## 9. 다음 단계

작업지시자 본 검토 문서 승인 후:

1. **시각 판정 1차** (SVG export-svg) — `output/svg/pr531/treatise sample_005.svg` 시각 확인
2. 통과 시 `pr_531_review_impl.md` 작성 (cherry-pick 절차)
3. 작업지시자 승인 후 cherry-pick 머지 + 결정적 검증 + WASM 빌드 + studio 동기화
4. **시각 판정 2차** (rhwp-studio web Canvas) — 머지 전 최종 게이트
5. 시각 판정 2차 통과 후 devel 머지 + push
6. `pr_531_report.md` 작성 + PR close + 이슈 #530 처리 (close 또는 open 유지)
7. README 기여자 목록 갱신
