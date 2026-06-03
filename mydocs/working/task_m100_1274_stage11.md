# Task 1274 Stage 11

## 대상

- Stage10 전체 sweep 이후 남은 overflow:
  - `2024-09-between20`: 35줄
- 첫 재현:
  - 14쪽 왼쪽 단 `pi=714` 이후
  - 14쪽 오른쪽 단 `pi=750` 이후

## 관찰

- `2024-09-between20`은 PDF와 페이지 수는 24쪽으로 맞지만, 14쪽부터 미주 흐름이 PDF보다 늦다.
- PDF 14쪽 왼쪽 단은 문18 꼬리 뒤 문19가 시작한다.
- rhwp 14쪽 왼쪽 단은 문16/문17/문18이 아래까지 밀려 있고, 문19가 오른쪽 단 상단에서 시작한다.
- overflow는 단순 로그 오탐이 아니라, 미주 사이 20mm 문서에서 누적 간격 또는 vpos 보정이 과하게 적용된 결과로 보인다.
- 13쪽 시작 문단 `pi=662`는 내부 `LINE_SEG` vpos가 첫 줄 뒤로 되감긴다.
- 기존 split 정책은 이 `split=1`을 실제 단 분할로 처리해 13쪽 왼쪽 단을 거의 비워 두었고, 그 결과 14쪽 이후 미주 흐름이 PDF보다 늦어졌다.
- 단순히 `split=1`을 제거하면 13/14쪽은 PDF와 가까워지지만, 뒤쪽에서 미주 사이 20mm 누적 간격이 부족해 전체가 23쪽으로 압축된다.
- `미주 사이` 전체값을 모든 경계에 예약하면 24쪽은 회복되지만, 10쪽 문12 꼬리가 다음 단으로 밀려 회귀가 발생한다.

## 목적

- `2024-09-between20`의 미주 사이 20mm 설정에서 한컴/PDF와 다른 누적 지연 원인을 찾는다.
- 구분선 아래 20mm(`2024-09-below20`)와 기본 간격 문서들은 유지한다.
- 문서/페이지/문항 번호 하드코딩 없이 endnote shape의 between-notes 설정과 vpos/line-spacing 처리 기준으로 보정한다.

## 검증 계획

- 진행 중 자동 테스트는 `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`만 사용한다.
- `cargo build --bin rhwp`
- `python3 scripts/task1274_visual_sweep.py --target 2024-09-between20`
- 필요 시 `python3 scripts/task1274_visual_sweep.py --target 2024-09-below20`로 20mm 관련 회귀를 함께 본다.
- 전체 CI급 테스트는 전체 목표 마지막에만 수행한다.

## 수정

- `typeset.rs`에서 미주 문단 내부 rewind split이 `split=1`이고, 해당 문단이 단 하단 fit 판정으로 먼저 다음 단에 놓인 경우에는 실제 분할을 제거한다.
- 이 보정이 발생한 뒤의 큰 미주 사이 문서에서는, 다음 미주 경계부터 `between_notes` 전체값을 pagination 예약값으로 사용한다.
- 보정 전 경계는 기존처럼 기본 7mm 초과분의 3/4만 예약해 10쪽 문8/문12의 기존 정합을 유지한다.
- 문항 번호나 페이지 번호 조건은 추가하지 않고, `split=1` 내부 rewind 해소 여부와 `between_notes > base_flow`라는 구조 조건만 사용했다.

## 검증 결과

- `cargo fmt`
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
  - 48개 통과
- `cargo build --bin rhwp`
- `python3 scripts/task1274_visual_sweep.py --target 2024-09-between20`
  - SVG/PDF/비교 PNG 24쪽 일치
  - overflow 35줄에서 8줄로 감소
- `python3 scripts/task1274_visual_sweep.py`

전체 sweep 결과:

- `2022-09`: SVG/PDF/비교 PNG 23쪽, overflow 없음
- `2023-09`: SVG/PDF/비교 PNG 20쪽, overflow 없음
- `2024-09-below20`: SVG/PDF/비교 PNG 23쪽, overflow 없음
- `2024-09-between20`: SVG/PDF/비교 PNG 24쪽, overflow 8줄
- `2022-10`: SVG/PDF/비교 PNG 18쪽, overflow 없음
- `2022-11-practice`: SVG/PDF/비교 PNG 21쪽, overflow 없음

## 시각 확인

- `output/task1274/2024-09-between20/compare/compare_013.png`
- `output/task1274/2024-09-between20/compare/compare_014.png`
- `output/task1274/2024-09-between20/compare/compare_017.png`
- `output/task1274/2024-09-between20/compare/compare_018.png`
- `output/task1274/2024-09-between20/compare/compare_023.png`

13쪽과 14쪽의 문15~문21 흐름은 PDF에 가까워졌고, 23쪽은 PDF처럼 문29 시작과 문30 시작이 같은 쪽에 배치된다.

## 다음 후보

- 남은 overflow는 `2024-09-between20` 17쪽 오른쪽 단 `pi=937..939` 8줄이다.
- 비교 PNG상 다음 대상은 18쪽 문29/문30 앞뒤 흐름이며, 문23/문24 꼬리가 오른쪽 단 하단에서 넘는 현상을 별도 Stage에서 보정한다.
