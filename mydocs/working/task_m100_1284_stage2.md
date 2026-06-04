# task 1284 stage2: 2024 미주사이20 문항 marker drift 원인 분석

## 배경

- stage1에서 PDF bbox 기반 `question_marker_drift` 검출을 추가했다.
- 전체 sweep 결과, 실제 후속 수정 대상은 `3-09월_교육_통합_2024-미주사이20.hwp`가 중심으로 확인됐다.

## sweep 관찰

- `2024-09-between20`: `question_marker_drift_pages=[13, 14, 18, 21, 23]`
- `2024-09-below20`: `question=[]`
- `2022-09`, `2022-10`, `2022-11-practice`: `question=[]`

대표 후보:

- 13쪽
  - `문15`: rhwp y=550.8, PDF y=624.5, `dy=-73.7`
  - `문16`: rhwp y=509.7, PDF y=597.7, `dy=-88.0`
  - `문17`: rhwp y=812.0, PDF y=900.2, `dy=-88.2`
  - `문18`: rhwp y=982.3, PDF y=1070.5, `dy=-88.2`
- 14쪽
  - `문19`: rhwp y=234.9, PDF y=312.1, `dy=-77.2`
  - `문20`: rhwp y=726.8, PDF y=816.3, `dy=-89.5`

## 원인 확인

- 같은 문서 계열의 `구분선아래20`은 `question=[]`인데 `미주사이20`만 위쪽 drift가 난다.
- 처음에는 `EndnoteShape.raw_unknown`에 저장된 한컴 UI “미주 사이 20mm” 간격이 compact 미주 단 흐름에서 누락되는 문제로 의심했다.
- 실제 dump와 PDF bbox를 대조하니, 12쪽 하단의 `pi=662` 문단이 마지막 단 frame 밖에 `FullParagraph[미주]`로 남고 있었다.
- PDF 기준으로는 `pi=662`의 `ㄱ. [참]` tail이 13쪽 첫머리에서 시작한 뒤 문15가 이어진다.
- rhwp는 12쪽 frame 밖에 이 문단을 그려 버렸기 때문에 13쪽의 문15~문20 marker가 PDF보다 약 74~90px 위로 당겨졌다.

## 수정 방향

1. `src/renderer/typeset.rs`의 compact 미주 흐름에서 `internal_rewind_split == Some(1)`인 문단을 재검토한다.
2. 큰 `미주 사이` 값을 가진 문서에서 마지막 단 하단에 첫 줄부터 vpos가 되감기는 문단은 현재 쪽에 남기지 않고 다음 쪽에서 통째로 시작하게 한다.
3. 단, PDF 기준으로 현재 쪽 하단에 남는 1줄짜리 문항 제목 tail은 기존처럼 허용한다. 문18 제목처럼 현재 쪽 끝에 남고 본문만 다음 쪽으로 이어지는 사례가 있기 때문이다.
4. `3-09월_교육_통합_2024-미주사이20.hwp` 13쪽 문15~문18 marker y를 PDF bbox 근처로 고정하는 회귀 테스트를 추가한다.

## 검증 계획

- 기존 `issue_1139_inline_picture_duplicate.rs`에 2024 미주사이20 문15~문18 marker y guard 추가
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
- `python3 scripts/task1274_visual_sweep.py --target 2024-09-between20`
- 필요 시 `python3 scripts/task1274_visual_sweep.py --target all`

## 검증 결과

- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
  - 52 passed
  - 신규 `issue_1284_2024_between20_page13_question_flow_matches_pdf` 포함
  - 기존 1261/1274 회귀 항목도 함께 통과

## 상태

- 원인 분석 및 1차 수정 완료.
- 이 커밋 후 `2024-09-between20` sweep를 다시 실행해 남은 marker drift/overflow 후보를 분석한다.
