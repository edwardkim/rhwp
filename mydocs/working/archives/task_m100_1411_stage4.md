# Task 1411 Stage 4 — `2024-09-below20-above20` 잔여 분류

## 목적

Stage 3 이후에도 남은 `2024-09-below20-above20` p19/p20/p22 후보를 분류한다.

공식 미주 모양값(`separatorAbove=20mm`, `betweenNotes=7mm`, `separatorBelow=20mm`) 계산
잔여인지, 이전 쪽의 tail overflow에 따른 cascade인지, 별도 layout 결함인지 구분한다.

## 입력

- baseline: `output/task1411_stage1_baseline/2024-09-below20-above20`
- Stage 3 이후: `output/task1411_stage3_after_fix_v2/2024-09-below20-above20`

## 확인 대상

- p19: 문29 tail overflow와 question marker drift
- p20: p19 이후 cascade인지, 독립적인 문23~문26 marker drift인지 확인
- p22: p19/p20 이후 누적 cascade인지 확인

## note shape 확인

`output/task1411_stage3_after_fix_v2/2024-09-below20-above20/analysis/note_shape.json`
기준 endnote 모양값은 다음과 같이 최신 공식 모델과 일치한다.

| 항목 | 값 |
| --- | ---: |
| `separatorAboveMm` | 19.999 |
| `betweenNotesMm` | 6.999 |
| `separatorBelowMm` | 19.999 |
| `separatorLengthMm` | 49.999 |

따라서 p19/p20/p22 잔여는 공식 미주 모양값 계산 잔여가 아니다.

## 관찰

### p19

문28 marker는 RHWP/PDF가 거의 일치한다.

| 문항 | RHWP | PDF | delta |
| --- | ---: | ---: | ---: |
| 문28 | 442.5 | 442.8 | -0.3 |
| 문29 | 1018.0 | 789.7 | +228.3 |

첫 갈림점은 문28 풀이 내부다. `dump-pages --page 18` 기준 문28(`src=s0:p334:ci0`)에는
큰 TAC shape line이 연속으로 들어간다.

| pi | note | line height | render y/h | 비고 |
| --- | --- | ---: | --- | --- |
| 980 | note5 | 14565 HU | 568.7 / 194.2px | `Shape tac=true` |
| 983 | note8 | 14185 HU | 90.7 / 189.1px | `Shape tac=true`, 다음 단 시작 |
| 994 | note19 | 4704/6897 HU | 777.6 이후 3 line | 문29 직전 큰 equation tail |

문29 tail overflow 후보도 문29 자체의 마지막 풀이(`pi=998..1000`)가 p19 frame bottom을
넘는 것으로 잡힌다. 즉 p19는 미주 간격이 아니라 문28의 큰 TAC shape/equation cluster split
위치가 문29 시작을 늦추고, 문29 tail이 그 결과로 frame 하단을 넘는 흐름이다.

### p20

p20은 p19의 문29 continuation으로 시작하고, 문30과 문23~문26이 연쇄적으로 늦어진다.

| 문항 | RHWP | PDF | delta |
| --- | ---: | ---: | ---: |
| 문30 | 718.1 | 568.6 | +149.5 |
| 문23 | 495.2 | 384.5 | +110.7 |
| 문24 | 665.9 | 528.9 | +137.0 |
| 문25 | 865.9 | 749.3 | +116.6 |
| 문26 | 1073.3 | 982.9 | +90.4 |

문26 tail overflow 후보(`pi=1077..1080`)는 p20 마지막 문항이 이미 낮은 위치에서 시작한 결과다.
`betweenNotes` marker gap 자체는 일부 짝에서 `±26px` 안팎이며, 20mm separator 모양값
계산 오류로 보이지 않는다.

### p22

p22의 문29는 RHWP가 page top에서 시작하고 PDF는 약 57.5px 낮다.

| 문항 | RHWP | PDF | delta |
| --- | ---: | ---: | ---: |
| 문29 | 90.7 | 148.2 | -57.5 |
| 문30 | 232.5 | 253.1 | -20.6 |

이 페이지는 p19/p20의 단순 누적이라기보다, 문29 제목/초반의 TAC shape top 배치가 별도 원인이다.
`dump-pages --page 21` 기준:

- `pi=1129`: 문29 제목 line에 `Shape tac=true` 포함
- `pi=1131`: 큰 `Shape tac=true`, `line_height=23190 HU`
- `pi=1169`: 이후 문30 풀이에 `Table tac=true`, `line_height=18772 HU`

따라서 p22는 large TAC shape/table을 포함한 미주 문항의 page-top anchoring/split 문제로 분류한다.

## 분류 결론

`2024-09-below20-above20` 잔여 3페이지는 공식 미주 모양 모델 잔여가 아니다.

- p19/p20: 문28/문29/문30의 큰 TAC shape 및 tall equation cluster split 차이에 따른 cascade
- p22: 문29 제목 및 초반 large TAC shape의 page-top 배치 차이

이 범위는 #1411의 "공식 미주 모양 모델 잔여 검증" 안에서는 문서화 대상으로 두고, 실제 보정은
별도 작은 작업에서 TAC shape/equation cluster split 정책으로 다루는 편이 안전하다.

## 검증

- `git diff --check`

완료:

- `git diff --check`: 통과
