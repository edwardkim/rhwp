# Task #1261 Stage6 보고 - PR CI hit-test 회귀 보정

## 배경

Stage5 커밋 후 PR #1264 GitHub Actions `Build & Test`를 다시 확인했다.
기존 `issue_1116` 회귀는 해소됐지만 `tests/issue_919_textbox_hit_test.rs`의 다음 테스트가 실패했다.

- `issue_919_textbox_outside_hit_returns_body`

## 원인

글상자 바깥 좌표 `x=400, y=50`은 정확한 본문/셀/글상자 bbox hit가 아니다.
그러나 `hit_test_native`의 마지막 "가장 가까운 줄" fallback이 본문 run뿐 아니라 글상자 내부 run까지 후보로 삼아, 글상자 바깥 클릭을 글상자 첫 문단으로 스냅했다.

## 수정 방향

- 정확한 `hit_cell`, `clicked_cell`, `textbox_hit`, `hit_body` 판정이 모두 실패한 뒤의 fallback은 본문 TextRun만 후보로 삼는다.
- 본문 TextRun이 없는 페이지에서 글상자 바깥을 클릭하면 글상자 내부로 들어가지 않고 페이지 본문 시작 위치를 반환한다.

## 검증 예정

- `cargo test --test issue_919_textbox_hit_test -- --nocapture`
- `cargo test --test issue_1116 -- --nocapture`
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
- `git diff --check`
