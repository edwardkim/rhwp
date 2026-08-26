---
kind: pr-review
status: review-complete-pending-merge
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-20
---

# PR #5761 검토 - rhwp-q-more 생성 probe 공통화

## 접수 메타데이터

| 항목 | 검토 기록 |
| --- | --- |
| PR / 작성자 | [#5761](https://github.com/edwardkim/rhwp/pull/5761) / `jangster77` self-review |
| 관련 issue | [#5760](https://github.com/edwardkim/rhwp/issues/5760) |
| base / code candidate | `devel` / `b9a56a11d288a9d4737827c10210131577878342` |
| source branch | `task_m100_5760` |
| 변경 규모 | 53 files, +364 / -890396 |
| 로컬 검토 기준 | `upstream/devel@bba0ba2d2deb4a8cd50a75a92a2c454686846afe` 위 rebase, 충돌 없음 |
| 라우팅 | `collaborator_self_merge` + `intake_and_review` + `local_validation` |
| CI 확인 시점 상태 | Open, non-draft, `MERGEABLE/CLEAN`; code candidate `b9a56a11`의 필수 CI와 Rust CodeQL 성공 |

## 변경 범위와 판정

`rhwp-q-more volume-probe`의 이전 구현은 50개 생성 shard에 slot별 280개의 거의 같은 문서 순회
함수를 두었다. 이 PR은 공통 `collect_stats()`가 문서를 한 번 순회해 paragraph, table cell, Control별
합계를 수집하고, 기존의 feature 순서, slot/probe 가중치, `u64` wrapping 산술을 규칙 테이블로
재현한다.

생성 shard 50개, 890,280 LOC를 제거했다. `src/bin/rhwp-q-more`의 Rust source 총량은 후보 적용 전
891,950 LOC에서 1,768 LOC가 됐다. CodeQL 제외나 query 범위 축소는 포함하지 않는다. 따라서 CodeQL이
분석해야 할 Rust AST를 줄이되, 보안 분석의 대상 경로를 임의로 제외하지 않는 변경이다.

`volume-probe`는 외부에 노출된 진단 명령이므로 결과 호환성을 계약으로 고정했다. form, picture,
equation 실제 HWP 3개와 slot 0..49의 기존 결과 150개를 golden 값으로 검증하며, 기존 범위 밖인 slot
50의 usage 거부도 유지한다.

renderer, layout, HWP/HWPX 저장 형식, fixture 또는 시각 출력 경로는 변경하지 않았다. 시각 검증은
적용 대상이 아니다.

## 로컬 검증

- `cargo fmt --all` 및 `cargo fmt --all -- --check`: 통과
- `git diff --check`, `node scripts/rust-test-suite-manifest.mjs --check`,
  `node scripts/rust-unit-test-tiers.mjs --check`: 통과
- `cargo build --locked --profile release-test --target-dir target/pr-review --bin rhwp-q-more`: 통과
- `cargo nextest run --locked --test regression_suite_025 -E 'test(agent_q_more_contract::)' --cargo-profile release-test --target-dir target/pr-review --no-fail-fast`:
  **5 passed**. help/usage와 150개 golden, 범위 밖 slot 거부를 확인했다.
- `cargo clippy --locked --profile release-test --target-dir target/pr-review --bin rhwp-q-more -- -D warnings`: 통과
- `cargo nextest run --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast`:
  **8,010 passed, 3 slow, 39 skipped, exit 0, 197.514초**. 실행 중 Cargo가 동등한 package 항목 순서만
  바꾼 `Cargo.lock` diff는 의존성 변경으로 포함하지 않고 복원했다.

전체 회귀는 rebase 전 코드 후보에서 완료했다. 이후 `upstream/devel`의 CI archive 분리와 운영 기록 두
커밋 위로 충돌 없이 rebase했으며, 사용자 지시에 따라 rebase 뒤에는 동일한 로컬 회귀를 반복하지 않았다.
최신 PR head의 GitHub CI가 이 기준선을 다시 검증한다.

`node scripts/run-rust-test.mjs agent_q_more_contract`는 현재 suite 018을 선택해 0개 테스트로 끝났다.
manifest의 실제 suite 025를 명시한 위 명령으로 5개 계약을 실행했다. 이 runner 매핑 불일치는 이번
CodeQL 원인과 별개이므로 PR 범위에 포함하지 않는다.

## GitHub CI와 CodeQL 실측

code candidate `b9a56a11`의 Build & Test aggregate, Lint, property roundtrip, adapter inter-diff와
Rust CodeQL이 모두 성공했다. CI에서 Native Skia, Frontend unit/package, WASM Build는 변경 경로상
skip됐으며, 현재 PR은 `MERGEABLE/CLEAN`이다.

Rust CodeQL은 동일한 CodeQL `2.26.3`, 36개 Rust query, `build-mode: none` 조건에서 성공했다. 실행 로그의
`Perform CodeQL Analysis`는 19분 5초, Rust job 전체는 20분 2초였다. 생성 shard가 있던 #5735 실행과의
실측 비교는 다음과 같다.

| 구간 | #5735 | #5761 | 변화 |
| --- | ---: | ---: | ---: |
| Rust CodeQL job 전체 | 29분 22초 | 20분 2초 | 9분 20초 단축 (31.8%) |
| `Perform CodeQL Analysis` | 28분 20초 | 19분 5초 | 9분 15초 단축 (32.6%) |
| 추출 및 database finalize | 7분 22초 | 6분 16초 | 1분 6초 단축 (14.9%) |
| 36개 query 실행 | 20분 51초 | 12분 36초 | 8분 15초 단축 (39.6%) |

이번 실행은 Rust 입력 829개 중 827개를 오류 없이 추출했고, extractor 진단 2건이 남았다. 이전 #5735의
성공 추출 876개와 비교하면 49개가 줄었다. `src/bin/rhwp-q-more/gen/s00.rs`~`s49.rs` 제거가 그 주된
원인이며, CodeQL 제외나 query 축소는 없었다. SARIF 업로드가 성공했고 branch 기준 Code Scanning open alert
조회는 0건이었다.

실측은 CodeQL run [#32353468251](https://github.com/edwardkim/rhwp/actions/runs/32353468251) Rust job의
09:22:45-09:41:50 로그를 근거로 한다. query 실행은 09:29:01-09:41:37에 완료됐다.

## 최종 권고

**수용 권고, merge 승인 대기.** 로컬 golden 계약과 전체 release-test 회귀, 최신 code candidate CI와
CodeQL 실측에서 차단 결함은 발견하지 못했다. 작업지시자 승인 후 merge하고, #5760의 실제 close 상태와
merge commit을 확인해 후속 comment 및 branch 정리를 `post_merge.md` 순서로 진행한다.
