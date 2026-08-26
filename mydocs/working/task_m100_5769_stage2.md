# Stage 2 완료 보고 — Task M100-5769: TS 조각 소비자 전환

- 일자: 2026-08-22
- 브랜치: `fix/5769-delete-inverse` @ `8b00956`
- 계획서: `mydocs/plans/task_m100_5769.md` Stage 2
- PR: https://github.com/edwardkim/rhwp/pull/5915

## 한 것

Rust 조각 저장소(Stage 1)를 TypeScript에서 소비하는 배선을 구현했다.

### FragmentDeleteCommand (신규)

`captureDeleteRange` → `deleteRange` → (undo) `restoreDeleteFragment` 순서로
스냅샷 2슬롯 없이 역연산한다.

- `type: 'deleteSelection'` — 양식 모드 게이트 정합 유지
- `selectionBefore()` — #3416 선택 복원 계약 유지
- `snapshotResourceCount()` — 조각 경로 0 (스냅샷 예산 무기여)
- `isNoOp()` — #2370 무변경 신호 전달
- `discard()` — 조각 해제
- redo: undo 가 조각을 소비하므로 문서를 다시 캡처해 삭제

### DeleteSelectionCommand 변경

- 비셀(basic text) 선택 → `FragmentDeleteCommand` 사용
- 셀 내 삭제 → 기존 `SnapshotCommand` 폴백 유지 (Stage 3에서 확장)
- `snapshot` / `fragment` 필드 중 하나만 실체화

### 기타

- `WasmBridge`: `captureDeleteRange`, `restoreDeleteFragment`, `discardDeleteFragment` 래퍼
- `MUTATING_METHODS` 제외: 조각 API 는 문서 IR 변경이 아니라 undo 인프라
- 소스 가드 테스트 업데이트: 조각/스냅샷 경로 분기 패턴 반영

## 검증

- TS 소스 가드 4/4 통과 (undo 위임, 커서 인자 순서, 텍스트 재조립 금지, 예산 참여)
- `command-history-snapshot` + `issue-5769-deferred-after-snapshot` 14/14 통과
- `mutation-routing-guard` 8/8 통과
- 전체 npm test: 992/997 통과 (4개 사전 존재 실패, 1개 환경 의존)

## 다음

Stage 3 — 붙여넣기 슬롯 절감 (4슬롯 → 목표 0). 선택 위 붙여넣기가
`deleteSelection()` 을 다시 호출해 4슬롯을 쓴다. Stage 2 의 조각 경로를
두 번째 소비자로 붙여 삽입분은 기존 텍스트 역연산 계열로 처리.
