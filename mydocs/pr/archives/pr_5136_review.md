---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-17
---

# PR #5136 검토 - 캡션 attr bit29 저장 계약

## 접수

| 항목 | 기록 |
| --- | --- |
| PR | [#5136](https://github.com/edwardkim/rhwp/pull/5136) |
| 작성자 / source | `planet6897` / `fix/hwp3-caption-attr-bit29` |
| 원 source head | `e2df9f9bb20e466300300a86b2ec380a447ce8fe` |
| 기준 | `devel` |
| 통합 검토 branch | `review/planet6897-20260817` |
| 원 PR 상태 | 작성 시점 `OPEN` / `DIRTY`; required checks와 CodeQL은 성공 또는 해당 없음 |
| 규모 | 7 files, +171 / -6 |
| 관련 이슈 | #5126 (작성 시점 `OPEN`) |

## 변경 범위

`serializer/control.rs`의 공통 개체 속성 `attr`에서 실제 캡션 유무와 bit29를 일치시키도록 공통
helper를 추가하고, 표·그림 control header 양쪽에 적용한다. 캡션 fixture와 계약 테스트, overflow
baseline·generated manifest·unit-tier manifest 변경도 함께 포함된다. 캡션이 있는 경우에만 bit29를
세우고 없는 경우에는 지우므로 두 경로의 레코드 스트림 판정이 같은 기준을 사용한다.

## 로컬 적용과 검증

`upstream/devel@d4cf27eeb` 기준 통합 branch에서 #5136의 세 commit을 적용했다. generated manifest
충돌은 #4984·#5172의 fixture 항목을 보존한 뒤 generator로 재생성했고, 최종 통합 head에서 다음을
실행했다.

- `cargo fmt --all -- --check` 통과
- `node scripts/rust-test-suite-manifest.mjs --check` 통과
- `node scripts/rust-unit-test-tiers.mjs --check` 통과
- `cargo nextest run --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast`
  통과: **6538 passed, 38 skipped, 8 slow**
- `cargo clippy --all-targets --target-dir target/pr-review -- -D warnings` 통과
- `git diff --check` 통과
- 신규 `issue_5136_caption_attr_bit29` focused test: 2 passed

serializer와 HWP fixture가 변경되므로 Rust 회귀 검증을 수행했다. renderer·Studio UI 변경은 없어
WASM·브라우저 시각 검증은 적용하지 않았다. source PR에 기록된 한컴 COM 실측은 확인했지만 Linux
검토 서버에서는 COM을 재실행하지 않았다.

## 판단

현재 통합 head에서 캡션 attr 계약과 기존 회귀를 깨는 문제는 발견하지 못했다. 원 PR은 작성 시점에
GitHub에서 `DIRTY`이므로 최종 통합 전 최신 head·mergeable·required checks를 다시 확인해야 한다.
추가 메인터너 코드 보정은 필요하지 않다. **로컬 통합 수용 권고.**
