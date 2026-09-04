# #6256 Stage 1: squash merge 뒤 trusted CodeQL 재사용 보완

## 배경

#6253의 squash merge 뒤 devel push에서 CodeQL과 CI 후속 workflow가 다시 전체 실행되었다.
[실행 33084109437](https://github.com/edwardkim/rhwp/actions/runs/33084109437)의
재사용 job은 reusable workflow 내부의 event가 workflow_call로 보이는 문제와, squash
commit의 부모가 하나인 문제 때문에 reuse=false로 종료했다.

## 원인

기존 정책은 reusable workflow가 직접 push/refs/heads/devel event를 보고, merge commit이
부모 두 개와 PR head 부모를 가진다고 전제했다. workflow_call과 GitHub squash merge에서는
각각 그 전제가 성립하지 않아 안전한 full lane으로 fallback했다.

## 보완 범위

- 각 caller가 원래의 event, ref, SHA를 reusable workflow에 명시적으로 전달한다.
- 부모 하나의 commit은 GitHub가 연결한 동일 저장소의 단일 merged PR일 때만 squash 후보로
  취급한다.
- 후보 PR head의 결과 tree, pre-merge base 관계, enforcement surface 비변경, 해당 workflow의
  최종 성공 run이 모두 일치해야 worker를 건너뛴다.
- direct push, merge queue, 모호한 PR 연결, tree 불일치, 정책 surface 변경,
  missing/pending/failed run은 full CI로 fail-closed한다.

## PR 전 계약 확인

archive label 또는 trusted reuse topology를 바꾸면 PR을 열기 전에 다음 두 묶음을 모두
실행한다.

- Node CI 정책과 trusted reuse 계약: ci-impact-classifier, ci-impact-policy, 기존 evaluator,
  squash evaluator test를 명시적으로 실행한다.
- Python workflow 계약 전체: python3 -m unittest discover -s scripts/tests -p test_*workflow.py

이 확인은 #6253에서 archive consumer 계약을 하나씩 놓쳐 후속 수정 커밋이 생긴 문제를
막는다. Studio E2E와 OS resource-limit 같이 무관한 테스트를 glob으로 섞어 환경 실패를
CI topology 회귀로 오판하지 않는다. 실제 GitHub Actions 재사용 성공은 PR CI와 squash
merge 뒤 devel run에서 별도로 확인한다.
