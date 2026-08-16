---
kind: pr-review
status: code-ci-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-17
---

# PR #4970 검토 - HWP5 BinData 순번 충돌 보정

| 항목 | 기록 |
| --- | --- |
| 원 PR | [#4970](https://github.com/edwardkim/rhwp/pull/4970) · @planet6897 |
| 원 source head | `1200c23921c795e202f17f24801a084eaecc61cf` |
| 보정 code head | `29b7d896339924a7906e10de8e84ccf3a1493797` |
| 기준선 | `upstream/devel@88b44de37b491a47494d2acac708b7b86a082951` |
| 처리 경로 | `collaborator_external_pr` 9.3.1 contributor source 직접 보정 |
| reviewer | @jangster77 |

## 검토 및 메인터너 보정

원 PR은 HWP5 본문의 `bin_data_id`가 1-based `BIN_DATA` 레코드 순번이고 `storage_id`와 별개라는
핵심 진단을 올바르게 반영했다. 다만 `resolve_bin_id`가 direct storage ID를 먼저 조회하고, 그 키가 없을
때만 순번 사상을 사용했다. storage ID 배열이 `[3, 1, 2]`처럼 재배열된 문서에서 순번 `1`은 첫 레코드의
storage `3`을 가리켜야 하지만, 기존 구현은 존재하는 direct key `1`을 먼저 선택해 다른 그림을 참조한다.

메인터너 보정 `29b7d8963`은 모든 실제 BIN_DATA 레코드에 순번→storage 사상을 만들고 이를 직접 조회보다
우선한다. `storage_id=0` Link와 사상이 없는 sparse ID(차트·수동 구성)는 기존 직접 조회를 유지한다. 충돌
순서 `[3, 1, 2]`가 `image3`, `image1`, `image2`로 해소되는 회귀 테스트를 추가해 두 축이 우연히 같은
숫자를 쓰는 경우를 고정했다. contributor의 기존 커밋은 재작성하지 않고 같은 source head 위에만 추가했다.

## 검증

- `cargo test --profile release-test --target-dir target/pr-review --lib hwp5_bin_sequence_precedes_colliding_storage_id -- --nocapture`: 통과
- `cargo test --profile release-test --target-dir target/pr-review --test issue_3893_bindata_sequence_ref -- --nocapture`: 2 passed
- `cargo fmt --check`: 통과
- `cargo nextest run --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast`: 6,386 passed, 38 skipped, 7 slow, 329.093초
- `cargo clippy --all-targets -- -D warnings`: 통과
- GitHub code candidate CI: [Full CI](https://github.com/edwardkim/rhwp/actions/runs/31959203731), [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/31959203598) 통과. Lint, archive 3개, regular 3개, slow shard, Build & Test 및 Rust 분석이 성공했고, 변경 범위 밖 frontend/WASM/Native Skia gate는 정책상 skip됐다.
- 최신 `upstream/devel` merge simulation과 `git diff --check upstream/devel...HEAD`: 통과

코드 후보 `29b7d8963`은 `MERGEABLE`/`CLEAN`이며 **수용 가능**이다. 이 기록은 해당 후보의 녹색 CI 뒤에 추가하는 review-only trailing commit이다.
