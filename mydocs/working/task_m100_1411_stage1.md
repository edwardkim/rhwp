# Task 1411 Stage 1 — baseline 잔여 재현

## 목적

PR #1410 merge 이후 `upstream/devel` 기준에서 #1293 최종 잔여 visual sweep 후보가 그대로
재현되는지 확인한다. 이번 단계는 코드 수정이 아니라 기준선 고정이다.

## 시작 기준

- 브랜치: `local/task_m100_1411`
- 기준 커밋: `a2a1b383` (`Merge pull request #1410 from jangster77/task_m100_1293`)
- 기존 근거 산출물: `output/task1293_stage122_rebase_full_sweep`
- 신규 산출물: `output/task1411_stage1_baseline`

## 실행 명령

```bash
cargo build --bin rhwp
python3 scripts/task1274_visual_sweep.py \
  --target 2022-10 \
  --target 2024-09-below20-above20 \
  --target 2024-11-practice-above0-between20-below2 \
  --out output/task1411_stage1_baseline \
  --rhwp-bin target/debug/rhwp
```

## 검증 결과

- `cargo build --bin rhwp`
  - 통과
  - `Finished dev profile [unoptimized + debuginfo] target(s) in 3m 12s`
- targeted sweep
  - 통과
  - summary: `output/task1411_stage1_baseline/summary.json`

| target | SVG/render/PDF | flagged | 주요 flag |
|---|---:|---:|---|
| `2022-10` | 18/18/18 | 1 | p14 `equation_text_overlap`, `line_band_drift`, `large_ink_region_drift` |
| `2024-09-below20-above20` | 23/23/23 | 3 | p19/p20 tail, p19/p20/p22 question/line/large |
| `2024-11-practice-above0-between20-below2` | 22/22/22 | 3 | p17/p20 tail, p17/p21 question, p21 content bottom |

총 `7/63`쪽이 flagged로 재현됐다. 이는 #1293 최종 보고서의 `7/323` 잔여 후보 중
해당 3개 target만 재실행한 결과와 일치한다.

## note shape 확인

| target | 구분선 위 | 미주 사이 | 구분선 아래 | 구분선 |
|---|---:|---:|---:|---|
| `2022-10` | 0.0mm | 6.999mm | 2.032mm | 표시 |
| `2024-09-below20-above20` | 19.999mm | 6.999mm | 19.999mm | 표시 |
| `2024-11-practice-above0-between20-below2` | 0.0mm | 19.999mm | 1.997mm | 표시 |

세 target 모두 공식 미주 모양 값은 #1293 최종 산출물과 같은 의미로 읽힌다.

## 페이지별 관찰

### `2022-10` p14

- question marker drift 후보는 없다.
- tail overflow 후보도 없다.
- 남은 `equation_text_overlap`은 문25 근처의 수식 bbox와 쉼표 텍스트 bbox가 `9px` 교차한 후보다.
- annotated 이미지 기준, 큰 흐름 붕괴보다는 수식/쉼표 bbox와 large ink coarse matching이 함께 잡힌 상태다.
- 다음 단계에서 실제 렌더 결함인지 detector 허용/분류 보강 대상인지 분리한다.

### `2024-09-below20-above20` p19/p20/p22

- p19:
  - 문29 marker가 RHWP `y=1018.0`, PDF `y=789.7`로 `+228.3px` 낮다.
  - 문29 tail 후보는 `pi=998..1000`에서 `11.6px`, `29.6px`, `65.3px` overflow로 잡힌다.
  - annotated 이미지 기준, 문28 이후 RHWP가 오른쪽 단 하단까지 더 많이 밀어 넣고 PDF는 문29가 오른쪽 단 중단에서 시작한다.
- p20:
  - 문23~문26, 문30 marker가 함께 지연된다.
  - 문26 tail 후보는 최대 `72.9px` overflow다.
- p22:
  - 문29 marker가 RHWP에서 `-57.5px` 높다.
  - p19/p20 이후 흐름 차이가 뒤쪽 페이지로 이어진 cascade로 보인다.
- `separatorAbove=20mm`, `separatorBelow=20mm`, `betweenNotes=7mm` 값은 정상 계측된다.

### `2024-11-practice-above0-between20-below2` p17/p20/p21

- p17:
  - 문27 marker가 RHWP `y=729.3`, PDF `y=674.6`으로 `+54.7px` 낮다.
  - 문27 tail 후보는 `5.0px` overflow다.
  - #1293 stage111의 문26 본문 높이 누적 분류와 같은 양상이다.
- p20:
  - 문28 tail 후보는 `39.9px`, `61.0px` overflow다.
  - 이 페이지의 `betweenNotes=20mm` marker gap은 최대 `0.5px` 차이라 미주 사이 계산식 자체는 맞는다.
- p21:
  - 문29 marker가 RHWP `y=750.6`, PDF `y=805.7`로 `-55.1px` 높다.
  - p20 이후 본문 흐름 차이가 다음 쪽 content bottom 차이로 이어진 형태다.

## 판정

- PR #1410 merge 이후에도 #1293 최종 잔여 후보는 동일하게 재현된다.
- Stage 1 범위에서는 공식 미주 모양 값 자체의 파싱/정규화 불일치는 확인되지 않았다.
- `2024-11-practice-above0-between20-below2` p20은 `betweenNotes=20mm` marker gap이 최대 `0.5px`
  차이로 맞아, 미주 사이 계산식보다 문28 본문/그림/수식 tail 문제가 우선이다.
- 다음 단계는 `2022-10` p14의 수식/bbox 후보를 먼저 분리한다.

## 다음 단계

- Stage 2: `2022-10` p14 수식/쉼표 bbox 후보가 실제 렌더 결함인지 detector 후보인지 분리한다.

