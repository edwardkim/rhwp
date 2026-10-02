# PR #7461 통합 검토 — #7458 install-action 갱신

## 최종 판정

**승인 — 로컬 검증과 Full CI 통과.** 검증 후보는
`1f170889dfeda896706275b0029943ea2786cc9d`이며 작업지시자가 CI 완료 후 병합과 후속 처리를 승인했다.
이 판정 기록을 포함한 최신 head의 required checks와 mergeability를 재확인하고 일반 merge한다.

## 접수와 적용

- 통합 PR: [#7461](https://github.com/edwardkim/rhwp/pull/7461), 작성자 jangster77의 self-review, reviewer 미지정.
- 원 PR: [#7458](https://github.com/edwardkim/rhwp/pull/7458), Dependabot 작성.
- 고정 base: `ba4fa3ccce2f1e2fe711a510bf36e04772451a4b` (`upstream/devel`).
- 원 source: `753565b0b1e7cd1ed861f6d0bf68b588227f5b45`.
- 체리픽·검증 후보: `1231f74fbb95c49ee14b07bcbc4892f5e4f4453b`.
- 작업 branch: `integration/dependabot-7458-20260928`.
- `devel`을 fast-forward 동기화하고 새 branch에서 `git cherry-pick -x`를 실행했다.
  충돌 없이 원 저자를 보존했다. 원 PR 종료·GitHub review·merge는 수행하지 않았다.
- 기본 경로: collaborator_self_merge. 보조: intake_and_review, review_template,
  local_validation, review_only_fast_pass, rework_and_exceptions. 공통 라우터·선택표와
  해당 문서, github_operations, docs_and_git_workflow를 읽고 적용했다.

## 변경과 검증

`build-nextest-archives.yml`, `run-nextest-archives.yml`의 install-action SHA 두 곳만
v2.87.15에서 v2.87.20으로 교체했다. 입력 `tool: nextest`, permission, trigger,
runner, job 이름, required check 배선은 변경하지 않았다.

검증 환경: Ubuntu 개발 서버, 위 체리픽 후보 SHA.

| 검사 | 실행·결과 |
| --- | --- |
| 공백 | `git diff --check upstream/devel...HEAD` 통과 |
| workflow 구문·ShellCheck | `actionlint .github/workflows/build-nextest-archives.yml .github/workflows/run-nextest-archives.yml` 통과 |
| archive 계약 | `python3 -m unittest discover -s scripts/tests -p 'test_nextest_archive_workflow.py'` 15 tests 통과 |
| 계약 테스트 CI 배선 | `python3 -m unittest discover -s scripts/tests -p 'test_workflow_contract_wiring.py'` 3 tests 통과 |
| Action SHA | 공식 tag API `repos/taiki-e/install-action/git/ref/tags/v2.87.20`의 commit이 `9983c65e42da123ff25d1f78505eb6de315aa172`과 일치 |

조판 원칙·Visual Sweep·HWP/HWPX/PDF 입력 커밋 확인은 **비해당**이다.
제품 Rust·WASM·Studio·fixture·baseline 변경이 없다. 제품 전체 로컬 빌드는 비해당이며
새 Action의 GitHub runner에서의 실제 nextest 설치는 builder A–D와 runner A–D 총 8개
job에서 모두 성공했다. archive 빌드·테스트 A–D도 모두 성공했다.

## CI와 후속 조건

후행 기록을 포함한 검증 후보 `1f170889d`의
[Full CI](https://github.com/edwardkim/rhwp/actions/runs/36384085507)는 2026-09-28 KST에 성공했다.
Lint, Native Skia, Frontend package gate, archive 빌드·테스트 A–D, Build & Test를 확인했다.
[CodeQL](https://github.com/edwardkim/rhwp/actions/runs/36384085538)의 언어별 분석과 GHAS CodeQL check,
[Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/36384085460),
[Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/36384085553)도 성공했다.
검증 후보의 최종 상태는 30 SUCCESS·4 SKIPPED, MERGEABLE/CLEAN이며 실패·대기는 없었다.
새 기록 commit의 최신 required checks는 별도로 확인하며 workflow 변경 PR의 재사용을 가정하지 않는다.
되돌리기는 위 체리픽 commit의 revert로 이전 Action SHA를 복원한다.

## Merge 후 contributor PR comment 계획

사용자가 병합과 후속 처리를 승인했다. 실제 merge SHA를 확인하고 review·오늘할일·Action SHA의
devel 포함을 확인한 뒤 #7461과 원 PR #7458에 한국어 존댓말로 결과를 게시한다.
원 PR에는 체리픽 통합 PR, 원/체리픽/merge SHA, 위 CI·로컬 검증 링크를 남긴 후 superseded로 닫는다.
관련 closing issue는 GraphQL 조회 결과 0개이며 별도 issue 종료는 없다.
시각 증적은 비해당이다. merge 후 duration 갱신의 성공 또는 증거 부족에 따른 보류 이유를
확인하고 최종 코멘트에 남긴다. 재검증 workflow를 시작하거나 재실행하지 않는다.
문서는 이미 이 PR의 archive 경로에 포함되어 추가 기록 PR·devel 직접 push는 필요 없다.
local devel을 fast-forward하고 이번 작업의 local/remote 통합 branch를 안전 조건 확인 후 정리한다.
