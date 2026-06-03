# Task 236 최종 완료 보고서

## 수행 결과

### 수정 1: 중첩 표 continuation 렌더링 수정

**파일**: `src/renderer/layout/table_partial.rs`

중첩 표 포함 셀의 PartialTable continuation에서 remaining 높이 계산 방식을 변경:

- **변경 전**: `cell.height - padding - offset` → cell.height가 실제 렌더링 높이보다 작아서 remaining ≈ 0
- **변경 후**: `calc_nested_split_rows().visible_height + om_top + om_bottom` → 내부 표의 실제 가시 행 높이를 직접 계산

핵심 로직:
1. 내부 표의 행 높이를 `resolve_row_heights`로 계산
2. `calc_nested_split_rows`로 split_start_content_offset 이후의 가시 행 범위 결정
3. `visible_height`에 내부 표의 `outer_margin_top/bottom` 추가

### 수정 2: 한글 폰트 메트릭 별칭 추가

**파일**: `src/renderer/font_metrics_data.rs`

| 한글 이름 | 영문 메트릭 |
|-----------|------------|
| 돋움, 함초롬돋움, 한컴돋움 | HCR Dotum |
| 바탕, 함초롬바탕, 한컴바탕 | HCR Batang |
| 맑은 고딕 | Malgun Gothic |
| 나눔고딕 | NanumGothic |
| 나눔명조 | NanumMyeongjo |

## 테스트 결과

- `cargo test`: 716 passed, 0 failed
- kps-ai.hwp 67-68페이지: 중첩 표 continuation 정상 렌더링 확인
- kps-ai.hwp 64페이지: space 너비 정상 확인
