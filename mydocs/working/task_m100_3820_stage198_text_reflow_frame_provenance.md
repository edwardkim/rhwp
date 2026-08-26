# Stage 198: 셀 텍스트 reflow의 source-frame provenance

## 문제

Stage 197은 `Table::dirty`로 원본 표와 셀 편집 표를 구분하려 했으나 실패했다.
`dirty`는 초기 측정 캐시 무효화에도 쓰이므로 sample16 HWP5 2010/2022의 원본
source-frame tail까지 막아 페이지 수가 각각 64에서 65/66으로 늘었다.

`LINE_SEG` tag도 충분한 provenance가 아니다. 셀 텍스트 reflow는 새 suffix line에
원본 line의 tag와 metrics를 계승할 수 있다. 따라서 line segment 값만으로 원본
frame과 편집 reflow line을 구분하면 giant-cell의 새 줄을 저장 frame에 다시 합친다.

## 보정

`Table::text_reflowed_after_edit` 런타임 상태를 추가했다.

- `replace_text_in_cell_native_impl`와 삭제 경로는 셀 문단 reflow와 vpos 재계산이
  완료된 뒤 해당 표에 provenance를 기록한다.
- native HWP5 stored-frame tail 연장은 이 provenance가 없는 원본 표에만 적용한다.
- 편집 reflow 표는 일반 capacity cut을 사용한다. 따라서 giant-cell의 새 줄은 기존
  source frame의 tail에 합쳐지지 않는다.
- 이 상태는 구조/측정 cache invalidation인 `dirty`와 분리되며, 저장 후 다시 열면
  새 `LINE_SEG`가 source가 되므로 기본값 `false`에서 시작한다.

문서 이름, 표 높이, 문단 수, 픽셀 기반 문서별 허용치는 판정에 사용하지 않는다.

## 검증 기준

- `issue_1035_alignment`: HWP5 2010/2018/2022/2024 sample16 모두 64페이지
- `issue_3820_hwp5_qa_rowbreak_tail_reduces_page_count`: HWP Q&A 383페이지 및 Q8 표제 위치
- `test_get_table_bbox_at_page_for_giant_multi_page_cell`: 원본 HWP/HWPX 115페이지
- #2214 deferred cache coherence와 #2424 resumable insert/delete: 편집 뒤 HWP/HWPX 115 fragment
- `issue_3930_hwpx_hwp_save_layout`: HWPX 383페이지

## PDF 증적

커밋에는 HWP 2020 MCP로 변환하고 `file`에서 `PDF document, version 1.7`을 확인한
sample16 PDF 여섯 개를 포함한다. `PDF 1.6 (zip deflate)`인 기존 2022 PDF는 MCP
2020 기준 산출물이 아니므로 증적에서 제외한다.
