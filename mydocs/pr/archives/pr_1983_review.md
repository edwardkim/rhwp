# PR #1983 리뷰 - #1981 BinData 빈 확장자 보존 + #1982 DRM/빈파일 감지 분류

## 메타

| 항목 | 내용 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/1983 |
| 제목 | 10k 서베이 수정: #1981 BinData 빈 확장자 보존 + #1982 DRM/빈파일 감지 분류 |
| 작성자 | planet6897 |
| base | `devel` |
| 문서 작성 시점 head SHA | `dfc28ff34028a96181dfd3881ce2fdf8c9ba1dd4` |
| 원 commits | `096accac66f7f9366ac8f3d7b0d4bdaaca7a333e`, `dfc28ff34028a96181dfd3881ce2fdf8c9ba1dd4` |
| 체리픽 commits | `1e320494a`, `03e35dafa` |
| 규모 | 4 files, +105 / -6 |
| 주요 변경 파일 | `src/serializer/hwpx/context.rs`, `src/parser/mod.rs`, `src/diagnostics/*_roundtrip_batch.rs` |
| 처리 방식 | planet6897 PR 8건 통합 체리픽 |

## 변경 범위

- #1981: BinData의 빈 확장자를 HWPX 재직렬화 시 `.bin`으로 치환하지 않도록 보존한다.
- #1982: DRM/보안 컨테이너와 빈 파일을 전용 감지 분류로 구분한다.
- 렌더 출력보다 파서/직렬화/진단 분류 충실도에 초점이 있다.

## 체리픽 검토

- 적용 순서: 3/8
- 충돌: 없음
- 선행 PR 의존: 없음
- PR 내부 2개 commit을 순서대로 적용했다.

## 시각 검증

직접 렌더 레이아웃을 변경하는 PR이 아니므로 visual sweep 대상은 아니다. 검증은 parser/serializer 단위 테스트와 전체 회귀 테스트로 수행했다.

## 로컬 검증

- `env CARGO_INCREMENTAL=0 cargo test --lib issue1981`: 1 passed
- `env CARGO_INCREMENTAL=0 cargo test --lib issue1982`: 2 passed
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`: 통과
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`: 통과
- `cargo fmt --check`: 통과
- `git diff --check`: 통과

## 검토 결과

변경 범위가 parser/serializer/diagnostics로 한정되어 있고 targeted test가 통과했다. 최종 권고는 통합 PR merge 후보다.

