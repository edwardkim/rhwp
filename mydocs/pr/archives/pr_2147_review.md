# PR #2147 검토 - Issue #2145 개요번호 ^n/^N 레벨 경로 자동코드

- PR: https://github.com/edwardkim/rhwp/pull/2147
- 작성자: `planet6897`
- base: `devel`
- 원 head: `841d3ab0c7d8fb02bef574c8834590988f9785d7`
- 체리픽 커밋: `9a32c90d4`
- 포함 README: 없음

## 결론

blocking finding 없음. 개요번호 format 문자열에서 `^n`/`^N`을 현재 레벨까지의
경로 번호로 확장하는 변경은 테스트와 구현이 일치한다.

## 변경 검토

- `src/renderer/layout/utils.rs`
  - `expand_numbering_format`에 `current_level`을 추가
  - `^n`/`^N`을 `1.2.3` 형태의 레벨 경로로 확장
- `src/renderer/layout/paragraph_layout.rs`
  - caller에서 현재 paragraph level 전달
- `src/renderer/layout/tests.rs`
  - digit, hangul, mixed format, level path 테스트 보강

## 검증

- reviewer assign 완료
- 체리픽 충돌 없음
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --lib expand_numbering_format -- --nocapture`
  - 4 passed
- 누적 브랜치 검증:
  - `git diff --check upstream/devel...HEAD` pass
  - `cargo fmt --check` pass
  - `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests` pass
  - `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings` pass

## 리스크

- 원 PR 본문상 실제 재현 문서/PDF는 포함되지 않았고, private fixture 기반 설명이 있다.
- 이번 검증은 단위 테스트 기반이다. 문서 렌더의 시각 기준 PDF 비교는 수행하지 않았다.

## 권고

누적 체리픽 PR에 포함 가능하다. 향후 번호/개요 렌더 PR에서는 가능한 경우 재현 HWP/HWPX와
기준 PDF를 같이 첨부하도록 권고한다.
