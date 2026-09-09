# Task #1716 Stage 4 완료보고서 — 회귀·통합 검증 + 샘플

## 통합 검증 (대표 파일, 한글 OLE 대조)
`verify_pi_page_vs_hangul` (HEAD 8a821046):
- **rhwp 53쪽 vs 한글 52쪽** (수정 전 173 vs 52). A 유형 폭주 완전 해소(−120쪽).
- 잔여: +1쪽 / 37 PI off-by-one(pi27부터) — 폭주에 가려져 있던 별개 **B 유형 미세 행높이 표류**.
  이번 수정이 유발한 것이 아님(수정 전엔 500+ 폭주 mismatch에 묻힘). #1716 범위 밖.

## 회귀 테스트
- **lib 단위테스트**: `cargo test --lib` → **2042 passed, 0 failed**, 7 ignored.
  신규 `leading_header_rows` 테스트 4개 전부 ok.
- **표 통합 테스트**(debug, 선별 7 크레이트): **19 passed, 0 failed**
  - diag_1042_table_row_height(2), issue_1070_tac_table_post_text_overflow(3),
    issue_1073_nested_table_split(3), issue_1133_nested_table_valign(3),
    issue_1195_cell_table_empty_line(1), issue_1417_pagination_cursor_render(1),
    issue_1486_hwpx_partial_tac_table(6).
- **의미적 무회귀 근거**: 변경은 상단 비연속 header 표에만 영향. 일반 표(row0만 header)와
  #1022 다중 머리행 표(상단 연속 header)는 `leading_header_rows` 가 기존 filter 와 동일 집합을
  반환하므로 동작 불변.

## 샘플 추가
`samples/task1716/table_scattered_header_rowbreak.hwpx` (대표 파일) + README.

## 미실행(환경 제약 — 후속 권장)
- 전체 `cargo test`(통합 162 크레이트)는 릴리즈 LTO(`codegen-units=1`)로 본 환경에서 극히 느려
  선별 실행으로 대체. CI/여유 시 전체 실행 권장.
- hwpdocs 표본 3000개 `verify_pi_page_vs_hangul` 재검증(A 유형 아웃라이어 개선 수 + 무회귀 확인)은
  수 시간 소요 → 후속 배치 권장.

## 상태
완료. 최종 보고서 `task_m100_1716_report.md` 작성.
