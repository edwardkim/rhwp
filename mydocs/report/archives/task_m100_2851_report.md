# task_m100_2851 처리 보고

## 이슈

#2851 — 필드 이름 입력 길이 미검증 → CTRL_DATA 길이 프리픽스 u16 캐스팅 랩어라운드로 저장 파일 손상

## 근본 원인

`rhwp-studio/src/ui/field-edit-dialog.ts`, `field-insert-dialog.ts`의 "필드 이름" 입력에는
길이 제한이 없었다. 이 값은 `rhwp-studio/src/command/commands/edit.ts`,
`insert.ts`를 거쳐 검증 없이 `wasm.updateClickHereProps` / `wasm.insertClickHereField`로
전달되고, 최종적으로 `src/serializer/control.rs:160`의 `(nlen as u16)` 캐스팅으로 CTRL_DATA
레코드의 길이 프리픽스에 기록된다. `nlen`(UTF-16 코드유닛 수)이 65536 이상이면 이 캐스팅이
랩어라운드되어, 기록된 길이 프리픽스와 실제로 뒤에 쓰인 바이트 수가 어긋난 손상된 레코드가
만들어진다.

## 수정 범위 (`.rs` 미수정)

작업 지시에 따라 Rust 직렬화기는 수정하지 않고, 손상 가능한 값이 wasm 호출까지 도달하지
않도록 프런트엔드 다이얼로그 단계에서 막는다.

- `rhwp-studio/src/ui/field-edit-dialog.ts`
  - `MAX_FIELD_NAME_LEN = 250` 상수 추가(65536보다 충분히 작은 안전한 상한).
  - `nameInput.maxLength`로 브라우저 레벨 제한.
  - `onConfirm()`이 길이 초과 시 오류 라벨을 표시하고 `false`를 반환해 다이얼로그를 닫지
    않도록 변경(기존에는 항상 `void`를 반환해 무조건 닫혔다).
- `rhwp-studio/src/ui/field-insert-dialog.ts`
  - 동일한 `MAX_FIELD_NAME_LEN` 가드를 `import`해 재사용, 동일 패턴 적용.

## 테스트

`rhwp-studio/tests/field-name-length-guard.test.ts` 신규 추가(소스 가드 방식):

1. `MAX_FIELD_NAME_LEN`이 u16 랩어라운드 지점(65536)보다 작은지 확인.
2. `FieldEditDialog.onConfirm`이 이름 길이 초과 시 `return false;` 경로를 갖는지 소스에서 확인.
3. `FieldInsertDialog`도 동일 가드를 갖는지 확인.

Red → Green:

- 수정 전: `MAX_FIELD_NAME_LEN` 상수와 길이 가드 코드 자체가 없어 세 테스트 모두 실패.
- 수정 후: 세 테스트 모두 통과.

## 검증 결과

- `npm test`: 502 tests, 501 pass, 1 fail — 실패한 테스트는 사전에 알려진
  `tests/cell-flow-boundary.test.ts` 뿐이며 이번 변경과 무관하다.
- `npx tsc --noEmit`: `TS2307` 2건(둘 다 `@wasm/rhwp.js` 모듈 선언 누락, 사전 known-issue) 외
  신규 오류 없음.

## 변경 파일

- `rhwp-studio/src/ui/field-edit-dialog.ts`
- `rhwp-studio/src/ui/field-insert-dialog.ts`
- `rhwp-studio/tests/field-name-length-guard.test.ts` (신규)
