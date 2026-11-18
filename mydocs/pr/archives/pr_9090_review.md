# PR #9090 리뷰 — 검증된 테스트 목록 뒤 문서 tail 재사용

## 최종 판정

**승인.** CI 재사용 판정의 로컬 검증과 정확한 code head의 전체 GitHub Actions를 완료했다.
[PR #9090](https://github.com/edwardkim/rhwp/pull/9090)은 2026-11-15 16:22:25 KST에
`705eb9aa4792d558c5dcacad1bd516b800a0dad6`로 병합됐다.
[Issue #9089](https://github.com/edwardkim/rhwp/issues/9089)는 devel push 자동화로 completed 종료됐다.
CI 정책 변경이므로 원 PR에 trailing commit을 추가하지 않고 옵션 2로 이 기록을 보존한다.

## 접수와 범위

| 항목 | 확인 결과 |
| --- | --- |
| 작성자·검토 역할 | `jangster77`, collaborator self PR |
| base | `e7ee272cfa33a960b0d0d924983d29e6509903e0`, `devel` |
| 검증한 code head | `bcdcfd2a9f900c26ea033794ced71954927843a0` |
| 제출 전 상태 | 본인 열린 PR 0건, 최신 base 동일, merge-tree 종료 코드 0 |
| 병합 직전 | 최신 head 전체 check 성공, `MERGEABLE`·`CLEAN`, head 고정 squash merge |
| 병합 결과 | 후보와 병합 tree 모두 `66a0bc6423706274cda8356b47b3b3af4987c4b2` |

변경은 Adapter·Proptest preflight, 기존 CI 계약 검사, 운영 문서·단계 보고서 다섯 파일이다.
제품 런타임·조판·저장·worker 실행 명령·event·권한·required check 이름은 변경하지 않았다.
따라서 로컬 Cargo·WASM·Studio·Visual Sweep은 비해당이다. 원격에서는 workflow 변경의
실행 영역 가드에 따라 Full CI가 실제로 수행됐다.

## 원인과 구현 검토

선행 [PR #6200](https://github.com/edwardkim/rhwp/pull/6200)의 실행 영역 가드는
테스트 목록 입력까지 실행 정의와 함께 PR 전체에서 차단했다. 실제 반례인
[PR #9088](https://github.com/edwardkim/rhwp/pull/9088)은 검증 완료 후보 뒤에 리뷰·오늘할일만
추가했어도 이전 목록 변경 때문에 worker를 재실행했다.

| 주장 | 실제 판정 경로와 검증 |
| --- | --- |
| 검증된 목록 변경 뒤 문서 tail을 재사용한다 | 목록은 여전히 코드로 분류하고 역방향 계보·후보 run 신원 검사 뒤 실제 worker 성공을 추가 확인한다. 실제 #9088 API의 변경 195개·커밋 41개를 재생해 수정 전 두 경로 full → 수정 후 동일 후보 `0a8afd62…` 재사용을 확인했다. |
| skip을 실제 성공으로 세지 않는다 | 최신 attempt의 worker 이름이 정확히 하나이며 run ID·attempt·head SHA가 일치하고 completed/success일 때만 수용한다. 문서 후보의 skipped는 앞선 실제 성공을 찾고, 실패·누락·중복·API 오류는 full로 처리한다. |
| 기존 실행·신뢰 경계를 유지한다 | workflow·runner·harness 변경과 실행 경로의 문서 rename은 full이다. 새 목록 변경 뒤 과거 성공 재사용, 다른 source·event·시각, 파일 목록 잘림과 계보 불일치를 거부한다. base bridge는 실제 트리 검증 대기를 유지한다. |
| 두 경로와 CI 호출을 일치시킨다 | 같은 worker 판정 함수의 mirror 검사와 실제 YAML inline script 실행 검사를 기존 CI 호출 파일에 추가했다. 새로운 중복 CI 호출은 없다. |

변경 전후 실행 명령·근거는 [단계 보고서](../../working/task_m100_9089_stage1_ci_tail_reuse.md),
최종 운영 계약은 [review-only fast-pass](../../manual/pr_review/review_only_fast_pass.md)에 있다.
단계 보고서는 제출 전 로컬 결과의 스냅샷이며, 이후 원격 검증과 병합 결과는 아래에 기록한다.

## 검증 결과

| 로컬 검사 | 실제 결과 |
| --- | --- |
| 수정 전 재현 | 두 경로 모두 실행 영역 가드 사유로 FAIL |
| 신규 CI 재사용 경계 | 19 PASS |
| Node classifier·controller·policy·report·evidence | 206 PASS, 실패·skip 0 |
| Python workflow·bridge·배선 | 152 PASS |
| actionlint | 두 workflow 종료 코드 0 |
| 문서·Git | 변경 문서 링크 오류 0, 전체 metadata 기존 오류 20건 동일·신규 0, diff 공백 오류 0 |

정확한 code head의 원격 검증은 아래 run에서 전부 success다.

| Workflow | 실제 run·결과 |
| --- | --- |
| CI | [43687566594](https://github.com/edwardkim/rhwp/actions/runs/43687566594), A~E 회귀·Native Skia·Frontend·lint·Build & Test 성공 |
| Adapter inter-diff | [43687566570](https://github.com/edwardkim/rhwp/actions/runs/43687566570), preflight·worker 성공 |
| Proptest roundtrip | [43687566968](https://github.com/edwardkim/rhwp/actions/runs/43687566968), preflight·worker 성공 |
| CodeQL | [43687566486](https://github.com/edwardkim/rhwp/actions/runs/43687566486), JavaScript/TypeScript·Python·Rust·Actions 성공 |
| CI Impact Policy | [최종 완료 감사 43689541932](https://github.com/edwardkim/rhwp/actions/runs/43689541932), exact-head status success |

[실제 lint 로그](https://github.com/edwardkim/rhwp/actions/runs/43687566594/job/132731196591)에
신규 `#9089` 검사 19건의 성공이 모두 포함돼 있다. 원격 preflight의
`adapter-execution-surface-change`·`proptest-execution-surface-change`도 확인해 workflow
실행 정의 변경의 full 경로가 유지됨을 확인했다. 실제 manifest-tail의 양성 판정은 원 #9088 API의
로컬 재생 근거이며, 이를 원격에 별도 시험 PR을 만든 결과로 표현하지 않는다.

## 옵션 2와 종료 처리

원 PR 병합 후 최신 devel 위 별도 문서 브랜치에서 이 리뷰와 오늘할일만 반영한다.
원 코드·검사·workflow를 재포함하지 않는다. 기록 PR 자체의 실제 review-only preflight와
required check·mergeability를 확인한 뒤 병합하며 또 다른 기록 PR은 만들지 않는다.

[Duration 자동 갱신 43689662464](https://github.com/edwardkim/rhwp/actions/runs/43689662464)가
성공했다. 검증된 CI run `43687566594`의 B/C/D/E attempt 1 실측을 사용했고, source merge
`705eb9aa4792d558c5dcacad1bd516b800a0dad6`의 메타데이터를
공유 `ci-metrics/nextest-target-durations` 브랜치의
[`7c9c29dec11b62669e3dfc3a5f48dca140954809`](https://github.com/edwardkim/rhwp/commit/7c9c29dec11b62669e3dfc3a5f48dca140954809)로
게시했다. 병합 후 검증 CI는 다시 실행하지 않았다.

필수 기록 반영 후 devel 동기화, 관련 이슈·원 PR 완료 기록과 이번 작업 소유 임시
브랜치·worktree·output/debug의 종료 정리를 정본 절차로 마친다.
