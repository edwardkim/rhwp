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


## 메인터너 보정 결과 (2026-09-01)

- 판정: **메인터너 보정 후 수용 가능**
- 원 contributor PR head: `8e4269db82cae5a45115f332c2fb80a467a45f32`
- 검토 브랜치 적용 commit: `b8041f23c`
- 메인터너 보정 commit: `0ff2e25b6` (`fix: 양수 offset 빈 host 표 앞 본문 흐름을 복원`)
- 원 contributor head는 직접 병합 대상으로 바꾸지 않는다. `review/planet6897-6514-6536-20260831`의 보정 commit을 포함한 후속 integration PR만 원격 검토 대상으로 한다.

### 보정 내용

양수 vertical offset을 가진 빈 host paragraph의 flow-with-text 표가 뒤따르는, 저장 line segment가 없는 생성 본문을 앞질러 배치하던 경로를 제한적으로 보정했다. 해당 본문을 표 앞의 저장 anchor로 되돌리고 표의 anchor는 유지해 Hancom의 문서 순서인 `2.` 본문 -> `연번` 표 -> `끝.`을 복원했다. 회귀 테스트는 페이지 수 1, 표 수, 그리고 세 대상의 세로 순서를 함께 고정한다.

### 검증

- Rust lint 묶음: `rust-test-suite-manifest --prepare/check`, `cargo fmt --all -- --check`, 일반/WASM/workspace Clippy, workspace build 모두 통과.
- focused: `issue_6535_page_anchored_block_stays_on_its_page` 통과.
- Native Skia: lib `3,946 passed, 13 ignored`; `issue_2225_missing_picture_placeholder` 2/2, `render_p37_direct_pdf_export` 4/4 통과.
- WASM: `scripts/wasm-pack-locked.sh --target web --out-dir pkg` 통과.
- 전체: `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast` 결과 `8,912 passed, 46 skipped`.

### Hancom 2020 PDF 시각 증적

- 기준 PDF: [`pr_6536_issue6535_p1_2020.pdf`](assets/pr_6536_issue6535_p1_2020.pdf), SHA-256 `d5a4a5f8702937d835aba7111c1c72dbbdfed6297c6d1ae3eff23ae656e8c66b`.
- 입력: `samples/issue6535/36404612_page_anchored_footer_block.hwpx`, physical page 1.
- 재실행: `pr_6536_issue6535_p1_maintainer`; SVG/PDF/render tree 각 1페이지, compared/review 1페이지, flagged 0.
- 자동 지표: pixel match `92.51028%`, ink match `16.28056%`. 서체 raster 차이에 민감한 보조 지표라 순서 판정의 단독 근거로 사용하지 않았다.
- 사람 검토: [`pr_6536_issue6535_p1_maintainer_review.png`](assets/pr_6536_issue6535_p1_maintainer_review.png)에서 rhwp와 Hancom 2020 PDF 모두 `2.` 본문 아래에 `연번` 표, 그 아래에 `끝.`이 한 페이지에 배치됨을 확인했다. 기존 #6536 footer 결과도 유지된다.
- 도구 실행 기록: visual sweep는 현재 스크립트가 선택한 `target/pr-review/debug/rhwp`를 사용했으며, 같은 source revision의 release-test focused/Native Skia/전체 nextest 검증을 별도로 통과했다.

### 병합 후 contributor PR comment 계획

후속 integration PR의 최신 head CI와 mergeability를 확인하고 승인된 원격 병합이 끝난 뒤에만 contributor PR에 다음을 게시한다.

- [시각 검증 절차](../manual/pr_review/visual_fixture_evidence.md)와 physical page 1, flagged 0, pixel/ink 지표를 링크한다.
- 병합된 자산 raw URL은 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr_6536_issue6535_p1_maintainer_review.png` 형식을 사용한다.
- 자산이 `devel`에 실제 존재하는지 API로 재확인한 뒤 게시한다.

원격 push, integration PR 생성, CI 확인, 병합, contributor comment는 아직 수행하지 않았다.
