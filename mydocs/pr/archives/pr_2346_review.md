# PR #2346 검토 — 셀 숫자 서식 히스토리 기록 (#2344, 연작 4)

- PR: https://github.com/edwardkim/rhwp/pull/2346 (lpaiu-cs) — Fixes #2344
- 충돌 0 (연작 1~3 merge 후 클린)

## 변경 본질

쉼표/자릿점 서식이 미기록 → 셀 문자 수 변화로 후속 undo 오프셋이 오염돼
**텍스트 손상**(undo 불가를 넘는 데이터 손상 계급). delete+insert 를
cellNumberFormat snapshot 하나로 **원자화**해 동파일 기존 패턴
(applyTableInsertRowColumn·safeTableOp)과 동형 라우팅. 읽기 계산은
operation 밖 유지, 수동 document-changed 제거(라우터 refresh 위임).
소스 가드 동반. 잔여(블록 계산)는 #2348 에서 처리 예정 명시.

## 로컬 재실증 (merged tree)

가드 2/2 · studio **343/343** · tsc 0 · e2e undo-contracts 24/0.
브라우저 손상 게이트(1234567→쉼표→undo 정확 복원)는 컨트리뷰터 실측.

## 판단

**merge 권고.** 데이터 손상 축의 원자화 정정 — 동형 패턴 준수로 위험 최소.
