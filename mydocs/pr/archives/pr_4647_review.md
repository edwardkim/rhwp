---
kind: pr-review
status: local-pass
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-12
---

# PR #4647 검토 - 문서 열기 압축 해제 예산

## 판정

메인터너 보정 후 로컬 수용. 원 PR은 HWP5 DocInfo·본문의 **압축 해제 결과** 누적 예산과 HWP3
압축 본문 상한을 선택한다. 검토 과정에서 원시 CFB 스트림을 먼저 `Vec`로 읽는 경로는 압축 해제
출력 상한만으로 제한되지 않는다는 공백을 확인했다.

보정은 strict·lenient·비밀번호·배포용 ViewText 경로 모두에서 원시 입력 상한을 먼저 집행하고,
그 뒤 복호화·압축 해제 결과를 기존 누적 예산으로 소비하도록 했다. lenient CFB fallback도 선언된
스트림 길이까지만 FAT/mini-FAT 체인을 읽으므로 손상된 체인의 뒷부분을 불필요하게 materialize하지
않는다. 하위 CFB/crypto API는 전역 제품 정책을 숨기지 않고 caller 제공 상한만 집행한다.

## 검토 기준

- 원격 head: `a274e67e782480c84adee25ffbfab28d559f4356`
- 로컬 누적 검토 브랜치: `review/humdrum00001010-20260812`
- 적용 순서: #4646 다음에 #4647의 6개 commit을 적용했다.
- 충돌 해소: #4646의 thumbnail 회귀를 유지한 채 `src/parser/mod.rs`에 누적 예산 회귀를 추가했다.

## 확인

- `cargo test --profile release-test --target-dir target/pr-review --lib hwp5_open_ -- --nocapture`: 9 passed.
- `cargo test --profile release-test --target-dir target/pr-review --lib open_decompression_stream_readers_enforce_limit -- --nocapture`: 1 passed.
- `cargo nextest run --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast`: exit 0.
- `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `git diff --check` 통과.

## 범위

제한 초과는 빈 섹션이나 preview fallback으로 본문을 계속 열지 않고 명시적 오류가 된다. 문서 열기
경로의 원시 CFB 입력은 기본 257 MiB, 단일 복호화·압축 해제 출력은 256 MiB, 전체 출력은 512 MiB로
제한한다. 별도 BinData와 thumbnail의 정책은 이 문서 열기 누적 예산에 섞지 않으며, raw-record
diagnostics는 자체 이름 붙은 제한을 선택한다.
