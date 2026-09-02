---
kind: pr-review-implementation
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-13
---

# PR #4716 메인터너 보정 구현 기록

## 라우팅

```text
base route: collaborator_external_pr
modifiers: intake_and_review, local_validation, review_only_fast_pass
```

외부 contributor source branch `kevin9327/rhwp:task_m100_spine`는 직접 수정하지 않았다.
최신 `upstream/devel` `e219134ae0` 위의 로컬 검토 브랜치
`review/kevin9327-20260813`에 원 head `a81bb8ff0b`의 기능 commit을 `81ab1910e4`로
cherry-pick하고, 메인터너 보정 `2fd830c606`을 추가했다. 결과는 원본 PR을 직접 merge하지 않는
통합 PR [#4722](https://github.com/edwardkim/rhwp/pull/4722)로 올렸다.

## stage

1. 원 PR head와 base를 고정하고, 표준 메타데이터 허용값 및 각 척추 표면의 실제 Markdown 연결을
   검토했다.
2. `kind: standard`가 메타데이터 검사에서 실패하고, 표면 파일이 존재해도 `gym/PARK.md`의 링크와
   `AGENTS.md`의 anchor 누락을 잡지 못하는 것을 확인했다.
3. `2fd830c606`에서 문서 kind를 `reference`로 보정하고, 링크·anchor 검증과 두 회귀 단위 검증을
   추가했다.
4. Windows PowerShell에서 Python 정적·단위·wiring·메타데이터·링크 검증을 모두 통과시켰다.
   Cargo incremental 환경 변수는 지정하지 않았고, Cargo 실행은 변경 범위에 필요하지 않아 수행하지
   않았다.
5. code candidate `2fd830c606`의 GitHub Full CI와 CodeQL이 녹색인 것을 확인했다.

## 경계와 rollback

가드는 선언 표면의 파일 존재뿐 아니라 표준 문서로 향하는 Markdown 링크와 필요한 heading anchor를
검증한다. 다만 AWS 표준 자체의 모든 서술 의미를 해석하는 lint로 확장하지는 않는다. 보정을 취소해야
하면 contributor 원 commit이나 fork branch를 변경하지 않고 통합 branch에 보정 commit을 추가하거나
되돌린다.

이 문서와 review 문서는 code candidate 뒤의 review-only trailing commit이다. push 뒤 최신 head의
fast-pass preflight와 Build & Test aggregate를 다시 확인한 다음 self-review와 merge 판단으로 진행한다.
