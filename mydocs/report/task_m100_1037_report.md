# Task #1037 최종 결과보고서 — HWP5 변환본 paragraph height 정합 + Dialog 한컴 정합

**Issue**: [#1037 HWP5 변환본 paragraph height 과대 측정](https://github.com/edwardkim/regression-rhwp/issues/1037)
**Branch**: `local/task1037` (rebased onto `local/task1035` / PR #1036)
**Status**: 완료, PR 준비
**Milestone**: M100 (v1.0.0)

---

## 1. 작업 배경

HWP5 변환본 (한컴 변환기로 HWP3 → HWP5 변환) 에서 paragraph height 가 한컴 정답 대비 **2배 (17.0 pt vs 8.5 pt)** 측정되는 문제 보고. 추가 시각 검증 결과 paragraph 모양 dialog (왼쪽/오른쪽 여백, 들여쓰기, 문단 위/아래) 가 HWP3/HWP5 변환본 모두 한컴 정답과 불일치.

본질적으로 **HWP5 변환본 의 raw ParaShape 값이 2× scaled** (한컴 변환기 quirk) + **dialog 표시 공식의 unit semantic 미정합** 두 가지 문제.

---

## 2. 작업 흐름 — Stage 1~4

### Stage 1 — 진단 (코드 변경 없음)

- HWP5 변환본 의 일부 paragraph (예: p23 영역, "세계3대물..." paragraph) 에 PARA_LINE_SEG 가 누락되어 합성 (synthesis) 처리되는 영역 식별
- 진단 보고서: [mydocs/working/task_m100_1037_stage1.md](mydocs/working/task_m100_1037_stage1.md)

### Stage 2 — ParaShape unit semantic normalize (parser + style_resolver)

**Hypothesis**: HWP5 변환본 의 raw ParaShape 값은 HWP3 의 2× scaled (한컴 변환기). 일관성 회복 위해 parser 단계 normalize.

**변경**:
1. [src/parser/mod.rs:228 직후](src/parser/mod.rs#L228) — `doc.is_hwp3_variant = true` 직후 ParaShape raw 값 halve (margin_left/right, indent, spacing_before/after)
2. [src/renderer/style_resolver.rs:745](src/renderer/style_resolver.rs#L745) — `variant_div` 분기 제거 (종전 4 vs 2), uniform `variant_div = 2.0`

**효과**:
- HWP5 변환본 dialog 문단 위: 17.0 → **8.6 pt** (한컴 8.5 정합) ✓
- rendering 무변동 (math: `raw × px/HU / divider` 동일값 유지)
- 보고서: [mydocs/working/task_m100_1037_stage2.md](mydocs/working/task_m100_1037_stage2.md)

### Stage 3 — Dialog margin/indent 한컴 정합 fix

**진단**:
- Stage 2 의 variant_div=2 가 dialog 표시값까지 절반으로 만들어 한컴 dialog 와 불일치
- margin/indent 의 unit semantic: HWP3 raw `margin_left` = continuation 라인 위치, HWP5 변환본 (Stage 2 normalize 후) raw `margin_left` = first-line 위치 (의미 다름)

**변경**:
- [src/document_core/commands/formatting.rs:681-735](src/document_core/commands/formatting.rs#L681) `build_para_properties_json`: margin/indent JSON 출력 산식 변경
  - `raw_ps` 직접 사용 (variant_div 미적용)
  - HWP3 native: 왼쪽 = `(margin_left + min(0, indent)) / 100 pt` (effective first-line)
  - HWP5 변환본 (`is_hwp3_variant=true`): 왼쪽 = `margin_left / 100 pt` (raw 직접)
  - 오른쪽/indent: 모두 raw / 100 pt (모든 format 통일)

**효과**:
- HWP3/HWP5 변환본 모두 dialog 한컴 정합 ✓
- rendering 무변경 (parser/renderer 영향 없음)
- 보고서: [mydocs/working/task_m100_1037_stage3.md](mydocs/working/task_m100_1037_stage3.md)

### Stage 4 — D 옵션 정량 평가 (negative result, 코드 변경 없음)

작업지시자 D 선택 (page break + line_seg 합성 모두 시도) 후 정량 평가:

| 시도 | 결과 |
|------|------|
| page break vpos==0 휴리스틱 | Recall 31.6%, FP 4개 — HWP3 [쪽나누기] 57개 vs HWP5 변환본 vpos==0 22개, 공통 18개. **부적합 폐기** |
| p23 overflow line_seg 합성 옵션 B' | Task #1010 Stage 2 회귀 (88 페이지 +24 over-split) + cross-correlation root cause 무관 입증. **시도 가치 낮음 폐기** |

정량 데이터로 D 두 가지 모두 본 task scope 내 안전한 fix 불가능 단언. 별도 task 분리 결정.

보고서: [mydocs/working/task_m100_1037_stage4.md](mydocs/working/task_m100_1037_stage4.md)

---

## 3. 종합 정량 결과 (sample16 p452 "계약상대자가 공급..." 기준)

| 필드 | 한컴 정답 | Task #1037 이전 | **Task #1037 완료** |
|------|---------|---------------|---------------------|
| dialog 왼쪽 여백 | 40.0 pt | HWP3 30 / HWP5 20 | **40 / 40** ✓ |
| dialog 오른쪽 여백 | 10.0 pt | 5 / 5 | **10 / 10** ✓ |
| dialog 내어쓰기 | 20.0 pt | 10 / 10 | **20 / 20** ✓ |
| dialog 문단 위 | 8.5 pt | HWP3 8.6 / HWP5 **17.0** (2×) | **8.6 / 8.6** ✓ |
| 페이지 수 | 64 | HWP3 64 / HWP5 64 | **64 / 64** ✓ |
| alignment (PR #1036) | — | 60/64 | **60/64** 유지 ✓ |

### 한컴 한글 시각 검증 (작업지시자 PC, 21페이지)

작업지시자가 한컴 한글 + rhwp-studio + 두 fixture (HWP3 + HWP5 변환본) 의 dialog 를 직접 비교:
- 한컴: 왼쪽 40, 오른쪽 10, 내어쓰기 20, 문단 위 8.5, 줄 간격 160%
- rhwp HWP3: 동일 ✓
- rhwp HWP5 변환본: 동일 ✓

---

## 4. 자동 검증

| 항목 | 결과 |
|------|------|
| `cargo build --release` | ✓ warning 0 |
| `cargo clippy --release --lib -- -D warnings` | ✓ clean |
| `cargo fmt --all -- --check` | ✓ clean |
| `cargo test --release --lib` | ✓ 1308 passed |
| `cargo test --release --tests` | ✓ FAILED 0 |

### 회귀 sweep (모든 fixture 페이지 수 무변동)
- 변환본 9 종 (sample/4/5/10/11/13/14/16/19-hwp5 + .hwpx): 무변동 ✓
- HWP3 native + 일반 (exam_*, aift, biz_plan): 무변동 ✓

---

## 5. 본 task scope 외 잔존 (별도 이슈 등록 권고)

### 5.1 HWP5 변환본 page break 정보 손실

작업지시자가 Stage 4 시각 검증 중 발견: HWP5 변환본 p440 "4. 서버통합 및 원격지 재해복구센터 시스템 구성요건" 헤더가 19페이지 끝에 잘려 들어감 (HWP3 native 에선 20페이지 시작에 정상).

**진단**:
- 한컴 변환기가 HWP3 → HWP5 변환 시 **모든 page break 정보 완전 손실**
- HWP5 변환본 전체 통계: `page_break_before` paragraph count = 0, `column_type=Page` count = 0, `control_mask` 모두 0
- HWP3 native [쪽나누기] 가 헤더에만 있는 게 아니라 일반 본문/이미지 paragraph 에도 광범위 분포 → text pattern / head_type 휴리스틱 복원 시 false negative 다수
- 안전한 복원 방법 추가 조사 필요

**별도 이슈 후보 제목**: "HWP3 → HWP5 변환본의 page_break_before 정보 손실로 paragraph 페이지 위치 정렬 미정합"

### 5.2 HWP5 변환본 p23 외곽선 overflow

Stage 2 보고서에서 잔존 항목으로 명시. paragraph height 측정 정확도 (Stage 1 의 PARA_LINE_SEG 합성 영역) 와 연관.

**별도 이슈 후보 제목**: "HWP5 변환본 p23 외곽선 overflow — paragraph height 측정 합성 정확도 영역"

---

## 6. 변경 파일 (3 개)

| 파일 | 변경 |
|------|------|
| [src/parser/mod.rs](src/parser/mod.rs) | is_hwp3_variant 시 ParaShape raw 값 halve (Stage 2) |
| [src/renderer/style_resolver.rs](src/renderer/style_resolver.rs) | variant_div uniform 2.0 (Stage 2) |
| [src/document_core/commands/formatting.rs](src/document_core/commands/formatting.rs) | build_para_properties_json margin/indent 공식 raw 직접 사용 (Stage 3) |

---

## 7. PR 본문 초안

**제목**: Task #1037: HWP5 변환본 ParaShape unit normalize + Dialog 한컴 정합 fix

**Summary**:
- HWP3 → HWP5 변환본 의 raw ParaShape 값이 2× scaled 인 한컴 변환기 quirk 를 parser 단계에서 normalize
- style_resolver variant_div 통일 (4→2), rendering 결과 byte-equivalent
- 문단모양 dialog margin/indent 표시 산식을 raw_ps 직접 사용 + HWP3/HWP5 변환본 의 unit semantic 차이 분기

**효과**:
- HWP5 변환본 dialog 문단 위 17.0 → 8.6 pt (한컴 8.5 정합)
- HWP3/HWP5 변환본 dialog 왼쪽/오른쪽/내어쓰기 한컴 정답 100% 정합 (40/10/20)
- rendering 64 페이지, alignment 60/64, 회귀 0

**Closes**: #1037
