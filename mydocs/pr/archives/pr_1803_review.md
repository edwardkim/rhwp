# PR #1803 리뷰 — Task #1793 BULLET 직렬화 char_shape_id / NBSP 코드 수정

## PR 메타

| 항목 | 내용 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/1803 |
| 작성자 | @planet6897 |
| base / head | `devel` / `planet6897:pr/devel-1793` |
| 관련 이슈 | #1793 |
| reviewer assign | @jangster77 요청 완료 |
| 적용 방식 | 비시리즈·샘플 미포함 PR 누적 cherry-pick |

## 변경 범위

- `src/model/style.rs`
- `src/parser/doc_info.rs`
- `src/serializer/body_text.rs`
- `src/serializer/doc_info.rs`
- `src/serializer/doc_info/tests.rs`
- 관련 계획/보고/오늘할일 문서

BULLET 레코드 직렬화에서 `char_shape_id` 4바이트를 보존하고, NBSP(U+00A0)를 HWP 코드 30(0x1E)으로
직렬화하도록 수정한다.

## 검토 결과

parser 는 이미 읽던 `char_shape_id`를 모델에 보존하고, serializer 는 문단 머리 정보 12바이트를 맞춘다.
누락 시 bullet_char offset 이 밀리는 문제를 테스트가 직접 확인한다. NBSP 는 기존 0x18 하이픈 코드 오기를
0x1E 로 정정해 재파싱 시 `-`로 손상되는 경로를 막는다.

## 검증

- 누적 cherry-pick 충돌 없음
- `git diff --check upstream/devel..HEAD`: 통과
- `cargo fmt --check`: 통과
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`: 통과
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`: 통과

## 결론

HWP record layout 과 NBSP 코드 매핑을 테스트로 고정했다. merge 후보로 판단한다.
