# PR #2324 검토 — undo 계약 e2e 실키 Ctrl+Z smoke (#2317)

- PR: https://github.com/edwardkim/rhwp/pull/2324 (lpaiu-cs)
- 이슈: #2317 (PR #2302 P2 후속, jangster77 요청) — 순수 테스트/하네스 변경
- base=devel, MERGEABLE/BEHIND, 앱 소스 무변경

## 변경 본질

1. **케이스 2b 신설**: 개체(그림) 스냅샷에 대한 **실키** 경로 검증 — 속성
   변경 → 실키 Escape(개체 모드 dispatch 분기까지 검증) → 실키 Ctrl+Z →
   복원 + 스택 소비 + dispatcher 오류 0. 종전 개체 3흐름은 performUndo
   직접 호출이라 키보드 dispatch 가 개체 계열에서 미검증이었음.
2. **case명 정직화 3건**: "Ctrl+Z 복원" → "performUndo 복원(개체 모드 키
   차단 우회)" — 커버리지 과대 표현 제거 + 헤더에 커버리지 지도.
3. **하네스 결함 수정**: dirty 문서에서 `createNewDocument` 가 미저장 가드
   모달을 잔존시켜 후속 케이스 오염 + 전 실키 소멸 — 이벤트가 이미 지원하는
   `skipUnsavedGuard: true` 전달로 해소 (앱 핸들러 지원 코드 확인:
   main.ts:1088 `canReplaceCurrentDocument(options?.skipUnsavedGuard)`).
   가드 자체 검증(unsaved-changes-guard)은 자체 emit 이라 무영향.

smoke 가 하네스 결함을 즉시 검출했다는 서사(수정 전 FAIL 증명, stash 분리)
포함 — 테스트가 테스트 인프라의 은폐 결함을 드러낸 좋은 사례.

## 로컬 재실증 (merged tree + 금일 6건 merge 반영 WASM 재빌드)

| 게이트 | 결과 |
|--------|------|
| undo-contracts (신규 smoke 포함) | **24 PASS / 0 FAIL**, 반복 2회 안정 |
| undo-object-selection-clear | 0 FAIL |
| unsaved-changes-guard | 6/6 (가드 무회귀) |
| studio 단위 / tsc | **307/307** / 0 에러 |

## 판단

**merge 권고.** P2 요청 범위(1종 smoke)에 정확히 부합 + 커버리지 표현
정직화 + 하네스 잠복 결함 수정의 3중 가치. 앱 소스 무변경이라 위험 표면
없음.
