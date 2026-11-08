---
kind: snapshot
status: archived
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-08
---

# PR #7464 보정 구현 기록

원 기능과 별도로 commit `282d5d77f6f7167a70047170dbad1a10bcce6ba5`에서 보정했습니다.

- `ModalDialog`의 document capture는 focused HTMLButtonElement의 Enter/Space를 직접 확인 버튼으로 치환하지 않습니다. browser default click을 허용하며 editor shortcut 전파는 차단합니다.
- `clampedCellAfterDelete`는 merged-cell rowSpan/colSpan 역조회 결과와 셀 문단 0, 복제한 마지막 path entry를 함께 반환합니다. 기존 history에 저장된 path를 mutate하지 않습니다.
- 셀 블록 구조 삭제와 일반 표 행/열 삭제가 같은 보정 값을 소비합니다. 표가 소멸하면 null로 본문 이동 경로를 유지합니다.
- depth-1 C3의 두 번째 문단에서 마지막 행/열을 지우는 실제 E2E는 삭제·Redo·이동 후 flat/path와 두 문단 좌표, rect 및 입력을 검사합니다. Undo에서 구조·내용·블록 복원도 확인합니다.
- Enter/Space 각각 삭제/남김/취소/닫기 실제 browser activation 검사와 direct clamp의 history 불변성 검사를 추가했습니다.

최종 결과와 원 commit 계보는 [검토 기록](pr_7464_review.md)에 있습니다. 실행 로그나 파생 manifest는 구현 산출물로 커밋하지 않습니다.
