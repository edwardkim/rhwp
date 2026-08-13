# Task M100-3438 Stage 2 — 한컴 모달 undo 판정과 도달 불가 구독 제거

## 한컴 2022 기준 판정

Windows의 한컴 Office 2022 새 빈 문서에서 `oracle3438`을 입력한 뒤 F6 스타일 대화상자를 열고
Ctrl+Z를 눌렀다. 대화상자는 열린 채로 남았고, Escape로 닫은 뒤 상태 표시줄은 `10글자`였다.
즉 한컴도 모달이 열린 동안 undo를 실행하지 않는다.

따라서 rhwp-studio의 `ModalDialog`가 modal capture 단계에서 Ctrl+Z를 편집기로 전달하지 않는 것은
한컴과 같은 계약이다. `StyleDialog`가 열린 동안 `history-jumped`가 발생할 수 없으므로 관련 구독,
동기화 메서드, 해제 핸들과 이를 고정한 소스 가드를 제거한다.

## 함께 보강할 저장 실패 처리

Stage 1의 반환 계약 처리는 유지하되, 실제 `false` 또는 예외면 `StyleEditDialog`의 확인 동작도
`false`를 반환해 대화상자를 닫지 않는다. 입력 핸들러 경로의 snapshot rollback과 services 미주입
fallback의 결과 처리를 같은 성공 여부로 묶어 실패를 정상 저장으로 표시하지 않는다.

## 범위

1. `StyleDialog`의 도달 불가 `history-jumped` 구독·동기화·해제를 제거한다.
2. `StyleEditDialog.onConfirm()`이 유효성 검사 및 저장 실패 때 modal을 유지하게 한다.
3. 가드 테스트를 실제 한컴 모달 계약과 WASM bool 실패 계약에 맞춰 갱신한다.

## 범위 외

- 모든 모달에서 Ctrl+Z를 허용하도록 `ModalDialog`의 capture 정책을 바꾸는 일
- 한컴 2022가 아닌 버전의 동작을 한컴 2022 결과로 일반화하는 일
- remote push, PR 생성, issue close

## 검증

- `node --test tests/style-undo-routing.test.ts tests/undo-noop-skip.test.ts`
- `npx.cmd tsc --noEmit`
- `npm.cmd test`
- `npm.cmd run build`
- 로컬 Studio에서 F6 스타일 대화상자 중 Ctrl+Z가 편집기를 바꾸지 않고, 닫은 뒤 Ctrl+Z가 동작하는지 확인
