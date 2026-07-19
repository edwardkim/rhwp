---
kind: memory
status: historical
canonical: mydocs/manual/memory/MEMORY.md
last_verified: 2026-07-17
name: project_lfs_quota_full
description: pdf-large/ LFS 파일이 git status에 M으로 뜨는 건 LFS 10GB 쿼터 초과로 객체 미수신 — 건드리지 말 것
metadata: 
  node_type: memory
  type: project
  originSessionId: 2560b31a-9f1c-4764-bbf1-7ba5fc27c7ce
---

rhwp의 Git LFS 사용량이 10GB 쿼터에 도달했다. 그 결과 `pdf-large/**/*.pdf`
(`.gitattributes`로 LFS 추적) 일부 파일의 **실제 LFS 객체를 pull하지 못한다**.

증상: `git status`에 해당 PDF가 `M`(modified)로 표시되고, git이 "file that should
have been a pointer, but wasn't" 경고를 낸다. ff/checkout 시 LFS smudge 단계에서
"Aborting"이 뜰 수 있다. (예: `pdf-large/hwpx/2026_oss_rst.pdf`, 2026-06-16 관측.)

**오진 금지**: 이 `M`은 origin 데이터 이슈도, 내 변경도 아니다. 쿼터 초과로 LFS blob을
못 받은 상태일 뿐이다. 브랜치 동기화(ff)는 정상 완료되며 작업에 지장 없다.

**하지 말 것**: 이 LFS 파일을 재커밋/덮어쓰기/`git add`/`git lfs migrate` 등으로 임의
정리하지 말 것 (쿼터·히스토리 영향). 쿼터 정리/증설은 메인테이너(edwardkim) 판단 사항.

ff 동기화 시 LFS 차단을 우회하려면 `GIT_LFS_SKIP_SMUDGE=1` 또는 `git branch -f
<branch> origin/<branch>`(작업트리 미변경)로 브랜치 포인터만 이동한다.

**브랜치 전환 차단 (릴리즈 시 자주 발생, 2026-06-19 관측)**: 이 LFS 파일이 작업트리에
`M`으로 남아 있으면 `git checkout main` 류가 "Please commit your changes or stash them
before you switch branches / Aborting"으로 **조용히 실패**한다(브랜치 안 바뀜). stash·
`git checkout -- <file>`·`update-index --assume-unchanged` 모두 LFS 필터 때문에 안
먹힌다. 해결: **`GIT_LFS_SKIP_SMUDGE=1 git checkout -f <branch>`** 로 강제 전환한다.
이 파일은 HEAD 내용과 md5 동일(쿼터로 smudge만 안 됨)이라 강제 전환해도 손실 없다.
devel→main 머지 같은 릴리즈 흐름에서 checkout이 실패하면 가장 먼저 이걸 의심한다.

관련: [[project_branch_policy]], [[feedback_close_issue_verify_merged]], [[feedback_release_sync_check]]
