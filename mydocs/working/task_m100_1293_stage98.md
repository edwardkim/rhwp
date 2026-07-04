# task 1293 stage98 - question title orphan advance 보정

## 목적

stage97에서 남은 실제 flow mismatch 중 대표 케이스는 `2023-09` p11 `문12` title orphan이다.
rhwp는 `문12` title만 왼쪽 단 하단에 남기고 본문은 오른쪽 단으로 넘기지만, PDF는 title도 오른쪽 단
상단에서 시작한다.

stage98에서는 새 미주 title이 현재 단 하단에 단독으로 남는 조건을 추적하고, 본문 첫 줄이 함께 들어가지
못하는 경우 다음 단으로 넘기도록 좁게 보정한다.

## 시작 기준

- 브랜치: `local/task_m100_1293`
- 시작 커밋: `aca70e6e task 1293: 잔여 tail question 후보 분류`
- 대표 후보:
  - target: `2023-09`
  - page: p11
  - rhwp: `문12` title `pi=570`이 왼쪽 단 하단
  - PDF: `문12` title이 오른쪽 단 상단

## 처리 방향

- `advance_for_new_endnote`와 question title tail 허용 조건을 확인한다.
- 현재 단에 title만 들어가고 다음 본문 첫 줄이 들어가지 않는 경우를 탐지한다.
- 기존 0 target인 `2024-09-between20`, `shape987`, zero/no-separator 조합에 회귀가 없도록 targeted sweep으로 확인한다.

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
  --out output/task1293_stage98_targeted
git diff --check
```

CI 전체 테스트와 PR은 작업지시자 지시에 따라 수행하지 않는다.

## 작업지시자 승인

2026-06-14 작업지시자가 자동 승인과 연속 커밋 진행을 지시했다.

## 구현 내용

- `src/renderer/typeset.rs`의 `default_question_title_tail_fits_by_line_height` 조건을 보정했다.
- 기존 조건은 기본 `미주 사이` + 보이는 구분선 + 첫 단 하단에서 새 미주 제목 한 줄이 들어가면
  `ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX`까지 허용해 `advance_for_new_endnote`를 막았다.
- `2023-09` p11 `문12`는 이 조건 때문에 `allow_default_line=true`, `advance_new=false`가 되어
  제목만 왼쪽 단 하단에 남았다.
- 기본 미주/보이는 구분선/현재 단 높이 95% 초과 조건에서는 제목+다음 본문 높이 검사에 하단 bleed를
  적용하지 않고 `available + 2px`까지만 허용하도록 좁혔다.

## 확인 결과

### 대표 케이스

```bash
RHWP_ENDNOTE_ADVANCE_DEBUG=1 target/debug/rhwp export-svg \
  samples/3-09월_교육_통합_2023.hwp \
  -o output/task1293_stage98_debug_after/svg \
  -p 10
```

- 수정 전: `note=12 ep=0`에서 `allow_default_line=true`, `advance_new=false`
- 수정 후: `note=12 ep=0`에서 `allow_default_line=false`, `advance_new=true`

```bash
target/debug/rhwp dump-pages samples/3-09월_교육_통합_2023.hwp -p 10
```

- 수정 전: `pi=570` `문12） 23_09 교육 12) ①`이 단 0 하단에 단독 배치
- 수정 후: `pi=570` `문12） 23_09 교육 12) ①`이 단 1 상단으로 이동

### 검증 명령

```bash
cargo fmt --check
cargo build --bin rhwp
python3 scripts/task1274_visual_sweep.py \
  --target 2023-09 \
  --target 2024-09-between20 \
  --target 2024-11-practice-shape987 \
  --target 2024-11-practice-above0-between0-below0 \
  --target 2024-11-practice-no-separator-above20-between20-below20 \
  --out output/task1293_stage98_targeted
git diff --check
```

### targeted sweep 결과

| target | 결과 |
|--------|------|
| `2023-09` | `flagged=2/20` (`stage96: 3/20`에서 p11 후보 해소) |
| `2024-09-between20` | `flagged=0/24` 유지 |
| `2024-11-practice-shape987` | `flagged=0/21` 유지 |
| `2024-11-practice-above0-between0-below0` | `flagged=0/21` 유지 |
| `2024-11-practice-no-separator-above20-between20-below20` | `flagged=0/23` 유지 |

CI 전체 테스트와 PR은 작업지시자 지시에 따라 수행하지 않았다.
