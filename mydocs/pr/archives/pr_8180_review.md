# PR #8180 리뷰 — 셀 선택·삭제 대상 일치

## 최종 판정

**승인 — 셀 주소 정규화와 선택·삭제 대상 일치 범위에서 추가 결함을 발견하지 않았다.**

## 검토한 구현과 검증


### 확인한 코드

table:create → CursorState.selectAllInCell → cellPathForEdit → DeleteSelectionCommand → ByPath core 호출을 추적했다. 완전한 flat 주소만 depth 1로 변환하며 중첩 path를 복사·보존한다. 셀 정체성 비교는 중간 문단 주소를 포함하고 최내곽 문단 범위만 허용한다. 구역·부모·path의 u32 상한 및 문단/문자 범위 검사가 snapshot·mutation보다 먼저 실행된다. redo는 기존 snapshot 복원을 유지한다.

공통 cellPathJson의 다른 소비자인 중첩 insert/delete, 문자 서식, split/merge도 읽어 변경 영향을 확인했다. 잘못된 셀 전체 선택이 본문 전체 선택으로 확장되지 않도록 예외를 전달한다. 새 actionable code finding 없음.

### 이번 직접 실행과 재사용 근거

- 선택·삭제 focused: **12 PASS / 0 FAIL / 0 SKIP**.
- 전체 frontend unit: **2,373 PASS / 0 FAIL / 2 SKIP** (총 2,375). 두 skip은 격리 checkout에 pkg-node가 없기 때문이다(#6788 실제 WASM 혼합 모양, pending format WASM 왕복).
- 로컬 tsc: **미완료**. generated @wasm/rhwp.js/타입 패키지가 없어 TS2307 발생. 변경 코드의 타입 결함으로 분류하지 않았다.
- 같은 최신 head의 [Frontend package gates](https://github.com/edwardkim/rhwp/actions/runs/39933164066/job/120346727664) 로그에서 tsc·production build 성공을 직접 확인했다.
- 작성 세션의 [최종 결과보고](https://github.com/edwardkim/rhwp/blob/9dbe4fa41df04fe5b2988de9c3c46f2b9f44cbad/mydocs/report/task_m100_8102_report.md)는 최종 code 076c6fd95626e141ff187a30af3ba3ed52286188의 fresh web/Node WASM, 2,375 PASS, 실제 browser E2E 4종과 invalid target 사전 거부 증거를 기록한다. 이번 로컬에서 이를 재실행했다고 주장하지 않는다.
- 기존 전역 E2E manifest의 #7998 누락은 원래 base에도 존재한 실패로 기록되어 있다. 이 PR의 새 회귀로 분류하지 않았다.

사용자 확인: 실제 문서 사본에서 생성 직후·클릭·Tab/Shift+Tab·undo 복원 진입을 비교하고, 다중 문단/중첩 셀 삭제 후 인접·바깥 셀 보존과 undo/redo를 확인한다. 기존 결과보고서의 정식 self-review 반영 항목은 이 문서를 같은 PR에 포함하는 것으로 처리한다.

## 2026-10-24 병합 승인과 검토 범위

- 검토 head: `9dbe4fa41df04fe5b2988de9c3c46f2b9f44cbad`. 검토 base: `def947c2149df5bc812d801e6e79be19a403b4d1`.
- 문서 반영 전 최신 devel: `e7a0dec55c6c6c6365e4ff50babe7583e1cf1c70`.
- 사용자가 각 PR의 리뷰 코멘트, 같은 source branch의 리뷰 문서 반영, 최신 CI 확인 후 merge를 명시적으로 승인했다. 운영 배포는 승인 범위에서 제외했다.
- [게시한 리뷰 코멘트](https://github.com/edwardkim/rhwp/pull/8180#issuecomment-6379331730). collaborator self 경로이며 GitHub APPROVE 이벤트를 제출하지 않는다.
- 사용자 실제 환경 확인은 미검증으로 유지한다. 이를 자동 검사로 대체하거나 완료로 표시하지 않는다. 사용자의 이번 병합 지시에 따라 검증된 변경 범위의 수용과 후속 사용자 확인을 구분한다.
- 이번 후행 변경은 리뷰·오늘할일 문서뿐이다. source/test/fixture/workflow/baseline을 변경하지 않는다. 최신 devel을 source branch에 merge/rebase하지 않는다.
- 코드 검토와 실행에서 추가 결함 없음. 조판/측정/paint/페이지 분할/저장 LineSeg 및 기준값 변경은 비해당이다. Visual Sweep을 실행했거나 한컴 출력과 일치한다고 주장하지 않는다.
- push 전 고정한 base/head의 merge-tree, 문서 상대 링크, 공백, 양쪽 오늘할일 기록 보존을 확인한다. 새 trailing head의 CI·실제 fast-pass·mergeability·head SHA를 다시 확인한 뒤에만 squash merge한다. 검사 실패·pending을 우회하지 않는다.

## Merge 후 contributor PR comment 계획

merge 후 실제 merge SHA와 이 리뷰 문서의 영구 링크, 마지막 head의 CI URL 및 fast-pass/reused candidate를 코멘트에 기록한다. 이번 직접 실행과 작성 세션의 기존 검증을 구분하며 미검증 범위를 그대로 남긴다. 연결 이슈 [#8102](https://github.com/edwardkim/rhwp/issues/8102)의 실제 상태를 조회해 기록하되 이슈 수동 종료·운영 배포는 수행하지 않는다. 별도의 후속 문서 PR은 만들지 않는다. 이번 검토 전용 clean worktree는 정리하고 다른 세션의 source branch와 공유 target은 보존한다.
