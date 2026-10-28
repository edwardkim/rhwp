# Stage 3 완료 보고 — Task M100-5769: 붙여넣기 슬롯 절감

- 일자: 2026-08-22
- 브랜치: `fix/5769-delete-inverse` @ `925fb67`
- 계획서: `mydocs/plans/task_m100_5769.md` Stage 3
- PR: https://github.com/edwardkim/rhwp/pull/5915

## 한 것

선택 위 붙여넣기에서 `deleteSelection()` 이 별도 히스토리 엔트리를 만들지 않도록
`deferRecord` 옵션을 추가했다.

### 변경

- `deleteSelection(options?: { deferRecord?: boolean })`: true 이면
  `history.execute()` 대신 `cmd.execute(this.wasm)` 으로 직접 실행.
  호출자의 SnapshotCommand 가 전체 undo 를 커버하므로 중복 엔트리 방지.
- pastePlainText, pasteControl, pasteInternal, pasteHtml 4곳에서 `deferRecord: true` 적용.
- 소스 가드 테스트 `functionBodyFrom` 시그니처 변경 반영.

### 효과

| 시나리오 | 종전 슬롯 | 현재 슬롯 |
|---------|----------|----------|
| 선택 위 텍스트 붙여넣기 | 4 (delete 2 + paste 2) | 1 (paste SnapshotCommand) |
| 비셀 선택 삭제만 | 2 | 0 (FragmentDeleteCommand) |

## 검증

- 전체 npm test: 992/997 통과 (4개 사전 존재 실패)
- `undo-delete-selection-multipara` 4/4 통과
- `command-history-snapshot` + `issue-5769-deferred-after-snapshot` 14/14 통과
- `issue-5690-block-selection-phase` 4/4 통과

## 다음

Stage 4 — 페이지·구역 설정 3곳 역연산화 (`pageSetup`·`pageMargin`·`sectionSettings`).
각 setter 의 부작용 추적(페이지 재조판 결과까지 복원)이 선행.
