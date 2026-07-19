# PR #2332 검토 — undo 스냅샷 상한 정합 + 예외 안전 스택 이동 (#2328)

- PR: https://github.com/edwardkim/rhwp/pull/2332 (lpaiu-cs, undo 연작)
- 이슈: #2328 — JS 전용 동작 변경 + Rust 는 교차참조 주석만 (동작 무변경 확인)

## 변경 본질

WASM(MAX_SNAPSHOTS=100 무통보 축출)과 JS(maxSize=1000) 의 상한 불일치로
스냅샷성 작업 50회+ 시 undo 가 죽은 스냅샷을 참조 → pop-먼저 구조로 엔트리
영구 소실 → 스택 오염. 3중 정정:

1. undo/redo 를 peek→op→성공 시 pop 으로 — 예외 시 스택 불변(재시도 가능)
2. SnapshotCommand.execute throw 시 beforeId discard (누수 차단)
3. **예산 정합**: SNAPSHOT_ID_BUDGET=98(WASM 100−2, 양방향 교차참조 주석)
   초과 시 front 연속 축출 + discard 즉시 반환 — WASM 무통보 축출 경로가
   **결코 발동하지 않게** 근원 제거 (contiguous bounded-history)

## 로컬 재실증 (merged tree)

| 게이트 | 결과 |
|--------|------|
| 신규 테스트 5건 | 5/5, **수정 전 실패 증명 재현** (JS 2파일 원복 → 5/5 FAIL → 복원 green) |
| studio 단위 / tsc | **316/316** / 0 |
| e2e | undo-contracts 24/0 · undo-object-selection 0 FAIL |
| Rust | 주석만 (빌드 확인) — 교차참조 상수 실재 확인 (WASM_MAX_SNAPSHOTS=100, BUDGET=−2) |

브라우저 60회 스냅샷 대조(수정 전 10건 예외 → 후 0)는 컨트리뷰터 실측 —
구조 검토와 테스트·e2e 무회귀로 방향 확인.

## 판단

**merge 권고.** #2329 가드가 "미기록"을 잡는다면 본 건은 "기록됐지만 축출로
죽는" 축 — undo 신뢰성의 남은 반쪽. 예외 안전(pop-후행)은 스냅샷 외 일반
커맨드에도 적용되는 구조 개선.
