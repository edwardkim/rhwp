---
kind: guide
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-07
---

# PR 접수와 리뷰 기록

이 가이드는 모든 정식 PR review의 접수·기본 판정·review 문서 작성 규칙이다. 역할별 push,
merge, merge 후속 처리는 기본 경로 문서와 [merge 후속 처리](post_merge.md)를 따른다.

## 2.1 reviewer assign 선행

개별 PR review를 시작하면 metadata 조사, local fetch, review 문서 작성보다 먼저 reviewer를 assign한다.
대량 PR은 [다수 PR과 update branch](multi_pr_update_branch.md)의 사전 분류 뒤, 각 원 PR마다 assign한다.

~~~bash
gh pr edit N --repo edwardkim/rhwp --add-reviewer <reviewer>
~~~

## 2.2 기본 metadata

아래 사실을 PR별 review 문서에 기록한다.

- base는 devel이어야 한다. main이면 재작업 경로를 적용한다.
- PR 설명에 closes #N 또는 관련 issue 참조가 있는지 확인한다.
- mergeable 및 mergeStateStatus, 최신 head SHA, required check 상태를 확인한다. 모두 작성 시점 참고값이며
  merge 직전에 다시 확인한다.
- 신규 mydocs/report 파일은 task_m100_{issue}_report.md 규칙과 본문의 Issue: #N을 대조한다.
  회차형 측정 기록은 문서·Git workflow의 예외를 따른다.

~~~bash
gh pr view N --repo edwardkim/rhwp --json \
  baseRefName,headRefName,headRefOid,mergeable,mergeStateStatus,isDraft,author,additions,deletions,files,commits
~~~

## 2.3 규모 분석

`additions`, `deletions`, `files`, `commits`로 변경 규모와 검토 범위를 확인한다. 1,000줄 초과 PR은
[재작업과 예외](rework_and_exceptions.md)의 대형 PR 경로를 추가한다. commit 수가 비정상적으로 많으면
오래된 base·이미 merge된 commit 혼입 여부도 함께 확인한다.

100줄 미만의 소형 PR은 maintainer 일반 경로에서 빠르게 판단할 수 있지만, 최신 head·required check·승인
조건을 생략하는 근거는 아니다.

## 2.4 작성자 확인

작성자가 first-time contributor인지, 이전 PR에서 이어진 변경인지, 같은 contributor의 선행·후속 PR이
있는지 확인한다. first-time contributor에게는 환영과 구체적인 피드백을 함께 제공하고, 기존 contributor는
이전 PR 맥락을 반영한다. 작성자 확인은 credit과 comment 언어·맥락을 위한 것이며 검증 수준을 낮추는
근거가 아니다.

## 2.6 렌더 영향과 시각 검증 필요 여부

Cargo 성공은 시각 검증 판정을 대체하지 않는다. 다음 중 하나라도 해당하면
[시각·fixture 증적](visual_fixture_evidence.md)을 보조 경로에 추가하고, review 문서에 판정과 이유를 먼저 적는다.

- src/renderer, src/wasm_api.rs, rhwp-studio의 Canvas/render 출력 경로가 바뀐다.
- typeset, layout, paint, pagination, page count, table split, wrap, clipping, margin/spacing이 바뀐다.
- PR이 기준 PDF, 한컴 출력, 페이지 수, render-diff, visual regression 해결을 주장한다.
- HWP/HWPX sample, 기준 PDF, golden, visual fixture를 추가·갱신한다.

개체 geometry 무회귀의 재실증은 다음 명령을 사용할 수 있다. 추적 개체가 없는 0→0 행은 근거로 삼지 않는다.

~~~bash
python tools/object_visual_regression.py --preset ovr5 -o output/poc/ovr --diff-against devel
~~~

## 3. 리뷰 문서

### 3.1 review 문서

maintainer 일반 경로는 처리 중 active 경로에 작성하고, 완료 후 archive로 이동한다.
collaborator self-merge와 collaborator 매개 외부 PR은 해당 기본 경로 문서가 정한 archive 경로를 처음부터 쓴다.
PR 번호는 PR 생성 시점에 확정하며 생성 전에 예측하지 않는다. collaborator self-merge는 PR을 생성해
번호를 받은 뒤 reviewer assign을 거쳐 번호 기반 review 문서를 같은 PR의 후속 commit으로 추가한다.

~~~text
mydocs/pr/pr_N_review.md
mydocs/pr/pr_N_review_impl.md
~~~

review 문서에는 최소한 다음을 포함한다.

- PR metadata 표: 번호, 작성자, base, 규모, mergeable 작성 시점 참고값
- 관련 issue 요약과 변경 범위: 핵심 기능, metadata 변경, 범위 밖 변경
- 렌더 영향과 visual sweep 필요 여부
- 선택한 로컬·CI·시각 검증 및 생략 이유
- 발견한 문제·risk·후속 이슈
- 최종 권고: merge, 보정, rebase 요청, 재작업, close, 보류 중 하나

### 3.2 implementation 계획서

다음 중 하나면 pr_N_review_impl.md를 추가한다.

- contributor 원 변경 위에 maintainer 또는 collaborator 보정 code를 추가한다.
- 여러 PR을 체리픽 통합하거나 conflict 해결 순서를 관리한다.
- merge, 후속 PR, issue 분리 등 작업지시자 선택이 필요한 단계가 둘 이상이다.
- review 문서만으로 실행 순서와 rollback 범위가 불명확하다.

커밋별 SHA·제목, 승인부터 cleanup까지의 stage, 작업지시자 결정 항목을 기록한다. 단순·소형 PR은
review 문서 안에 처리 계획을 적고 implementation 계획서를 생략할 수 있다.

### 3.3 volatile 상태값

draft, mergeable, head SHA, CI 상태는 확정 사실처럼 쓰지 않는다. 다음 표현을 쓴다.

- 문서 작성 시점 참고값
- merge 전 최신 상태 확인 필요
- 최종 merge 조건: 최신 PR head의 GitHub Actions 통과와 작업지시자 승인

과거 CI 통과, 특정 SHA, CLEAN 상태만으로 최종 merge 가능을 단정하지 않는다.

### 3.4 완료 검증 기록의 시제

local validation이 끝난 review 문서는 검증 계획서가 아니다. 완료한 Cargo·npm·lint·fixture·
시각 검증은 명령과 결과를 과거형으로 쓴다. "실행한다", "확인할 예정이다", "통과해야 한다"는
아직 실행하지 않은 항목에만 사용한다.

GitHub Actions와 mergeability는 작성 뒤에도 변하는 외부 상태이므로, 최신 head 재확인 필요와
merge 전 조건으로 구분해 기록한다. 이 규칙은 로컬 검증 결과를 미래 약속처럼 약화하거나,
반대로 대기 중 CI를 완료 사실처럼 쓰는 일을 함께 막는다.

### 3.5 가설 기각·재분류 PR

조사 PR이 초기 가설을 기각하거나 다른 원인 계통으로 재분류하는 목적이면, 기각 자체는 merge 보류 사유가 아니다.
다만 최종 보고서·stage 문서·README·sample 설명이 같은 결론을 가리키고, 기각 근거와 후속 issue가 명확해야 한다.
초기 가설을 최종 사실처럼 남긴 문서가 있으면 수정 요청 또는 보정 뒤에 판단한다.

시각 검증을 실제 판단 근거로 쓸 때의 asset·기준 PDF·MCP·comment 규칙은
[시각·fixture 증적](visual_fixture_evidence.md)을 따른다.
