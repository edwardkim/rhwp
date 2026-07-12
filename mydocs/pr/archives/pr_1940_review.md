# PR #1940 리뷰

## 메타

| 항목 | 내용 |
| --- | --- |
| PR | #1940 |
| 작성자 | planet6897 |
| 제목 | Issue #1917 잔여(XML 축): MAX_XML_SIZE 32→256MB — 75MB section XML 실문서 수용 |
| base | devel |
| 문서 작성 시점 참고값 | mergeable: MERGEABLE, mergeStateStatus: BEHIND |
| 변경 규모 | 1 files, +36/-3 |

## 변경 범위

- `src/parser/hwpx/reader.rs`의 HWPX XML 엔트리 상한을 32MB에서 256MB로 상향한다.
- BinData 한도 상향 뒤에도 남아 있던 대형 `Contents/section*.xml` 실문서 로드 실패 축을 처리한다.
- zip-bomb 방어를 없애지 않고 절대 상한만 넓히며, 대형 정상 XML 수용 테스트를 추가했다.

## visual sweep 판정

- 렌더러 출력 경로 변경은 아니다.
- 파서의 HWPX ZIP/XML 읽기 한도 변경이므로 visual sweep 대상이 아니라 로드/라운드트립 게이트와 회귀 테스트가 핵심이다.

## 로컬 검증

누적 검토 브랜치 `review/planet-1940-1960`에서 #1940, #1942, #1953, #1957, #1958, #1959, #1960을 오래된 순서로 cherry-pick해 확인했다.

- `git diff --check upstream/devel...HEAD`: 통과
- `cargo fmt --check`: 통과
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --lib`: 통과, 2123 passed / 0 failed / 6 ignored
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`: 통과
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`: 통과

## 검토 메모

- #1942와 같은 파일의 테스트 추가 위치에서 충돌이 있었지만, 두 PR의 테스트를 모두 보존하는 방식으로 해결했다.
- 변경은 대형 정상 XML 수용에 한정되어 있고, 기존 bomb 거부 게이트를 유지한다.

## 결론

오래된 순서 누적 merge 후보로 판단한다. merge 전에는 PR head 최신 커밋 기준 GitHub Actions가 여전히 통과 상태인지 개별 재확인한다.
