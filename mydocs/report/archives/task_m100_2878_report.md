# Task #m100-2878 처리 결과 보고

## 이슈

#2878 — 누름틀(ClickHere) 안내문(guide)/메모(memo) 입력 길이 미검증 → CTRL_DATA(command) `u16` 캐스팅
랩어라운드로 저장 파일 손상. #2851(필드 이름)/#2862(책갈피 이름)/#2866(스타일 이름)와 동일 근본 원인의
형제 필드 재발 인스턴스.

## 스윕 경과

`rhwp-studio/src/ui/*.ts`, `rhwp-studio/src/engine/*.ts` 전수 조사(`<input>`/`<textarea>`/`prompt()` →
`wasm.*` 경로) 결과, 남은 후보 다수는 SAFE로 확인됨(폰트셋 이름·검색/바꾸기·히스토리 라벨 등은 wasm/직렬화기에
도달하지 않거나 u32/텍스트 경로를 씀). 다음 두 후보는 같은 `write_hwp_string`(`src/serializer/byte_writer.rs:70-76`,
`utf16.len() as u16`) 싱크로 향하는 **잠재 취약 소견**으로 확인했으나, 이번 작업 스코프에서는 제외하고 별도 이슈로
분리할 것을 권고함:

- 개체 설명문(`rhwp-studio/src/ui/picture-props-dialog.ts` `descInput`) → `src/serializer/control.rs:1905`
- 수식 스크립트(`rhwp-studio/src/ui/equation-editor-dialog.ts`, `equation-props-dialog.ts` `scriptArea`) → `src/serializer/control.rs:2386`

guide/memo는 즉시 조치 가능한 명확한 인스턴스로 판단해 이번 작업에서 수정함.

## 근본 원인

`rhwp-studio/src/ui/field-edit-dialog.ts`/`field-insert-dialog.ts`의 `guideInput`/`memoInput`이 길이
제한 없이 `wasm.updateClickHereProps`/`wasm.insertClickHereField`로 전달되고, `src/serializer/control.rs`의
누름틀 CTRL_DATA 직렬화가 guide/memo를 포함한 `f.command` 전체 UTF-16 길이를 `cmd_len as u16`으로 기록한다.
합산 길이가 65536 코드 유닛 이상이면 랩어라운드되어 손상된 레코드가 생성된다. `name` 필드는 #2851에서
`MAX_FIELD_NAME_LEN(250)` 가드가 추가되었으나 `guide`/`memo`는 미가드 상태였다.

## 수정 (TypeScript만, `.rs` 미변경)

- `rhwp-studio/src/ui/field-edit-dialog.ts`: `MAX_FIELD_GUIDE_LEN(250)`, `MAX_FIELD_MEMO_LEN(1000)` 상수
  추가. `guideInput`/`memoInput`에 `maxLength` 속성과 에러 라벨을 추가하고, `onConfirm()`에서 길이 검증 실패
  시 `false`를 반환해 다이얼로그가 닫히지 않도록(적용을 막도록) 변경.
- `rhwp-studio/src/ui/field-insert-dialog.ts`: 동일 상수를 `field-edit-dialog.ts`에서 import해 동일 가드
  적용.

## 검증(적색→녹색)

- 적색: 수정 전 `guideInput`/`memoInput`에 `maxLength`/길이 검증이 전혀 없었음(코드 확인).
- 신규 소스 가드 테스트: `rhwp-studio/tests/clickhere-guide-memo-length-guard.test.ts`
  — `MAX_FIELD_GUIDE_LEN`/`MAX_FIELD_MEMO_LEN` 상수가 존재하고 합산 한도가 u16 랩어라운드 지점(65536)보다
  충분히 작은지 검증. 수정 후 통과.
- `cd rhwp-studio && npm test`: 500개 중 499 통과, 1건(`tests/cell-flow-boundary.test.ts`) 실패 — 이 실패는
  wasm 빌드 산출물 부재로 인한 기존(베이스라인) 실패이며 `field-edit-dialog.ts`/`field-insert-dialog.ts`를
  참조하지 않음(grep 확인, 무관 확인).
- `npx tsc --noEmit`: `@wasm/rhwp.js` 모듈을 찾을 수 없다는 기존(베이스라인) 오류만 있음(wasm 빌드 산출물
  부재), 이번 변경으로 인한 신규 타입 오류 없음.

## 변경 파일

- `rhwp-studio/src/ui/field-edit-dialog.ts`
- `rhwp-studio/src/ui/field-insert-dialog.ts`
- `rhwp-studio/tests/clickhere-guide-memo-length-guard.test.ts` (신규)
- `mydocs/report/task_m100_2878_report.md` (본 보고서)
