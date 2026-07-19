# Task 214 수행계획서: 단일 패스 레이아웃 엔진 전환 — Phase 1: 문단 조판

## 목표

현재 3단계 파이프라인(height_measurer → pagination → layout)을 단일 패스 조판 엔진으로 전환한다.
Phase 1에서는 **문단(비-표) 요소**의 조판을 단일 패스로 처리하여 측정과 배치를 통합한다.

## 현재 문제

1. height_measurer에서 측정한 높이와 layout에서 실제 배치하는 높이의 괴리
2. pagination이 측정값을 신뢰하므로 오차가 누적되어 overflow, 빈 페이지 발생
3. 피드백 루프가 없어 한 케이스를 고치면 다른 케이스가 깨지는 악순환 (7일+ 반복)

## 전환 전략

기존 684개 테스트를 보호하면서, **새로운 typeset 경로**를 기존 경로와 **병렬로** 구축한다.
새 경로가 동일한 결과를 내는 것이 검증되면 기존 경로를 제거한다.

## 구현 단계

### 1단계: TypesetEngine 프레임워크 구축

**목표**: 단일 패스 조판의 핵심 구조체와 인터페이스 정의

- `TypesetEngine` 구조체 신설 (src/renderer/typeset.rs)
- `PageState` — 현재 페이지의 남은 높이, 단 정보 관리
- `TypesetResult` — 조판 결과 (페이지별 PageContent 목록)
- 핵심 메서드:
  - `typeset_section()` — 진입점 (paragraphs + page_def + styles → TypesetResult)
  - `format_paragraph()` — 문단 높이 계산 (기존 height_measurer 로직 통합)
  - `fits()` — 남은 공간에 들어가는지 판단
  - `place()` — 현재 페이지에 배치 확정
  - `move_to_next_page()` — 다음 페이지로 이동 (높이 리셋)

**검증**: 구조체 정의 + 빈 구현 컴파일 확인

### 2단계: 문단 조판 구현

**목표**: 비-표 문단의 format → fits → place/split 흐름 구현

- `format_paragraph()`: 기존 `HeightMeasurer::measure_paragraph()` 로직을 통합
  - line_seg 기반 줄 높이 계산
  - spacing_before/after 반영
  - 비-인라인 이미지 높이 포함
- `fits()`: 문단 전체 높이 ≤ 남은 높이인지 판단
- `place()`: PageItem::FullParagraph 생성, y_offset 갱신
- `split_paragraph()`: 줄 단위 분할
  - 들어가는 줄까지 PageItem::PartialParagraph (master)
  - 나머지 줄을 다음 페이지에서 계속 (follow)
- 페이지/단 넘김: force_new_page, column_break 처리

**검증**: 비-표 문서(간단한 다중 페이지 문단)로 기존 pagination과 동일 결과 비교

### 3단계: 특수 케이스 처리 및 기존 경로 통합

**목표**: 다단, 머리말/꼬리말, 각주 등 특수 케이스 처리 및 기존 코드와 연결

- 다단(multi-column) 처리: ColumnDef에 따른 단 전환
- 머리말/꼬리말 영역 높이 차감
- 각주(footnote) 영역 높이 차감
- 페이지 번호 위치 결정
- 표/Shape는 기존 방식 유지 (Phase 2에서 전환)
  - 표를 만나면 기존 MeasuredTable 활용하여 fits/place/split
- `DocumentCore::paginate()`에 새 경로 연결
  - 기존 경로와 새 경로 모두 실행하여 결과 비교 (디버그 모드)

**검증**: 684개 테스트 PASS, kps-ai.hwp/hwpp-001.hwp overflow 0건, WASM 빌드 성공

### 4단계: 기존 경로 제거 및 정리

**목표**: 검증 완료 후 기존 3단계 파이프라인의 문단 관련 코드 정리

- `HeightMeasurer::measure_paragraph()` 제거 (표 측정은 유지)
- `Paginator` 내 문단 pagination 로직을 TypesetEngine으로 위임
- 불필요한 중간 데이터 구조 정리
- LayoutOverflow 자가 검증 0건 확인

**검증**: 684개 테스트 PASS, 전체 문서 시각적 정확도 유지, WASM 빌드 성공

## 위험 요소

1. **기존 테스트 regression**: 병렬 경로로 점진 전환하여 위험 최소화
2. **표 처리 호환**: Phase 1에서는 표를 기존 방식으로 유지, 인터페이스만 맞춤
3. **vpos 보정 로직**: 기존 layout.rs의 vpos 기반 y_offset 보정은 새 엔진에서 불필요해야 함

## 참고 문서

- [단일 패스 레이아웃 설계서](../../tech/archive/single_pass_layout_design.md)
