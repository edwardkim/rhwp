# Task 1293 Stage 40: shape987 page17 compact 미주 overflow 보정

## 배경

Stage39 이후 focused sweep에서 `2024-11-practice-shape987`
(`3-11월_실전_통합_2024-구분선위9미주사이8구분선아래7.hwp`)에
renderer overflow가 남아 있었다.

초기 후보는 page 11 `pi=571`과 page 16/17 `pi=818~820`이었다. 실제 시각
확인과 `RHWP_VPOS_DEBUG=1 export-render-tree -p 16` 비교 결과, page 17 하단의
문29 tail이 frame 아래로 렌더링되는 문제가 우선 수정 대상이었다.

## 원인

문27의 `pi=800`은 저장 vpos가 본문 흐름보다 되감기는 internal rewind 미주
문단이다. 이 문단이 page 16 첫 단 하단에서 page 17 둘째 단으로 넘어갈 때
pagination은 하단 기준의 축약 높이(`en_fit`)를 재사용했다.

하지만 renderer는 새 단에서 저장 vpos와 미주 간격을 적용해 실제 문단을 더
아래에 그린다. 그 결과 pagination의 `current_height`가 renderer보다 낮게
계산되어, 뒤따르는 문28/문29 흐름이 page 17 하단에 과도하게 남았고
`pi=818~820`이 frame 아래로 내려갔다.

## 수정 내용

- 보이는 구분선이 있고 `미주 사이`가 비기본 compact 값인 internal-rewind 문단은
  하단 split 후보가 1줄뿐일 때 억지로 제거하지 않고 유지한다.
- 단 하단에서 다음 단/쪽으로 이동된 internal-rewind 미주는 새 단 시작 시 축약된
  `en_fit` 대신 문단 전체 line advance와 저장된 `미주 사이` gap을 소비하도록
  보정했다.
- 마지막 단에서도 compact 미주 tail이 하단에 너무 가까우면 다음 쪽으로 넘길 수
  있도록 late tail overflow risk 조건을 확장했다.

이 변경은 개별 문항 번호가 아니라 `구분선 있음 + 비기본 compact 미주 사이 +
internal vpos rewind + 단 하단 이동` 구조에 걸리는 공통 흐름 보정이다.

## 검증

### 단일 target 재검증

```bash
cargo build --bin rhwp
python3 scripts/task1274_visual_sweep.py \
  --target 2024-11-practice-shape987 \
  --out output/task1293_stage40_shape_trial13 \
  --rhwp-bin target/debug/rhwp
```

결과:

- SVG/PDF/render tree 페이지 수: `21/21/21`
- `overflow_lines`: `[]`
- `visual_metrics.frame_overflow_pages`: `[]`
- page 17에서 기존 overflow 문단 `pi=818~820`은 다음 페이지로 이동했다.
- `compare_017.png` 기준 frame 하단 overwrap은 제거되었다.

주의: page 17/18의 PDF 대비 문항 흐름은 아직 완전 일치하지 않는다. 이번 stage는
renderer overflow 제거까지 확인했고, 남은 시각 drift는 후속 stage에서 계속 본다.

### focused sweep

```bash
python3 scripts/task1274_visual_sweep.py \
  --target 2024-11-practice-above20-between7-below2 \
  --target 2024-11-practice-above20-between0-below20 \
  --target 2024-11-practice-shape987 \
  --target 2024-11-practice-above0-between0-below0 \
  --out output/task1293_stage40_focused \
  --rhwp-bin target/debug/rhwp
```

결과:

| target | frame overflow | render-tree overflow |
|---|---:|---:|
| `2024-11-practice-above20-between7-below2` | 0 | 0 |
| `2024-11-practice-above20-between0-below20` | 0 | 0 |
| `2024-11-practice-shape987` | 0 | 0 |
| `2024-11-practice-above0-between0-below0` | 0 | 14 |

`above0-between0-below0`의 잔여 render-tree overflow는 page 9/10/12/13/16/17에
남아 있으며 다음 stage에서 별도 분석한다.

### focused test

```bash
cargo test --lib compact_endnote -- --nocapture
cargo fmt --all -- --check
git diff --check
```

결과:

- `compact_endnote` 관련 28개 테스트 통과
- formatting check 통과
- whitespace diff check 통과

## 남은 작업

- `2024-11-practice-above0-between0-below0`의 14개 render-tree overflow를 다음
  stage에서 분석한다.
- `shape987` page 17/18의 PDF 대비 흐름 차이도 후속 sweep/시각 비교에서 계속
  추적한다.
