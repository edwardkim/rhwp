---
kind: snapshot
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-14
---

# PR #7142 메인터너 보정·반영 순서

[최종 검토·증적](pr_7142_review.md). rubidus-api의 첫 기여자 지침을 적용한다.

1. contributor 원 head `7e57741eb6df800921de836d4e0df7286dd71864`를 보존하고 동일 가시성 branch에서 메인터너 code-only `0308b476baed1e1c57f93af27a27edf47d18109c`를 추가했다. 확장 자산 복사·필수 gate·계약 검사, DOM 자식 보존·집중 테스트만 변경했다.
2. 로컬 집중 40/40·타입검사·두 확장 패키징·배포 계약 3/3·실제 Chrome bootstrap/DOM 검증을 완료했다. dry-run 성공 후 사용자 승인으로 code-only SHA를 원 PR에 push했다.
3. 첫 기여자 CI 승인 후 정확한 code head의 CI·CodeQL·Render Diff·Adapter·Proptest·정책 성공을 확인했다. review·증적·오늘할일을 코드와 분리한 단일 trailing commit으로 준비한다.
4. 최신 devel의 다른 오늘할일 기록을 보존하는 section에 이번 항목만 추가한다. 실제 merge-tree에서 충돌·공백·링크·기록 보존을 검사하고 base/head가 바뀌지 않았을 때 push한다. source에 무관한 devel 이력을 넣지 않는다.
5. 최종 trailing CI의 실제 reuse/skip 여부 및 aggregate, exact head·MERGEABLE/CLEAN을 확인한다. 사용자 merge·후속처리 승인은 받았으며 조건 충족 후 merge한다.
6. merge 뒤 duration 갱신 결과·devel 동기화·이슈 #5852 OPEN 유지와 1단계 반영 comment·첫 기여자 환영 PR comment를 완료한다. 이미지/문서는 실제 merge SHA에 존재할 때만 게시한다.
7. 이번 clean 로컬 branch·fetch ref·검토 임시 디렉터리를 정리한다. 별도 worktree나 새 Cargo target은 만들지 않았다. contributor fork branch·기본 작업공간·공유 target/dist/pkg는 보존한다.

보정 철회가 필요하면 contributor 이력을 rewrite하지 않고 메인터너 code commit만 별도 revert한다. merged/closedAt 등의 확정값은 merge 뒤 GitHub 후속 comment로 보존하고 기록 PR을 반복 생성하지 않는다.
