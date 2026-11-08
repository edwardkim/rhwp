# #8102 invalid target의 전체 선택 fallback 차단 — 2단계

Issue: #8102

## 분석과 계획

1단계 코드 `c21f2de99`의 정상 셀 경로·전체 npm·E2E는 통과했다. 최종 호출 경로 검토에서
`handleSelectAll`이 `selectAllInCell() === false`이면 본문 전체 선택을 실행하는 것을 확인했다.
잘못된 셀 대상은 선택·편집 전에 거부한다는 #8102 완료 조건을 이 호출자까지 확인한다.

수정 범위는 `cursor.ts`와 해당 검사다. 셀 내부로 판정된 뒤 유효한 대상이 없으면 오류를
전달한다. 셀 밖의 비적용만 false로 유지한다. 오류 payload나 호스트 복구 계약은 바꾸지 않는다.
제품 shortcut handler에서 invalid target이 본문 선택으로 확장되지 않는지 실제 브라우저로
확인하고, 같은 최종 코드에서 필수 package 검증과 관련 E2E를 재실행한다.

## 결과

- `selectAllInCell`은 셀 내부의 누락 부모·문단 부재·조회 실패를 false로 숨기지 않고 throw한다.
  셀 밖의 비적용만 false로 유지한다. 기존 오류 전달 경로를 사용하며 #8104 계약은 추가하지 않는다.
- 기준 devel의 실제 shortcut handler: `rejected=false`, `selected=true`, `stillInCell=false`로
  본문 선택에 진입했다. 새 브라우저 회귀가 이 차이로 실패함을 확인했다.
- 수정 후보는 `rejected=true`, `selected=false`, `stillInCell=true`, HWP bytes unchanged로 통과했다.
- 제품 경로 단위 검사 10 PASS / 0 FAIL / 0 skip. #8102 브라우저 회귀의 기존 여섯 진입 경로,
  중첩·글상자·invalid target도 모두 통과했고 pageerror 0건이었다.
- 최종 commit의 package 전체 검증과 관련 네 E2E를 재실행할 예정이다. 이 기록은 focused 결과이며
  1단계 전체 검증을 새 코드의 최종 검증으로 재사용하지 않는다.

## 최종 commit 검증

`7ab412becb03a8db55d4c01f33020f103349a838`에서 fresh web/Node WASM, TypeScript,
전체 npm 2373 PASS / 0 FAIL / 0 skip, production build, 관련 E2E 4종과 두 diff 공백 검사가
모두 통과했다. 네 E2E의 실제 WASM 응답 해시는 모두 빌드 manifest와 일치했다.
최종 기록은 [결과보고서](../report/task_m100_8102_report.md)에 반영했다.
