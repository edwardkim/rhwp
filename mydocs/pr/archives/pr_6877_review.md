# PR #6877 검토 기록: Rust CodeQL PR 내부 통계 쿼리 축소

## 판정: 승인

작성자 self-review다. 통계 쿼리 3개만 제외되고 기존 보안 쿼리와 추출 진단이 유지되는 것을 실제 CI 로그로 확인했다.
단일 실행의 시간 차이를 반복 재현되는 성능 개선이나 동일 소스 A/B 실험 결과로 확대하지 않는다.
사용자 승인에 따라 원 PR을 먼저 병합하고 옵션 2의 별도 문서 전용 PR으로 이 기록과 오늘할일을 보존한다.

## 검토 경로와 metadata

- base route: `collaborator_self_merge.md`
- modifiers: `intake_and_review.md`, `review_only_fast_pass.md`, `post_merge.md`
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`, 위 역할별 문서, `visual_fixture_evidence.md`의 옵션 2
- current head: 아래 코드 후보와 merge SHA를 구분한다. 문서 PR의 head/CI는 별도 확인 대상이다.

| 항목 | 확인한 사실 |
| --- | --- |
| 원 PR | [#6877](https://github.com/edwardkim/rhwp/pull/6877) |
| 관련 이슈 | [#6876](https://github.com/edwardkim/rhwp/issues/6876) |
| 작성자 / 대상 | `jangster77` / `devel` |
| 코드 후보 | `04c71852cf7ada9acc59eef60058b2256cac5afa` |
| 코드 후보의 기반 | `b5eee9c50d95edca43431a7590e59d6101a30a67` |
| 규모 | `.github/codeql/rust-pr.yml` 한 파일, 17줄 추가 |
| 병합 직전 | 최신 head의 checks 성공 또는 정책상 skip, `MERGEABLE` / `CLEAN` 확인 |
| 병합 | 일반 merge commit, 2026-09-08 15:18:25 KST |
| merge SHA | `9947ad8633215de8966a844b881c150008cd00f5` |

## 변경과 범위

PR Rust 분석 전용 설정에서 다음 ID와 `kind: metric`이 모두 일치하는 쿼리만 제외했다.

- `rust/summary/summary-statistics`
- `rust/summary/reduced-summary-statistics`
- `rust/summary/query-sink-counts`

확인한 CodeQL 2.26.4 / rust-queries 0.1.41의 해당 쿼리에는 `exclude-from-incremental` 태그가 없어
태그만 지정하는 대신 정확한 ID를 사용했다. 기존 product paths, 기본 보안 쿼리, 추출 오류/경고,
성공 파일 수/커버리지 진단은 변경하지 않았다. devel push/schedule의 full-scan 설정도 변경하지 않았다.
렌더러, 문서 샘플, PDF, golden, baseline 변경이 없으므로 시각 증적은 필요하지 않다.

## 완료한 로컬 검증

- `python3 -m unittest scripts.tests.test_codeql_workflow`: 16개 통과.
- YAML 파싱, 정확한 제외 ID 3개와 metric 조건, 기존 product paths, 기본 쿼리 override 부재 검사: 통과.
- `git diff --check`: 통과.
- 로컬 CodeQL CLI가 없어 query resolution과 분석 시간은 아래 GitHub 실행으로 검증했다.
- Rust source/test 변경이 없어 로컬 Cargo 전체 회귀와 시각 검증은 생략했다. GitHub에서는 아래 Full CI가 실행됐다.

## PR #6867과 실제 실행 비교

사용자가 지정한 [PR #6867](https://github.com/edwardkim/rhwp/pull/6867)을 주 비교 대상으로 사용했다.

| 항목 | #6867 변경 전 | #6877 변경 후 | 관측 차이 |
| --- | --- | --- | --- |
| Rust job 전체 | 16분 49초 | 14분 38초 | 2분 11초 감소, 약 12.98% |
| Perform CodeQL Analysis | 14분 33초 | 12분 48초 | 1분 45초 감소, 약 12.03% |
| Checkout | 1분 52초 | 1분 27초 | 25초 감소 |
| DB finalize 구간 | 64.791초 | 59.666초 | 5.125초 감소 |
| 쿼리 실행부터 결과 해석 시작 전까지 | 531.320초 | 439.920초 | 91.400초 감소, 약 17.20% |
| 로드된 쿼리 | 36개 | 33개 | 지정한 통계 3개만 제외 |
| 보안 쿼리 | 16개 | 동일 16개 | 목록 동일 |
| 추출 진단 쿼리 | 기존 목록 | 기존 목록 | 목록 동일 |

- 변경 전: [CodeQL run 34187344129 / Rust job 101938289235](https://github.com/edwardkim/rhwp/actions/runs/34187344129/job/101938289235), PR head `ed196de9d2129adfae41698099ec0a468e604093`.
- 변경 후: [CodeQL run 34192743968 / Rust job 101954032377](https://github.com/edwardkim/rhwp/actions/runs/34192743968/job/101954032377), PR head `04c71852cf7ada9acc59eef60058b2256cac5afa`.
- 양쪽 모두 `ubuntu-latest`, CLI `2.26.4`, Rust query pack `0.1.41`, `--threads=4`, diff-informed 분석이다.
- 자동 RAM은 각각 14574 MB / 14575 MB였다. 별도 유료 runner나 thread 증설을 적용하지 않았다.
- 두 후보의 공통 기반은 `b5eee9c50d95edca43431a7590e59d6101a30a67`이다. 다만 #6867에는 `paragraph_layout.rs`의 오른쪽 탭 보정과 테스트/샘플이 있으므로 동일 소스의 통제 실험은 아니다.
- checkout 25초 차이는 필터 효과가 아니다. 공유 계산, runner 부하, 소스 차이 때문에 전체 131초 또는 쿼리 91.4초 차이 전부를 필터에 귀속하지 않는다.
- 더 오래된 [job 101772210665](https://github.com/edwardkim/rhwp/actions/runs/34131388576/job/101772210665)와 비교하면 전체 14분 43초에서 14분 38초로 5초, 분석 단계는 22초 감소였다. 비교 기준에 따라 관측 차이가 달라지므로 고정된 단축률을 보장하지 않는다.
- 확정한 결과는 필터 적용과 쿼리 목록 보존이다. 동일 소스 반복 A/B 성능 검증과 SARIF 경고 결과의 동등성 실험은 수행하지 않았다.

## 실제 PR CI

코드 후보의 실행 worker와 최종 상태를 확인했다.

- [CI 34192744011](https://github.com/edwardkim/rhwp/actions/runs/34192744011): Build & Test, Lint, Native Skia, Frontend package gates, archive A/B/C/D 및 각 shard 성공.
- [CodeQL 34192743968](https://github.com/edwardkim/rhwp/actions/runs/34192743968): preflight, Rust/JavaScript/Python 분석 및 GHAS CodeQL check 성공.
- [Adapter inter-diff 34192743974](https://github.com/edwardkim/rhwp/actions/runs/34192743974): 성공.
- [Proptest 34192743977](https://github.com/edwardkim/rhwp/actions/runs/34192743977): 성공.
- CI Impact Policy 성공을 확인했다. WASM Build, Frontend unit gates, workflow promotion, PR의 duration refresh는 정책상 skip이었다.

## 병합 후 devel 검증

merge SHA `9947ad8633215de8966a844b881c150008cd00f5`의 결과다.

- [CI 34194092961](https://github.com/edwardkim/rhwp/actions/runs/34194092961): 성공. Build & Test와 duration refresh 성공, heavy worker는 정책상 skip.
- [CodeQL 34194092851](https://github.com/edwardkim/rhwp/actions/runs/34194092851): 성공. preflight 성공, Analyze worker는 정책상 skip.
- [Adapter 34194093014](https://github.com/edwardkim/rhwp/actions/runs/34194093014), [Proptest 34194092904](https://github.com/edwardkim/rhwp/actions/runs/34194092904): 성공, 실제 worker는 정책상 skip.
- 네 workflow 모두 `reuse=true reason=exact-green-pr-workflow-reused`를 실제 로그에서 확인했다.
- CI는 PR run `34192744011`의 duration 자료를 재사용했고 [refresh job 101958042908](https://github.com/edwardkim/rhwp/actions/runs/34194092961/job/101958042908)이 성공했다.
- [Close Issues 34194092742](https://github.com/edwardkim/rhwp/actions/runs/34194092742): 성공. 원 PR의 closing issue references는 비어 있고 #6876은 OPEN이다. 자동 close로 보고하지 않는다.
- 로컬 `devel`을 해당 `upstream/devel`까지 fast-forward했다.

## 옵션 2 기록과 후속 처리 계획

- 이 문서와 [오늘할일](../../orders/20260908.md)만 동일한 문서 전용 후속 PR에 포함한다.
- 원 PR에는 trailing self-review commit을 추가하지 않는다. 문서 PR에 source/test/workflow/sample/LFS/임시 로그를 포함하지 않는다.
- 문서 PR의 최신 head CI와 merge 조건은 별도로 확인한다. 원 코드 PR의 녹색 결과를 문서 PR의 결과로 쓰지 않는다.
- 문서가 devel에 반영된 뒤 원 PR에 merge SHA, 실제 PR/devel CI, 위 비교 수치와 한계를 한 번만 `--body-file`로 기록하고 API로 본문을 재조회한다. 기존 동일 기록이 있으면 중복 등록하지 않는다.
- 시각 검증 대상이 아니므로 PNG/PDF 증적을 만들지 않는다. 비교 job과 이 문서의 direct link를 사용한다.
- #6876은 반복 성능 측정 등 잔여 범위가 있어 별도 판단 전 OPEN을 유지한다. #6867은 비교 대상일 뿐 이 작업에서 merge/comment/close하지 않는다.
- 문서 PR 종료 뒤 이번 작업 전용 local branch만 정리한다. 원격 branch, 다른 작업의 worktree, 기본 작업공간, 공유 `target/pr-review`는 임의 삭제하지 않는다.
