# Task 315 수행계획서: 다단 설정 기능 구현 (v2)

## 1. 아키텍처 이해

### 조판 흐름
```
SectionDef (용지/여백) → ColumnDef (단 분할) → PageLayoutInfo (단별 영역) → Pagination (배치)
```

### ColumnDef의 위치와 역할
- **문단 컨트롤**로 저장 (`Control::ColumnDef`)
- 구역 첫 문단에 초기 ColumnDef (단정의) 존재 — SectionDef와 함께
- 구역 내 여러 ColumnDef 가능 (다단 설정 나누기)
- `find_initial_column_def()`: 구역의 초기 ColumnDef 추출
- `find_column_def_for_paragraph()`: 특정 문단에 적용되는 ColumnDef

### SectionDef와의 관계
- SectionDef: 용지 크기, 여백 → **body_area** 결정
- ColumnDef: body_area를 **단별 영역**으로 분할
- 두 컨트롤은 독립적 — ColumnDef만 수정해도 다단 변경 가능
- SectionDef 변경 API(setSectionDef)는 이미 구현됨

## 2. 구현 계획

### 2.1 단계 1: Rust API — setColumnDef
- 현재 구역의 **기존 ColumnDef를 찾아 수정** (find_initial_column_def 활용)
- 없으면 첫 문단에 삽입
- 수정 후: recompose_section + paginate + invalidate_page_tree_cache
- 파라미터: section_idx, column_count, column_type, same_width, spacing

### 2.2 단계 2: WASM 바인딩 + 프론트엔드
- wasm_api.rs에 setColumnDef 노출
- wasm-bridge.ts에 메서드 추가
- page.ts의 stub(col-1/2/3/left/right) → 실제 구현으로 교체
- index.html 메뉴 disabled 제거

### 2.3 단계 3: 테스트
- 1단→2단→3단→1단 전환
- 다단 상태에서 Ctrl+Shift+Enter (단 나누기)
- cargo test 716개 통과

## 3. 영향 범위
- `src/document_core/commands/text_editing.rs` — setColumnDef API
- `src/wasm_api.rs` — WASM 바인딩
- `rhwp-studio/src/command/commands/page.ts` — 프리셋 커맨드
- `rhwp-studio/src/core/wasm-bridge.ts` — WASM 브릿지
- `rhwp-studio/index.html` — 메뉴 활성화
