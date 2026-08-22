---
kind: pr-review
status: trailing-docs-pending-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-23
---

# PR #5935 검토 - 한컴오피스 2010 마지막 저장 버전 식별

## 접수 메타데이터

| 항목 | 작성 시점 확인값 |
| --- | --- |
| PR / 작성자 | [#5935](https://github.com/edwardkim/rhwp/pull/5935) / [@jangster77](https://github.com/jangster77) |
| 관련 issue | [#5934](https://github.com/edwardkim/rhwp/issues/5934) |
| base / code candidate | `devel` `61e4390436644d306bd2e0402bc24d841066b671` / `14432bd356a8d7a3f95049a580bfc2711313cca2` |
| 변경 규모 | 4 files, +5 / -2 |
| 작성 시점 상태 | non-draft, `MERGEABLE`, `BLOCKED` (GitHub required checks 진행 중) |
| reviewer | 작성자 본인 self-review, 별도 reviewer 미지정 |

GitHub mergeability와 CI 상태는 작성 시점 참고값이다. 이 trailing 기록 commit의 최신 head가
required check를 통과하고 작업지시자가 승인한 뒤에만 merge한다.

## 변경과 판단

- HWP5 `HwpSummaryInformation.revisionNumber`의 주 버전 8을 `hancom-office-2010`으로 분류한다.
- 기존 추적 fixture `samples/basic/Hyper(hwp2010).hwp`의 `8.0.0.460`을 `info --json` 계약에 추가했다.
- 기존 2018/2022/2024 분류와 알 수 없는 주 버전의 `product:null` 동작은 유지한다.
- CLI 매뉴얼과 에이전트 지식 지도의 `lastSavedWith` 설명을 2010 범위까지 갱신했다.

renderer, layout, sample 파일, 기준 PDF는 바뀌지 않았다. 기존 fixture를 읽는 JSON 계약 변경이므로
시각 검증은 요구하지 않았다.

## 로컬 검증

- `node scripts/run-rust-test.mjs info_hancom_save_version_contract -- --locked --cargo-profile release-test --target-dir target/pr-review`를 통과했다.
  - HWP 2010/2018/2022/2024 fixture는 각각 대응하는 `lastSavedWith.product`와 버전을 반환했다.
  - HWP3와 HWPX는 `lastSavedWith:null`을 유지했다.
- 한컴오피스 2010으로 새로 저장한 별도 HWP 표본도 `8.0.0.466`과 `hancom-office-2010`을 반환했다.
- `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast`를 exit code 0으로 통과했다.
- `cargo clippy --locked --all-targets --target-dir target/pr-review -- -D warnings`,
  `cargo fmt --all -- --check`, `git diff --check`를 통과했다.
- `node scripts/rust-test-suite-manifest.mjs --prepare` 및 `--check`,
  `node scripts/rust-unit-test-tiers.mjs --check`를 통과했다.

모든 Cargo 검증은 `--locked`와 공유 검토 산출물 `target/pr-review`로 순차 실행했다. 파생 harness와
manifest는 검증에만 사용했고 PR diff에 포함하지 않는다.

## 최종 판정

**수용 권고, trailing CI 대기.** 확인된 2010 저장 메타데이터를 분류하되, 근거가 없는 주 버전에는
계속 제품 연도를 부여하지 않는다. 최신 trailing head의 GitHub Actions가 모두 성공하고 작업지시자가
승인한 뒤 merge한다.
