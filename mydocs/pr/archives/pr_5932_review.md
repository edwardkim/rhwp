---
kind: pr-review
status: trailing-docs-pending-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-23
---

# PR #5932 검토 - HWP5 마지막 저장 한컴오피스 버전

## 접수 메타데이터

| 항목 | 작성 시점 확인값 |
| --- | --- |
| PR / 작성자 | [#5932](https://github.com/edwardkim/rhwp/pull/5932) / [@jangster77](https://github.com/jangster77) |
| 관련 issue | [#5931](https://github.com/edwardkim/rhwp/issues/5931) |
| base / code candidate | `devel` `2078a2629c0d1cfd85437383cbbfb7b72418fe7b` / `f78fcad44abc8aa77dc7bea34b524f9c244407bb` |
| 변경 규모 | 8 files, +227 / -10 |
| 작성 시점 상태 | non-draft, `MERGEABLE`, `BLOCKED` (GitHub checks 대기) |
| reviewer | 작성자 본인 self-review, 별도 reviewer 미지정 |

GitHub mergeability와 CI 상태는 작성 시점 참고값이다. 이 trailing 기록 commit의 최신 head가
required check를 통과하고 작업지시자가 승인한 뒤에만 merge한다.

## 변경과 판단

- HWP5 FileHeader의 `5.1.1.0`은 2022와 2024 모두에서 나타나므로 저장 제품 연도 판정 근거로 쓰지
  않는다.
- HWP5 `HwpSummaryInformation.revisionNumber`의 주버전 12와 13을 각각
  `hancom-office-2022`, `hancom-office-2024`로 반환한다. 10은 2018로 함께 식별한다.
- `info --json`, MCP `hwp_info`, capabilities, CLI 매뉴얼과 에이전트 지식지도가
  `lastSavedWith:{product,version,confidence}` 계약을 같이 선언한다.
- HWP3/HWPX, 요약정보 누락·손상, 알 수 없는 주버전은 제품 연도를 추정하지 않는다. 전자는
  `lastSavedWith:null`, 후자는 `product:null`이다.
- 이 값은 원 작성 제품이 아니라 수정 가능한 마지막 저장 메타데이터다.

renderer, typeset, layout, fixture, 기준 PDF는 바뀌지 않았다. 이번 변경은 메타데이터 읽기와 JSON
계약이므로 visual sweep은 요구하지 않았다.

## 로컬 검증

- `node scripts/run-rust-test.mjs info_hancom_save_version_contract -- --locked --cargo-profile release-test --target-dir target/pr-review`를 통과했다.
  - 저장소 표본의 HWP 2018/2022/2024는 각각 다른 `lastSavedWith.product`와 버전을 반환했다.
  - HWP3와 HWPX는 `lastSavedWith:null`을 반환했다.
- `node scripts/run-rust-test.mjs knowledge_map_field_dictionary_contract -- --locked --cargo-profile release-test --target-dir target/pr-review`를 통과했다.
- `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast`:
  `8173 passed, 5 slow, 39 skipped` (217.261s).
- `cargo clippy --locked --all-targets --target-dir target/pr-review -- -D warnings`,
  `cargo fmt --all -- --check`, `git diff --check`를 통과했다.
- `node scripts/rust-test-suite-manifest.mjs --prepare` 및 `--check`를 통과했다.
- `node scripts/rust-unit-test-tiers.mjs --check`를 통과했다.

모든 Cargo 검증은 `--locked`와 공유 검토 산출물 `target/pr-review`로 실행했다. 파생 harness와
manifest는 검증에만 사용했고 PR diff에 포함하지 않는다.

## 발견 사항과 보정

초기 전체 회귀에서 새 capabilities `recordFields`의 `lastSavedWith`가 에이전트 지식 지도 전수 사전에
누락된 것을 발견했다. `mydocs/manual/agent_knowledge_map.md`의 필드 행과 합계를 325/333으로 보정한
뒤 해당 계약 및 전체 회귀를 다시 실행해 통과했다.

## 최종 판정

**수용 권고, trailing CI 대기.** 2022와 2024를 FileHeader로 추정하지 않고 실제 마지막 저장
메타데이터로 구분하며, 불확실한 입력에는 제품 연도를 부여하지 않는다. 최신 trailing head의 GitHub
Actions가 모두 성공하고 작업지시자 승인을 받은 뒤 merge한다.
