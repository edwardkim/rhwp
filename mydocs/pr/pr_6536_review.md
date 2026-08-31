# PR #6536 검토 - page-anchored occupied block stored vpos

- 검토일: 2026-09-01
- 작성자: `planet6897`
- base: `devel` (`upstream/devel@891e395bb`)
- 원 PR head: `8e4269db82cae5a45115f332c2fb80a467a45f32`
- 통합 commit: `b8041f23c`
- 상태: 변경 요청

## 범위

- 쪽-앵커 `TopAndBottom` 자리차지 블록 자신의 저장 `vpos`가 절대좌표 산물일 때, 이를 본문 흐름 동기화에 쓰지 않는다.
- `36404612_page_anchored_footer_block.hwpx` fixture와 1페이지 회귀 테스트를 추가한다.

## 발견 사항

### P1. 1페이지를 맞추는 대신 본문과 표의 문서 순서가 뒤집힌다

- [src/renderer/typeset.rs:22779](/home/tsjang/rhwp/src/renderer/typeset.rs#L22779)는 쪽-앵커 블록이면 `sync_h` 동기화를 건너뛴다. 이로써 빈 페이지는 제거하지만 host 문단의 나머지 본문과 표의 상대 흐름 순서는 보장하지 않는다.
- Hancom 2020 기준 PDF p1에서는 `2.` 문단이 본문 표 **앞**에 있다. 통합본 `rhwp` p1에서는 같은 문단이 표 **뒤**로 이동했다. 페이지 수는 모두 1이지만, 문서 읽기 순서가 달라져 사용자 출력의 의미와 레이아웃이 바뀐다.
- [p1 review 패널](assets/pr_6536_issue6535_p1_review.png)에서 좌측 `rhwp`와 중앙 Hancom PDF를 비교할 수 있다. visual sweep은 flagged `0`이지만, 현 자동 규칙이 이 문단-표 순서 교차를 검출하지 못한 경우다.
- [tests/cases/issue_6535_page_anchored_block_keeps_page.rs:38](/home/tsjang/rhwp/tests/cases/issue_6535_page_anchored_block_keeps_page.rs#L38)는 `page_count == 1`만 확인하고, [같은 파일:47](/home/tsjang/rhwp/tests/cases/issue_6535_page_anchored_block_keeps_page.rs#L47)는 표 개수만 확인한다. 따라서 표가 존재하더라도 본문 뒤로 앞질러 배치되는 회귀를 놓친다.

## 검증 증적

- `issue_6535_page_anchored_block_keeps_page`는 `release-test` 종료 코드 `0`으로 통과했지만, 위 시각 오류를 포착하지 못한다.
- Hancom 2020 direct-dll-host PDF: `pdf/pr_6536_issue6535_p1_2020.pdf`, SHA-256 `d5a4a5f8702937d835aba7111c1c72dbbdfed6297c6d1ae3eff23ae656e8c66b`.
- visual sweep: physical p1 single-page fallback, pixel match `92.48729%`, ink match `16.26395%`, flagged `0`; 사람 검토 결과는 불합격이다.
- Rust format, native/WASM/workspace/all-target Clippy, workspace build와 full nextest는 통과했다.

## 요청 변경

- 쪽 수와 표 존재 외에, fixture의 `2.` 본문과 footer/표의 상대 y-order 또는 render-tree 순서를 Hancom 기준으로 고정하는 regression을 추가한다.
- 동기화 예외가 문단의 후속 본문을 표 뒤로 보내지 않도록 placement/flow 소비 조건을 보정한다.
- 수정 뒤 최신 head에서 CI와 Hancom p1 visual sweep을 다시 제시한다.

