# 타스크 36: 표 테두리 처리 고도화 - 최종 결과보고서

## 개요

표 렌더링의 두 가지 핵심 문제를 해결했다:
1. 그라데이션 배경이 투명으로 렌더링되는 문제
2. 인접 셀 테두리가 이중으로 렌더링되는 문제

추가로 헤더행 반복 시 일부 셀이 누락되는 버그도 수정했다.

## 수행 내역

### 1단계: 그라데이션 채우기 렌더링

**문제**: k-water-rfp.hwp 1페이지 표의 상단/하단 장식 셀 배경이 투명으로 출력됨

**원인**: 렌더링 파이프라인 4개 레이어 모두 그라데이션을 지원하지 않았고, HWP 파서의 그라데이션 필드 크기가 스펙 문서 오류로 잘못 파싱되어 OOM 발생

**해결**:
- `GradientFillInfo` 구조체 정의 및 렌더 트리 노드 확장
- Style Resolver에서 `FillType::Gradient` 해소 로직 추가
- SVG: `<defs>` 섹션 관리, `<linearGradient>/<radialGradient>` 생성
- Canvas: `createLinearGradient()/createRadialGradient()` API 활용
- HWP 파서: 그라데이션 필드 크기 수정 (스펙 문서 오류, 레퍼런스 구현 기반)

### 2단계: 인접 셀 테두리 중복 제거

**문제**: 각 셀이 4방향 테두리를 독립적으로 그려 인접 셀 경계에서 테두리가 2회 렌더링됨

**해결**: 엣지 기반 수집/병합/렌더링으로 전환
- `h_edges[row_boundary][col]`, `v_edges[col_boundary][row]` 그리드 구조
- `merge_border()`: 두 테두리 중 우선순위 높은 것 선택 (굵기 > 종류)
- `collect_cell_borders()`: 셀의 4방향 테두리를 그리드에 수집 (병합 셀 지원)
- `render_edge_borders()`: 연속 같은 스타일 세그먼트를 하나의 Line으로 병합 렌더링
- 4개 테이블 레이아웃 함수 모두 적용 (`layout_table`, `layout_partial_table`, `layout_nested_table`, `layout_embedded_table`)

### 추가 수정: 헤더행 반복 버그

**문제**: `layout_partial_table()`에서 제목행 반복 시 `is_header` 속성이 없는 개별 셀을 건너뛰어 일부 셀이 누락됨

**원인**: HWP 편집기는 행 0에 `is_header` 셀이 하나라도 있으면 해당 행의 **모든 셀**을 반복하지만, 코드는 개별 셀의 `is_header` 플래그를 체크

**해결**: `if is_repeated_header_cell && !cell.is_header { continue; }` 조건 제거

## 변경 파일

| 파일 | 변경 내용 |
|------|-----------|
| `src/parser/doc_info.rs` | 그라데이션 파싱 필드 크기 수정 (스펙 오류 대응) |
| `src/parser/control.rs` | 관련 수정 |
| `src/renderer/mod.rs` | `GradientFillInfo` 구조체 정의 |
| `src/renderer/style_resolver.rs` | `ResolvedBorderStyle` 확장, gradient 해소 로직 |
| `src/renderer/layout.rs` | 엣지 기반 테두리 4개 헬퍼 함수, 4개 레이아웃 함수 수정, 헤더행 반복 버그 수정 |
| `src/renderer/svg.rs` | SVG gradient 렌더링 (`<defs>`, `<linearGradient>`, `<radialGradient>`) |
| `src/renderer/web_canvas.rs` | Canvas gradient 렌더링 |

## 검증 결과

- **단위 테스트**: 416개 전체 통과
- **k-water-rfp.hwp**: 30페이지 전체 SVG 내보내기 성공
  - 1페이지: 장식 행에 `<radialGradient>` 정상 렌더링
  - 6페이지: 헤더행 4개 셀 모두 반복 표시
- **전체 샘플**: 20개 HWP 파일 모두 정상 내보내기 확인
- **WASM 빌드**: 정상 완료

## 미수행 항목

구현 계획서의 3단계(꼭짓점 처리 및 페이지 분할 경계선)와 4단계(대각선 테두리)는 이번 타스크에서 수행하지 않았다. 필요 시 후속 타스크로 진행 가능하다.
