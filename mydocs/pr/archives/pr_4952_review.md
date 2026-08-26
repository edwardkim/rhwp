---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-16
---

# PR #4952 검토 - 고아 fieldEnd 문단 보존

## 접수

| 항목 | 기록 |
| --- | --- |
| PR | [#4952](https://github.com/edwardkim/rhwp/pull/4952) |
| 작성자 / source | @planet6897 / `fix/4398-orphan-fieldend` |
| 원 source head | `ca77e38cf48eb2bf6750177cac9fac4b24ea9a13` |
| 기준 devel | `76e407b127c261427854172990bde6b2e1793edf` |
| 가시성 검토 branch | `review/planet6897-20260816-r6` |
| local 적용 commit | `575610644fb8ddfe185daafda7ab394d2e60c9cc` |
| 원 PR 상태 참고값 | `MERGEABLE` / `CLEAN` |

다문단 필드의 종료 마커가 다음 문단에 단독으로 남는 경우, serializer가 이를 빈 문단으로 접어
`PARA_TEXT`와 control mask를 생략하면 `char_count`가 9에서 1로 붕괴한다. 방출 가능한
`orphan_field_ends.begin_ctrl_id != 0` 조건을 `has_content`와 `compute_control_mask`에 같은 기준으로
반영해 record header와 본문 bytes를 일치시킨다.

## 검증

| 범위 | 명령 또는 근거 | 결과 |
| --- | --- | --- |
| HWPX→HWP5→HWPX | `cargo test --profile release-test --target-dir target/pr-review --test issue_4398_orphan_fieldend -- --nocapture` | 1 passed; `char_count=9`, 고아 fieldEnd 보존 |
| 누적 전체 Rust | `cargo nextest run --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast` | 6,514 passed, 38 skipped, 7 slow, 378.554초 |
| 품질 | `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `git diff --check upstream/devel...HEAD` | 통과 |
| 원 source CI | 최신 head의 Build & Test와 필수 분석 job | 성공; CodeQL aggregate는 `NEUTRAL` |

전용 회귀 실행 중 기존 HWP5 CTRL_DATA의 알려진 `ClickHere` 파라미터 손실 경고(#4396)가 표준 오류에
나왔지만, 테스트의 HWP5 재파싱·최종 HWPX 고아 fieldEnd/`char_count` 단언은 모두 성공했다.

## 판단

실제 방출 조건과 문단 존재·mask 판정을 동일하게 만들어 손상된 header/body 조합을 막는다. 최신 devel에
적용할 때 자동 병합됐고 추가 메인터너 보정은 필요하지 않았다. **통합 수용 권고.**
