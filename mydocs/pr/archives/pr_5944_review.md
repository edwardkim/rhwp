---
kind: pr-review
status: trailing-docs-pending-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-23
---

# PR #5944 검토 - HWPX 마지막 저장 한컴오피스 버전 판별

## 접수 메타데이터

| 항목 | 작성 시점 확인값 |
| --- | --- |
| PR / 작성자 | [#5944](https://github.com/edwardkim/rhwp/pull/5944) / [@jangster77](https://github.com/jangster77) |
| 선행 작업 | [#5932](https://github.com/edwardkim/rhwp/pull/5932), [#5935](https://github.com/edwardkim/rhwp/pull/5935) |
| base / code candidate | `devel` `bf30bd792b9198cec23ce5bb0f897f7d94bf6463` / `345373b1d90c37b3c0aaf043368e3bee4e2a5f69` |
| code candidate 변경 규모 | 13 files, +198 / -59 |
| 작성 시점 상태 | non-draft, `MERGEABLE`, `CLEAN`; code candidate required checks 성공 |
| reviewer | 작성자 본인 self-review, 별도 reviewer 미지정 |

GitHub mergeability와 CI 상태는 작성 시점 참고값이다. 이 review·오늘할일 trailing commit의 최신 head가
required check를 통과하고, merge 직전에 같은 head SHA와 `MERGEABLE/CLEAN`을 다시 확인한 뒤에만 merge한다.

## 변경과 판단

- HWPX 파서가 이미 보존한 `version.xml`에서 `HCFVersion` 또는 `version` 루트의 `appVersion`을 읽는다.
  ZIP을 다시 열지 않으며 HWPX 보조 항목을 한 번 읽는 기존 경로를 재사용한다.
- 쉼표로 구분된 앞 네 버전 구성요소를 정규화하고, 주 버전 10·11·12·13을 각각
  `hancom-office-2018`, `hancom-office-2020`, `hancom-office-2022`, `hancom-office-2024`로 분류한다.
- `version.xml` 또는 유효한 `appVersion`이 없으면 `lastSavedWith:null`을 유지한다. 알 수 없는 주 버전은
  임의 제품 연도를 부여하지 않고 정규화한 버전과 `product:null`만 반환한다.
- HWP5의 기존 `HwpSummaryInformation.revisionNumber` 판별과 HWP3의 `null` 계약은 유지한다.
- CLI, 에이전트 지식 지도, 버전 대조표와 HWP 2020/2024 MCP 선택 문서를 같은 계약으로 갱신했다.

`appVersion`은 수정하거나 삭제할 수 있는 마지막 저장 메타데이터다. 원 작성 제품, 원본성 또는 한컴오피스
설치 상태를 증명하는 값으로 확대 해석하지 않는다는 제한도 문서와 JSON의 `confidence:metadata`에 유지했다.

## 원본 fixture와 시각 검증 판정

한컴오피스에서 저장한 HWPX 세 개를 `samples/pr5935/`에 바이트 보존하고 계약 테스트가 직접 읽게 했다.

| 파일 | `appVersion` | SHA-256 |
| --- | --- | --- |
| `samples/pr5935/test-2018.hwpx` | `10.0.0.5060` | `724ea8c1bfbcb169b51f1eea10b0d338c234f55acb52d438daa84c4d14eac4f4` |
| `samples/pr5935/test-2022.hwpx` | `12.0.0.4605` | `eeec58972c304cf237bc5760e0f5d6765855f0725b78c28497a1d5dd7d4f73ba` |
| `samples/pr5935/test-2024.hwpx` | `13.0.0.3379` | `c9b3ce7a3816d2e867c985876a0705a20c6f926255ffd7966a0afd059011c9a0` |

새 sample은 `version.xml/appVersion` 메타데이터 계약의 입력이며 renderer·typeset·layout·paint 또는 한컴 기준
PDF 일치를 주장하지 않는다. 따라서 별도 MCP PDF와 visual sweep asset은 만들지 않았다. 이 범위 판정과 별개로
code candidate의 Render Diff `Canvas visual diff`는 성공했다.

## 로컬 검증

- 정확한 integration suite `regression_suite_025`에서 `info_hancom_save_version_contract` 3건을 통과했다.
  테스트 이름 기반 편의 실행기는 suite 030으로 잘못 라우팅해 0건을 보고했으므로, manifest가 지정한 suite를
  직접 실행해 exit code 0을 확인했다.
- 실물 HWPX `rhwp info --json`은 2018 `10.0.0.5060`, 2020 `11.0.0.7257`,
  2022 `12.0.0.4605`, 2024 `13.0.0.3379`를 각각 대응 제품과 `confidence:metadata`로 반환했다.
- `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests
  --test-threads 12 --no-fail-fast`를 exit code 0으로 통과했다: 8,166 passed, 39 skipped, 412.050s.
- `cargo clippy --locked --all-targets --target-dir target/pr-review -- -D warnings`,
  `cargo fmt --all`, `cargo fmt --all -- --check`, `git diff --check`를 통과했다.
- `node scripts/rust-test-suite-manifest.mjs --prepare`와 `--check`를 통과했다:
  881 sources, 4,126 static test attributes, 32 suites + 9 exceptions.
- `node scripts/rust-unit-test-tiers.mjs --check`를 통과했다: 4,225 tests / 299 modules.
- 지식 지도 계약 2건과 Markdown 내부 링크 검사 601개 문서를 통과했다.
- 문서 메타데이터 전수 검사는 변경과 무관한 기존 기술 문서 5개의 front matter 누락 20건을 보고했다.
  이번 PR의 변경 문서는 오류 목록에 없었고, 이 기존 기준선은 PR 변경으로 위장해 수정하지 않았다.

모든 Cargo 검증은 `--locked`와 공유 검토 산출물 `target/pr-review`를 사용했다. 파생 integration harness와
manifest는 검증에만 사용했고 PR diff에 포함하지 않는다.

## GitHub Actions

code candidate `345373b1d90c37b3c0aaf043368e3bee4e2a5f69`에서 다음 실행을 모두 통과했다.

- [CI](https://github.com/edwardkim/rhwp/actions/runs/32627173290): Build & Test, Lint, Native Skia,
  Frontend package gates와 archive A/B/C shard 성공
- [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/32627173203): Rust, Python,
  JavaScript/TypeScript 분석과 aggregate 성공
- [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/32627173233) 성공
- [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/32627173281) 성공
- [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/32627173197) 성공

WASM Build와 Frontend unit gates의 skip은 preflight 분류에 따른 정상 결과였다. code candidate 뒤에는 이
review와 오늘할일만 추가하므로 최신 head는 review-only fast-pass 조건과 aggregate 성공 여부를 다시 확인한다.

## 최종 판정

**수용 권고, trailing CI 대기.** HWPX 저장 제품 판별이 문서 내부의 명시적 메타데이터에만 근거하고,
누락·손상·미지원 값에 대해 fail-closed한다. 최신 trailing head의 required checks가 모두 성공하고
`MERGEABLE/CLEAN` 및 exact head SHA를 다시 확인한 뒤 squash merge한다.
