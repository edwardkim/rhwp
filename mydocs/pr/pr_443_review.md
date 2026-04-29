# PR #443 검토 — Task #439 Square wrap 표 직후 col 0 over-fill 정정 (exam_kor 22→20)

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#443](https://github.com/edwardkim/rhwp/pull/443) |
| 작성자 | [@planet6897](https://github.com/planet6897) (Jaeuk Ryu) — 신뢰 컨트리뷰터, 본 사이클 9번째 PR |
| 이슈 | [#439](https://github.com/edwardkim/rhwp/issues/439) (closes), PR #442 의 잔여 작업 1번 |
| base / head | `devel` ← `planet6897:local/task439` |
| 변경 규모 | +2,261 / -17, 13 files (11 commits → 본질 4 commits) |
| mergeable | `MERGEABLE` (BEHIND) |
| 검토 일자 | 2026-04-29 |

## 11 commits 누적 정황

| 영역 | commits | 처리 |
|------|---------|------|
| Task #435 (4) | `e0cdc2e`, `0879f11`, `bd08d6b`, `0a096fc` | PR #442 로 흡수 완료 (방금 머지) |
| merge commits (3) | `2da9c8d`, `a6416bf`, `50b02f3` | skip |
| **Task #439 (4)** | `3884305`, `4ea2746`, `99f1596`, `df2aff3` | **본질 변경 — 분리 cherry-pick** |

## 본질

`samples/exam_kor.hwp` 페이지 14 col 0 의 4 개 Square wrap (어울림) 표 (pi=33, 37, 40, 47, 모두 wrap=어울림 vert=문단 1.4mm) 처리 시 **호스트 텍스트 + 표 별도 누적** 하던 버그 정정.

### 원인

호스트 문단 텍스트가 `PartialParagraph` 로 push 되며 `current_height` 누적 + 이어서 표가 push 되며 `table_total_height` 도 추가 누적 → 두 누적이 **합산** 되지만, HWP layout 에서 어울림 표는 **본문이 표 옆을 흐름** (같은 수직 영역 공유).

페이지 14 col 0 의 4 표 누적 측정:

| pi | pre_h (host) | table_total | 합산 (현재) | max (정답) | 차이 |
|---:|-------------:|------------:|-----------:|-----------:|-----:|
| 33 |        98.03 |       84.93 |     182.96 |      98.03 | -84.93 |
| 37 |        73.52 |       60.75 |     134.27 |      73.52 | -60.75 |
| 40 |        49.01 |       38.45 |      87.46 |      49.01 | -38.45 |
| 47 |        73.52 |       60.75 |     134.27 |      73.52 | -60.75 |
| **합** | 294.08 | 244.88 | **538.96** | **294.08** | **-244.88** |

→ col 0 used = 1225.8 px = 정상 누적 + **244.88 px 과다 누적** = 1211.3 본문 한계 +14.5px 초과 → pi=48 fit 실패 → col 1 advance → col 1 64.3 px (pi=48,49 만) → pi=50+ 페이지 15 로 강제 이동.

### 정정 (32 라인)

```diff
+let is_wrap_around_table = !table.common.treat_as_char
+    && matches!(table.common.text_wrap, crate::model::shape::TextWrap::Square);

-if pre_table_end_line > 0 && is_first_table {
-    let pre_height: f64 = fmt.line_advances_sum(0..pre_table_end_line);
+let pre_height: f64 = if pre_table_end_line > 0 && is_first_table {
+    let h = fmt.line_advances_sum(0..pre_table_end_line);
     st.current_items.push(PageItem::PartialParagraph { ... });
-    st.current_height += pre_height;
-}
+    h
+} else { 0.0 };

 st.current_items.push(PageItem::Table { ... });
-st.current_height += table_total_height;
+
+// [Task #439] 누적 정책:
+// - Square wrap (어울림): max(pre_height, v_off + table_total)
+// - 그 외 (TopAndBottom 등): pre_height + table_total 합산 (기존 동작)
+if is_wrap_around_table && pre_height > 0.0 {
+    let v_off_px = crate::renderer::hwpunit_to_px(vertical_offset as i32, self.dpi);
+    let table_bottom = v_off_px + table_total_height;
+    st.current_height += pre_height.max(table_bottom);
+} else {
+    st.current_height += pre_height + table_total_height;
+}
```

PageItem 자체는 `PartialParagraph` + `Table` 모두 push 유지 (layout 렌더링 보존).

### 부수 발견 — 활성 엔진 정확 식별

이슈 본문 추정 (`engine.rs::Paginator` 의 prev_is_table 분기) 이 **활성 엔진 아님** — `RHWP_USE_PAGINATOR=1` 일 때만 활성. 기본은 `typeset.rs::TypesetEngine`. 작성자가 활성 엔진 정확히 식별 후 typeset.rs 정정.

### 결과 메트릭

| 메트릭 | Before | After |
|---|---|---|
| exam_kor.hwp 페이지 수 | 22 | **20** |
| 페이지 14 col 0 used | 1225.8 px (+14.5 over) | **1036.1 px** (under) |
| 페이지 14 col 1 items | 2 | **18** |
| 페이지 14 col 1 used | 64.3 px | **1016.9 px** |
| 페이지 14 + 15 통합 | 분리 | **통합** |

## 처리 방향

**옵션 A — Task #439 본질 4 commits 분리 cherry-pick** (PR #442 와 같은 패턴).

## dry-run cherry-pick 결과

`local/pr443` 브랜치 (`local/devel` 분기) 에서 4 commits cherry-pick — 작성자 attribution 보존:

| commit (cherry-pick) | 작성자 | 내용 |
|---------------------|--------|------|
| `93348d9` (← `3884305`) | @planet6897 | Stage 1: 베이스라인 측정 + 진단 (활성 엔진 확인) |
| `eeca209` (← `4ea2746`) | @planet6897 | Stage 2: 원인 확정 + 구현 계획서 |
| `f37f616` (← `99f1596`) | @planet6897 | Stage 3: Square wrap 표 누적 정책 max 적용 (32 lines) |
| `033781c` (← `df2aff3`) | @planet6897 | Stage 4: 최종 결과보고서 + 오늘할일 갱신 |

cherry-pick 결과:
- Stage 1, 2, 3 자동 적용
- Stage 4 의 `mydocs/orders/20260429.md` add/add 충돌 → HEAD 유지 후 `--continue`

## 검증 (cherry-pick 후)

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | ✅ **1066 passed** (회귀 0) |
| `cargo test --test svg_snapshot` | ✅ 6/6 passed |
| `cargo test --test issue_418` | ✅ 1/1 (Task #418 보존) |
| `cargo clippy --lib -- -D warnings` | ✅ warning 0건 |
| WASM 빌드 (Docker) | ✅ 1m 45s, 4,178,523 bytes |

## 광범위 byte 단위 비교

10 샘플 / 307 페이지 SVG 비교 (devel ↔ PR #443):

| 결과 | 카운트 |
|------|------|
| byte 동일 | **298 / 307** |
| 차이 발생 | **9 / 307** |

차이 분포:
- **exam_kor 9 페이지** (페이지 14+15 통합 + 후속 페이지 재조판)
- **다른 9 샘플 회귀 0건** ✅

작성자 본인 검증 (149 개 sample HWP 전수 페이지 수): exam_kor 만 22→20 변경, 나머지 148 동일.

전체 SVG 수: 307 → **305** (exam_kor 22→20).

## 시각 판정

**한컴 정답지 (한컴 2010 / 2020 / 한컴독스 + hwp 원본) vs rhwp 출력** — 작업지시자 직접 판정.

| 경로 | 시각 판정 결과 |
|------|--------------|
| SVG 내보내기 | ✅ **통과** |
| Canvas (rhwp-studio 웹 에디터, WASM 4,178,523 bytes) | ✅ **통과** |

작업지시자 코멘트:
> "시각적 판정은 svg, canvas 모두 통과입니다. 점점 개선이 되어가고 있는게 보입니다. 좋은 공략 전략입니다."

→ 정정의 누적 개선 흐름 평가. @planet6897 의 잔여 작업 분리 (#439/#440/#441) + 즉시 처리 전략 검증.

## 본 PR 의 좋은 점

1. **부수 발견 정확** — 이슈 본문 추정 (engine.rs::Paginator) 의 함정을 정확히 식별. 활성 엔진 (typeset.rs::TypesetEngine) 에 정정
2. **정량 측정** — 페이지 14 col 0 의 4 표 누적 244.88 px 과다 정확 산출 (pi 별 수치 모두 명시)
3. **메트릭 정합** — col 0 1225.8 → 1036.1 px (-189.7), col 1 64.3 → 1016.9 px (+952.6) — 두 단으로 균형 분배
4. **149 개 샘플 회귀 검증** — 작성자 본인 광범위 점검 (메인테이너 10 샘플 byte 비교와 정합)
5. **PR #442 연속 처리** — 22→20 정합의 첫 번째 잔여 작업 즉시 처리 (메인테이너 머지 직후)
6. **케이스별 명시 가드** — `is_wrap_around_table` 명시 + Square wrap 만 max 정책 적용 (TopAndBottom 등 기존 동작 보존). 메모리 [feedback_hancom_compat_specific_over_general](https://github.com/edwardkim/rhwp/blob/main/mydocs/manual/memory/feedback_hancom_compat_specific_over_general.md) 부합

## v0.7.8 + 후속 사이클 누적

@planet6897 본 사이클 PR 머지 누적 (가나다순):
- PR [#401 v2](https://github.com/edwardkim/rhwp/pull/401) Task #398
- PR [#406](https://github.com/edwardkim/rhwp/pull/406) Task #402
- PR [#408](https://github.com/edwardkim/rhwp/pull/408) Task #404
- PR [#410](https://github.com/edwardkim/rhwp/pull/410) Task #409
- PR [#415](https://github.com/edwardkim/rhwp/pull/415) Task #352
- PR [#424](https://github.com/edwardkim/rhwp/pull/424) Task #412
- PR [#434](https://github.com/edwardkim/rhwp/pull/434) Task #430
- PR [#442](https://github.com/edwardkim/rhwp/pull/442) Task #435, #393
- **PR [#443](https://github.com/edwardkim/rhwp/pull/443) Task #439 (본 PR)**

총 **9 PR** — 본 사이클 가장 활발한 컨트리뷰터.

## 메모리 원칙 부합

| 메모리 | 부합 여부 |
|--------|----------|
| 외부 PR 머지 전 검증 게이트 | ✅ cargo test 1066 + svg_snapshot 6/6 + clippy 0 + WASM |
| 시각 판정 게이트 (push 전 필수) | ✅ 작업지시자 SVG + Canvas 양 경로 통과 |
| 한컴 호환은 일반화보다 케이스별 명시 가드 | ✅ `is_wrap_around_table` 명시 + Square wrap 만 max 정책 |
| 작은 단위 PATCH 회전 | ✅ 22→20 정합 작업을 #439 단독으로 분리 처리 |
| 보고서는 타스크 브랜치에서 커밋 | ✅ `local/pr443` 에서 커밋 |
| 이슈 close 시 commit devel 머지 검증 | ✅ 머지 + push 후 close |

## 다음 단계

1. 본 보고서 + 오늘할일 갱신 commit
2. `local/pr443` → `local/devel` → `devel` 머지 + push
3. PR #443 close + 작성자 댓글 (이슈 #439 자동 close)

## 참고

- PR: [#443](https://github.com/edwardkim/rhwp/pull/443)
- 이슈: [#439](https://github.com/edwardkim/rhwp/issues/439)
- 이전 PR: [#442](https://github.com/edwardkim/rhwp/pull/442) (Task #435, exam_kor 24→22)
- 잔여 작업: [#440](https://github.com/edwardkim/rhwp/issues/440), [#441](https://github.com/edwardkim/rhwp/issues/441) (다음 PR 후보)
