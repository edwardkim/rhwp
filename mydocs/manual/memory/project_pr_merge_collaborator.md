---
kind: memory
status: historical
canonical: mydocs/manual/memory/MEMORY.md
last_verified: 2026-07-17
name: jangster77-pr-collaborator
description: "collaborator 2명(jangster77, postmelee)이 외부 PR 머지 게이트 분담 — origin/devel의 모르는 머지는 누락 아닌 분업"
metadata: 
  node_type: memory
  type: project
  originSessionId: 2560b31a-9f1c-4764-bbf1-7ba5fc27c7ce
---

**collaborator 2명 (2026-07-04 기준, API 확인)** — 작업지시자(edwardkim, admin) 외에
write 권한으로 **외부 PR 머지 게이트를 분담**한다:

- **jangster77 (Taesup Jang)** — write. task 기반 기여자(미주/수식 정합 계열)이면서 타 기여자
  PR을 머지하는 메인테이너 역할 겸임. CI 실패 안내 코멘트도 담당(#1573 사례).
- **postmelee (Taegyu Lee)** — write. 2026-07 추가. 확장·CI 캐시·측정(#1664/#1667) 시리즈
  기여자이면서 머지 분업. **머지 커밋 author "Taegyu Lee" <meleeisdeveloping@gmail.com> =
  postmelee 동일인** — 커밋 로그에서 별개 인물로 오인 금지.

**Why**: Claude가 한 이슈를 처리하는 동안 origin/devel에 모르는 커밋 수십 개가 병렬로 들어올
수 있다(실측: 하루 50~171커밋). 머지 직전 fetch에서 "behind origin by N"으로 나타나는데,
**누락이나 사고가 아니라 collaborator들의 PR 분업 결과**다. author 분포에 Taesup Jang /
Taegyu Lee가 다수인 것도 정상.

**How to apply**:
- 머지 직전 `git fetch origin devel` 후 behind 발견 시: origin/devel 위로 rebase + 통합 트리
  CI 재검증(release-test) 후 진행.
- collaborator가 이미 머지한 외부 PR은 Claude가 사후 검토 문서(pr/)를 남길 필요 없음 —
  다른 메인테이너가 게이트를 통과시킨 것. 작업지시자 명시 요청 시에만.
- 외부 PR 착수 점검([[feedback_check_open_prs_first]]) 시 collaborator가 곧 머지할 수 있음을
  감안 — 겹치는 열린 PR은 mergeable/CI 상태 함께 확인.
- **이슈 assignee 는 이 3계정(edwardkim/jangster77/postmelee)만 가능** — 그 외(planet6897 포함)는
  collaborator 아니라 assign 불가(#1728 실측, 2026-07-01).
- postmelee/jangster77 의 자기 PR 은 여전히 검토 대상(self-merge 아닌 경우) — 응대는 내부
  분업 톤으로, 외부 첫-PR 환영 패턴 아님([[project_external_contributors]]).

관련: [[project_external_contributors]] [[feedback_check_open_prs_first]] [[user_role_identity]]
