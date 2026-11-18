# seo-rii P120–P130 승인된 원격 처리안과 통합 결과

작업지시자는 원 PR 머지와 후속 처리 전체를 승인했다. 아래는 승인된 실행 순서이며 1–4의 승인 리뷰·통합·원 11건의 MERGED 확인을 완료했다. 운영 기록 반영 뒤 댓글·정리도 같은 승인으로 수행한다.

## 고정 대상

- 원 PR: #8962·#8963·#8964·#8965·#8966·#8967·#9128·#9129·#9130·#9131·#9132.
- 마지막 원 head #9132: `4379aa080b01a5da26f03f154287057ff6fb8269`.
- 검증 base: `443dafb08107152e066689544e88d90988339567`.
- 실제 예상 merge tree: `cb9c303399b1769d137751da9fe5c1ccdd8c1842`.
- [원 head·최신 CI](live-final-checks.json), [조상·개별 병합](simulation.json), [원격 후보/로컬 실행 관계](scope-and-remote-candidate.json).

## 승인 시 실행 순서

1. 원 11개 head·reviewer·CI/required checks와 최종 #9132의 MERGEABLE을 재조회한다. 원 head 변경 시 덮어쓰지 않으며 새 검토를 수행한다. 최신 devel을 fetch하고 exact #9132 merge-tree를 다시 산출한다. base가 전진하면 변경된 기능의 실제 보존 증거와 충돌 여부를 다시 확인한다.
2. 원 PR 11건의 검토 결과를 각 원 head의 APPROVE review로 게시한다. 실제 후보 SHA를 지정한다. 현재 제품 보정이 없으므로 원 fork/source branch에는 push하지 않는다.
3. #9132 한 건을 `--merge --match-head-commit 4379aa080b01a5da26f03f154287057ff6fb8269`로 통합한다. 보호 설정·CI 기준을 바꾸거나 실패 check를 우회하지 않는다. 새 통합 PR을 만들지 않는다.
4. merge SHA·devel 포함·실제 tree를 조회한다. 선행 원 PR 10건의 상태도 개별 조회하여 간접 MERGED 여부를 확인한다. 자동 반영이 없으면 검증한 수용 근거 댓글과 완료 close로 처리하고 MERGED라고 잘못 기록하지 않는다. 기여 SHA 11개·fork branch는 그대로 보존한다.
5. post_merge 7.1의 maintainer 직접 반영 경로로 archive review·mydocs/pr/assets·오늘할일만 하나의 운영 기록 commit으로 devel에 push한다. 제품 source/test/workflow/baseline이나 새 LFS를 섞지 않는다. 현재 `.py.txt`·`.dart.txt`는 활성 test가 아닌 관측 증거 사본이다. 기록 링크·base 보존·diff를 먼저 검증한다. 기록 전용 새 PR을 임의로 생성하지 않는다.
6. 최종 devel fast-forward·clean을 확인한다. 부모 #536·#8701의 OPEN을 유지하며 새로운 issue close는 하지 않는다. 기여자 감사 댓글은 실제 merge SHA와 devel에 존재하는 증거를 연결해 각 PR에 게시하고 API로 본문과 링크를 재조회한다.
7. 승인된 후속 처리의 종료 gate로 이번 작업의 clean local branch/worktree·자체 Windows 검증 checkout/runner를 정리한다. 현재 사용이 끝난 기본 작업공간 `target/pr-review/debug`와 저장소 루트 `output/` 전체를 canonical 절차로 삭제·검증한다. user/다른 도구의 WIP·다른 worktree·공유 target의 다른 profile·host SDK·contributor fork branch는 보존한다. 임시 upstream branch는 만들지 않았으므로 원격 branch 삭제는 없다.

새 commit·기능 보정이나 추가 CI rerun이 필요하면 기존 원 head의 재사용으로 완료 처리하지 않는다. GitHub workflow 취소·재실행과 보안 metadata 변경은 이번 제안 범위에 포함하지 않는다. macOS 기여자 주장 수용 결정은 유지하며 새로운 검증 면제나 물리 GPU 지원 완료로 확장하지 않는다.

실제 code merge: `97575816999577744f0e2e7f2e16a41d5961bebf`. 원 11건 모두 MERGED이므로 supersede close는 실행하지 않았다. 부모 #536·#8701은 OPEN을 유지한다.
