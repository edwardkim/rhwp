# PR #8542 원격 처리안

2026-11-06 KST. 이 문서는 승인 요청을 위한 계획이며 원격 처리 완료 기록이 아니다.

## 고정 기준

- 원 PR: `edwardkim/rhwp#8542`, 작성자 `johndoekim`.
- 원 repository·branch: `johndoekim/rhwp`, `fix/issue-8539-rowbreak-stored-restart-cut`.
- 검토 시작 head: `dc7553366f6c85035d510eaeb50eff2f25d83242`.
- 고정 devel: `ece9c3383bb2c73e0f055459ede727a9d3101ac6`.
- 검증 code/data 후보: `790ef21567d041fe272253673fa63fb1ce7809df`.
- 실제 실행한 제품 src tree: `ceaa7c40545ca8036c71c880b8d4a8c447435a4e`.
- 로컬 branch: `review/pr-8542-rowbreak-stored-restart`.
- 최종 문서 후행 commit은 위 code/data를 바꾸지 않으며, push할 실제 HEAD는 승인 요청과 실행 때 확인한다.

원 기여의 제품 수정·기존 assertion을 유지했다. 메인테이너가 추가한 것은 최신 devel 병합,
실측 주석·독립 PDF의 3쪽 원장 등록·현재 Native/Canvas 증적·검토 기록이다.
GitHub API에서 `maintainer_can_modify=true`를 확인했으나 실제 push 권한 성공은 아직 주장하지 않는다.

## 승인 뒤 실행 순서

1. 원 PR이 Open/non-Draft이고 원격 head가 검토 기준과 같은지 확인한다. 최신 devel을 읽고
   충돌·실제 기능 보존을 재확인한다. 다른 사람의 새 head를 덮지 않는다.
2. 로컬 `gh` 인증을 쓰는 Git 경로로 원 기여 branch에 fast-forward push한다. 원 기여 SHA를
   보존하며, fork branch의 삭제·강제 덮어쓰기는 포함하지 않는다.
3. 원 기여 설명을 보존하고 메인테이너의 검증 범위·CI 구분·최종 head 고정 Native/Canvas 이미지를
   PR 본문에 보충한다. UTF-8 파일로 전송한 뒤 REST API로 head·본문·이미지를 확인한다.
4. 새 head에서 필수 Full CI·별도 검사를 완료하고 self-review한다. code/data 보정이 필요하면
   해당 변경의 로컬 검증과 새 head CI를 먼저 완료한다. CI 성공 뒤 불필요한 로컬 전체 재실행이나
   문서 후행 push를 추가하지 않는다.
5. 최신 head·required checks·mergeability와 실제 현재-base 병합 tree를 재조회한다. 같은 head에
   `--merge --admin --match-head-commit`을 사용해 원 PR을 병합한다.
6. 실제 merge SHA와 병합 후 duration 갱신 또는 정당한 갱신 보류를 확인한다. 병합 후 검증 CI를
   새로 시작하거나 재실행하지 않는다.
7. maintainer 운영 기록 범위에서 review를 archive로 옮기고 오늘할일·필요한 확정값을 반영해 devel에
   한 commit으로 push한다. code·test·baseline을 이 직접 반영 commit에 넣지 않는다.
8. devel fast-forward와 #8539의 완료 상태를 확인한다. 승인 범위의 이슈 완료 반영과 원 PR 감사
   댓글을 게시한다. 대표 이미지 URL은 실제 merge SHA에 고정하고 응답·해시·API/화면을 검증한다.
9. 작업 전용 local branch/worktree, 원본 저장소에 이 작업으로 만든 임시 branch가 있을 경우 해당
   branch, 루트 output 전체와 사용 종료한 `target/pr-review/debug`를 정리한다. contributor fork,
   공유 release cache, 사용자와 다른 작업의 소스·worktree·글꼴은 보존한다.

workflow 취소·재실행·cache 삭제·CI 예산/정책 변경은 현재 실제 장애가 없으므로 제안하거나 실행하지 않는다.
권한 거부로 원 기여 branch를 갱신할 수 없으면 그 결과를 보고하고 새 원격 branch/통합 PR 경로를
별도로 확정한다. 승인하지 않은 원 PR close나 임의 supersede는 수행하지 않는다.

## 검토·증적

- [검토 기록](../../archives/pr_8542_review.md)
- [전쪽 TSV·빌드·입력·글꼴·직접 판독](visual-validation.md)

최종 전체 회귀 12,640 PASS/45 skipped, Native 공식 3종 합계 4,064 PASS/12 ignored와 14개 gate를 완료했다. push할 문서 후행 HEAD를 확정해 이 계획을 원격 승인 질문에 연결한다.
