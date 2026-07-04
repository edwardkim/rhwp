# Task 1293 Stage 62: 0mm 미주 lazy base와 그림 y 기준 분석

## 목적

Stage61에서 문13 제목을 강제로 현재 단에 남기는 실험은 폐기했다. 제목 fit 조건만 완화하면
pagination은 현재 단에 남지만 renderer에서 frame 밖으로 내려가고, bottom-fit으로 당기면 직전
수식 tail과 겹친다.

이번 단계는 `문13` 제목 자체가 아니라 직전 `문12` 후반부의 기준 y가 PDF/Hancom보다 낮아지는
공통 원인을 추적한다.

## 대상

- 샘플: `samples/3-11월_실전_통합_2024-구분선위0미주사이0구분선아래0.hwp`
- 기준 PDF: `pdf/3-11월_실전_통합_2024-구분선위0미주사이0구분선아래0.pdf`
- 비교 page: display 11쪽, 내부 page index 10
- 우선 문단:
  - `pi=514..516`: page-base에서 lazy-base로 전환되는 구간
  - `pi=537`: 비TAC 그림/그래프 문단
  - `pi=538`: 문12 tail
  - `pi=539`: 문13 제목

## 분석 질문

1. `pi=516`에서 lazy base가 만들어질 때 anchor paragraph/control이 무엇인지 확인한다.
2. `pi=537` 그림의 renderer bbox가 PDF보다 약 30px 낮게 시작하는 이유가 저장 vpos, paragraph
   line advance, TopAndBottom shape block 중 어디에서 오는지 분리한다.
3. `layout_shape_item`의 TopAndBottom 그림 host line advance 보정이 0/0/0 미주 하단에서
   한컴보다 큰 공백을 만드는지 확인한다.
4. 수정은 문항 번호나 특정 `pi`가 아니라 다음 공통 조건으로만 허용한다.
   - 미주 영역
   - 구분선 위/미주 사이/구분선 아래가 모두 0인 compact 설정
   - 비TAC 그림 또는 저장 vpos 되감김이 문항 tail 앞에 있는 경우

## 검증 계획

- `cargo build --bin rhwp`
- 기준 로그:
  - `RHWP_VPOS_DEBUG=1 target/debug/rhwp export-svg ...`
  - `target/debug/rhwp dump-pages ... -p 10`
  - `target/debug/rhwp export-render-tree ...`
- target sweep:
  - `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-above0-between0-below0 --out output/task1293_stage62_zero_lazy_base --rhwp-bin target/debug/rhwp`
- focused regression:
  - `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
  - `cargo test --test issue_1050_footnote_serialize -- --nocapture`
  - `cargo test --lib compact_endnote`

## 분석 결과

### 기준 재현

Stage61 HEAD 기준 target sweep을 다시 실행했다.

```sh
python3 scripts/task1274_visual_sweep.py \
  --target 2024-11-practice-above0-between0-below0 \
  --out output/task1293_stage62_zero_lazy_base_sweep \
  --rhwp-bin target/debug/rhwp
```

결과:

- page count: SVG/PDF/render tree `21/21/21`
- renderer `LAYOUT_OVERFLOW`: 0
- hard gate: frame/title/order/equation 모두 0
- qflow 후보: `[11, 12, 13, 17, 19, 20]`
- display 11쪽은 PDF/Hancom과 달리 `pi=539` 문13 제목이 우측 단 상단으로 넘어간다.

`dump-pages -p 10` 기준 page 11 좌측 단은 다음 순서다.

| 순서 | 항목 | 설명 |
|---:|---|---|
| 24 | `pi=536` | 문12 tail 직전 텍스트 |
| 25 | `pi=537` FullParagraph | 빈 host 문단 |
| 26 | `pi=537` Shape | 비TAC 그림, `vpos=207548` |
| 27 | `pi=538` FullParagraph | `따라서` + 수식 tail |
| 28 | `pi=539` FullParagraph | 문13 제목 |

`RHWP_DEBUG_TAC_CURSOR=1` 기준 주요 진행량은 다음과 같다.

```text
FullPara pi=536 y_in=878.7 y_out=896.7 dy=18.0
FullPara pi=537 y_in=896.7 y_out=896.7 dy=0.0
Shape pi=537 ci=0 y_in=896.7 y_out=1053.9 dy=157.2
FullPara pi=538 y_in=1053.9 y_out=1095.8 dy=41.9
FullPara pi=539 y_in=1089.8 y_out=1107.8 dy=18.0
```

따라서 문13 제목만 문제가 아니라, `Shape pi=537`의 선언 bbox 하단과 `pi=538` line advance가
합쳐져 문13 제목이 단 하단 밖으로 밀린다.

### 폐기한 실험

1. 제목 tail guard 완화
   - `zero_question_title_tail_fits_by_line_height`에서 첫 단 `line_advance` fit 요구를 제거했다.
   - 결과: pagination은 `pi=539`를 좌측 단에 남기지만 renderer에서 frame 하단에 걸린다.
   - 판단: Stage61의 폐기 결론과 동일하게, 제목만 현재 단에 남기는 방향은 원인 해결이 아니다.

2. 0/0/0 미주에서 빈 TopAndBottom host line advance 생략
   - `layout.rs`의 Task #683 보정을 0/0/0 미주 host에서 생략하는 실험을 했다.
   - 결과: `pi=538`은 약 18px 올라가지만 `pi=539`는 여전히 frame 하단 밖에 걸린다.
   - 판단: #683 한 줄 보정만으로는 부족하다.

3. 0/0/0 미주에서 vpos rewind reset 억제
   - 같은 단 안에서 lazy base를 더 오래 보존하는 실험을 했다.
   - 결과: display 11쪽은 일부 올라가지만 display 14쪽에서 하단 outside pixel이 크게 증가했다.
   - 판단: vpos reset 억제는 공통 로직으로 쓰기에는 너무 넓다.

4. 직전 content bottom으로 다음 문항 제목 y를 당기는 실험
   - `last_item_content_bottom`으로 `pi=539`를 좌측 단 하단에 맞추면 display 11쪽은 PDF에 가장
     가까워진다.
   - 그러나 display 14쪽 `pi=657` 수식-only 문단 뒤 `pi=658` 문26 제목, display 19쪽
     `pi=954` 수식-only 문단 뒤 `pi=955` 문29 제목이 같은 y로 겹친다.
   - 판단: content bottom 보정은 수식-only 문단에는 적용할 수 없다.

5. content bottom 보정을 수식-only가 아닌 문단으로 좁힌 실험
   - 수식-only 제목 겹침은 사라졌지만, display 11쪽 문13 제목은 다시 PDF보다 낮게 붙는다.
   - 결과 sweep:

```sh
python3 scripts/task1274_visual_sweep.py \
  --target 2024-11-practice-above0-between0-below0 \
  --out output/task1293_stage62_zero_refined \
  --rhwp-bin target/debug/rhwp
```

```text
analysis: 2024-11-practice-above0-between0-below0
flagged=18/21 frame=[] red=[10, 11, 12, 13, 14, 16, 17, 19, 20, 21]
qflow=[11, 12, 13, 17, 19, 20]
line=[9, 11, 13, 14, 17, 20]
title=[] order=[] large=[9, 10, 11, 12, 13, 14, 16, 17, 19, 20, 21]
```

render tree:

- `pi=538`: `y=1053.9`, `h=35.9`
- `pi=539`: `y=1089.8`, `h=12.0`

판단: 겹침 회귀는 줄였지만 문13 제목은 여전히 한컴/PDF보다 낮고, 실제 정합 해결이 아니다.

## 결론

Stage62의 모든 코드 실험은 폐기한다. 현재 소스는 Stage61 HEAD 상태로 되돌렸다.

이번 단계에서 확정한 원인은 다음이다.

1. `pi=516` lazy base 전환 자체는 page 11 후반을 낮추는 배경이지만, 단독으로 조정하면 다른 page의
   vpos 흐름을 흔든다.
2. `pi=537` 비TAC Shape는 render tree bbox상 `y=898.0`, `h=155.9`로 선언 bbox 하단까지 진행한다.
3. PDF/Hancom 흐름은 `pi=537`의 선언 bbox 하단이 아니라 실제 보이는 그림/host 흐름 하단에 더
   가깝게 `pi=538`을 붙이는 것으로 보인다.
4. 다음 stage에서는 `Shape/Picture`의 선언 bbox, 실제 image ink bbox, host paragraph line box,
   `last_item_content_bottom`을 분리해 renderer가 “다음 flow y”로 삼아야 할 값을 찾아야 한다.

## 다음 단계

Stage63에서는 `pi=537` Shape의 실제 image ink bbox를 산출한다.

- render tree bbox: `Image y=898.0 h=155.9`
- compare/PDF PNG의 실제 graph ink bbox를 같은 좌표계로 검출한다.
- `layout_column_item`의 Shape result_y가 `common.height`/host line advance/실제 image content 중
  어느 값을 쓰는지 계측한다.
- 수정이 필요하면 0/0/0 미주 profile과 비TAC TopAndBottom 객체의 flow advance 기준으로 제한한다.
