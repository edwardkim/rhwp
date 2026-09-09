# Task #1763 Stage 2 완료보고서 — 선언높이 권위 가드 + 테스트

## 수행 내용 (`src/renderer/height_measurer.rs`)
- 셀 마지막 줄 trailing ls 포함분(`cell_last_trailing_ls`)을 별도 산출
  (가로쓰기 + 비중첩 + 다문단 + **비 RowBreak** — TAC 여부와 무관하게 RowBreak 표 전체 제외).
- RowBreak 제외를 TAC 포함 전체로 넓힌 근거: TAC RowBreak 표(rowbreak-problem-pages
  p11~13)의 분할 배치는 trailing 포함 측정에 정합되어 있어, clamp 적용 시 셀높이
  축소 → 하류 배치 이동으로 body 하단 overflow 회귀 (통합테스트 3건 검출).
- required 판정에 가드: `required > 선언높이` 이고 초과분이 전적으로 trailing ls 때문
  (`content − trailing + pad ≤ 선언`)이면 **선언높이로 clamp**. 콘텐츠가 진짜 초과하는
  기존 보존 케이스(#874/#1086 — aift/KTX)는 조건 미충족으로 불변.
- 통합테스트 `tests/issue_1763_cell_trailing_ls_expand.rs` (render tree row0 셀 높이
  ≈142.2px 단언, serde_json 파싱).

## 검증
- 재현 파일: row0 셀 149.1 → **142.2px** (선언높이 = 한글 142.1 정합),
  표 전체 524.7 → **517.8px** (한글 517.5, 잔여 +0.3 = 측정 노이즈 수준).
- 통합테스트 통과, cargo check/fmt 통과.

## golden 스냅샷 갱신 (svg_snapshot::issue_677)
- clamp 로 복학원서.hwp p1 첫 표(TAC 5x4)의 총 필요높이가 선언 표높이 안으로
  들어가 비례 축소가 사라짐 → 행 높이가 선언값 그대로 렌더.
- 한글 2022 PDF(`pdf/복학원서-2022.pdf`) 픽셀 대조: 행 간격 60.0/51.0/60.0/53.5px 가
  갱신 렌더(60.05/50.72/60.05/53.81)와 정확 일치하고 PDF 대비 오프셋도 전 경계 상수
  (+8.6px). 구 golden(비례 축소치 58.58/49.47/58.58/56.03)은 오프셋 10.7→15.1 드리프트
  — **갱신 렌더가 정답 정합**이라 golden 재생성(UPDATE_GOLDEN=1).
- 참고: 원 CI 는 rowbreak 실패에서 fail-fast 로 멈춰 svg_snapshot 이 실행되지 않아
  본 diff 가 뒤늦게 드러남.

## 상태
완료. Stage 3 (회귀 검증 + 최종보고) 진행.
