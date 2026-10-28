# #3790 main 긴급 적용 기록

- 작성: 2026-09-14 14:57 KST
- main 기준: `cac9b4f7cc743535cd7c00fe4f286abd67e7145b`
- 검증된 devel 원본: `93ffc3dd59c120bd54df4c2ac6d1ddbe630f8a2d` (PR #7129 병합)
- 승인: 메인테이너가 정규 릴리즈를 기다리지 않는 main 긴급 적용을 요청하고, 세 소비자의 v6 대응 범위를 확인한 뒤 주의하여 진행하도록 승인했다.

## 적용 경계

이번 승인에 한해 PR 없이 main에 non-force push한다. 제품 소스, 패키지, 버전, 태그 및 branch protection은 변경하지 않는다. #3790의 종료 판정과 CI Impact Policy의 advisory 지위를 유지한다. 대상은 devel을 base로 하는 PR이며 devel→main은 감사·게시 대상에서 제외한다.

1. `.github/workflows/ci-impact-policy.yml`: 검증된 devel YAML 적용. 대상 식별·게시 직전 재검증, 완료 이벤트의 취소 경합 방지, 증거 수집 연결을 포함한다.
2. `.github/workflows/{ci,codeql,render-diff}.yml`: `hasTrustedReviewReuse`만 devel 구현으로 정렬한다. v5→v6, devel/same-repository 경계, 중복·빈 필드 거부를 포함한다. CI에는 신규 계약 테스트 실행 한 줄을 연결한다.
3. `scripts/tests/test_ci_impact_policy_workflow.py`, `scripts/tests/ci-impact-controller-contract.test.cjs`: devel 검증본 적용.
4. `scripts/tests/test_workflow_contract_wiring.py`: main의 기존 목록에 신규 계약 테스트만 추가한다.

main의 정책 발행 helper는 이미 v6다. Controller는 감사 대상 PR의 신뢰된 devel base SHA에서 helper를 checkout한다. `scripts/ci-workflow-evidence.cjs`가 해당 devel에 존재함을 확인했으며, main에 helper 전체를 복사하지 않는다. devel의 다른 CI 개편도 가져오지 않는다.

## push 전 검증

- main CI YAML의 `Validate CI impact classifier` 실행문 전체: PASS.
- main CI YAML의 `Validate workflow contracts` 실행문 전체: PASS. wiring 테스트를 명시적으로 포함한다.
- 네 YAML 파싱, 11개 github-script의 Node 구문 검사: PASS.
- 네 workflow의 트리거·최상위 권한 보존: PASS. 세 소비자의 concurrency 보존: PASS. Controller concurrency 변경은 위의 승인 범위다.
- Controller 및 두 계약 테스트가 지정한 devel blob과 일치: PASS.
- `git diff --check`: PASS.

## 운영 영향과 확인

기존 main push 트리거를 유지하므로 CI, CodeQL 및 Pages 배포가 실행될 수 있다. 패키지 릴리즈·태그 생성은 하지 않는다. push 직전 원격 main의 이동 여부를 다시 확인한다. push 후 원격 SHA와 변경 경로, 발생한 workflow run을 확인한다. 신규 Controller의 실제 devel PR 감사는 배포 이후 자연 발생한 이벤트에서 확인하며, 배포 전 실행을 재실행한 결과로 새 YAML의 활성화를 주장하지 않는다.

기능 배포와 원격 CI 완료는 별개의 상태다. 아직 관측하지 않은 실행 결과를 성공으로 기록하지 않는다. 긴급 변경이 원인인 운영 장애가 확인되면 이 긴급 커밋을 `git revert`하는 방식으로 복구하며, main reset/force push 및 보호 규칙 변경은 하지 않는다.
