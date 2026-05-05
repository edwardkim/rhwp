# PR #593 처리 보고서 — 핀셋 cherry-pick 머지 + 시각 판정 ★ 통과 (SVG + 웹 캔바스)

**PR**: [#593 fix: Square wrap 표 horz_rel_to=단 속성 정합 (closes #590)](https://github.com/edwardkim/rhwp/pull/593)
**작성자**: @planet6897 (PR 등록) / Jaeook Ryu = @jangster77 (commit author)
**처리 결정**: ✅ **핀셋 cherry-pick 머지 + push + PR close + Issue #590 close**
**처리일**: 2026-05-05

## 1. 처리 결과 요약

| 항목 | 결과 |
|------|------|
| 결정 | ✅ 핀셋 cherry-pick (`5d3b3e2d` 단독) + devel merge + push + PR close |
| 시각 판정 | ★ **통과** (SVG + 웹 캔바스 양쪽) |
| Devel merge commit | `0e543e6` |
| **PR mergeable** | **MERGEABLE** (PR #592 와 함께 본 사이클 두 번째 케이스) |
| Cherry-pick 충돌 | 0 건 |
| Author 보존 | ✅ Jaeook Ryu (@jangster77) 보존 |
| Issue #590 | CLOSED (수동 close + 안내 댓글) |
| 광범위 페이지네이션 sweep | 164 fixture / 1,614 페이지 / 페이지 수 회귀 0 |

## 2. 본질 결함 (PR 진단)

### 2.1 결함 가설

`src/renderer/layout.rs:2285-2300` (Issue #480 도입 분기) 가 모든 Square-wrap 표를 무조건 문단 좌측 가장자리(`col_area.x + effective_margin`) 기준으로 강제 배치하며 `horz_rel_to` 속성을 무시.

### 2.2 정량 측정 + 수식 검증

문단 2 ParaShape `margin_left=1700, indent=+2000` → effective_margin = 24.67 px → 우측 24.7 px (=6.5mm) 시프트:

```
table_x = col_area.x + effective_margin + h_offset
        = 117.17    + 24.67            + 9.44
        = 151.28 px ← SVG 실측치와 정확히 일치
```

→ 사용자 보고 "오른쪽으로 6.5mm 치우침" 의 본질 = effective_margin (24.67 px = 6.5 mm) 강제 적용.

## 3. 본질 정정 — 분기 가드 1줄 추가

```diff
- } else if !is_tac && tbl_is_square {
+ } else if !is_tac && tbl_is_square
+     && matches!(t.common.horz_rel_to, crate::model::shape::HorzRelTo::Para) {
      // [Issue #480 / #590] horz_rel_to=Para 인 Square wrap 표만 paragraph 영역
      // (col_area + margin) 기준으로 정렬. horz_rel_to=Column/Page/Paper 는
      // compute_table_x_position 의 기본 분기에서 명세대로 처리한다.
```

**핵심 가드 정합성:**
- `horz_rel_to=Para` 한정 → Issue #480 분기는 Para 기준 표만 처리
- `Column/Page/Paper` → `compute_table_x_position` 명세 기반 분기로 위임 (HWP 표준)
- 케이스별 명시 가드 (`feedback_hancom_compat_specific_over_general` + `feedback_rule_not_heuristic` 정합)

### 3.1 적용 영역 / 미적용 영역

**적용 (위치 변경):**
- Square wrap (`wrap=어울림`) 표
- `treat_as_char=false`
- `horz_rel_to ∈ {Column, Page, Paper}`

**미적용 (이전 동작 유지):**
- TAC 표 (`treat_as_char=true`)
- `horz_rel_to=Para` Square wrap 표 (#480 분기 동작 유지)
- 글뒤로 / 글앞으로 wrap

## 4. PR 의 1 commit 분석 (특이 사례)

| commit | 설명 | 본 환경 처리 |
|--------|------|-------------|
| **`5d3b3e2d` fix + 보고서 + orders 갱신** | `layout.rs` +5/-4 + plans/working/report + orders | ⭐ cherry-pick |

**특이 사항**: 이전 PR 들 (PR #561~#592) 은 본질 commit + plans/working/report/orders 를 **별도 commit 으로 분리**했지만, 본 PR 은 모든 것이 1 commit. orders 충돌 우려 있었으나 본 환경 임시 cherry-pick test 결과 auto-merge 정합.

## 5. cherry-pick 진행

### 5.1 대상 commit (1개, 충돌 0)

```
3682cff fix: Square wrap 표 horz_rel_to=단 속성 정합 (closes #590)
```

`Jaeook Ryu <jaeook.ryu@gmail.com>` author 보존.

### 5.2 변경 영역

| 파일 | 변경 |
|------|------|
| `src/renderer/layout.rs` | +5 / -4 (분기 가드 1줄 + 주석 갱신) |
| `mydocs/orders/20260504.md` | +1 (PR #593 항목) |
| `mydocs/plans/task_m100_590.md` 외 3 working/report | +525 (단계별 보고서) |

## 6. 결정적 검증 결과

| 게이트 | 결과 |
|--------|------|
| `cargo build --release` | ✅ Finished |
| `cargo test --lib --release` | ✅ **1134 passed** / 0 failed / 2 ignored (회귀 0) |
| `cargo test --test svg_snapshot` | ✅ 6/6 passed |
| `cargo test --test issue_546` | ✅ 1 passed (Task #546 회귀 0) |
| `cargo test --test issue_554` | ✅ 12 passed |
| `cargo clippy --release --lib` | ✅ 0건 |
| Docker WASM 빌드 | ✅ **4,581,465 bytes** (1m 33s, PR #592 baseline -62 bytes — 가드 1줄 추가의 LLVM 최적화 효과) |

## 7. 광범위 페이지네이션 회귀 sweep

본 환경 `samples/` 폴더 전체 자동 sweep:

| 통계 | 결과 |
|------|------|
| 총 fixture | **164** (158 hwp + 6 hwpx) |
| 총 페이지 (devel baseline) | **1,614** |
| 총 페이지 (cherry-pick 후) | **1,614** |
| **fixture 별 페이지 수 차이** | **0** |

→ Square wrap 표 horz_rel_to 정정이 페이지네이션에 영향 없음.

## 8. exam_kor 4 페이지 의도된 정정 (PR 본문 100% 재현)

| 페이지 | halign | 변화 |
|---|---|---|
| **page 17 [A]** | Left | 151.28 → **126.61** (-24.7 px, **사용자 보고 정정**) |
| page 18 [B] x2 | Left | 동류 정정 |
| page 19 [B] | Left | 동류 정정 |
| page 14 [A] x4 | Right | 515.60 → 508.05 (-7.55 px, **명세 정합 향상**) |

### 8.1 page 14 (halign=Right) 명세 정합 향상의 의미

- 이전 동작: `inline_x_override` 경로가 Right 정렬 시에도 h_offset 을 ADD 하던 모순 (Right 는 SUBTRACT 해야 함)
- `compute_table_x_position` 의 명세 기반 분기로 통일
- → **명세 정합 향상** (회귀 아님)

### 8.2 page 17 [A] 글자 정량 검증 (시각 판정용)

| 글자 | Before (devel) | After (cherry-pick) | 변화 |
|---|---|---|---|
| 셀 내부 `[` | translate(**152.28**, 579.39) | translate(**127.61**, 579.39) | **-24.67 px** |
| 셀 내부 `A` | translate(**157.50**, 579.39) | translate(**132.83**, 579.39) | **-24.67 px** |
| 다른 위치 (x=622) `[A]` | 변경 없음 | 변경 없음 | 0 (회귀 0) |

→ PR 본문 effective_margin (24.67 px = 6.5 mm) 과 **정확히 일치** 정량 검증 완료. 다른 영역 [A] 글자는 변경 없음 — 케이스별 명시 가드 정합 입증.

## 9. 시각 판정 (★ 게이트)

### 9.1 SVG 자료 + WASM 환경

- `output/svg/pr593_before/exam_kor/` (devel 기준, 20 페이지)
- `output/svg/pr593_after/exam_kor/` (cherry-pick 후, 20 페이지)
- WASM: `pkg/rhwp_bg.wasm` 4,581,465 bytes (다양한 hwp 직접 검증용)

### 9.2 작업지시자 시각 판정 결과

> 웹 캔바스 시각 판정 통과 입니다. svg 도 17 페이지 내보내기 해보세요. (page 17 SVG 추출 검증 후) svg, 웹 모두 시각 판정 통과입니다.

→ ★ **통과** (SVG + 웹 캔바스 양쪽). PR #584 와 달리 Canvas/SVG 경로 차이 의심 영역 없이 양쪽 일관 정합.

## 10. PR / Issue close 처리

### 10.1 PR #593 close
- 댓글 등록 (cherry-pick 결과 + 결정적 검증 + 광범위 sweep + page 17 [A] 정량 검증 + 본 PR 의 본질 (분기 가드 1줄 + 명세 위임) + 명세 정합 향상 + 컨트리뷰터 협업 인정)
- close 처리

### 10.2 Issue #590 수동 close
- closes #590 키워드는 PR merge 가 아닌 close 로 자동 처리 안 됨 (PR #564/#570/#575/#580/#584/#592 와 동일 패턴)
- 수동 close + 안내 댓글 (cherry-pick 처리 완료 + page 17 [A] 정량 검증)

## 11. 메모리 정합

- ✅ `feedback_visual_regression_grows` — 시각 판정 게이트 정합 운영 (★ 통과, SVG + Canvas 양쪽)
- ✅ `feedback_v076_regression_origin` — 수식 검증 (table_x = 117.17 + 24.67 + 9.44 = 151.28) 으로 결함 origin 정량 식별
- ✅ `feedback_hancom_compat_specific_over_general` — 케이스별 명시 가드 (horz_rel_to=Para 한정)
- ✅ `feedback_rule_not_heuristic` — `compute_table_x_position` 명세 기반 분기로 위임
- ⚠️ `feedback_pdf_not_authoritative` — hancomdocs PDF 비교는 참고 + 작업지시자 시각 판정으로 보정 완료
- ✅ `feedback_per_task_pr_branch` — Task #590 단일 본질 PR 정합
- ✅ `feedback_pr_comment_tone` — close 댓글 차분/사실 중심 + 컨트리뷰터 협업 인정
- ✅ `feedback_check_open_prs_first` — PR OPEN 상태 확인 후 진행
- ✅ `feedback_small_batch_release_strategy` — 본 사이클 (5/1 ~ 5/5) 활발한 외부 기여의 빠른 회전 (20번째 PR 처리)

## 12. 본 PR 의 본질 — 단일 줄 분기 가드 + MERGEABLE + 명세 정합 향상

본 PR 의 처리 본질에서 가장 우수한 점:

1. **단일 줄 분기 가드** — `layout.rs` 의 단 한 줄 추가 (`&& matches!(t.common.horz_rel_to, HorzRelTo::Para)`) 로 결함 정정
2. **MERGEABLE 표시** — PR #592 와 함께 본 사이클 두 번째 케이스 (PR base 가 본 devel 과 정합한 시점)
3. **HWP 표준 룰 위임** — `compute_table_x_position` 명세 기반 분기로 위임 (휴리스틱 아닌 규칙)
4. **명세 정합 향상** — page 14 (halign=Right) 의 `inline_x_override` ADD/SUBTRACT 모순 통일 (회귀 아님, 부수 효과로 명세 정합도 향상)
5. **단일 commit 에 본질 + 보고서 + orders 통합** — 이전 PR 들과 다른 새 패턴 (auto-merge 정합)
6. **SVG + Canvas 양쪽 시각 판정 통과** — PR #584 의 Canvas/SVG 경로 차이 의심 영역 없이 양쪽 일관 정합

## 13. 본 사이클 사후 처리

- [x] PR #593 close (cherry-pick 머지 + push)
- [x] Issue #590 수동 close (안내 댓글)
- [x] 처리 보고서 (`mydocs/pr/archives/pr_593_report.md`, 본 문서)
- [ ] 검토 보고서 archives 이동 (`mydocs/pr/pr_593_review.md` → `mydocs/pr/archives/pr_593_review.md`)
- [ ] 5/5 orders 갱신 (PR #593 항목 추가)
