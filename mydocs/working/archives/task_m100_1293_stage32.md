# Task 1293 Stage 32: 반복 TAC 그림 pagination/render y 차이 분석

## 목적

Stage31에서 `Shape` 항목의 overflow 오탐은 줄였지만,
`2024-11-practice-no-separator-above20-between20-below20` page 14 표시분의 `pi=613`은
실제 TAC 그림 bbox가 body frame 밖으로 나가는 문제가 남았다.

Stage31에서 두 후보를 폐기했다.

- TAC picture-only rewind를 단순 순차 누적으로 바꾸면 page count가 `24/23`으로 증가한다.
- 반복 TAC picture-only tail을 보수적으로 다음 단에 넘겨도 page count가 `24/23`으로 증가한다.

따라서 이번 단계는 넘김 조건을 바로 늘리지 않고, 같은 page count를 유지하면서 renderer y와
pagination current height가 어디서 벌어지는지 수치화한다.

## 분석 대상

- target: `2024-11-practice-no-separator-above20-between20-below20`
- 페이지: page 14 표시분(0-based page 13)
- 문단 묶음:
  - `pi=607`, `pi=609`, `pi=611`, `pi=613` TAC picture-only 반복
  - `pi=607`은 저장 vpos가 직전 제목 쪽으로 되감기지만 renderer는 실제 y를 되감지 않는다.

## 확인할 질문

1. `tac_picture_rewind_height` 축약으로 pagination current height가 얼마나 낮아지는가?
2. renderer `HeightCursor`가 같은 구간에서 되감김을 적용하지 않는 이유는 무엇인가?
3. page count를 유지하려면 `pi=613`을 다음 단으로 넘기는 대신, 같은 단 안에서 그림 묶음의 시작 y를
   한컴/PDF 기준에 맞춰 위로 재정렬해야 하는가?
4. no-separator 문서의 `구분선 위/아래` 값이 실제로는 첫 미주 block top이나 column start에 흡수되어
   이 그림 묶음의 시작 y를 결정하는지 확인한다.

## 검증 계획

- `RHWP_DEBUG_TAC_CURSOR=1 RHWP_VPOS_DEBUG=1 export-render-tree`로 page 13/14 렌더 y를 재확인한다.
- `dump-pages -p 13`의 `used`와 render tree bbox bottom을 비교한다.
- 수정 후보가 나오면 아래 focused target으로 검증한다.
  - `2024-11-practice-no-separator-above20-between20-below20`
  - `2024-11-practice-above0-between20-below2`
  - `2024-11-practice-above20-between0-below20`

## 관찰 결과

Stage31 출력의 compare PNG를 직접 확인했다.

- `output/task1293_stage31_no_separator_target/.../compare/compare_013.png`
- `output/task1293_stage31_no_separator_target/.../compare/compare_014.png`

결론:

- `pi=613`은 실제 그림 bbox overflow이지만, 단독 tail 문제가 아니다.
- rhwp page 13/14는 PDF 기준과 같은 page count를 유지하면서도 단별 내용 순서가 크게 어긋난다.
  - page 13 PDF는 우측 단에 `문20`, `문21` 진입부가 이미 보인다.
  - rhwp page 13은 우측 단에 `문17`~`문19`가 남아 있어 PDF보다 미주 흐름이 늦다.
  - page 14 PDF는 좌측 단 상단부터 문21 그래프 묶음이 이어지지만, rhwp는 이전 풀이 텍스트가
    더 남아 있고 하단에서 `pi=613` 그림이 frame을 침범한다.
- 따라서 `pi=613`을 다음 단으로 넘기는 방식은 증상만 밀어내며 page count 회귀를 만든다.

## 폐기한 방향

Stage31에서 이미 확인한 두 typeset 후보는 stage32 관찰 기준에서도 폐기한다.

1. `tac_picture_rewind_height`를 순차 누적으로 바꾸는 방식
   - page count가 `24/23`으로 증가한다.
2. 반복 TAC picture-only tail을 단 하단에서 보수적으로 넘기는 방식
   - page count가 `24/23`으로 증가한다.

## 다음 단계 방향

Stage33에서는 no-separator 미주 block의 공식 설정 해석을 다시 기준으로 잡는다.

- 구분선이 없을 때 `구분선 위/구분선 아래`가 실제로 separator line 주변이 아니라 미주 block 시작/끝
  또는 단 전환의 reserve/padding으로 소비되는지 확인한다.
- 현 상태는 page count만 맞고 단별 content order가 틀리므로, `overflow_lines`만 줄이는 목표로는
  부족하다.
- 다음 수정은 `pi=613` 개별 넘김이 아니라 no-separator 미주 block의 누적 current height와 column
  start 계산을 대상으로 해야 한다.
