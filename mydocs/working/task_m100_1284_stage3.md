# task 1284 stage3: 18쪽 하단 문24/25 overflow 이월 보정

## 배경

- stage2 수정 후 `2024-09-between20` sweep를 다시 실행했다.
- 13~14쪽 문15~문20 marker drift는 `question` 후보에서 사라졌다.
- 남은 핵심 후보는 18쪽 오른쪽 단 하단의 문24/25 overflow와, 그 여파로 19쪽 문26~문28이 PDF보다 위로 당겨지는 문제다.

## 관찰

- `render_tree.log` 기준 18쪽 오른쪽 단에서 다음 문단들이 frame 밖에 그려진다.
  - `pi=937` 문24: y=1140.6, col_bottom=1092.3, overflow=48.3px
  - `pi=938`: y=1174.2, overflow=81.9px
  - `pi=939`: y=1210.9~1269.9, overflow=177.7px
  - `pi=940` 문25: y=1357.5, overflow=265.3px
- PDF 기준 18쪽 오른쪽 단에는 문23까지만 보이고, 문24/25는 19쪽 왼쪽 단 상단으로 넘어간다.
- rhwp는 문24/25가 18쪽 render tree에는 있으나 실제 화면에서는 하단 밖에 있어 보이지 않고, 19쪽은 곧바로 문26부터 시작한다.

## 수정 방향

1. stage2에서 추가한 `allow_large_between_question_title_tail`이 너무 넓게 적용되어 frame 밖의 문항 제목까지 허용하는지 확인한다.
2. 1줄짜리 문항 제목 tail은 한컴/PDF처럼 frame 안에 실제로 보이는 경우에만 허용한다.
3. title tail 뒤의 후속 문단이 같은 단에서 frame 밖으로 이어지는 경우에는 제목부터 다음 쪽/단으로 넘겨, 후속 문단이 보이지 않는 영역에 남지 않게 한다.
4. `issue_1139_inline_picture_duplicate.rs`에 2024 미주사이20 18~19쪽 문24/25 이월 guard를 추가한다.

## 수정 내용

- `src/renderer/typeset.rs`
  - 큰 `미주 사이` 문항 제목 tail 후보에서 현재 단의 기존 item들을 render `HeightCursor` 방식으로 재생해 다음 제목의 실제 render y를 예측한다.
  - 예측한 제목 bottom이 body frame을 넘으면 현재 쪽/단에 남기지 않고 `advance_column_or_new_page()`로 다음 쪽/단에서 시작하게 한다.
  - 이 보정은 `미주 사이`가 기본값보다 큰 compact 미주, 마지막 단, 새 문항 제목 1줄 후보에만 제한했다.
- `tests/issue_1139_inline_picture_duplicate.rs`
  - `issue_1284_2024_between20_page19_question24_continues_from_pdf_top` 추가.
  - 18쪽에는 `pi=937` 문24가 남지 않고, 19쪽이 문24→문25→문26 순서로 시작하는지 확인한다.
  - 문24/25/26/27/28 제목 y를 PDF bbox 기준 근처로 고정한다.

## 검증 계획

- `cargo test --test issue_1139_inline_picture_duplicate issue_1284_2024_between20_page13_question_flow_matches_pdf -- --nocapture`
- 신규 18~19쪽 guard 테스트
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
- `python3 scripts/task1274_visual_sweep.py --target 2024-09-between20`

## 검증 결과

- `cargo test --test issue_1139_inline_picture_duplicate issue_1284_2024_between20 -- --nocapture`
  - 2 passed
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
  - 53 passed
- `python3 scripts/task1274_visual_sweep.py --target 2024-09-between20`
  - `question` 후보가 `[16,18,19,20,21,22,23]`에서 `[18,21,23]`으로 감소했다.
  - `frame=[]`로 frame overflow 후보는 없다.
  - 19쪽은 `question_marker_drift_candidates=[]`가 됐다.
  - 남은 실제 후속 후보는 21쪽 문23 column drift다.

## 상태

- 수정 및 자동 검증 완료.
- 다음 stage에서는 21쪽 문23이 PDF와 달리 오른쪽 단 상단으로 이동한 문제를 분석한다.
