---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_6662_stage9_terminal_clip_boundary.md
last_verified: 2026-09-06
---

# 열린 이슈 재검증 9단계: 마지막 셀 복원의 본문 경계

Issue: #6712, 집계 #6662. 시작 HEAD: `ef80a0e88`.

## 재현 및 원인 가설

- 8단계는 가정통신문 마지막 문구를 복원했지만 교육과정 문서의 글자 겹침이
  94건에서 97건으로 증가했다. 마지막 조각만 복원하는 후보도 같은 실패를 냈다.
- 실제 diagnostic에는 Table/TextRun과 Footer/TextRun의 교차가 있다. 마지막
  조각이라는 이유만으로 쪽 여백까지 글줄을 노출하면 꼬리말 영역을 침범할 수 있다.
- 복원할 글줄 전체가 현재 본문 경계 안에 있을 때만 마지막 셀 clip을 확장한다.
  기존 비종료 조각 복원·잔여물 억제, 페이지 밖 진단 기준은 변경하지 않는다.

## 수정 및 검증 계획

1. 기존 `current_body_area` 경계를 이용하고 값이 미설정인 독립 probe만 물리 쪽으로 fallback한다.
2. terminal 복원은 줄의 top이 아니라 bottom이 해당 경계 안에 있는지 확인한다.
3. 실패한 overlap partition 14, #6712의 footer/행 전진, #5862/#5863/#2007 계약을 재실행한다.
4. 통과 시 전체 overlap/overflow/off-canvas를 재검증하고 결과와 함께 별도 커밋한다.

기준선 완화, 샘플 예외 등록, 로그/임시 이미지 커밋, PR/push는 수행하지 않는다.

## 가설 분리와 원인 확정

- 본문 경계만 제한한 후보: #6712 계약 11개 통과, overlap partition 14 실패
  (12개 중 11 통과, 3.480초, exit 100). 겹침은 여전히 94 -> 97건이다.
- 교육과정 원본의 문단/중첩 표를 순회한 결과 Square Picture가 없다.
  이번 그림 앵커 변경이 아니라 마지막 셀 clip 복원 경로를 조사했다.
- 복원된 5개 셀은 `sec=1/pi=3/cell=83,87`, `sec=1/pi=10/cell=56`,
  `sec=5/pi=11/cell=558`, `sec=28/pi=2/cell=337`이다. 모두 본문 안에 있고,
  각각 행 끝이 29/46, 20/32, 92/114, 64/83으로 표의 마지막 행이 아니다.
- `end_unit == usize::MAX`는 해당 셀 분할의 끝이지 표 전체의 끝이 아니다.
  이후 행이 있는 셀까지 확장하면서 그 행의 글자와 교차했다.
- 마지막 조각 복원에 `cell_end_row == row_count`를 추가한다. 본문 경계 제한은
  꼬리말 침범 방지 조건으로 유지한다. 임시 추적 코드는 원인 확인 후 제거했다.

## 검증 결과와 다음 단계

- 마지막 행 조건을 추가한 후보: 80 tests passed, 2 slow, 997 filtered/skipped,
  87.937초, exit 0. #6712 11계약, #5862/#5863/#2007, 전체 16분할
  text-overlap/overflow-cell/off-canvas를 포함한다.
- 실패했던 text-overlap partition 14와 중국어 마지막 문구 가시성이 동시에 통과했다.
  기준선 파일은 변경하지 않았다. `cargo fmt --all`, `git diff --check`도 exit 0이다.
- 이 결과는 8단계에서 추가된 겹침 회귀의 국소 수정 완료다. 전체 9,082건 재실행이나
  #6712의 전체 시각적 정합 완료를 의미하지 않는다. 다음 단계는 글자가 있는 Square
  anchor 다음 문단이 같은 그림을 피해 흐르지 못하는 문제다.
- GitHub 목록 재확인: 최초 21개 중 #6354와 #6674가 닫혔고 19개는 열린 상태다.
  이번 단계에서는 이슈를 추가 종료하거나 PR/push하지 않았다.
