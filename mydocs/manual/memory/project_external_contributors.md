---
name: project_external_contributors
description: "외부 컨트리뷰터 누적 명단 (32명, 2026-07-04 기준) — 첫 PR/재기여 식별용"
metadata: 
  node_type: memory
  type: project
  originSessionId: 2560b31a-9f1c-4764-bbf1-7ba5fc27c7ce
---

**외부 컨트리뷰터 누적 (약 32명, 2026-07-04 기준).** 작업지시자(edwardkim)는 이 프로젝트의
메인테이너다([[user_role_identity]]).

**활동 상위 (머지 PR 기준, 2026-07-04 집계)**:
planet6897(82건, 페이지네이션·한글 오라클·측정 인프라) / postmelee(45건, 확장·CI 캐시·측정) /
oksure(12) / seo-rii(10, CanvasKit P-시리즈) / johndoekim(8) / seanshin(5) / seunghan91(5) /
kkyu8925(4, float 표 레이아웃) / humdrum00001010(4, Skia perf·fidelity) / oleg-sung(2, 러시아,
셀 레이아웃)

**그 외 누적**: ahnbu / bapdodi / cskwork(donga-csk) / DanMeon / dragonnite1221-lgtm /
dreamworker0 / dyjung150605 / ggoban / marsimon / metahan88-droid / nameofSEOKWONHONG /
ubermensch1218 / xogh3198 / yl-star7 / Mireutale / HaimLee-4869 / snvtac / physwkim / mrshinds /
Martinel2 / chkwon

**5/14→7/4 신규 진입(10명)**: kkyu8925, humdrum00001010, oleg-sung, Mireutale, HaimLee-4869,
snvtac, physwkim, mrshinds, Martinel2, chkwon — 증가세 지속.

**Why**: PR 처리 시 "첫 PR vs 재기여" 식별이 환영 표현·응대 패턴에 영향. 2026-06-30 oleg-sung을
"첫 러시아 기여자"로 잘못 표기했다가 정정한 사례(#1489 선행 머지 존재) — 환영 표현 전 누적
이력 확인 필수([[feedback_contributor_cycle_check]]).

**How to apply**:
- PR 처리 보고서에 첫 PR 여부 명시. 첫 기여: 환영+base 동기화 안내([[feedback_first_pr_courtesy]]).
  재기여: 직전 처리 보고서 참고해 일관 응대.
- **머지 PR 집계만으로 첫 PR 판단 금지** — cherry-pick 통합/close 된 기여(구 명단 cskwork·DanMeon·
  ggoban 등)는 merged 집계에 안 잡힌다. `gh pr list --author X --state all` + 본 명단 대조.
- **jangster77(Taesup Jang)·postmelee(Taegyu Lee)는 외부 기여자에서 출발했으나 현재 write 권한
  collaborator 2명으로 분업**([[project_pr_merge_collaborator]], 2026-07-04 API 확인) — 외부
  컨트리뷰터 응대 패턴(첫 PR 환영 등) 대상 아님. 명단 집계에선 기여 이력으로 포함하되 역할 구분.
- 이슈 assignee 지정은 edwardkim/jangster77/postmelee 3계정만 가능 — planet6897도 2026-07-01
  기준 assign 불가였음(#1728).

관련: [[feedback_contributor_cycle_check]] [[feedback_first_pr_courtesy]] [[project_pr_merge_collaborator]] [[user_role_identity]]
