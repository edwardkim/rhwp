# Task 1411 Stage 5 — `2024-11-practice-above0-between20-below2` 잔여 분류

## 목적

Stage 3 이후에도 남은 `2024-11-practice-above0-between20-below2` p17/p20/p21 후보를 분류한다.

공식 미주 모양값(`separatorAbove=0mm`, `betweenNotes=20mm`, `separatorBelow=2mm`) 계산
잔여인지, 큰 그림/수식/table tail에 따른 layout 잔여인지 구분한다.

## 입력

- baseline: `output/task1411_stage1_baseline/2024-11-practice-above0-between20-below2`
- Stage 3 이후: `output/task1411_stage3_after_fix_v2/2024-11-practice-above0-between20-below2`

## 확인 대상

- p17: 문27 marker drift 및 tail 후보
- p20: 문28 tail overflow 후보
- p21: p20 이후 continuation/cascade인지 확인

## note shape 확인

`output/task1411_stage3_after_fix_v2/2024-11-practice-above0-between20-below2/analysis/note_shape.json`
기준 endnote 모양값은 다음과 같다.

| 항목 | 값 |
| --- | ---: |
| `separatorAboveMm` | 0.0 |
| `betweenNotesMm` | 19.999 |
| `separatorBelowMm` | 1.997 |
| `separatorLengthMm` | 49.999 |

최신 공식 미주 모양 모델과 일치한다.

## 관찰

### p17

문26은 PDF와 거의 일치하지만 문27만 약 55px 늦게 시작한다.

| 문항 | RHWP | PDF | delta |
| --- | ---: | ---: | ---: |
| 문26 | 408.0 | 408.2 | -0.2 |
| 문27 | 729.3 | 674.6 | +54.7 |

`betweenNotes` marker gap에서도 p26→p27 구간만 `+55px` 차이가 난다. 직전 문항 문26의
마지막 풀이 `pi=786`은 5개 line segment를 가진 tall equation cluster이며 마지막 segment의
gap이 `5669 HU`다. 문27 자체도 시작 직후 `pi=788`에 `Shape tac=true`,
`line_height=10963 HU`의 큰 그림을 가진다. p17의 tail 후보는 문27 마지막 `pi=792`가 frame
bottom을 약 `5px` 넘는 작은 bleed다.

따라서 p17은 20mm 미주 사이 값 계산 오류가 아니라, 문26 tall equation cluster 뒤 문27 제목
배치와 문27 large TAC shape/tail 처리 잔여로 분류한다.

### p20

p20의 `betweenNotes` marker gap은 PDF와 거의 완전히 맞는다.

| gap pair | RHWP | PDF | delta |
| --- | ---: | ---: | ---: |
| 1 | 143.5 | 144.0 | -0.5 |
| 2 | 188.5 | 188.0 | +0.5 |
| 3 | 340.5 | 341.0 | -0.5 |
| 4 | 511.0 | 511.0 | 0.0 |

잔여 후보는 문28 tail이다.

- `pi=936`: `즉, [EQ] [EQ]①`, frame bottom overflow `39.9px`
- `pi=937`: `아래 그림의 사각형 [EQ]에서`, frame bottom overflow `61.0px`

같은 문항은 다음 페이지 p21에서 `pi=939` `Shape tac=true`, `line_height=12201 HU`의 큰
그림으로 이어진다. p20은 미주 모양값이 아니라 문28 large TAC shape continuation/tail split
잔여다.

### p21

p21은 문29만 약 55px 위에 놓이고 문30은 PDF와 거의 일치한다.

| 문항 | RHWP | PDF | delta |
| --- | ---: | ---: | ---: |
| 문29 | 750.6 | 805.7 | -55.1 |
| 문30 | 955.6 | 956.0 | -0.4 |

p21 앞부분은 p20 문28 continuation이다.

- `pi=939`: `Shape tac=true`, `line_height=12201 HU`
- `pi=962`: 문29 내부 `Shape tac=true`, `line_height=17563 HU`
- `pi=964`: 문29 내부 `Shape tac=true`, `line_height=11353 HU`

문29의 위치 차이는 p20에서 넘어온 문28 large shape continuation의 높이/분할 차이에 따른
page-local cascade로 보는 것이 자연스럽다. 문30이 다시 맞는 점도 전역 미주 모양값 문제가
아님을 뒷받침한다.

## 분류 결론

`2024-11-practice-above0-between20-below2` 잔여 3페이지는 공식 미주 모양 모델 잔여가 아니다.

- p17: 문26 tall equation cluster 뒤 문27 제목 gap 및 문27 large TAC shape/tail 잔여
- p20: 문28 large TAC shape continuation/tail overflow
- p21: p20 문28 continuation 이후 문29 시작 위치 cascade, 문30은 재정합

실제 보정은 20mm `betweenNotes` 산식이 아니라 TAC shape/equation/table continuation split
정책으로 분리하는 편이 안전하다.

## 검증

- `git diff --check`

완료:

- `git diff --check`: 통과
