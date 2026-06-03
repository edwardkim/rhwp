# Task 337 수행계획서: TAC 이미지 후속 문단 위치 조판 오류 수정

## 현상

`samples/hwpspec-w.hwp` 24페이지에서 이미지(pi=134) 다음 캡션 이후의 문단(pi=135)이 이미지보다 **위에** 배치되는 버그.

### 디버그 데이터

```
# dump-pages 결과
FullParagraph  pi=134  h=215.4 (sb=2.7 lines=203.4 sa=9.3)
Shape          pi=134 ci=0  wrap=TopAndBottom tac=true
FullParagraph  pi=135  h=20.0 (sb=6.7 lines=13.3 sa=0.0)

# overlay y-trace
pi=133 y=201.7  (마지막 텍스트 문단)
pi=134 y=471.8  (이미지 문단 — 위치 비정상)
pi=135 y=461.1  (후속 문단 — 이미지보다 위!)
```

### IR 분석 (dump -s 2 -p 134)

- **ParaShape**: ps_id=33, align=Center, spacing_before=400, spacing_after=1400
- **LINE_SEG**: vpos=13560, lh=15255 (이미지 높이 포함), ls=628
- **Shape**: 묶음(Group) 17개 자식, treat_as_char=true, wrap=TopAndBottom
- Shape 크기: 90.7mm × 47.5mm (25704×13455 HU)

## 원인 추정

pi=134는 TAC(treat_as_char=true) Shape 문단으로, LINE_SEG의 `lh=15255`에 이미지 높이가 포함되어 있다. 이 문단은 FullParagraph와 Shape 두 개의 PageItem으로 분리되어 처리된다.

layout_column_item에서 FullParagraph pi=134 처리 시:
1. `has_block_table` 체크에서 Shape는 포함되지 않을 수 있음 → 일반 layout_paragraph로 처리
2. layout_paragraph가 LINE_SEG lh=15255 기반으로 높이를 계산하여 y_offset을 크게 전진
3. 이후 Shape pi=134도 별도로 처리되어 이미지가 렌더링됨
4. pi=135는 y_offset이 이미 전진한 상태에서 배치

그런데 vpos 보정 코드가 pi=134의 LINE_SEG(vpos=13560)를 기반으로 pi=135의 y_offset을 역보정할 수 있음. 이 역보정이 이미지 높이를 고려하지 않아 pi=135가 이미지 위로 올라감.

## 구현 계획

### 1단계: 원인 정밀 분석

- layout_column_item에서 pi=134(FullParagraph + Shape) 처리 흐름 추적
- vpos 보정 코드가 TAC Shape 문단에 대해 어떻게 동작하는지 확인
- pi=134 처리 후 y_offset 값과 pi=135 처리 전 y_offset 값 비교

### 2단계: 수정 구현

- 원인에 따라 vpos 보정 또는 layout_paragraph에서 TAC Shape 높이 처리 수정
- 기존 TAC 표 처리(exam_social 10번 등)와의 호환성 확인

### 3단계: 검증

- hwpspec-w.hwp p24: pi=134→135 간격 정상화 확인
- 기존 테스트 716건 통과 확인
- exam_kor, exam_social 등 TAC 처리가 관련된 샘플 SVG 비교 확인

## 영향 범위

- `src/renderer/layout.rs` — vpos 보정 또는 FullParagraph TAC Shape 처리
- `src/renderer/layout/paragraph_layout.rs` — layout_paragraph의 TAC Shape 높이 반영 (필요 시)
