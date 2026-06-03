# Task 390: TAC 표 다음 문단 vpos 간격 과대 수정

## 현상

- **파일**: samples/synam-001.hwp 8페이지
- **문단**: s0:pi=82 ci=0 (TAC 표 19x4) → s0:pi=83 (텍스트)
- **증상**: pi=82 표 하단(≈920px)에서 pi=83(963.5px)까지 **43.5px 과대** 간격
- **정상**: vpos 기반 gap = 0 HU → 표 직후에 문단 배치

## 원인 분석

마지막 TAC의 `line_end` 보정(Task 375/386)에서:
```rust
line_end = para_y + (seg.vpos + seg.lh) / 7200 * 96
```
- `seg.lh = 61313` HU (LINE_SEG에 기록된 높이)
- 표 실제 높이 = `61031` HU (common.size.height)
- **차이 = 282 HU = 3.8px** → line_end가 표 실제 하단보다 높음
- `line_end > y_offset` → y_offset이 과대하게 설정
- 후속 lazy_base 역산에서 추가 drift 발생

## 수정 방향

마지막 TAC의 `line_end` 보정에 **상한 제한** 추가:
- `line_end`를 `layout_table 반환값(표 실제 하단) + line_spacing`으로 clamp
- 표 실제 높이와 LINE_SEG lh의 차이가 과대 간격을 만들지 않도록

## 구현 계획 (3단계)

### 1단계: line_end clamp 구현
- `layout_table` 반환값을 `table_y_end`로 저장
- `line_end = line_end.min(table_y_end + ls_px)` 상한 적용

### 2단계: 검증
- synam-001.hwp p8: pi=82→pi=83 간격 정상화 확인
- kps-ai.hwp p19: TAC 연속 표 간격 정상 유지 확인
- bodo-01/02: 기존 수정 유지 확인
- cargo test 전체 통과

### 3단계: 커밋 + merge
