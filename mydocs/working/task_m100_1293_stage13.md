# Task 1293 Stage 13: shape987 p12 내부 rewind 분할 보정

## 목적

Stage12 이후 `3-11월_실전_통합_2024-구분선위9미주사이8구분선아래7.hwp`의 p12에서
왼쪽 하단 표/수식 흐름이 한컴/PDF와 다르게 우측 컬럼 앞부분에 중복되어 들어가고, 그 영향으로
뒤 문항들이 아래로 밀리는 문제가 남았다.

이번 단계에서는 p12의 내부 `vpos` rewind 문단을 한컴처럼 컬럼 경계에서 분할하고, 빈 spacer 뒤
미주 간격이 중복 적용되는 현상을 함께 보정한다.

## 현재 기준

- 직전 산출물: `output/task1293_stage12_sample_check_all/summary.json`
- 대상 샘플: `samples/3-11월_실전_통합_2024-구분선위9미주사이8구분선아래7.hwp`
- 한컴/PDF 기준 p12 우측 컬럼은 `pi=571`의 뒤쪽 줄부터 시작해야 한다.
- 기존 rhwp는 `pi=571` 전체를 다음 컬럼으로 넘기면서 앞쪽 2줄이 우측 컬럼에 중복되어 보였다.

## 원인

`pi=571`은 문단 내부에서 2번째 줄 이후 `vpos`가 크게 되감긴다. 이 경우 한컴은 현재 컬럼에 들어갈
수 있는 앞쪽 줄은 그대로 배치하고, 되감긴 줄부터 다음 컬럼에서 이어간다.

기존 `typeset`의 fit 판정은 내부 rewind split 후보가 있어도 문단 전체 높이 기준으로 다음 컬럼
advance를 먼저 수행했다. 그래서 `pi=571`의 앞쪽 줄까지 다음 컬럼으로 이동했고, p12 하단 표/수식
흐름과 뒤 문항 marker가 연쇄적으로 밀렸다.

추가로 p12 우측 컬럼의 `pi=574` 빈 spacer 뒤에서 `미주 사이` 간격이 이미 `y_offset`에 반영된 뒤
한 번 더 더해져 `문16)` 이후 marker가 한컴보다 아래로 이동했다.

## 수정 내용

- `src/renderer/typeset.rs`
  - 내부 rewind split 후보가 있고, split 앞쪽 줄 높이가 현재 컬럼 남은 높이 안에 들어가면 문단
    전체를 다음 컬럼으로 advance하지 않는다.
  - 기존 partial paragraph emit 경로가 `lines=0..split`을 현재 컬럼에 두고 `lines=split..end`를
    다음 컬럼에 배치하도록 한다.

- `src/renderer/height_cursor.rs`
  - compact 미주에서 빈 spacer 문단의 stale note gap은 `line_spacing`을 다시 더하지 않고 현재
    `y_offset`을 그대로 사용한다.
  - 제목 tail backtrack은 직전 제목의 실제 하단보다 위로 올라가지 않게 제한하면서 `y_offset`을
    넘지 않도록 cap을 명확히 했다.
  - lazy path의 미주 하단 제목 tail 기준을 0.88로 낮춰 shape987 계열 p14 제목 tail이 frame 안에서
    먼저 정리되도록 했다.

## 확인 결과

`dump-pages` 기준 p12의 `pi=571`은 아래처럼 분할된다.

```text
단 0 ... PartialParagraph pi=571 lines=0..2 vpos=294185..295715
단 1 ... PartialParagraph pi=571 lines=2..6 vpos=252335..258731
```

`output/task1293_stage13_all_threshold_check/summary.json` 기준:

| 대상 | 페이지 수 | 주요 결과 |
|---|---:|---|
| `2024-11-practice-shape987` | 21/21/21 | p12 `line_order_overlap` 제거, p12 `frame_overflow` 제거 |
| `2024-11-practice-above0-between7-below20` | 21/21/21 | frame 후보 없음 |
| `2024-11-practice-above20-between7-below2` | 21/21/21 | frame 후보 없음 |

남은 후보:

- `2024-11-practice-shape987`: p11, p14, p18, p19, p20, p21 frame 후보가 남아 있다.
- `2024-11-practice-no-separator-above20-between20-below20`: p17, p20 frame 후보가 남아 있다.
- 이들은 p12 내부 rewind 중복과는 별도 원인으로 보여 다음 스테이지에서 분리 분석한다.

## 검증

- `cargo fmt --all -- --check`: 통과
- `cargo build --bin rhwp`: 통과
- `cargo test height_cursor --lib -- --nocapture`: 37개 통과
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`: 52개 통과
- `cargo test --test issue_1139_inline_picture_duplicate issue_1274_2022_oct_page11_question20_equation_tail_stays_in_frame -- --nocapture`: 통과
- `python3 scripts/task1274_visual_sweep.py --target all --out output/task1293_stage13_all_threshold_check --rhwp-bin target/debug/rhwp`: 완료
