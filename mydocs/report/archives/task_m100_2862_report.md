# Task m100-2862 처리 결과

## 이슈

#2862 — 책갈피 이름 입력 길이 미검증 → CTRL_DATA 길이 프리픽스 u16 캐스팅 랩어라운드로 저장 파일
손상 (#2851/#2854 재발).

## 원인

`BookmarkDialog`(`rhwp-studio/src/ui/bookmark-dialog.ts`)의 이름 입력("넣기" `nameInput`,
"이름 바꾸기" `prompt()`)에 길이 제한이 없어, 매우 긴 이름이 `wasm.addBookmark`/
`wasm.renameBookmark`로 그대로 전달되면 `src/serializer/control.rs`의
`serialize_bookmark_ctrl_data`에서 `utf16.len() as u16` 캐스팅이 랩어라운드되어 CTRL_DATA
레코드가 손상될 수 있었다. `.rs`는 수정하지 않고, 손상 가능한 값이 wasm 호출까지 도달하지
않도록 프런트엔드 다이얼로그 단계에서 `MAX_BOOKMARK_NAME_LEN(250)` 가드를 추가했다.

## Red → Green

- 수정 전(red): `rhwp-studio/tests/bookmark-name-length-guard.test.ts`가 `MAX_BOOKMARK_NAME_LEN`
  상수 부재로 실패.
- 수정 후(green): 동일 테스트 통과.

## 변경 파일

- `rhwp-studio/src/ui/bookmark-dialog.ts` — `MAX_BOOKMARK_NAME_LEN` 상수, `nameInput.maxLength`,
  `doAdd()`의 길이 초과 검사, `doRename()`의 `prompt()` 반환값 길이 초과 검사(`alert`로 안내 후
  중단).
- `rhwp-studio/tests/bookmark-name-length-guard.test.ts` — 신규 소스 가드 테스트.
- `mydocs/report/task_m100_2862_report.md` — 본 보고서.

## 검증

- `cd rhwp-studio && npm test` — 500 테스트 중 499 통과, 1 실패(`cell-flow-boundary.test.ts`,
  기존에도 실패하던 무관 테스트로 확인됨). 신규 테스트는 통과.
- `cd rhwp-studio && npx tsc --noEmit` — `error TS2307` 2건(`@wasm/rhwp.js` 모듈 부재)만 존재,
  기존 베이스라인과 동일. 신규 에러 없음.

## 스코프 밖 관찰 (후속 이슈 후보)

같은 근본 원인(`write_hwp_string`의 `utf16.len() as u16`, `src/serializer/byte_writer.rs:70-77`)
이 `serialize_style`(`src/serializer/doc_info.rs:665-670`)의 스타일 이름 직렬화에도 존재하며,
`rhwp-studio/src/ui/style-edit-dialog.ts`의 `nameInput`/`enNameInput`에도 길이 제한이 없다.
이번 작업 범위에는 포함하지 않았으며, 별도 이슈로 후속 처리가 필요하다.
