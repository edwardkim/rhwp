# #3790 정상화 N3 — 통합 검증·PR 준비

- Issue: [#3790](https://github.com/edwardkim/rhwp/issues/3790)
- 시각: 2026-09-14 13:07 KST 기준
- 상태: **최초 제출 후 CI 실패 원인 최소 보정·로컬 검증 완료. 아래 §5가 최신 상태다.**
- 선행: [N2 구현 결과](task_m100_3790_normalize_n2.md)
- PR 초안: [제출 본문과 명령](../plans/task_m100_3790_normalize_pr.md)

## 1. 경로와 정확한 후보

내부 타스크 PR 준비는 `docs_and_git_workflow.md`를 기본으로, 번호 확정 뒤 self-review는
`collaborator_self_merge.md` 경로를 적용한다. `pr_review_workflow.md`, 선택표,
`intake_and_review.md`, `local_validation.md`, `github_operations.md`를 읽었다.
문서 포함 총 diff가 1,000줄을 넘을 수 있어 `rework_and_exceptions.md`의 대형 PR 경계도 확인했다.
별도 self-review·최신 CI·병합 승인을 생략하지 않는다. PR 번호와 오늘할일은 미리 만들지 않았다.

- branch: `task_m100_3790_normalize`
- 검증 head: `adadcb4e212fe487de5f366e4ad4a2c8c90a3f70`
- 제품 외 CI 구현 commit: `822a1f76d`
- fetch한 devel: `922946438fd89346a022df82e76648f88472f0cc`
- fetch한 main: `cac9b4f7cc743535cd7c00fe4f286abd67e7145b`
- `upstream/devel...HEAD`: behind 0 / ahead 5. 원격 전진 없음; 불필요한 merge/rebase 미수행.
- `git merge-tree --write-tree upstream/devel HEAD`: 충돌 없음.
- merge tree: `fd8d4756da417b1855500eff077ad8266e1d6d59`; 검증 head tree와 동일.
- live devel 보호: `Build & Test`, app_id=15368, enforcement=`non_admins`; 변경하지 않음.
- issue OPEN, assignee edwardkim/postmelee 유지. 이 branch의 열린 PR은 아직 없음.

## 2. 통합 검증

환경: Node v24.15.0, Python 3.12.3. 다음 결과는 위 exact head에서 실제 실행했다.

```bash
node --test scripts/tests/ci-impact-controller-contract.test.cjs \
  scripts/tests/ci-impact-classifier.test.cjs scripts/tests/ci-impact-policy.test.cjs \
  scripts/tests/ci-workflow-evidence.test.cjs scripts/tests/ci-impact-report.test.cjs \
  scripts/tests/collect-postmerge-duration-data.test.mjs \
  scripts/tests/verify-trusted-postmerge-ci-reuse.test.mjs \
  scripts/tests/verify-trusted-postmerge-ci-reuse-squash.test.mjs
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s scripts/tests -p 'test_*workflow.py'
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  scripts.tests.test_workflow_promotion_preflight scripts.tests.test_workflow_promotion_evidence
```

- Node **437/437 통과**, 6.322초. 로그: `output/3790/n3-node.log`.
- Python workflow **180/180 통과**, 9.771초. 로그: `output/3790/n3-workflows.log`.
- Python promotion **33/33 통과**, 0.614초. 로그: `output/3790/n3-promotion.log`.
- 변경 YAML 4개 BaseLoader 파싱, inline JS 11개 syntax 통과.
- base 대비 4개 workflow의 trigger·권한·concurrency 불변 확인.
- CI·CodeQL·Render Diff의 preflight 외 job은 기존 테스트 명령 1개 추가 외 동일함을 구조 비교.
  main 승격 preflight와 `Build & Test`의 연결도 workflow 테스트로 통과했다.
- `git diff --check upstream/devel...HEAD`, 작업 diff 공백 검사 통과.
  관련 변경 문서 8개 상대 링크 검사 통과 (`output/3790/n3-links.log`).
  N3 후속 변경은 계획·보고·PR 초안 문서뿐이며 실행 파일을 변경하지 않았다.

actual classifier 입력을 바꿔 확인한 결과:

| 입력 | 결과 |
| --- | --- |
| 이번 후보 전체 diff | `full`, `fail-closed:workflow-contract`; Rust·Frontend·Render·Native·CodeQL 검증 요구 |
| Studio 렌더 경로 대조군 | Rust=false, Native=false, Frontend=package, Render=true |
| Rust renderer 대조군 | Rust=true, Native=true, Frontend=none, Render=true |

따라서 선택 실행의 기존 영향축을 유지하며 이번 정책 변경 PR 자체는 경량 통과로 분류하지 않는다.
GitHub runner 시간 절감률은 미측정이다. 로컬 테스트 시간을 CI 절감 시간으로 제시하지 않는다.

## 3. 검토 판정·제한

N3에서 추가 코드 보정은 필요하지 않았다. main PR의 trusted reuse polling 0회, 모호한 신원의
감사·게시 차단, 게시 직전 base/head 재확인, v6 발행·소비, 기존 pending/failure 의미를 확인했다.
빈/복수 연결은 최소 live 조회가 남고, 확인 중 PR 상태가 바뀌면 무게시하므로 모든 run 기록이나
모든 경쟁 조건의 계산 비용까지 제거했다는 뜻이 아니다.

조판 원칙·HWP/HWPX/PDF 입력 검증은 **비해당**: 제품 코드·sample·baseline을 변경하지 않았다.
Rust/Studio 전체 빌드는 O3의 이번 변경 범위에 해당하지 않아 실행하지 않았다.
actionlint 부재와 기존 전역 문서 메타데이터 오류는 [N2 제한](task_m100_3790_normalize_n2.md)에 따른다.
실행형 모의 API 검증은 GitHub 실제 webhook·runner 검증을 대신하지 않는다.

## 4. 다음 조치와 복구

승인 후 최신 base 재조회·병합 시뮬레이션 → branch push → devel 대상 Open PR 생성 → 실제 번호로
self-review·필요한 오늘할일 기록 → 최신 head CI 확인 → 별도 병합 승인 순서다.
이 문서 작성 자체로 push·PR·댓글·workflow dispatch·merge를 수행하지 않았다.

구현 병합과 운영 활성화는 다르다. main 등록형 YAML의 정상 승격 및 실제 devel/main 경계 관찰까지
#3790을 OPEN으로 유지한다. PR 본문에 자동 close 키워드를 넣지 않는다.
부작용은 `822a1f76d` 구현 범위의 revert PR로 검증·복구한다. 문서 이력을 삭제하거나 required check를
끄지 않으며, 직접 main push/자동 재실행으로 우회하지 않는다.

## 5. 2026-09-14 후속 최소 보정과 종료 범위 확정

위 §1–4는 최초 PR 제출 전 기록이다. 승인 후 `b0b150081`을 push하여
[PR #7129](https://github.com/edwardkim/rhwp/pull/7129)를 제출했다.
[CI 34805247198](https://github.com/edwardkim/rhwp/actions/runs/34805247198)의
`Validate workflow contracts`에서 기대 테스트 파일 목록 누락으로 실패했다.

신규 `ci-impact-controller-contract.test.cjs`는 실제 CI 실행 단계에는 연결되어 있었으나,
`test_workflow_contract_wiring.py`의 고정 기대 목록에는 없었다. 기존 로컬 명령의
`test_*workflow.py` 패턴은 `test_workflow_contract_wiring.py`를 포함하지 않는다.
따라서 앞의 180개 통과 기록은 해당 명령의 결과이지 CI 계약 검사 전체 통과 증거가 아니다.
이 범위 누락은 작업자의 검증 누락이며, 해당 배선 검사는 CI에서 의도대로 결함을 검출했다.

- 보정 전 `python3 -m unittest scripts.tests.test_workflow_contract_wiring`: 3개 중 1개 실패 재현.
- 보정: 기대 목록에 신규 연결 테스트 파일명만 추가. workflow·제품 코드는 추가 변경 없음.
- 보정 후 `.github/workflows/ci.yml`의 `jobs.lint.steps`에서 아래 두 단계의 `run`을 읽어
  `bash -e -o pipefail -c`로 순서대로 실행했다 (`PYTHONDONTWRITEBYTECODE=1`).
  - `Validate CI impact classifier`: 등록된 Node·Python 명령 전체 통과.
  - `Validate workflow contracts`: 등록된 Node·Python 명령 전체 통과.
    마지막 배선 검사 3개도 모두 통과했다.
- 로컬 수정은 `b0b150081` 위의 테스트 목록 보정과 이번 결정 문서다.
  원격에 보정을 push하지 않았으므로 새 CI 성공·병합·이슈 종료는 아직 아니다.

작업지시자의 최종 결정에 따라 advisory와 required `Build & Test`를 유지하고 v6 정합성을
복원한 후보가 CI·self-review·devel 병합을 통과하면 결정과 결과를 게시하고 #3790을 종료한다.
§4의 main 활성화까지 OPEN 유지 조건은 [최종 수행계획 §8](../plans/task_m100_3790_normalize.md)로
대체한다. main YAML 활성화는 정규 승격 시점이며 종료와 동일시하지 않는다.
실행 횟수·순서 재설계는 추가하지 않고, 새 문제가 생길 때 후속 검토한다.
