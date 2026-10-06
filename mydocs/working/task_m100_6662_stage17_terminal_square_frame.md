---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_6662_stage17_terminal_square_frame.md
last_verified: 2026-09-06
---

# 열린 이슈 재검증 17단계: 마지막 Square 셀의 테두리

Issue: #6712. 시작 HEAD: `7da3db8c5`.

## 분석과 계획

- 중국어 2쪽 마지막 TextLine은 종료 조각 복구로 clip에 포함되지만 테두리는 복구 전
  `grid_row_y`로 그려져 footer 글자를 가로지른다. Table bbox 확장만으로는 선이 움직이지 않는다.
- native HWP5의 저장 Square 그림 어울림이 입증된 셀, 마지막 원본 행의 종료 조각만
  대상으로 복원한 글줄 아래의 유효 padding과 프레임 경계를 일치시킨다.
- 다음 조각, 다음 행, 본문 밖까지 확장하지 않는다. paginator cut/논리 소비 높이는 그대로 둔다.
  `common.height`를 모든 페이지 높이로 강제하는 식의 미입증 변경은 하지 않는다.
- footer와 수평 테두리의 분리 계약, 관련 제어군, 전체 회귀, 필수 lint 및 최종 시각 증적으로 검증한다.

## 구현과 단계 결과

- 마지막 source row의 terminal clip이 자기 TextLine을 복구한 경우, 검증된 Square 셀에만
  원본의 유효 하단 padding을 복원한다. 확장은 현재 body 하단으로 제한한다.
- native HWP5 루트 표의 마지막 continuation에서 같은 셀의 paint 경계와 마지막
  border grid를 일치시킨다. 반환하는 논리 소비 높이는 변경하지 않는다.
- 신규 footer/frame 계약 포함 **18 passed, 148 skipped, 0.286s, exit 0**.
  한국어·중국어 두 원본의 2쪽/어울림/본문 소유/중첩 표 위치 계약도 함께 통과했다.
- `cargo fmt --all` 통과. 이 단계에서는 focused 결과만 확정한다.
  다음 최종 검증 단계에서 전체 corpus와 lint, 새 sweep을 확인하기 전까지 PR 준비 완료나
  #6712 close를 선언하지 않는다. 로그/임시 산출물은 커밋하지 않는다.
