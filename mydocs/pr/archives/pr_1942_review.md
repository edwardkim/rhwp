# PR #1942 리뷰

## 메타

| 항목 | 내용 |
| --- | --- |
| PR | #1942 |
| 작성자 | planet6897 |
| 제목 | Issue #1932: UTF-16/UTF-8 관용 디코딩 폴백 — 부분 손상 실문서 로드 |
| base | devel |
| 문서 작성 시점 참고값 | mergeable: MERGEABLE, mergeStateStatus: BEHIND |
| 변경 규모 | 2 files, +70/-8 |

## 변경 범위

- `src/parser/byte_reader.rs`에서 UTF-16 문자열을 `from_utf16_lossy` 기반으로 읽어 lone surrogate로 인한 전체 로드 실패를 완화한다.
- `src/parser/hwpx/reader.rs`에서 UTF-8 XML 엔트리 읽기 실패 시 lossy 폴백을 적용한다.
- PR 본문에서 암호화 HWPX 2건은 인코딩 손상이 아니라는 점을 재분류하고, 복호화는 별도 축으로 남긴다.

## visual sweep 판정

- 렌더링/레이아웃 코드 변경은 아니다.
- 문서 로드 실패 완화가 목적이므로 parser unit test와 라운드트립/CI 검증으로 판단한다.

## 로컬 검증

누적 검토 브랜치 `review/planet-1940-1960`에서 오래된 순서로 cherry-pick했다.

- `git diff --check upstream/devel...HEAD`: 통과
- `cargo fmt --check`: 통과
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --lib`: 통과, 2123 passed / 0 failed / 6 ignored
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`: 통과
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`: 통과

## 검토 메모

- #1940과 같은 `reader.rs` 테스트 영역에서 충돌이 있었으나, 두 PR의 테스트가 독립이라 모두 보존했다.
- lossy 폴백은 손상 문자열을 U+FFFD로 치환해 문서 전체 거부를 피하는 방향이며, 암호화 문서는 #1958에서 별도 분류한다.

## 결론

누적 merge 후보로 판단한다. #1940 이후 순서로 merge하는 것이 충돌 위험이 가장 낮다.
