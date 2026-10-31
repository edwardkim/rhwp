# #8103 선택 오버레이 배치 완료 동기화

Issue: #8103

## 구현 전 분석

- 기준: 최신 `upstream/devel` = `def947c2149df5bc812d801e6e79be19a403b4d1`.
  `codex/issue-8103-selection-layout`에서 독립 구현한다. 이슈 작성자와 인증 계정은
  `postmelee`이며 댓글·연결 PR·본인 열린 PR을 확인했다. 중복 구현 PR과 본인 열린 PR은 없다.
  이슈 담당자를 현재 계정으로 지정했다. push·PR 생성은 이번 사용자 지시에 포함된다.
- 포함: `InputHandler.afterEdit` 직후의 임시 선택 좌표를 확정된 VirtualScroll 배치 완료에서
  현재 선택으로 다시 계산한다. 일반 선택 재투영과 `CaretLayoutReveal` 자동 스크롤은 분리한다.
  비동기 renderer 선택의 폐기·문서 교체·새 revision 경계를 검사한다.
- 제외: #8102 셀 경로 정규화, 호스트 오류 복구, 조판·pagination·문서 저장 속성 변경.
  다른 후보 PR을 전제로 하지 않는다. merge·release·배포·이슈 종료를 하지 않는다.
- 독립 기대 계약: 삭제 undo는 지우기 전 유효한 선택을 복원한다(#3416 원 PR #5654,
  통합 #5670 `c2a36398dba0eb46eb0744933ae9e70cd0d6f10d`). redo는 선택을 해제한다.
  기존 #7539 `e1ecaa248ecf7f667d8fccab4d9938e70a253392`는 Enter 자동 스크롤만
  예약하며 일반 선택 삭제 undo를 포함하지 않는다. 해당 PR 본문·댓글·변경 목록과
  [기존 보고서](../report/task_m100_7486_report.md)를 확인했다.
- 좌표 경로: WASM `getSelectionRects`의 page-local rect →
  `CanvasView.refreshPagesForMutation`의 renderer await → `refreshPages` →
  `VirtualScroll.setPageDimensions` → `SelectionRenderer.render`의 쪽 원점 + rect × zoom.
  `afterEdit`의 즉시 `updateCaret`은 이전 쪽 배치를 소비한다. 완료 이벤트에서 최신 선택을
  읽어 재투영하면 선택을 재설정하거나 scroll/cursor를 옮길 필요가 없다.
- 시각 검증 분류: 문서 paint·줄 구성·쪽 분할·인쇄 출력은 바꾸지 않는 Studio DOM UI 변경이다.
  시각 거버넌스의 `studio/확장 UI (렌더 엔진 무관)` 행을 적용한다. 한컴 Print PDF
  Visual Sweep·문서 silhouette gate는 비해당이며 실제 Studio 선택 DOM·브라우저를 검사한다.

## 검증 계획

400개의 공개 합성 문단을 실제 paste 경로로 입력하고 전체 선택·삭제·undo·redo를 실키로
실행한다. 첫/중간/마지막 쪽의 음영을 완료된 쪽 원점과 새 전체 선택 대조군에 비교한다.
100%/66%·스크롤·지연 중 새 역방향 선택/줌·anchor/focus·스크롤 보존을 검사한다.
CanvasView 실제 메서드의 epoch/current/dispose fencing을 지연 promise로 검사한다.
fresh dev WASM, TypeScript, 전체 unit, production build, 관련 선택·Enter E2E,
manifest·문서 링크·`git diff --check`를 실행한다. 미실행·실패를 성공으로 기록하지 않는다.
사용자의 실제 브라우저 스크롤·줌 중 음영과 실제 삭제 범위 일치 확인은 별도 절차로 남긴다.

## 구현과 선행 검증 결과

- 완료 이벤트에서 현재 선택을 동기적으로 다시 조회한다. `CaretLayoutReveal`이 예약된 기존
  명령만 캐럿 좌표와 자동 스크롤을 갱신하고, 일반 선택은 `updateSelection`만 실행한다.
- auto mutation renderer의 metrics 적용·후속 완료에서 epoch/current/dispose를 함께 검사한다.
  `refreshPagesForMutation`의 await 뒤 dispose도 검사한다.
- 기준 SHA에서 locked wrapper의 fresh dev WASM 빌드 성공(3분 40초). root pkg/public의
  JS·WASM SHA-256이 각각 일치했다. 이는 release 최적화 결과가 아니다.
- 실제 Chromium E2E 선행 버전: 400문단·10쪽·400개 음영, 100%/66% 삭제·undo·redo,
  첫/중간/마지막 쪽·새 선택 대조·스크롤·지연 중 새 선택/줌을 검사했다.
  수정 전 41 PASS / 32 FAIL(exit 1), 수정 후 73 PASS / 0 FAIL(exit 0).
  단위 재현 2건도 수정 전 0 PASS / 2 FAIL, 수정 후 2 PASS / 0 FAIL이다.
- 수정 전 중간 쪽 캡처를 직접 열어 본문은 있으나 음영이 없는 것을 확인했다.
  테스트를 보강해 active selection의 zoom 전환 및 대기 중 선택 해제도 최종 회차에서 검사한다.
- 초기 `npm ci`는 기본 npm cache의 쓰기 권한 때문에 실패했다. 명령 범위에
  `npm_config_cache=/workspace/work/npm-cache`를 지정해 재실행했고 성공했다.
  해당 환경 실패를 제품 결함 재현이나 검사 통과로 세지 않는다.
- 전체 TypeScript/unit/production build·최종 SHA fresh WASM·확장 E2E 결과는 아직 선행
  결과에 포함하지 않는다. 로그·HTML·브라우저 캡처·생성 WASM은 Git에 포함하지 않는다.
  최종 검증과 사용자 절차는 [결과보고](../report/task_m100_8103_report.md)에 기록한다.
