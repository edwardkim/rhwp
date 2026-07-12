# PR #1911 Review Impl — task 1655: HWPX 수식 flowWithText 보존

## Stage 1. 사전 확인

완료.

- #1655는 `OPEN` 상태였고, 같은 범위를 처리하는 열린 PR은 없었다.
- `Equation.common`은 `CommonObjAttr`를 보유하므로 `flow_with_text`를 표현할 수 있다.
- HWPX parser는 수식의 `hp:pos@flowWithText`를 `CommonObjAttr.flow_with_text`로 읽는다.
- serializer의 `render_equation`에는 `flowWithText="1"` 하드코딩이 남아 있었다.
- roundtrip diff gate는 표의 `flowWithText`만 비교하고 수식 arm에는 누락되어 있었다.

## Stage 2. 구현

완료.

- `render_equation`이 `Equation.common.flow_with_text`를 `flowWithText` 값으로 방출하도록 수정했다.
- 수식 roundtrip 테스트 `task1655_equation_flow_with_text_roundtrips`를 추가했다.
- `diff_documents`의 `Control::Equation` arm에 `diff_flow_with_text` 호출을 추가했다.
- gate 테스트 `task1655_equation_flow_with_text_in_gate`를 추가했다.

## Stage 3. 검증

완료.

- `env CARGO_INCREMENTAL=0 cargo test --lib task1655 -- --nocapture`: 통과, 2 passed
- `cargo fmt --check`: 통과
- `env CARGO_INCREMENTAL=0 cargo test --lib serializer::hwpx -- --nocapture`: 통과, 275 passed
- `git diff --check`: 통과
- `env CARGO_INCREMENTAL=0 cargo clippy --lib -- -D warnings`: 통과

## Stage 4. merge 전 확인

대기.

- PR head 최신 커밋 기준 GitHub Actions 통과 확인
- 작업지시자 merge 승인 확인
- merge 후 #1655 auto-close 여부 확인 및 후속 코멘트 수행
- merge 후 브랜치 정리

