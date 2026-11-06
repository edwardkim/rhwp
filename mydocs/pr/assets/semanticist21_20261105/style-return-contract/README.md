# PR #8526 스타일 생성 실패 반환과 UI 소비 계약

PR #8526은 선택한 위치의 스타일 삽입을 추가하고 잘못된 위치나 표현할 수 없는 목록 중간 삽입을 `-1`로 거절한다. 원 구현·계약은 `tests/cases/inserted_style_keeps_references.rs`와 실제 fresh WASM 공개 API에서 확인했다.

## 기존 검사와 해결 이력

PR #3400·issue #3387의 스타일 undo 배선과, issue #3438 보정 commit `51801e44e1c849bea2a4dfff08236aacbbaf89c6`을 먼저 읽었다. [당시 분석](../../../../working/archives/task_m100_3438_stage1.md)은 당시 API에 실패 경로가 없음을 근거로 음수 검사를 제거했다. 그때의 판단을 소급해 잘못됐다고 부르지 않는다. PR #8526의 계약 확장 뒤에는 Vec `push`, 특정 반환 지역 변수 이름, 음수 반환 경로 부재를 요구하는 Node 소스 검사가 맞지 않는다.

전체 Node 실행 2,721건은 2,720건 통과·이 소스 검사 1건 실패였다. 추가·삽입·실패 시 문서 보존은 실제 API를 실행하는 기존 Rust 회귀로 유지하고, 기존 ID 갱신의 false 검사도 유지한다. 오래된 생성 내부 구현 문자열 세 항목을 현재 동작의 검출 증거로 사용하지 않는다.

## 실제 소비 지점 보정과 반증

[StyleEditDialog](../../../../../rhwp-studio/src/ui/style-edit-dialog.ts)의 생성 ID가 음수이면 모양 적용과 성공 후속 처리를 하기 전에 false를 반환한다. 기존 snapshot operation은 이 결과를 null로 전달한다. 일반 UI는 삽입 index를 생략해 기존처럼 끝에 추가한다. 새 실패 반환 계약이 소비자에 도달했을 때의 실패 처리도 보호한다.

[Node 회귀](../../../../../rhwp-studio/tests/style-undo-routing.test.ts)는 실제 `onConfirm` 메서드를 공용 balanced extractor와 TypeScript 제거로 실행한다. 모양 변경 유무·라우터/직접 경로에서 음수 ID의 false·모양 적용 생략·성공 알림 생략·snapshot null을 확인하며, 성공 ID 0과 3의 정상 처리를 함께 검사한다. UI에 실패 ID를 공급하는 소비 경계 시험이고, 실제 자연 발생 사용자 입력이라고 표현하지 않는다.

- [보정 전](../validation-r6/runtime-checks.md): 7건 중 5 PASS / 2 FAIL. 실제 확인 경로가 실패를 true로 반환하는 반례와 배선 가드가 실패한다.
- [보정 후](../validation-r6/runtime-checks.md): 해당 7건과 기존 undo no-op 7건, 합계 14 PASS, exit 0.
- [실제 최적화 fresh WASM API](../validation-r6/runtime-checks.md): 제품 source `a314d7a24d3234305abff43149f575853db31105`, 잘못된 위치 -1·23은 실제 -1을 반환하고 스타일 목록·본문을 보존한다. index 생략 추가는 ID 22, 가운데 삽입은 ID 1이다.
- TypeScript·production build도 통과했다. [실제 검증 WASM과 pkg/public 동기화 SHA](../validation-r6/runtime-checks.md)를 보존하며 새 UI 소비 보정 때문에 Rust·WASM 제품을 다시 빌드하지 않는다.

이 검사는 UI 트랜잭션·API 계약이며 실제 문서의 px 좌표나 모양을 고정하지 않는다. 다른 입력의 시각 수치를 이 검사의 측정값으로 쓰지 않는다. 원 PR의 문서 시각 증거는 개별 PR 리뷰에서 같은 원본의 Native/fresh WASM TSV와 대표 페이지로 별도 확인한다.

실제 브라우저의 [모달·undo 결과](../validation-r6/runtime-checks.md)도 통과했다. 잘못된 index를 실제 WASM에 전달해 -1을 받은 경우 모달·스타일 목록·undo 이력을 유지하고, 정상 생성 후에는 모달 닫기·이력 1건·undo 복원을 확인했다. [초기화 때 받은 WASM 바이트](../validation-r6/runtime-checks.md)는 R3 최적화 producer SHA와 일치했다. 브라우저는 기존 headless shell 155를 명시해 재시도했고, 첫 Chrome 기동 timeout은 제품 실패와 구분했다. 통합 전체 회귀·보호 문서 등 필수 게이트가 끝나기 전에는 최종 PR 수용이나 제출 완료로 판정하지 않는다.
