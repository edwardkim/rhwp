# PR #7129 self-review — devel 전용 CI Impact Policy 정상화

- Issue: [#3790](https://github.com/edwardkim/rhwp/issues/3790)
- PR: [#7129](https://github.com/edwardkim/rhwp/pull/7129)
- 검토 시각: 2026-09-14 14:20 KST
- 작성자 self-review: `edwardkim`. 본인 PR이므로 reviewer assign과 GitHub approve는 수행하지 않았다.
- base route: `collaborator_self_merge.md`
- modifiers: `intake_and_review.md`, `local_validation.md`, `rework_and_exceptions.md`, `review_only_fast_pass.md`
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`, 위 기본·보조 문서 및 `github_operations.md`, `codex/docs_and_git_workflow.md`.

## 1. 검토 대상과 규모

| 항목 | 확인값 — 작성 시점 참고값 |
| --- | --- |
| code candidate | `b7ab8624087aad2137e64d539681e39229b560f3` |
| source branch | `task_m100_3790_normalize` |
| 최신 fetch devel | `922946438fd89346a022df82e76648f88472f0cc` |
| base / author | devel / edwardkim |
| 규모 | 15 files, +1121 / -41, 7 commits; 이 review 추가 전 |
| 상태 | OPEN, non-draft, MERGEABLE / CLEAN |
| required check | `Build & Test`, app_id=15368, strict=false |
| advisory | `CI Impact Policy`는 required 목록에 없음. 보호 설정 변경 없음 |
| merge simulation | `1038d3a1b16b349a881670b457651e199b4bfb24`, 충돌 없음; candidate tree와 동일 |

1,000줄 초과이므로 자동/admin merge하지 않고 전체 실행 변경과 테스트를 검토했다.
대부분의 추가량은 계획·단계 보고서와 실행형 계약 테스트다. Rust/Studio/샘플/기준값 변경은 없다.
초기 생성·계획 단계 기록은 당시 상태이며, 최신 범위는
[수행계획 §8](../../plans/task_m100_3790_normalize.md)과 [N3 §5](../../working/task_m100_3790_normalize_n3.md)를 따른다.

## 2. 의도와 실제 코드 검토

최종 범위는 기존 advisory 유지, devel PR 경계와 v6 발행·소비 정합성 복원이다.
Controller를 scheduler로 바꾸거나 실행 횟수·순서를 재설계하지 않는다.

| 검토 항목 | 직접 확인한 호출 경로·반례 | 판정 |
| --- | --- | --- |
| v6 연결 | `ci-impact-policy.cjs::statusDescription` 출력 → CI/CodeQL/Render Diff의 실제 `hasTrustedReviewReuse` 함수. v6 수용, v5/미지원 버전·중복·누락 필드 거부 | 충족 |
| 신뢰 경계 | exact head의 status, Actions bot, base SHA, Controller run 경로·repository·event·결과 검증 유지. main/fork의 특수 trusted reuse는 조회 0회 | 충족 |
| devel 대상 | job 조건 → resolve의 open/base/repository/head/branch 검증 → active=true에만 trusted checkout. 명확한 비devel·비PR 제외 | 충족 |
| 연결 정보 | 명시 PR 번호 우선, 다른 같은-head PR로 대체하지 않음. 빈 연결은 유일한 live 후보 재조회, 모호함·11개 이상 연결·신원 누락은 활성화하지 않음 | 충족 |
| 게시 직전 검증 | `Publish exact-head policy status`에서 live base/head SHA와 저장소·branch·open 상태 재확인. retarget/close/base 전진 시 무게시 | 충족 |
| privileged 실행 | checkout은 resolve한 trusted base SHA, PR head/산출물 실행 경로 추가 없음. 권한 확대 없음 | 충족 |
| 실패 의미 | 기존 blocked/failure/pending/성공 조건 유지. API 실패를 active/success로 합성하지 않음. 제외 사유와 실제 내부 오류를 reporter가 구분 | 충족 |
| 기존 CI 보존 | producer/classifier/evidence helper 변경 없음. 소비자 preflight 외 변경은 Node 연결 테스트 실행 추가. 기존 main·release·promotion 및 Full fallback 유지 | 충족 |
| 테스트 배선 | 신규 `.cjs` 실행과 `test_workflow_contract_wiring.py` 기대 목록 동시 등록. 버전 assertion 삭제로 통과시키지 않고 실제 producer/consumer 연결 검증 사용 | 충족 |

소비 함수의 성공만으로 최종 skip하지 않고 기존 candidate 결과·계보 검증을 계속하는 경로도 확인했다.
일반 fork PR의 선택 실행은 유지하며, same-repository 전용 trusted reuse를 fork까지 확장하지 않았다.
합성 API 테스트는 실제 GitHub webhook 실행과 구분한다.

## 3. 검증 증거

아래 CI는 모두 위 code candidate의 완료 결과다.

| 검증 | 결과·근거 |
| --- | --- |
| Full CI | [34808066941](https://github.com/edwardkim/rhwp/actions/runs/34808066941) success; Build & Test, Rust lint, Frontend package, Native Skia 및 archive worker 성공 |
| CodeQL | [34808066983](https://github.com/edwardkim/rhwp/actions/runs/34808066983) success |
| Render Diff | [34808066825](https://github.com/edwardkim/rhwp/actions/runs/34808066825) success |
| Adapter inter-diff | [34808066943](https://github.com/edwardkim/rhwp/actions/runs/34808066943) success |
| Proptest roundtrip | [34808066974](https://github.com/edwardkim/rhwp/actions/runs/34808066974) success |
| Policy 최종 감사 | [34808962973](https://github.com/edwardkim/rhwp/actions/runs/34808962973) success; exact-head status `v=6;cv=7;mode=full;rfp=0` |
| self-review 집중 재실행 | Node 연결 테스트 22/22, Python 정책·배선 검사 합계 19/19 통과 |

이번 self-review에서 실제 실행한 명령:

```bash
node --test scripts/tests/ci-impact-controller-contract.test.cjs
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  scripts.tests.test_ci_impact_policy_workflow scripts.tests.test_workflow_contract_wiring
git merge-tree --write-tree upstream/devel HEAD
```

선행 로컬 전체 관련 계약 검사는 [N3 §5](../../working/task_m100_3790_normalize_n3.md)에 기록되어 있다.
최초 discover 패턴이 배선 검사를 빠뜨려 원격 CI가 실패했던 사실은 삭제하지 않았다.
`b7ab86240`에서 목록을 보정했고, 최신 CI의 `Validate CI impact classifier`와
`Validate workflow contracts`가 모두 성공한 것을 확인했다.

음성 대조: [N2](../../working/task_m100_3790_normalize_n2.md)의 구현 전 19개 중 14개 실패는
당시 19개 suite의 기록이다. 이번 22개 재실행과 같은 개수로 오인하지 않는다.
후속 배선 결함은 보정 전 3개 중 1개 실패, 보정 후 3개 통과로 별도 확인했다.

제품 코드·Rust test/helper 변경이 없고 정확한 후보의 Full CI가 이미 녹색이므로 광범위 Cargo/WASM
검증을 중복 실행하지 않았다. actionlint는 기존 로컬 설치 부재로 미실행이다.

## 4. 조판 원칙·검증 입력 커밋 확인

- 조판 원칙 준수 검토: **비해당**. renderer/layout/paint·문서 입출력·폰트·baseline/golden/래칫을 바꾸지 않는 CI 정책 변경이다.
- 시각 검증: **비해당**. 실제 문서 렌더 개선을 주장하지 않으며 별도 HWP/HWPX/PDF 입력을 사용하지 않았다.
- 검증 입력 커밋 확인: **비해당**. 테스트는 커밋된 JS/Python과 YAML의 모의 API 입력을 사용하며 문서 fixture가 없다.
- 상수·정책·호출 경로의 동작 기반 검증: **충족**. 실제 발행 함수 및 YAML 소비/resolve/publish 코드를 실행하고 구버전·위조·main·fork·stale·모호한 신원 반례를 확인했다.

## 5. 잔여 제한과 최종 판정

- 새 차단 사유는 발견하지 못했다. 코드 검토·집중 검사·최신 Full CI가 확정 범위를 충족한다.
- `v=6`은 정책 버전, `cv=7`은 classifier 버전이다. 서로 같아야 하는 번호가 아니다.
- 원격의 `rfp=0` 성공은 Full CI 감사 성공이다. 새 review-only trailing commit의 `rfp=1` 재사용 성공을 아직 입증하지 않는다.
- default branch 등록형 Controller YAML의 변경은 main 정규 승격 이후 활성화된다. 지금의 원격 녹색 상태를 새 main 경계 로직의 운영 검증으로 승격하지 않는다.
- 빈/복수 PR 연결의 최소 조회와 workflow 기록은 남을 수 있다. 실행 횟수 0이나 시간 절감률을 보증하지 않는다.
- 최종 사용자 결정에 따라 main 활성화를 기다리며 이번 이슈를 계속 확장하지 않는다. 최신 검토 기록 포함 head의 CI와 승인된 devel 병합 후 결정·결과를 게시하고 #3790을 종료한다.

**최종 판정: 승인**

이는 code candidate에 대한 로컬 self-review 판정이다. GitHub approve나 merge 승인이 아니다.
다음 순서는 이 기록의 로컬 커밋 → 승인된 문서-only push → 최신 head CI 확인 → 승인된 병합 및
이슈 종료다. 새 실행 코드·테스트 보정은 추가하지 않았다. 병합 직전 head·base·mergeability·CI를 다시 확인한다.
main 직접 push·branch protection 완화·자동 재실행은 수행하지 않는다.
