# task 1293 stage99 - 2023-09 p13 미주 흐름 잔여 후보 분석

## 목적

stage98 이후 `2023-09`는 p11 `문12` title orphan이 해소되어 `flagged=3/20`에서 `flagged=2/20`으로 줄었다.
남은 p13, p19 중 p13은 `question_marker_drift`, `red_marker_drift`, `large_ink_region_drift`,
`render_tree_frame_tail_overflow`가 함께 잡히므로 실제 흐름 차이가 더 크다.

stage99에서는 `2023-09` p13의 대표 후보를 추적해, 미주 title/body advance 규칙으로 고칠 수 있는 실제 mismatch인지
아니면 sweep 과검출인지 먼저 분리한다.

## 시작 기준

- 브랜치: `local/task_m100_1293`
- 시작 커밋: `71afdd23 task 1293: 기본 미주 제목 고아 배치 보정`
- Stage98 targeted sweep:
  - `2023-09`: `flagged=2/20`
  - 남은 페이지: p13, p19
  - p13 주요 후보:
    - `question_marker_drift`: `문16` y drift 42px
    - `render_tree_frame_tail_overflow`: `문19` `pi=695` `[EQ][EQ]` overflow 24.5px

## 처리 방향

- `2023-09` p13의 dump-pages, render-tree 후보, question flow를 확인한다.
- `문16` marker drift가 실제 문항 시작점 차이인지, p13 하단 tail overflow가 다음 쪽 흐름에 영향을 주는지 확인한다.
- 코드 수정이 필요하면 새 미주 흐름 조건을 좁게 보정하고, `2023-09` 및 stage98에서 0을 유지한 target을 targeted sweep으로 재확인한다.

## 검증 계획

```bash
cargo fmt --check
cargo build --bin rhwp
python3 scripts/task1274_visual_sweep.py \
  --target 2023-09 \
  --target 2024-09-between20 \
  --target 2024-11-practice-shape987 \
  --target 2024-11-practice-above0-between0-below0 \
  --target 2024-11-practice-no-separator-above20-between20-below20 \
  --out output/task1293_stage99_targeted
git diff --check
```

CI 전체 테스트와 PR은 작업지시자 지시에 따라 수행하지 않는다.

## 분석 결과

`2023-09` p13의 `문16` marker drift는 `문16` 자체가 아니라 직전 `문15` 마지막 빈 guide 문단과
새 미주 경계 gap이 중복 소비되는 문제였다.

- PDF 기준:
  - `문15` marker y는 rhwp/PDF가 거의 동일하다.
  - `문16`부터 `문19`까지 rhwp가 PDF보다 약 42px 아래에 있었다.
- rhwp render-tree 기준:
  - 수정 전 `문16` `pi=671` y=`450.3px`, PDF y=`408.3px`
  - `문15` 마지막 `pi=670`은 두 줄짜리 빈 문단이고, boundary 단계에서 마지막 line spacing이
    `1984HU`로 커졌다.
  - 이어서 `문16` 제목 cap도 `h4f 12.0px + 미주 사이 26.5px = 38.5px`를 다시 소비했다.
- 즉 p13은 기본 미주 사이가 직전 tail 렌더 spacing과 새 제목 cap에 이중 반영된 케이스다.

## 구현 내용

- `src/renderer/typeset.rs`에서 기본 미주 + 보이는 구분선 + 마지막 단 중간 tail 조건을 추가했다.
- 같은 조건에서:
  - 직전 미주 마지막 paragraph의 render trailing에 `미주 사이`를 다시 주입하지 않는다.
  - 새 미주 제목의 `new_endnote_between_notes_px` cap을 `0px`로 계산한다.
- 조건은 다음처럼 좁혔다.
  - `visible_nonzero_default_between_notes`
  - 같은 쪽/단에 직전 미주 tail이 이어짐
  - 마지막 단
  - 현재 단 사용 높이가 `25%` 초과, `50%` 미만

## 확인 결과

### 대표 케이스

```bash
RHWP_ENDNOTE_ADVANCE_DEBUG=1 RHWP_EN_SSOT_DEBUG=1 \
  target/debug/rhwp export-svg samples/3-09월_교육_통합_2023.hwp \
  -o output/task1293_stage99_debug_final/svg \
  -p 12
```

- 수정 전: `note=16 ep=0` `gap=Some(26.45)`, `pi=671 adv=38.5`
- 수정 후: `note=16 ep=0` `gap=Some(0.0)`, `pi=671 adv=12.0`
- render-tree `문16` 위치: `450.3px` → `423.8px`
- Stage98의 `2023-09` p11 `문12` 단 1 상단 배치는 유지됐다.

### targeted sweep 결과

```bash
python3 scripts/task1274_visual_sweep.py \
  --target 2023-09 \
  --target 2024-09-between20 \
  --target 2024-11-practice-shape987 \
  --target 2024-11-practice-above0-between0-below0 \
  --target 2024-11-practice-no-separator-above20-between20-below20 \
  --out output/task1293_stage99_targeted
```

| target | 결과 |
|--------|------|
| `2023-09` | `flagged=1/20` (`stage98: 2/20`, p13 후보 해소, p19 잔류) |
| `2024-09-between20` | `flagged=0/24` 유지 |
| `2024-11-practice-shape987` | `flagged=0/21` 유지 |
| `2024-11-practice-above0-between0-below0` | `flagged=0/21` 유지 |
| `2024-11-practice-no-separator-above20-between20-below20` | `flagged=0/23` 유지 |

### 검증 명령 결과

- `cargo fmt --check`: 통과
- `cargo build --bin rhwp`: 통과
- `git diff --check`: 커밋 전 수행 예정

CI 전체 테스트와 PR은 작업지시자 지시에 따라 수행하지 않았다.
