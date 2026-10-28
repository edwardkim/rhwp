# #3508 2단계 후속 — 실증 실패, 원인 확정, 이중 트리거 핫픽스

Issue: #3508
브랜치: `task/3508-reaper-dual-trigger` (핫픽스)

## 실증 시도와 실패 (2026-07-28 12:02 UTC)

작업지시자가 fork PR #3503 에 수동 Update branch 를 실행해 실증을 시도했다.

| 관찰 | 결과 |
|---|---|
| synchronize 이벤트 | 발생 — 새 head `963940ee1` 의 CI·CodeQL 정상 시작(12:02:40Z) |
| reaper run | **0건** — 저장소 전체에서 `pull_request_target` 이벤트 run 역대 0건 |
| main 의 reaper 파일 | 구버전(`pull_request` 트리거) 그대로 |

이전 SHA(`1a1a39bd1`) run 이 이미 completed 라 취소 대상이 없던 것과 별개로, **run 자체가
뜨지 않았다.**

## 원인 — 실측으로 확정

**`pull_request_target` 트리거는 default 브랜치(main)의 워크플로 파일 기준으로 등록된다.**
devel 에만 있는 정의는 발동하지 않는다.

- 계획서와 #3509 는 "base 브랜치(devel) 파일로 동작하므로 main 불필요" 를 전제로 했는데,
  이는 "base 컨텍스트에서 실행된다" 는 문서 서술을 트리거 등록까지 확장 해석한 **오판**이다.
  collaborator 의 원 불만("main 에 머지되어야 한다") 이 실측으로 옳았다.
- 2차 피해: 트리거를 `pull_request_target` 단독으로 바꾼 #3509 merge 는 **same-repo 자동
  cancel 까지 끊었다.** `pull_request` run 은 PR merge-ref 의 파일을 쓰는데, update branch
  후 merge-ref 가 devel 의 새 파일(트리거에 `pull_request` 없음)을 담아 run 이 안 뜬다.
  #3503 에서 reaper run 이 어느 이벤트로도 0건인 것이 그 증거다(완료 조건 4 위반 상태).

## 핫픽스 — 이중 트리거

`on:` 에 `pull_request` 와 `pull_request_target` 을 모두 선언하고 job 가드로 이중 발동을
막는다.

| 이벤트 | 대상 | 발동 시점 |
|---|---|---|
| `pull_request` | same-repo 만 (가드) | **즉시** — merge-ref 파일로 동작, #3509 이전 동작 복원 |
| `pull_request_target` | fork 만 (가드) | 이 파일이 **릴리즈로 main 에 실린 뒤** |

- concurrency group 에 `event_name` 을 포함해 이벤트별로 분리 — main 반영 후 same-repo
  synchronize 에 두 이벤트가 다 뜰 때, group 을 공유하면 skip 될 pull_request_target run 이
  실동작 중인 pull_request run 을 cancel-in-progress 로 죽일 수 있다.
- 스크립트 본문·안전 경계·최소 권한은 #3509 그대로.
- `multi_pr_update_branch.md` 2.5 의 "fork 포함" 서술을 "main 반영 후 개시" 로 정정.

## 교훈

- **워크플로 트리거 변경은 merge-ref 를 통해 열린 PR 전체에 소급 적용된다** — 구 트리거를
  제거하면 그 이벤트의 run 이 즉시 전면 중단된다. 트리거 교체는 이중 선언 → main 반영 확인
  → 구 트리거 제거의 2단계로 해야 한다.
- `pull_request_target`·`schedule`·`workflow_run` 류의 등록형 트리거는 default 브랜치
  기준이다. base 브랜치 서술("runs in the context of the base")과 혼동하지 말 것.
- 격리 저장소 사전 검증(#3406 전례)을 생략한 대가를 클릭 한 번으로 배웠다 — 워크플로 트리거
  변경은 사전 격리 검증을 기본 게이트로 삼는다.

## 검증·잔여

- YAML 파싱, 문서 게이트 (아래 커밋 전 실행).
- same-repo 복원 실증: merge 후 same-repo PR synchronize 에서 `pull_request` reaper run 확인.
- fork 커버 실증: **다음 릴리즈로 main 반영 후** fork PR update branch 에서 확인. #3508 은
  그때까지 OPEN 유지.
