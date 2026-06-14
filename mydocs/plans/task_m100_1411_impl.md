# Task 1411 구현 계획서

## 원칙

이번 작업의 첫 목적은 잔여 visual sweep 후보의 분류 확정이다. 새 수치 보정부터 넣지 않는다.
공식 미주 모양 계산식 불일치가 재확인될 때만 코드 수정을 검토하고, tail/cascade 또는 비교 지표
후보라면 근거를 문서화한 뒤 작은 후속 단위로 분리한다.

소스 수정은 작업지시자 승인 후에만 진행한다.

## 1단계: 최신 기준 baseline 재현

- `local/task_m100_1411`이 `upstream/devel` `a2a1b383`에서 분기됐는지 확인한다.
- 기존 산출물 `output/task1293_stage122_rebase_full_sweep`의 3개 잔여 target을 요약한다.
- 필요한 경우 아래 targeted sweep을 실행해 최신 작업 브랜치 기준 산출물을 새로 만든다.

```bash
cargo build --bin rhwp
python3 scripts/task1274_visual_sweep.py \
  --target 2022-10 \
  --target 2024-09-below20-above20 \
  --target 2024-11-practice-above0-between20-below2 \
  --out output/task1411_stage1_baseline \
  --rhwp-bin target/debug/rhwp
```

산출물은 단계 보고서 `mydocs/working/task_m100_1411_stage1.md`에 기록한다.

## 2단계: `2022-10` p14 분류

- `equation_text_overlap_candidates`의 9px 쉼표/수식 bbox 교차를 확인한다.
- `line_band_drift`, `large_ink_region_drift`가 실제 쪽 흐름 차이를 뜻하는지, coarse matching noise인지 확인한다.
- `dump-pages`와 render tree에서 문항/문단 위치가 PDF 흐름과 크게 갈라지는지 확인한다.
- 결론은 다음 중 하나로 분류한다.
  - 실제 코드 수정 대상
  - sweep detector 허용/분류 보강 대상
  - 문서화만으로 충분한 잔여 후보

## 3단계: `2024-09-below20-above20` p19/p20/p22 분류

- p19의 첫 갈림점인 문28 이후 문29 지연을 render tree와 question flow에서 확인한다.
- p20/p22가 p19 이후 cascade인지, 별도 separator/between-notes 간격 문제인지 분리한다.
- `separatorAbove=20mm`, `separatorBelow=20mm`, `betweenNotes=7mm` 값이 최신 note shape dump에서도 유지되는지 확인한다.
- 만약 그림/수식 continuation 높이 문제로 확정되면, #1411 안에서 직접 수정할지 별도 후속 이슈로 분리할지 판단한다.

## 4단계: `2024-11-practice-above0-between20-below2` p17/p20/p21 분류

- p17 문26 본문 높이 누적과 문27 marker drift를 확인한다.
- p20 문28 tail overflow와 `betweenNotes=20mm` marker gap이 독립적으로 맞는지 확인한다.
- p21이 p20 이후 본문 흐름 차이인지 확인한다.
- 공식 미주 모양 계산식이 아니라 본문/그림/수식 tail 문제라면 수정 범위를 별도 작은 작업으로 분리한다.

## 5단계: 처리 방향 결정

- 세 target의 결과를 표로 정리한다.
- 코드 수정이 필요한 경우에는 새 stage 문서를 먼저 만들고 작업 목적/판단/검증 대기를 기록한 뒤, 작업지시자에게 소스 수정 승인을 요청한다.
- 코드 수정 없이 분류로 충분하면 #1411 최종 보고서에서 잔여 후보와 후속 이슈 후보를 명확히 적는다.
- 별도 이슈 생성이나 GitHub 코멘트가 필요하면, 등록 전 초안을 먼저 작업지시자에게 보여주고 승인받는다.

## 검증 계획

문서/분류만 수행한 경우:

```bash
git diff --check
```

스크립트나 소스를 수정한 경우:

```bash
cargo fmt --check
cargo build --bin rhwp
python3 scripts/task1274_visual_sweep.py \
  --target 2022-10 \
  --target 2024-09-below20-above20 \
  --target 2024-11-practice-above0-between20-below2 \
  --out output/task1411_after_fix \
  --rhwp-bin target/debug/rhwp
git diff --check
```

PR 준비 단계까지 진행하는 경우, macOS 로컬 필수 검증은 별도 승인 후 다음 순서로 수행한다.

```bash
cargo build --release
cargo test --release --lib
cargo test --profile release-test --tests
cargo fmt --check
```

WASM 영향이 있으면 다음 명령을 사용한다.

```bash
wasm-pack build --target web --out-dir pkg
```

## 승인 대기

이 구현 계획은 아직 승인 전이다. 승인 전에는 소스 수정, GitHub 코멘트 등록, 이슈 close를 수행하지 않는다.

