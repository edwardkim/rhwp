# task 1293 stage90 - visible separator 20mm 미주 head group anchor 분석

## 목적

stage89에서 `FootnoteShape` attr 해석과 수식 TAC 판정 정리는 커밋했다. stage90에서는 남은
`visible separator + 20mm betweenNotes` 계열의 p11/p18/p19 후보를 새 stage에서 분리해 다룬다.

## 시작 기준

- 브랜치: `local/task_m100_1293`
- 시작 커밋: `beb7b212 task 1293: FootnoteShape attr와 수식 TAC 정리`
- stage89 기준 확인:
  - `2024-09-between20`: `flagged=3/24`
  - 남은 페이지: p11, p18, p19
  - `2024-11-practice-shape987`: `flagged=1/21`
  - `2024-11-practice-above0-between0-below0`: `flagged=0/21`
  - `2024-11-practice-no-separator-above20-between20-below20`: `flagged=0/23`

## 남은 문제

1. p19에서 `문28` marker가 PDF/한컴 기준보다 약 54px 위에 있다.
2. p19의 `문27` tail과 `문28` title/body/head visual 경계에서 `미주 사이` 소비 위치가 맞지 않는다.
3. p11/p18에서는 visual/TAC/equation tail의 저장 `lineSeg.vertical_pos`와 실제 render-tree bbox가
   sequential pagination 판단과 다르게 나타난다.
4. stage89의 여러 실험에서 `미주 사이` 값을 단순히 더하거나 빼면 p20~p22 흐름이 회귀했다.

## 처리 방향

- 개별 문항 번호 기준 보정은 하지 않는다.
- `미주 사이` 수치 자체를 단/쪽 시작 offset으로 일괄 가산하지 않는다.
- `이전 note tail + 다음 note title/body/head visual`을 하나의 head group으로 보고,
  이 group의 anchor가 저장 vpos, 현재 column sequential flow, page/column split 중 어디에서 와야 하는지
  `lineSeg` 단위로 분류한다.
- `visible separator + large betweenNotes`에서만 필요한 판단과 compact/zero/no-separator clean 조합을
  명확히 분리한다.

## 우선 분석 대상

1. `2024-09-between20` p19:
   - `문27` 마지막 수식/텍스트 tail
   - `문28` title, 첫 본문, 첫 TAC 그림
   - PDF/한컴 기준 marker y와 render-tree y 차이
2. `2024-09-between20` p18:
   - equation-only tail의 frame bottom 후보가 실제 overflow인지 sweep 과검출인지 분리
3. `2024-09-between20` p11:
   - visual tail 저장 vpos가 순차 텍스트 흐름보다 앞서는 패턴인지 확인

## 검증 계획

사용자 승인 전 full CI는 수행하지 않는다.

```bash
cargo build --bin rhwp
cargo test --test issue_1139_inline_picture_duplicate issue_1293_ -- --nocapture
python3 scripts/task1274_visual_sweep.py \
  --target 2024-09-between20 \
  --target 2024-11-practice-shape987 \
  --target 2024-11-practice-above0-between0-below0 \
  --target 2024-11-practice-no-separator-above20-between20-below20 \
  --out output/task1293_stage90_targeted
```

## 작업지시자 승인

2026-06-14 작업지시자가 "승인 진행"으로 stage90 착수를 승인했다.

## 진행 시작

stage90에서는 위 기준으로 p11/p18/p19의 head group anchor 모델을 분석한다. 소스 수정은 이 문서 기준으로
진행한다.

## 구현 내용

- `src/renderer/layout.rs`의 textless equation tail 뒤 새 미주 제목 보정에서, 직전 `PageItem`을
  `HeightCursor` 저장 paragraph보다 우선 사용하도록 조정했다.
- `visible separator + large betweenNotes` 프로필에서만, 직전 미주가 textless 수식 tail이고 다음 미주
  제목 뒤에 보이는 본문이 먼저 나온 뒤 큰 TAC 그림/도형이 이어지는 경우를 head group anchor 후보로 본다.
- 이 후보에서는 저장 `LINE_SEG`의 head anchor gap을 현재 렌더 `y_offset`에 추가해 p19 `문28` 제목/body
  group을 PDF 기준 위치로 내린다.
- p14처럼 제목 바로 다음 문단이 큰 TAC 그림인 head는 보정하지 않도록 제외했다.
- no-separator 계열은 `구분선 위/아래`가 큰 프로필이므로 이번 보정 대상에서 제외했다.

## 확인 결과

- `2024-09-between20` p19 `문28` marker:
  - stage89 render y: 749.6px
  - stage90 render y: 804.8px
  - PDF 기준 y: 803.5px
- p14 `문21` marker는 직접 TAC head라 보정 제외 후 stage89 위치 644.2px로 유지했다.
- p20 `문29` 흐름도 stage89 위치 398.2px로 유지했다.

## 검증

```bash
cargo fmt --check
cargo build --bin rhwp
cargo test --test issue_1139_inline_picture_duplicate issue_1293_equation_control_is_not_always_treat_as_char -- --nocapture
python3 scripts/task1274_visual_sweep.py \
  --target 2024-09-between20 \
  --target 2024-11-practice-shape987 \
  --target 2024-11-practice-above0-between0-below0 \
  --target 2024-11-practice-no-separator-above20-between20-below20 \
  --out output/task1293_stage90_targeted_v3
```

검증 결과:

- `cargo fmt --check`: 통과
- `cargo build --bin rhwp`: 통과
- focused test: 통과 (`issue_1293_equation_control_is_not_always_treat_as_char`)
- `2024-09-between20`: `flagged=3/24`, p11/p18/p19 유지. 단, p19의 `question_marker_drift`는 제거됐다.
- `2024-11-practice-shape987`: `flagged=1/21` 유지
- `2024-11-practice-above0-between0-below0`: `flagged=0/21` 유지
- `2024-11-practice-no-separator-above20-between20-below20`: `flagged=0/23` 유지

## 남은 문제

- p19는 `문28` marker drift는 해소됐지만, `문28` tail의 p981 overflow와 p27 tail overflow가 남아
  `tail/large` 계열 flag가 유지된다.
- p18의 equation-only tail overflow 후보와 p11의 visual/vpos mismatch 후보는 아직 남아 있다.
- 다음 stage에서는 p19 tail overflow와 p18/p11 후보를 별도로 분리한다.
