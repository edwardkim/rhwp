# Task 1274 Stage 19 - issue_241 HWPX 도장 host 줄 예약

## 대상

- 테스트: `tests/issue_241.rs`
- 샘플: `samples/hwpx/issue_241.hwpx`
- 위치: 1쪽 하단 `pi=9` 도장 그림 host, `pi=10` 날짜, `pi=11` 서명

## 현상

PR CI에서 `issue_241_hwpx_stamp_host_paragraph_keeps_flow_line_height`가 실패한다.
`upstream/devel`에서도 같은 실패가 재현되지만, task 1274 현재 화면에서도 한컴과 다르게
날짜/서명/도장 하단 배치가 맞지 않는다. 따라서 task 1274 안에서 함께 보정한다.

## 원인

`pi=9`는 텍스트 없는 non-TAC `InFrontOfText` 도장 그림 host 문단이다.
렌더러는 phantom 빈 `TextLine`과 overflow 오탐을 막기 위해 텍스트 없는 non-TAC 그림
host 문단의 `layout_paragraph`를 건너뛰고 있다. 이 처리 자체는 필요하지만, HWPX의
`InFrontOfText + vert_rel_to=Para` host는 한컴처럼 빈 텍스트를 그리지 않더라도 host
문단의 line advance는 본문 흐름에 예약해야 한다. 현재는 그 advance도 함께 빠져
후속 `pi=10` 날짜 문단이 도장 host 위치로 당겨진다.

## 수정 방향

- 텍스트 없는 non-TAC 그림/도형 host의 phantom text 렌더 생략은 유지한다.
- 단, `InFrontOfText`이고 `vert_rel_to=Para`인 host 문단은 원본 line segment 높이만큼
  `y_offset`을 진행시킨다.
- `BehindText`는 기존처럼 line advance를 예약하지 않는다. 기존 HWP5 배경 로고 문서에서
  빈 첫 문단 advance가 끼면 표가 아래로 밀리는 회귀가 있기 때문이다.

## 검증 계획

- `cargo test --test issue_241 -- --nocapture`
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
- `python3 scripts/task1274_visual_sweep.py --target 2022-11-practice`

## 검증 결과

- `cargo test --test issue_241 -- --nocapture`
  - 2개 테스트 통과.
  - 도장 overlay 위치 기준 테스트와 host paragraph line advance 테스트가 모두 통과했다.
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
  - 49개 테스트 통과.
  - task 1274의 2022-11 non-TAC 그림 host phantom overflow 회귀도 통과했다.
- `git diff --check`
  - 통과.
- `cargo fmt --all -- --check`
  - 통과.
- `cargo clippy -- -D warnings`
  - 통과.
- `python3 scripts/task1274_visual_sweep.py --target 2022-11-practice`
  - SVG/PDF 페이지 수 21/21.
  - `manifest.json`의 `overflow_lines` 0건.
