# #8102 주소 상한과 같은 셀 거부 회귀 보정 — 3단계

Issue: #8102 · PR: https://github.com/edwardkim/rhwp/pull/8180

## 분석과 계획

재검토한 head는 `27a2415a57f7be8bd574ea07c4bd52ee07c08ac0`이며 기준 devel은
`def947c2149df5bc812d801e6e79be19a403b4d1`이다. 보정 시작 시 원격 head·devel은 같고,
리뷰어 지정·추가 댓글·GitHub review는 없었다. 사용자가 같은 PR의 보정·재검토·반영을 요청했다.

- `Number.isSafeInteger`는 `2^32`를 허용하지만 WASM의 section/parent 인자는 `u32`다.
  실제 WASM에서 구역 또는 부모 문단 `4294967296`이 0으로 변환되어 `alpha`를 삭제했다.
  두 경우 모두 snapshot 1회·mutation 1회였고 사전 거부 회귀는 2건 실패했다.
- 다른 셀·다른 부모 거부 테스트의 시작 문단은 1, 끝 문단은 0이었다. 같은 셀 검사만 메모리에서
  제거해도 기존 10개 검사가 통과했으므로 잘못된 순서 검사와 대상을 분리해야 한다.

공통 주소 검사의 상한을 WASM 주소 범위로 제한하고 기존 문서 존재 여부 조회를 유지한다.
flat/path 각각의 상한 초과와 최대 허용 정수 경계를 확인한다. 같은 문단의 정방향 범위에서
셀·control·부모·중간 경로 좌표를 하나씩 바꿔 snapshot·mutation 전 거부를 검증한다.
fresh WASM 브라우저에서도 내용이 존재하는 상태에서 상한 초과를 주입해 HWP bytes 보존을 확인한다.

적용 절차는 collaborator self PR의 보정 경로와 local_validation 4.3이다. Studio 단독 변경이므로
TypeScript·전체 npm·production build·fresh WASM 브라우저와 관련 E2E를 완료한 뒤 같은 PR에
push한다. 주소 표현·검증만 보정하며 #8104·호스트 #594·Rust/core·다른 이슈 구현은 추가하지 않는다.
정식 PR review·오늘할일 trailing 기록은 해당 code head CI 성공 전 만들지 않는다.

## 결과

- 공통 `cellPathForEdit`는 0 이상 `0xffff_ffff` 이하의 safe integer만 허용한다. 기존 존재 여부
  조회는 유지한다. `2^32`와 `Number.MAX_SAFE_INTEGER`의 flat/path 좌표는 조회 전 거부한다.
- 같은 문단의 정방향 범위에서 셀·control·부모 주소를 바꾸고, 중첩 경로의 중간 문단·control·셀을
  각각 바꾼다. 역방향 범위 검사는 별도로 유지한다. 모두 snapshot·mutation 0회를 확인한다.
- 실제 WASM 브라우저 회귀는 삭제 undo로 내용을 먼저 복원하고 invalid target 9건을 확인한다.
  상한 초과 4건(flat/path × 구역/부모), 최대 u32의 존재하지 않는 주소 2건 및 기존 3건이다.
  HWP bytes 보존과 snapshot·mutation 0회도 검사하도록 보강했다.
- 보정 전 새 단위 검사의 `sectionIndex=4294967296` 사전 거부 assertion은 실패했다.
  보정 후 제품 단위 검사는 12 PASS / 0 FAIL / 0 skip이다. TypeScript도 통과했다.
- 메모리의 `sameCell` 거부 조건만 제거한 mutation 검사에서 모듈 적용 1회를 확인했고, 보강한
  flat 및 중간 경로 거부 테스트가 모두 실패했다. 정식 source를 변경하거나 실패를 skip하지 않았다.
- 기존 실제 WASM 경계 회귀는 보정 전 2 FAIL에서 보정 후 2 PASS / 0 FAIL / 0 skip으로 바뀌었다.
  두 경우 모두 snapshot 0회·mutation 0회·HWP bytes 보존이다.

## 최종 code commit 검증과 재검토

`076c6fd95626e141ff187a30af3ba3ed52286188`에서 fresh web/Node WASM, TypeScript, 전체 npm
2375 PASS / 0 FAIL / 0 skip, production build, 관련 E2E 4종과 두 diff 검사가 모두 통과했다.
네 브라우저가 실제 수신한 WASM 해시는 build manifest와 일치했다. #8102 브라우저의 invalid
target 9건은 snapshot 0회·mutation 0회·HWP bytes 보존이었다. pageerror는 0건이다.

첫 전체 검사의 샌드박스 stdout 전달 실패는 보존하고 최소 재현으로 실행 환경 원인을 확인했다.
게이트·명령을 바꾸지 않은 재실행 결과만 최종 통과로 집계했다. 기존 E2E 원장의 #7998 행 누락
1건은 이번에도 같은 실패이며 다른 이슈의 파일을 수정하지 않았다.

P2 상한 누락과 P3 거부 테스트 허점은 재검토에서 해소됐다. 정상 경로의 선택·삭제·undo/redo도
보존됐고 이 범위의 추가 blocker는 찾지 못했다. 원격 새 head CI와 사용자 실제 문서는 이 로컬
판정으로 대체하지 않는다. 상세 명령·증적·미검증·사용자 절차는
[최종 결과보고서](../report/task_m100_8102_report.md)에 갱신했다.
