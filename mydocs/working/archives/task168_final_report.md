# 타스크 168 최종 결과보고서: 스타일 시스템 구현

## 개요

HWP 문서의 스타일(바탕글, 본문, 개요 1~7 등)을 편집기에서 조회·적용할 수 있도록 전체 파이프라인을 구현했다.

## 구현 범위

### 1단계: WASM API (Rust)

`src/wasm_api.rs`에 5개 메서드 추가:

| API | 기능 |
|-----|------|
| `getStyleList()` | 문서 전체 스타일 목록 JSON 반환 |
| `getStyleAt(sec, para)` | 문단의 스타일 조회 |
| `getCellStyleAt(sec, para, ctrl, cell, cellPara)` | 셀 내부 문단 스타일 조회 |
| `applyStyle(sec, para, styleId)` | 문단에 스타일 적용 |
| `applyCellStyle(sec, para, ctrl, cell, cellPara, styleId)` | 셀 내부 문단에 스타일 적용 |

### 2단계: 네이티브 스타일 적용 로직 (Rust)

`src/document_core/commands/formatting.rs`:

- `apply_style_native()` / `apply_cell_style_native()` — 스타일 적용 핵심 로직
- `resolve_style_para_shape_id(style_id, current_psid)` — 스타일별 ParaShape 결정
  - 개요 문단: `numbering_id` 보존 + `para_level`/`margin_left`만 변경
  - 일반 문단: 참조 문단의 ParaShape 사용 또는 스타일 기본값 폴백
- `find_reference_para_shape_for_style()` — 동일 스타일 기존 문단에서 ParaShape 참조
- `find_para_shape_with_nid_and_level()` — nid+head_type+level 일치 ParaShape 검색
- `parse_outline_level_from_style()` — 스타일명에서 개요 수준 파싱

### 3단계: Studio UI (TypeScript)

| 파일 | 변경 |
|------|------|
| `wasm-bridge.ts` | 5개 스타일 API 래퍼 |
| `toolbar.ts` | `initStyleDropdown()` + `cursor-style-changed` 이벤트 리스너 |
| `input-handler.ts` | `applyStyle()`, `changeOutlineLevel()`, `cursor-style-changed` 이벤트 발행 |
| `format.ts` | `format:apply-style`, `format:level-increase`, `format:level-decrease` 커맨드 |
| `index.html` | 스타일 드롭다운 기본값 제거, 수준 증가/감소 메뉴 활성화 |
| `main.ts` | `toolbar.initStyleDropdown()` 호출 |

## 해결한 버그

| 문제 | 원인 | 해결 |
|------|------|------|
| 커서 이동 시 스타일 드롭다운 미갱신 | 이벤트 미발행 | `cursor-style-changed` 이벤트 추가 |
| 스타일 변경 시 여백/들여쓰기 리셋 | 스타일 기본 ParaShape(여백 0) 사용 | 참조 문단 ParaShape 활용 |
| 개요 수준 변경 시 후속 번호 리셋 | `numbering_id` 변경으로 NumberingState 리셋 | 현재 문단의 `numbering_id` 보존 |
| PartialParagraph 번호 카운터 누락 | `build_page_tree` 리플레이에서 누락 | PartialParagraph 리플레이 추가 |
| 블록 표/인라인 표 번호 카운터 누락 | 표 경로에서 `apply_paragraph_numbering` 생략 | 표 경로에 카운터 진행 추가 |

## 테스트

- Rust 단위 테스트: 613개 통과 (3개 신규 NumberingState 테스트 포함)
- WASM 빌드: 성공
- Studio 빌드: 성공
- 브라우저 검증: `samples/biz_plan.hwp` 5페이지 개요 수준 변경 시 후속 번호 정상 재계산 확인

## 커밋 이력

| 커밋 | 내용 |
|------|------|
| `66082de` | 타스크 168: 스타일 시스템 구현 (WASM API + Studio UI + 번호 보존) |
| `6f753ae` | 타스크 168 보완: 한 수준 증가/감소 메뉴 활성화 + 경쟁 우위 추적 문서 |

## 부산물

- `mydocs/report/competitive_advantages.md` — 경쟁 우위 기능 추적 문서 신규 작성 (29개 항목)

---

*작성일: 2026-02-27*
