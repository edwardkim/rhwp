---
kind: memory
status: historical
canonical: mydocs/manual/memory/MEMORY.md
last_verified: 2026-07-17
name: github-author-mailmap
description: "컨트리뷰터 커밋이 GitHub 계정 미연결 author(이메일에 @ 없음 등)로 박힌 경우, history rewrite 금지 — .mailmap 으로 비파괴 매핑"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: da1865ca-614e-44a5-8c3e-ce3fe8956096
---

외부 컨트리뷰터 PR 커밋이 GitHub 계정과 연결 안 되는 author(이메일이 `@` 없는 단순 문자열 등)로 박혀 기여 집계에서 누락된 경우, **.mailmap** 으로 정정한다. 공개 히스토리 rewrite(filter-repo + force push) 는 금지 — 9명+ 활발한 컨트리뷰션 사이클의 모든 clone/fork SHA 를 불일치시킨다.

**Why**: 2026-05-29, @xogh3198(taeho) 의 PR #571/#799(rhwp-studio 문서 비교·이력) 커밋 4개가 `thlee2 <thlee2>` author 로 main/devel 에 기록됨(전체 40 author 중 이메일에 `@` 없는 유일 케이스). 작업지시자가 ".mailmap(안전)" 선택.

**How to apply**:
- 누락 author 진단: `git log --all --format='%an <%ae>' | sort -u | grep -vE "@"` 로 이메일 비정상 author 식별
- `.mailmap` 형식: `정규이름 <정규이메일> 기록된이름 <기록된이메일>` (예: `taeho <92752011+xogh3198@users.noreply.github.com> thlee2 <thlee2>`)
- 검증: `git shortlog -sne` 에서 통합 확인
- 한계: `.mailmap` 은 git log/shortlog/blame/GitHub 소스뷰 표시만 통합. GitHub 프로필 기여 잔디 그래프는 커밋 이메일 자체가 계정 연결돼야 찍히므로 소급 미반영 가능 — 이는 history rewrite 없이는 불가하며 그 비용이 더 큼

[[feedback_pr_supersede_chain]] 동일 컨트리뷰터 누적 PR 점검과 함께 author 일관성도 확인.
