---
kind: pr_review
status: active
pr: 4769
issue: 4768
last_verified: 2026-08-14
---

# PR #4769 검토: Subsecond 대화형 개발 제어 활성화

## 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#4769](https://github.com/edwardkim/rhwp/pull/4769) |
| 관련 이슈 | [#4768](https://github.com/edwardkim/rhwp/issues/4768) |
| 작성자 | `jangster77` |
| base / head | `devel` / `task_m100_4768` |
| code candidate | `712873a50a6eae2b6d7e5a17c81dcb43c6c2984a` |
| 작성 시점 merge 상태 | `MERGEABLE`, `CLEAN` |
| 규모 | 3 files, +34 / -1 |

base route: collaborator_self_merge
modifiers: intake_and_review.md, local_validation.md
loaded documents: pr_review_workflow.md, pr_review/README.md, collaborator_self_merge.md, intake_and_review.md, local_validation.md
current head: `712873a50a6eae2b6d7e5a17c81dcb43c6c2984a`

## 변경 범위와 판정

- `subsecond:serve`만 Dioxus 대화형 제어를 켠다.
- loopback `7711`, hot-patch feature와 Vite `7700` 프록시 계약은 유지한다.
- 일반 `npx vite` 및 `npm run dev`는 수정한 npm script를 호출하지 않는다.
- renderer 출력, fixture, baseline은 바꾸지 않는다. Render Diff는 Studio 설정 변경에 대해 canvas/PDF 비교를
  실제로 실행해 성공했다.

## 검증 결과

- `--interactive true`의 동일 Dioxus hot-patch 명령에서 `/` 단축키 메뉴가 나타나는 것을 확인했다.
- `npm run subsecond:serve -- --help`는 package script가 `--interactive true`와 `--addr 127.0.0.1`을
  전달함을 확인했다.
- 실제 `npm run subsecond:serve` 장기 서버는 사용자 터미널의 기존 Dioxus watcher와 target 충돌을 피하기 위해
  중복 기동하지 않았다. 대화형 동작은 동일 `dx serve` 옵션의 사용자 확인 결과로 판정했다.
- [CI run 31794523416](https://github.com/edwardkim/rhwp/actions/runs/31794523416): Frontend package gates와
  Build & Test aggregate 성공
- [CodeQL run 31794523255](https://github.com/edwardkim/rhwp/actions/runs/31794523255): JavaScript/TypeScript,
  Python, Rust 분석 성공
- [Render Diff run 31794523202](https://github.com/edwardkim/rhwp/actions/runs/31794523202): canvas/PDF visual
  diff 성공

## 결론

차단 결함은 발견하지 못했다. 이 문서와 오늘할일, self-merge 예외 규칙을 포함한 trailing head의 CI와
mergeability를 다시 확인한다. 작업지시자가 명시한 경우에만 maintainer `--admin` 예외를 적용하며, 실패 또는
대기 중인 check를 우회하지 않는다. merge 후 #4768 종료 상태와 branch 정리를 post-merge 절차로 확인한다.
