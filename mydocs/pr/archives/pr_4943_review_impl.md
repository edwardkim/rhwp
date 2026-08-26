---
kind: pr-review-implementation
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-16
---

# PR #4943~#4952 planet6897 누적 검토 적용 기록

이 기록은 다수 PR의 누적 적용 순서만 설명한다. 개별 변경의 검토·위험·수용 판단은
[#4943](pr_4943_review.md), [#4945](pr_4945_review.md), [#4946](pr_4946_review.md),
[#4948](pr_4948_review.md), [#4952](pr_4952_review.md) 기록이 정본이다.

## 기준과 적용

| 항목 | 기록 |
| --- | --- |
| 기준 devel | `76e407b127c261427854172990bde6b2e1793edf` |
| 가시성 검토 branch | `review/planet6897-20260816-r6` |
| 누적 후보 head | `575610644fb8ddfe185daafda7ab394d2e60c9cc` |
| 적용 방식 | 최신 원 PR head를 fetch한 뒤 오래된 번호 순서로 cherry-pick. 원 contributor branch는 rewrite하거나 직접 push하지 않음 |

| 순서 | 원 PR / source commit | local commit | 처리 |
| --- | --- | --- | --- |
| 1 | #4943 `7cef482a0741100ea86b009dd7f81bcaf87cd744` | 적용 생략 | DocInfo raw provenance는 이미 `upstream/devel`의 #4941에 포함되어 빈 cherry-pick이 됨 |
| 2 | #4943 `c2b5042c30fd8a99ec76d3c80325abe55e739b08` | `ce5a99ba28c2223e1303c6eae7f21e61aae53ba8` | 본문 Section·하위 컨트롤 raw provenance 적용 |
| 3 | #4945 `6a985214ff89f5552a736e1243895fec7bea4f35` | `04bc2a90d957d28ad3c51117960d84cb20bbb772` | booleanParam lexical 표기 보존 적용 |
| 4 | #4946 `62fe2dc775f1b2aaa2d6bb6ac16121246af51c2b` | `328e6e886aeb97d3d61164113a427d9c325bd9ac` | HWP5 글꼴 기본 이름 실측표 적용 |
| 5 | #4948 `43acd85f1705ded12f859e217e77b508a282be92` | `83d2996df4c4d8baaf0fd5dcb44f9b11ddf66d76` | 구역 시작 `secd`/`cold` 순서 보존 적용 |
| 6 | #4952 `ca77e38cf48eb2bf6750177cac9fac4b24ea9a13` | `575610644fb8ddfe185daafda7ab394d2e60c9cc` | 고아 `fieldEnd` 문단의 HWP5 왕복 보존 적용 |

원 #4943은 GitHub에서 당시 `CONFLICTING`/`DIRTY`였지만, 이는 오래된 source base 때문이다. 위 기준선에서
본문 provenance commit은 자동 병합됐으며 수동 conflict 해소나 메인터너 코드 보정은 없었다.

## 완료 검증

1. 원 PR 5개의 source head를 fetch해 기록한 SHA와 일치함을 재확인했다.
2. 각 전용 회귀 `issue_4488_4495_body_provenance` 6건, `issue4437_boolean_lexical_round_trips_verbatim`,
   `issue4898_face_name_fills_measured_default_font_name`, `issue_3367_secd_cold_order`,
   `issue_4398_orphan_fieldend`가 모두 통과했다.
3. `samples/hwpx_sample2.hwpx`를 HWP5로 실제 변환해 `--verify --json`의 `identical:true`,
   `diffCount:0`을 확인했다.
4. 누적 후보에서 `cargo nextest run --cargo-profile release-test --target-dir target/pr-review --tests
   --test-threads 12 --no-fail-fast`를 실행해 6,514 passed, 38 skipped, 7 slow, 378.554초로 종료했다.
5. `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `git diff --check`를 통과했다.

## 다음 단계

1. 이 누적 후보와 개별 archive review 기록을 하나의 `devel` 대상 통합 PR로 올린다.
2. merge 전에는 원 PR별 최신 source head, mergeable 상태, 필수 GitHub Actions를 다시 확인한다.
3. 작업지시자 merge 승인 뒤 통합 PR만 병합하고, 원 PR에는 통합 반영 사유를 남겨 close한다.
