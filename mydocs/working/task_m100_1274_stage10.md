# Task 1274 Stage 10

## 대상

- Stage9 전체 sweep 이후 남은 `2022-10` overflow 3줄:
  - 16쪽 왼쪽 단 `pi=841`, `pi=842`
  - `pi=841`: `문30）   12`
  - `pi=842`: `함수 의 한 부정적분을 라 하자.`

## 관찰

- PDF 16쪽은 문30 제목뿐 아니라 `pi=842`에 해당하는 첫 본문 줄도 왼쪽 단 하단에 남긴다.
- rhwp도 시각 배치는 PDF처럼 왼쪽 단 하단에 남지만, `pi=841`, `pi=842`를 overflow로 기록한다.
- 기존 late question tail 정책은 문29/문30 하단 꼬리를 허용하는 의도였고, 이번 케이스도 그 허용 범위에 들어간다.
- 따라서 본문을 다음 단으로 넘기는 조판 변경은 오답이며, 실제 문제는 late tail 허용 배치를 overflow 수집기가 다시 잡는 오탐이다.

## 목적

- 기본 미주 간격 문29/문30 late title 허용은 유지한다.
- 문30 제목과 첫 본문 줄의 기존 시각 배치는 유지한다.
- 문서/페이지 하드코딩 없이 late question tail과 overflow 검출 조건을 같은 기준으로 맞춘다.

## 검증 계획

- 진행 중 자동 테스트는 `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`만 사용한다.
- `cargo build --bin rhwp`
- `python3 scripts/task1274_visual_sweep.py --target 2022-10`
- 필요 시 전체 sweep으로 페이지 수와 overflow 수를 다시 확인한다.
- 전체 CI급 테스트는 전체 목표 마지막에만 수행한다.

## 원인

- Stage8/9의 기본 미주 late tail 정책은 문29/문30 하단 꼬리를 시각 분기상 허용한다.
- `pi=841`은 컬럼 마지막 항목이 아니라 마지막에서 두 번째 문항 제목이어서 item-level overflow 예외를 타지 못했다.
- `pi=842`는 마지막 항목이지만, 9pt 줄 높이 반올림까지 포함한 draw bottom이 기존 24px bleed 허용폭을 약 1.8px 초과했다.
- 조판 기준의 24px 허용폭을 키우면 2022-11 p14 미주 분기가 흔들리므로, 배치 기준과 렌더 overflow 로그 허용폭을 분리해야 한다.

## 수정

- `ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX`는 24px로 유지해 typeset 단 넘김 기준을 보존했다.
- 렌더 overflow 로그 수집에는 별도 28px 허용폭을 사용해 한컴/PDF처럼 페이지 테두리 안쪽에 남는 9pt 미주 하단 꼬리를 오탐하지 않게 했다.
- item-level overflow 검출은 미주 flow 컬럼의 마지막 항목뿐 아니라 마지막에서 두 번째 문항 제목도 late tail 항목으로 인정한다.
- 문30 본문을 다음 단으로 넘기는 조판 변경은 제거했다.

## 검증 결과

- `cargo fmt`
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
  - 48개 통과
- `cargo build --bin rhwp`
- `python3 scripts/task1274_visual_sweep.py --target 2022-10`
  - SVG/PDF/비교 PNG 18쪽 유지
  - `2022-10` overflow 3줄에서 0줄로 감소
  - `dump-pages` 기준 16쪽 왼쪽 단 하단은 `pi=841`, `pi=842`를 유지하고, 오른쪽 단은 `pi=843`부터 시작한다.
- `python3 scripts/task1274_visual_sweep.py`

전체 sweep 결과:

- `2022-09`: SVG/PDF/비교 PNG 23쪽, overflow 없음
- `2023-09`: SVG/PDF/비교 PNG 20쪽, overflow 없음
- `2024-09-below20`: SVG/PDF/비교 PNG 23쪽, overflow 없음
- `2024-09-between20`: SVG/PDF/비교 PNG 24쪽, overflow 35줄
- `2022-10`: SVG/PDF/비교 PNG 18쪽, overflow 없음
- `2022-11-practice`: SVG/PDF/비교 PNG 21쪽, overflow 없음

## 시각 확인

- `output/task1274/2022-10/compare/compare_016.png`
- `2022-10` 16쪽에서 문30 제목과 첫 본문 줄이 PDF처럼 왼쪽 단 하단에 남고, 오른쪽 단은 `조건 (가)에서`로 시작한다.

## 다음 후보

- 남은 overflow는 `2024-09-between20` 35줄이다.
- 첫 재현은 14쪽 왼쪽 단 `pi=714` 이후 묶음이다.
