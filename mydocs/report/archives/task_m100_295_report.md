# Task #295 최종 보고서 — exam_math.hwp 12쪽 좌단 레이아웃 붕괴 수정

## 개요

`samples/exam_math.hwp` 12쪽 다단 레이아웃의 좌측 컬럼 콘텐츠가 페이지 하단으로 밀려 압축·겹침 발생하던 문제를 해소.

## 원인 (확정)

`src/renderer/layout.rs::layout_table_item`의 `renders_above_body` 분기는 **머리말 자리(본문 위)** 의 vert=Paper 앵커 TopAndBottom 표만 본문 흐름에서 분리(out-of-flow)했음.

12쪽 pi=22 ctrl[0] 푸터 표는 `vert=Page valign=Bottom`(페이지 하단 앵커)이라 두 가지로 분기에서 누락:
1. `vert` 종류가 Paper만 처리 → Page 누락
2. 위치 조건이 `tbl_y < body.y`(본문 위)만 → 본문 아래 누락

결과: 푸터 표 처리 시 `y_offset`이 푸터 위치(≈y=1371)로 점프 → 후속 좌단 본문(pi=23..27) 모두 본문 영역 하단/이하에 배치 → LAYOUT_OVERFLOW 18건.

## 수정 (`src/renderer/layout.rs`)

| 위치 | 변경 |
|------|------|
| `renders_outside_body` (≈1995) | vert: Paper → Paper \| Page; 조건: `tbl_y < body.y \|\| tbl_y + tbl_h > body_bottom` |
| `is_outside_body` (≈2103) | 동일 확장 (표 아래 spacing 추가 가드) |
| `tbl_inline_x` Square 분기 (≈1978) | 좌측 강제 → halign(Left/Right/Center/Outside)에 따라 배치 |
| 어울림 호출 가드 제거 (≈2086) | wrap_around_paras 비어 있어도 자가 wrap host 처리 호출 |
| `layout_wrap_around_paras` 호스트 텍스트 (≈2509) | 첫 줄만 → 전체 줄 렌더링 |

## 검증

| 항목 | 수정 전 | 수정 후 |
|------|---------|---------|
| 12쪽 LAYOUT_OVERFLOW | 18건 | **0건** |
| pi=23 (29번 본문) y | 1340.1 ❌ | 178.7 ✅ |
| pi=27 표 머리행 | 누락 | 표시 |
| pi=27 호스트 본문 5줄 | 첫 줄만 | 5줄 모두 |
| pi=27 표 위치 | 좌측 강제 | halign=Right 반영 |
| `cargo test --release` | - | 1028 passed, 0 failed |
| 주요 샘플 회귀 | - | LAYOUT_OVERFLOW 0건 |

회귀 점검: `exam_math.hwp` 1쪽(머리말 vert=Page 표) 정상 유지, `exam_math_no.hwp`/`equation-lim.hwp`/`text-align-2.hwp` 전체 LAYOUT_OVERFLOW 0건.

## 파일

- 수정: `src/renderer/layout.rs`
- 보고서: `mydocs/working/task_m100_295_stage{1,3,4}.md`, 본 문서

## 잔여 별건 (분리)

- 머리말 페이지번호 4↔2 불일치 (1단계 보고서에서 분리 권고)
- 12쪽 우측 컬럼 단락 높이 과대 → **이슈 #297**로 신규 등록 (사전 존재 버그, #295 무관)
