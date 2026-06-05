# task 1284 stage4: 21쪽 문23 column drift 분석

## 배경

- stage3 후 `2024-09-between20` sweep 결과 `question=[18,21,23]`만 남았다.
- 19쪽 문24/25 이월 문제는 해소됐고, frame overflow 후보도 없다.
- 남은 실제 후보 중 가장 큰 차이는 21쪽 문23이다.

## 관찰

- sweep 후보:
  - rhwp: page 21, column 1, `pi=1054`, y=90.7, text=`문23）   ④`
  - PDF: page 21, column 0, y=1073.2, text=`문23）`
  - `column_drift`, `y_delta=-982.5px`
- 시각 비교:
  - PDF는 왼쪽 단 하단에 문23 제목만 남고, 오른쪽 단은 문24~문26으로 이어진다.
  - rhwp는 왼쪽 단에서 문30 풀이를 더 길게 유지한 뒤, 문23을 오른쪽 단 상단으로 넘긴다.

## 1차 가설

- 문23 직전의 문30 tail 또는 그 이전 TAC/수식 paragraph가 PDF보다 더 많은 높이를 차지해 왼쪽 단 하단에 문23 제목 tail이 들어갈 공간을 잃은 것으로 보인다.
- stage3의 title-tail render overflow 예측은 frame 밖으로 나가는 제목만 넘기는 방어로직이므로, 이번처럼 PDF가 하단 제목 tail을 허용하는 방향에는 별도 조건이 필요하다.

## 원인 확인

- `typeset.rs`의 큰 `미주 사이` 문항 제목 tail 허용 조건이 마지막 단에만 열려 있었다.
- PDF page 21은 중간 단인 왼쪽 단 하단에 문23 제목 1줄만 남기고, 문23 본문은 오른쪽 단 상단에서 이어진다.
- rhwp는 왼쪽 단의 title-tail을 허용하지 않아 문23 제목과 본문 전체가 오른쪽 단으로 넘어갔다.
- title-tail을 허용한 뒤에는 `prev_tac_seg_applied` 경로에서 `HeightCursor`가 bypass되어 q23 제목이 frame 밖 y=1126.3에 남았다.

## 수정 내용

- `src/renderer/typeset.rs`
  - 큰 `미주 사이` 문항 제목 tail 허용 조건에서 마지막 단 제한을 제거했다.
  - 대신 `current_height > available * 0.85`와 frame-inside 예측을 함께 요구해 중간 단의 이른 제목 tail은 허용하지 않는다.
- `src/renderer/height_cursor.rs`
  - compact 미주 문항 제목이 column bottom을 소폭 넘는 경우 제목 visual line-height 기준으로 하단 안쪽에 맞추는 bottom-fit 보정을 추가했다.
- `src/renderer/layout.rs`
  - TAC/수식 직후 `HeightCursor`가 bypass되는 경우에도 compact 미주 문항 제목 1줄이 frame 안에 들어갈 수 있으면 직접 bottom-fit을 적용한다.
  - bottom-fit이 적용된 제목에는 기존 미주 제목 간격 보존 로직을 다시 적용하지 않도록 막았다.
- `tests/issue_1139_inline_picture_duplicate.rs`
  - `issue_1284_2024_between20_page21_question23_title_stays_in_left_tail` 추가.
  - page 21 왼쪽 단 하단 문23 제목, 오른쪽 단 상단 문23 본문, 이어지는 문24~문26 위치를 PDF bbox 기준으로 고정했다.

## 검증 계획

- 신규 21쪽 문23 guard 테스트
- `cargo test --test issue_1139_inline_picture_duplicate issue_1284_2024_between20 -- --nocapture`
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
- `python3 scripts/task1274_visual_sweep.py --target 2024-09-between20`

## 검증 결과

- `cargo test --test issue_1139_inline_picture_duplicate issue_1284_2024_between20 -- --nocapture`
  - 3 passed
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
  - 54 passed
- `python3 scripts/task1274_visual_sweep.py --target 2024-09-between20`
  - `question=[18,21,22,23]`
  - page 21의 `문23 column_drift` 후보는 사라졌다.
  - page 21에는 후속 후보로 q25/q26 약 +57px drift와 `pi=1082/1083` tail overflow 로그가 남았다.
  - page 22에는 문28 약 -75.5px drift가 새 후속 후보로 남았다.

## 상태

- 수정 및 자동 검증 완료.
- 다음 stage에서는 page 21 오른쪽 단 하단의 `pi=1082/1083` q26 tail overflow와 그 여파로 보이는 page 22 문28 drift를 분석한다.
