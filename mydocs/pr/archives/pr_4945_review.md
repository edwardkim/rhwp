---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-16
---

# PR #4945 검토 - booleanParam lexical 표기 보존

## 접수

| 항목 | 기록 |
| --- | --- |
| PR | [#4945](https://github.com/edwardkim/rhwp/pull/4945) |
| 작성자 / source | @planet6897 / `fix/4437-boolean-lexical` |
| 원 source head | `6a985214ff89f5552a736e1243895fec7bea4f35` |
| 기준 devel | `76e407b127c261427854172990bde6b2e1793edf` |
| 가시성 검토 branch | `review/planet6897-20260816-r6` |
| local 적용 commit | `04bc2a90d957d28ad3c51117960d84cb20bbb772` |
| 원 PR 상태 참고값 | `MERGEABLE` / `CLEAN` |

HWPX `booleanParam`의 의미값만 보존하면 `false`/`true`와 `0`/`1`의 원본 lexical 표기가 정규화되어
바이트 왕복이 달라진다. 이 변경은 유효한 xs:boolean 표기만 보관해 재방출하고, 프로그램이 새로 만드는
값은 기존 `0`/`1` 정규화를 유지한다.

## 검증

| 범위 | 명령 또는 근거 | 결과 |
| --- | --- | --- |
| 모델 단위 | `cargo test --profile release-test --target-dir target/pr-review --lib issue4437_boolean_lexical_round_trips_verbatim -- --nocapture` | 1 passed |
| 누적 전체 Rust | `cargo nextest run --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast` | 6,514 passed, 38 skipped, 7 slow, 378.554초 |
| 품질 | `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `git diff --check upstream/devel...HEAD` | 통과 |
| 원 source CI | 최신 head의 Build & Test와 필수 분석 job | 성공; CodeQL aggregate는 `NEUTRAL` |

모델·HWPX parser/serializer의 lexical 보존 변경이며 레이아웃이나 paint 경로를 바꾸지 않는다. 별도 시각
대조 대신 유효 표기 재렌더와 프로그램 생성값 정규화의 단위 계약 및 전체 회귀를 적용했다.

## 판단

입력 lexical과 의미값의 경계를 명확히 지키며 기존 생성값의 canonical 동작도 유지한다. 최신 누적 후보에서
추가 메인터너 보정이나 충돌 해소가 필요하지 않았다. **통합 수용 권고.**
