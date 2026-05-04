# Task #574 최종 결과 보고서

**제목**: exam_science.hwp 페이지 쪽번호 색·굵기가 진함 (HY견명조 heavy display 오분류 정정)
**브랜치**: `local/task574`
**이슈**: https://github.com/edwardkim/rhwp/issues/574
**Milestone**: M100 (v1.0.0)
**기간**: 2026-05-04 (Stage 0 → Stage 5)

---

## 1. 본질

`is_heavy_display_face` (`src/renderer/style_resolver.rs:601`) 의 hardcoded list 에
"HY견명조" 가 잘못 포함되어 CharShape.bold=false 무시 → SVG 에 `font-weight="bold"`
강제 적용.

이슈 본문 가설 (바탕쪽 / 회색) 일부 정정:
- 쪽번호 "1" 출처 = 바탕쪽이 아닌 **본문 [6] 표 셀 paragraph[0] 의 Shape (사각형,
  InFrontOfText) TextBox** 내부 literal text "1"
- IR 색상은 #000000 (검정) — "회색" 가설 잘못. 본질은 **굵기만**

## 2. 단계 요약

| Stage | 산출물 | 결과 |
|-------|-------|------|
| Stage 0 | `examples/inspect_574.rs` + `working/task_m100_574_stage0.md` | 본질 확정 — `is_heavy_display_face` HY견명조 오분류 |
| Stage 1 | `plans/task_m100_574_impl.md` | 구현 계획 (단일 줄 수정 + TDD + 7개 샘플 sweep) |
| Stage 2 | `tests.rs:938` 단위 테스트 갱신 + `integration_tests.rs:797` 통합 테스트 추가 | RED 확인 |
| Stage 3 | `style_resolver.rs:610` `\| "HY견명조"` 제거 | RED → GREEN |
| Stage 4 | `working/task_m100_574_stage4.md` | 7개 샘플 sweep + 1120 lib tests + clippy 검증 |
| Stage 5 | 본 보고서 | 한컴 PDF 시각 판정 대기 |

## 3. 핵심 변경 (단일 줄)

**`src/renderer/style_resolver.rs:608-612`**:

```diff
 matches!(primary,
     "HY헤드라인M" | "HYHeadLine M" | "HYHeadLine Medium"
-    | "HY견고딕" | "HY견명조" | "HY견명조B"
+    | "HY견고딕" | "HY견명조B"
     | "HY그래픽" | "HY그래픽M"
 )
```

doc 주석에 Task #574 변경 사유 명기.

**보존**:
- `"HY헤드라인M"`, `"HYHeadLine M"`, `"HYHeadLine Medium"`: Task #146 v4 본질 케이스
- `"HY견고딕"`: Heading 전용 굵은 고딕
- `"HY견명조B"`: 명시 Bold variant (B 접미)
- `"HY그래픽"`, `"HY그래픽M"`: 그래픽 굵은 face

**제거**:
- `"HY견명조"`: 한컴 일반 두께 명조 — heavy display 가 아님

## 4. 회귀 검증 결과

### 4.1 7개 샘플 SVG sweep

| 샘플 | 변경 페이지 | 변경 본질 |
|------|-----------|----------|
| exam_science.hwp | 4/4 | HY견명조 텍스트 font-weight 해제만 |
| exam_kor.hwp | 20/20 | 동일 |
| exam_eng.hwp | 8/8 | 동일 |
| exam_math.hwp | 20/20 | 동일 |
| 복학원서.hwp | 1/1 | 동일 |
| synam-001.hwp | 0/35 | HY견명조 미사용 |
| text-align.hwp | 0/1 | **Task #146 v4 base — HY헤드라인M 보존** ✓ |

**변경 라인 분석**:
- 모든 변경 라인의 100% 가 HY견명조 사용 텍스트
- HY견명조外 폰트 회귀 0건
- `font-weight="bold"` 제거 후 diff = 0 → **순수 font-weight 변경만**

### 4.2 단위/통합 테스트

```
$ cargo test --release --lib
test result: ok. 1120 passed; 0 failed; 1 ignored
```

추가/갱신:
- `test_is_heavy_display_face_matches_known_heavy_faces` (HY견명조 단언 위치 이동)
- `test_574_page_number_not_force_bold_for_hy_kyun_myeongjo` (신규 통합 테스트)

### 4.3 clippy

Task #574 변경 파일 (`style_resolver.rs`, `tests.rs`, `integration_tests.rs`) 한정
신규 경고 0건.

## 5. 시각 검증 (Stage 5 작업지시자 판정 게이트)

### 5.1 fix 후 SVG (페이지 1 쪽번호 "1")

```xml
<text transform="translate(924.36,114.87) scale(0.9000,1)"
      font-family="HY견명조,..." font-size="44"
      fill="#000000">1</text>
```

→ `font-weight` 속성 미적용. CharShape cs_id=0 (`bold=false`) IR 권위 회복.

### 5.2 한컴 PDF 비교 (작업지시자 판정 필요)

`samples/exam_science.pdf` 페이지 1 우상단 쪽번호 "1" 의 굵기 시각 비교:
- fix 후 rhwp SVG: `font-weight` 미적용 (HY견명조 regular weight, browser fallback
  에 따라 Batang/바탕 regular)
- 한컴 PDF: 작업지시자 시각 판정 필요

→ **작업지시자 판정 대기**.

## 6. 산출물 목록

### 6.1 코드 변경

| 파일 | 변경 |
|------|------|
| `src/renderer/style_resolver.rs:610` | `"HY견명조"` 제거 + Task #574 doc 주석 |
| `src/renderer/layout/tests.rs:938` | `test_is_heavy_display_face_matches_known_heavy_faces` 갱신 |
| `src/renderer/layout/integration_tests.rs:797` | `test_574_page_number_not_force_bold_for_hy_kyun_myeongjo` 신규 |
| `examples/inspect_574.rs` | 진단 스크립트 (Stage 0 — 보존) |

### 6.2 문서

| 파일 | 내용 |
|------|------|
| `mydocs/plans/task_m100_574.md` | 수행 계획서 |
| `mydocs/plans/task_m100_574_impl.md` | 구현 계획서 |
| `mydocs/working/task_m100_574_stage0.md` | 본질 확정 보고서 |
| `mydocs/working/task_m100_574_stage2.md` | TDD RED 확인 보고서 |
| `mydocs/working/task_m100_574_stage3.md` | Fix 적용 RED→GREEN 보고서 |
| `mydocs/working/task_m100_574_stage4.md` | 회귀 sweep + 테스트 + clippy |
| `mydocs/report/task_m100_574_report.md` | 본 최종 보고서 |

## 7. 메모리 룰 준수

- **[feedback_essential_fix_regression_risk]**: 본질 미확정 시 fix 금지. Stage 0 진단으로
  본질 (heavy display 오분류) 확정 후 Stage 3 단일 줄 수정.
- **[feedback_visual_regression_grows]**: 7개 샘플 sweep + cargo test 1120 + clippy.
  HY견명조外 폰트 회귀 0건 정량 증명.
- **[feedback_pdf_not_authoritative]**: 한컴 PDF 보조 ref. 작업지시자 시각 판정 게이트
  (Stage 5).
- **[feedback_rule_not_heuristic]**: 화이트리스트 단일 룰 — heavy display 의미 face 만
  포함. HY견명조 (일반 두께) 제거. HY견명조B (명시 Bold variant) 보존.

## 8. 커밋 이력 (`local/task574`)

```
8437d03 Task #574 Stage 4: 광범위 회귀 sweep + 전체 테스트 + clippy
8926611 Task #574 Stage 3: is_heavy_display_face HY견명조 제거 (RED→GREEN)
bdc5c22 Task #574 Stage 2: TDD 통합 테스트 + 단위 테스트 갱신 (RED 확인)
c6ad464 Task #574 Stage 1: 구현 계획서 (HY견명조 heavy display 오분류 정정)
c6688ee Task #574 Stage 0: 정밀 진단 + 본질 확정 보고서 (코드 무수정/진단 스크립트만)
4906d88 Task #574 Stage 0: 수행 계획서 (페이지 쪽번호 색·굵기 정정)
```

## 9. 결정 요청 (작업지시자)

1. **한컴 PDF 시각 판정**: `samples/exam_science.pdf` 페이지 1 쪽번호 "1" 굵기 vs
   fix 후 SVG (`/tmp/sweep574/after/exam_science/exam_science_001.svg`).
2. **승인 시 절차**:
   - 본 보고서 + 오늘할일 갱신 커밋
   - `local/task574` → `local/devel` merge
   - `local/devel` → `devel` merge + push
   - `gh issue close 574` (또는 closing 커밋 메시지에 `closes #574`)
3. **이슈 본문 가설 정정 사항** (closing 시 코멘트 권고):
   - 출처는 바탕쪽이 아닌 **본문 표 셀 Shape (사각형) TextBox**
   - 색상은 IR/PDF 모두 검정 — 본질은 **굵기만**
   - 본질은 `is_heavy_display_face` 의 HY견명조 오분류

---

본 보고서는 작업지시자 시각 판정 + 승인 대기 상태입니다.
