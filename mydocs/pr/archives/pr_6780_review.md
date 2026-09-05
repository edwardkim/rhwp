# PR #6780 검토 기록

## 최종 판정

판정: 승인

로컬 검증을 완료한 후보에 대한 작성자 self-review 판정이다. 최신 GitHub Actions 결과, merge 가능 상태, 작업지시자의 merge 승인은 별도 조건이며 아직 완료로 기록하지 않는다.

## 검토 대상

| 항목 | 내용 |
| --- | --- |
| PR | [#6780](https://github.com/edwardkim/rhwp/pull/6780) |
| 관련 이슈 | [#6779](https://github.com/edwardkim/rhwp/issues/6779) |
| 작성자 및 검토 방식 | jangster77, collaborator 작성자 self-review |
| 대상 base | devel |
| 작업 branch | fix/6779-frontend-postmerge-reuse-20260905 |
| 구현 기준 base | fdba74164ef7003c68fdb14980a3ee1023957fed |
| 검증한 구현 commit | 21f89bf429e10a900f0c9610a268b83e1f12bacf |
| 구현 변경 규모 | workflow 2개, JavaScript 검증기 1개, 계약 테스트 2개; 314줄 추가, 29줄 삭제 |
| 작성일 | 2026-09-05 |
| 원격 CI 및 merge 상태 | 작성 시점 최신 결과를 별도로 조회하지 않았다. merge 직전에 최종 head 기준으로 확인해야 한다. |

이 기록과 오늘할일은 구현 검증 뒤 같은 PR에 추가한 문서 전용 후속 commit이다. 저장소 owner를 reviewer로 자동 지정하지 않았다.

## 원인과 변경 범위

PR #6777의 [PR CI](https://github.com/edwardkim/rhwp/actions/runs/33967701594)는 Frontend package lane으로 성공했다. 그러나 [devel push CI](https://github.com/edwardkim/rhwp/actions/runs/33968179761)의 재사용 판정은 `candidate-full-lane-evidence-unavailable`로 거부되었다. Rust가 필요 없는 PR에도 Rust B/C/D nextest duration artifact가 있어야 재사용 후보가 되는 결합이 원인이었다.

PR #6772의 `devel -> main` 승격 검증과 이번 devel post-merge 재사용 판정은 다른 경로다.

이번 변경은 다음 범위로 제한했다.

- merge 전 신뢰된 base에서 classifier와 verifier를 로드한다.
- GitHub API의 PR 변경 파일을 분류해 Rust와 Native Skia가 불필요한 Frontend unit/package 변경만 추가 경로의 대상으로 삼는다.
- 최종 PR head의 성공한 CI에서 preflight, aggregate, 선택된 Frontend worker, 예상된 Rust skip을 확인한다.
- 19개 job 계약에서 누락, 중복, 알 수 없는 job, pending, 실패, 취소, 예상과 다른 skip/success를 거부한다.
- 기존 PR identity, repository, head, 실행 시각, merge-tree 및 enforcement-surface 검사를 유지한다.
- 검증되지 않은 최종 review head 뒤에 있는 과거 Frontend run으로 대체하지 않는다.
- Frontend-only 재사용에서는 duration 갱신을 건너뛰고, Rust CI 재사용에는 기존 B/C/D duration artifact 요구를 유지한다.
- 만료된 duration artifact를 증거에서 제외한다.

제품 source, renderer, sample, golden, baseline은 변경하지 않았다.

## 완료한 검증

| 명령 | 실제 결과 |
| --- | --- |
| `node --check scripts/verify-trusted-postmerge-ci-reuse.mjs` | 통과 |
| `node --test scripts/tests/verify-trusted-postmerge-ci-reuse.test.mjs` | 30개 통과, 실패 0개, skip 0개 |
| `python3 -m unittest scripts.tests.test_trusted_postmerge_ci_reuse_workflow scripts.tests.test_ci_impact_workflow` | 43개 통과 |
| `actionlint -shellcheck= .github/workflows/trusted-postmerge-ci-reuse.yml .github/workflows/ci.yml` | 통과; 외부 ShellCheck는 이 명령에서 비활성화 |
| `git diff --check` | 구현 후보에서 통과 |

기존 계약 테스트를 약화하지 않고 Frontend unit/package 수용과 실패·취소·pending·identity 불일치·증거 누락·과거 head 거부 사례를 추가했다. 완료한 테스트는 커밋·PR 생성 단계에서 다시 실행하지 않았다. 원시 실행 로그는 임시 디렉터리에만 두고 PR에 포함하지 않았다.

## 생략한 검증과 시각 증적

Rust source와 제품 동작을 변경하지 않아 Cargo 전체 회귀 테스트, WASM build, Studio 테스트는 실행하지 않았다. 렌더링·레이아웃·문서 출력 변경이 없어 visual sweep, PDF 변환, 대표 PNG 산출은 필요하지 않다.

로컬 계약 테스트 통과를 실제 GitHub workflow 실행 성공으로 표현하지 않는다. 새 경로를 적용한 실제 post-merge Frontend-only 성공 사례는 아직 확인하지 않았다.

## 위험 및 merge 전 조건

- job 이름이나 topology가 바뀌면 계약이 일치하지 않아 재사용을 거부한다. 조용히 검증 범위를 줄이는 대신 기존 실행 경로로 돌아간다.
- 이 PR은 controller 및 CI enforcement surface를 바꾸므로 최초 merge의 devel push가 Full CI로 실행되는 것은 예상된 안전 동작이다.
- 모든 workflow를 무조건 skip하거나 모든 PR에서 duration 갱신만 실행하도록 변경하지 않는다.
- 최종 PR head의 required check, 관련 aggregate와 실행된 worker, `MERGEABLE`, `CLEAN`을 확인한 뒤 작업지시자 승인을 받아야 merge할 수 있다.

## Merge 후 issue 및 PR comment 계획

[post_merge.md](../../manual/pr_review/post_merge.md)를 따른다. 실제 merge SHA의 devel CI와 관련 aggregate/worker 결과가 확인된 뒤에만 후속 처리한다.

- PR 본문의 `Closes #6779`와 실제 issue 상태를 확인한다.
- issue #6779의 기존 comment를 확인해 중복 게시를 피하고, merge SHA, 실제 PR/devel CI URL, Frontend 재사용 및 Rust duration 분리 범위, 최초 Full CI가 예상된 이유를 기록한다.
- 본 검토 문서는 확정된 merge SHA에 고정한 GitHub 파일 링크로 안내한다. 시각 증적을 사용하지 않았으므로 관련 없는 이미지나 임시 로그를 첨부하지 않는다.
- 게시가 승인된 comment는 UTF-8 body file로 전달하고 API로 본문을 재조회한다.
- 별도 후속 문서 PR을 만들거나 이미 처리한 PR #6777의 source PR comment/close를 반복하지 않는다.
