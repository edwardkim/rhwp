# PR #7901 메인터너 보정 — ClipboardEvent 개체 Cut의 알림 순서

## 범위와 근거

원 PR head `e4cc258d8e79e7e37d684133efc6971641aedeb6`의 복사 성공 확인·async 개체 Cut 보호를
통합 head `d818c3e95`에 적용한 뒤 두 진입점을 대조했다. 명령형 `cutSelectedObject()`는
삭제 뒤 선택 해제 알림을 보냈지만, `input-handler-keyboard.ts::onCut()`는 알림 뒤 삭제했다.
공개 EventBus의 동기 observer가 문서를 교체하면, 이전 문서에서 얻은 ref를 새 문서에 사용한다.
사용자가 필요한 메인터너 보정을 승인한 범위에서 이 알림 순서를 보정했다.

## 반례와 보정

`clipboard-cut-safety.test.ts`는 실제 키보드 모듈과 InputHandler 위임을 실행한다.
선택 해제 observer가 문서를 교체하도록 하고, 삭제 대상이 복사한 원래 문서이며 observer가 연
새 문서는 보존되는지 검사했다. 기대값은 원본 소유·작업 순서 계약이다.

- 수정 전: 새 반례에서 실제 삭제 대상이 `교체된 문서`로 관측되어 assertion이 실패했다.
  상위 test까지 집계한 실행 결과는 56개 중 54 PASS / 2 FAIL이며, 독립 결함은 신규 반례 1건이다.
- 보정: 기존 snapshot 삭제를 마친 뒤 `picture-object-selection-changed`를 발행한다.
  복사 payload·대상 주소·삭제 API·Undo 경로는 유지했다. 기존 정상 경로 검사의 알림 순서도 함께 교정했다.
- 수정 후: clipboard 이벤트·async 개체 Cut·탭 단위·간격 clamp·기존 단축키/표 owner 경로
  focused 143 PASS / 0 FAIL / 0 SKIP. 새 반례도 통과했다.

실행 명령:

```sh
node --test rhwp-studio/tests/clipboard-cut-safety.test.ts
node --test rhwp-studio/tests/clipboard-cut-safety.test.ts \
  rhwp-studio/tests/clipboard-object-cut-safety.test.ts \
  rhwp-studio/tests/tab-units.test.ts \
  rhwp-studio/tests/section-settings-spacing-clamp.test.ts \
  rhwp-studio/tests/issue-3414-shortcut-routing.test.ts \
  rhwp-studio/tests/table-object-copy-cellpath.test.ts
```

원시 로그는 ignored `output/pr-review/semanticist21-20261015/logs/`의
`clipboard-event-observer-before-20261015.log`, `frontend-focused-after-20261015.log`에 있다.
조판 수치·저장 모델·Rust 코드에는 영향이 없는 Studio 이벤트 순서 보정이다.

## 통합 검증 연결

Studio 타입 검사·전체2010개 단위 검사·production build와 실제 topmost Cut/clipboard 우선순위
브라우저 검증을 완료했다. TypeScript는 보정 뒤 그대로이며 새 Rust/WASM의 공개 저장 API도
실제 초기 WASM 응답 SHA를 확인한 브라우저에서 재검증했다.
전체 Rust·fresh WASM·Print 및 최신 통합 head CI의 단계별 결과는
[개별 review](pr_7901_review.md)와 [공통 검증 기록](../assets/semanticist21_20261015/validation.md)에 연결한다.
로컬 검증과 원격 CI·merge 완료를 같은 상태로 표기하지 않는다.
