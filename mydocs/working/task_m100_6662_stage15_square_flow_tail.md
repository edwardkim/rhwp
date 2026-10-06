---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_6662_stage15_square_flow_tail.md
last_verified: 2026-09-06
---

# 열린 이슈 재검증 15단계: Square 셀 흐름과 마지막 테두리

Issue: #6712. 시작 HEAD: `429f5ac04`.

## 범위

- 한국어 중첩 표 뒤 관리 문단의 약 -9.9px, 한국어 2쪽 중첩 표 약 +6.7px,
  중국어 2쪽 footer와 겹치는 마지막 테두리의 원인을 구분한다.
- 원본 저장 줄, 문단 간격, nested table flow, 실제 페인트 경계와 기준 PDF를 대조한다.
  fixture 이름이나 쪽 번호에 맞춘 오프셋은 넣지 않는다.
- 기존 PDF를 재사용하고 새 sweep으로 최종 검증한다. 코드 변경이 필요하면 독립 계약과
  관련 대조군을 실행한다. 로그/중간 PNG/SVG/JSON은 커밋하지 않는다.
- #6712 완료 뒤 #6714/#6699/#6708을 포함한 현재 수정 범위의 PR 준비로 전환한다.
  나머지 열린 이슈의 추가 구현은 이 단계에서 하지 않는다.

## 분석과 수정

- native HWP5 Square 중첩 표 뒤의 빈 wrap 문단들은 이미 접히지만, 다음 문단의
  위치 보정은 첫 빈 문단만 읽었다. 빈 run 뒤 실제 본문의 저장 위치를 사용해야 한다.
- `stored_nested_table_wrap_successor`는 단일 paragraph-relative Square 표와 기존
  empty-wrap 증거, 단조 증가하는 실제 LINE_SEG, control 없는 후속 본문을 확인한다.
  일반/분할 셀 렌더러가 현재 조각에 포함된 successor에만 기존 vpos snap을 적용한다.
- 한국어 관리 문단 앞 간격은 182.8px에서 저장 값 192px로 복원됐다. 한국어 1쪽
  대응 본문 중앙 오차는 -0.5px이며, 두 원본 모두 기준 PDF와 같은 2쪽이다.

## 검증

- focused 계약: 16 passed, 148 skipped, 0.259s, exit 0.
- 6712/5862/5863/2007 및 text-overlap/off-canvas/overflow 제어군:
  **85 passed, 997 skipped, 88.036s, exit 0**. focused와 중복 합산하지 않는다.
- `cargo fmt --all` 통과. 전체 회귀와 PR 직전 필수 lint는 최종 수정 뒤 다시 실행한다.
- 기존 두 PDF로 render-tree/SVG 좌표를 비교했다. 한국어 1/2쪽은 새로운 visual sweep의
  `review_001.png`, `review_002.png`를 직접 확인했다. 로그/중간 산출물은 `/tmp`에만 둔다.

## 다음 단계

- 중첩 Square 표 자체의 y 오차와 중국어 마지막 테두리는 아직 남아 있다.
  이 단계의 성공을 #6712 전체 해결 또는 PR 준비 완료로 표시하지 않는다.
- 이후 단계도 #6712로 한정하며, 구현/검증 보고와 일반 커밋을 분리해 진행한다.
