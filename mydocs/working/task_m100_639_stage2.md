# Task #639 Stage 2 — 결정적 룰 fix 구현

**상태**: Stage 2 완료, 작업지시자 승인 대기
**작성일**: 2026-05-06
**브랜치**: `local/task639`

---

## 1. 목표

Issue #639 의 결정적 룰을 코드에 구현. Stage 1 RED 테스트 5건 모두 GREEN 전환 +
전체 cargo test sweep 회귀 0 확인.

## 2. 구현 발견 — 두 페이지네이션 경로 존재

구현 시작 시 의도한 위치 (`Paginator::finalize_pages` at `src/renderer/pagination/engine.rs`)
에만 fix 를 적용했으나 Stage 1 테스트가 여전히 FAIL. 원인 조사 결과:

`render_page_svg_native` → `build_page_tree` → `find_page` → `self.pagination` 경로의
페이지네이션은 **`TypesetEngine::typeset_section`** 으로 수행됨 (`src/document_core/queries/rendering.rs:1042-1068`):

```rust
let use_paginator = std::env::var("RHWP_USE_PAGINATOR").map(|v| v == "1").unwrap_or(false);
let mut result = if use_paginator {
    paginator.paginate_with_measured_opts(...)  // Paginator path (env opt-in fallback)
} else {
    use crate::renderer::typeset::TypesetEngine;
    let typesetter = TypesetEngine::new(self.dpi);
    typesetter.typeset_section(...)  // ★ 기본 경로
};
```

즉 `TypesetEngine::finalize_pages` (`src/renderer/typeset.rs:2131`) 가 기본 main path 이며,
`Paginator::finalize_pages` (`src/renderer/pagination/engine.rs:1892`) 는 fallback.

**결론**: 두 경로 모두에 동일 룰 적용 필요.

## 3. 변경 사항

### 3.1 `src/renderer/pagination/engine.rs` (Paginator path, fallback)

```rust
// (a) 호출 측 시그니처 갱신 (line 498)
Self::finalize_pages(&mut st.pages, paragraphs, &hf_entries, ...);

// (b) finalize_pages 시그니처에 paragraphs 추가 (line 1892)
fn finalize_pages(
    pages: &mut [PageContent],
    paragraphs: &[Paragraph],   // ← 신규
    hf_entries: &[(usize, HeaderFooterRef, bool, HeaderFooterApply)],
    ...
)

// (c) 기존 PageHide 적용 직후 cover-style 룰 추가 (line 1982~)
if page.page_hide.is_none() && Self::is_cover_style_page(page, paragraphs) {
    page.page_hide = Some(crate::model::control::PageHide {
        hide_page_num: true,
        ..Default::default()
    });
}

// (d) 신규 헬퍼 (line 2018~)
fn is_cover_style_page(page: &PageContent, paragraphs: &[Paragraph]) -> bool {
    if page.column_contents.len() != 1 { return false; }
    let items = &page.column_contents[0].items;
    if items.len() != 1 { return false; }
    let (para_idx, ctrl_idx) = match &items[0] {
        PageItem::Table { para_index, control_index } => (*para_index, *control_index),
        _ => return false,
    };
    let Some(para) = paragraphs.get(para_idx) else { return false; };
    let Some(Control::Table(t)) = para.controls.get(ctrl_idx) else { return false; };
    !t.common.treat_as_char
}
```

### 3.2 `src/renderer/typeset.rs` (TypesetEngine path, 기본)

동일한 (a) (b) (c) (d) 변경 적용 (line 768, 2131, 2196~, 2207~).

### 3.3 변경 통계

| 파일 | LOC 추가 | LOC 삭제 |
|------|---------|---------|
| `src/renderer/pagination/engine.rs` | +27 | -2 |
| `src/renderer/typeset.rs` | +27 | -2 |
| **합계 (코드)** | **+54** | **-4** |

## 4. Stage 1 테스트 GREEN 전환

```
$ cargo test --release --lib test_639

running 5 tests
test renderer::layout::integration_tests::tests::test_639_aift_page6_shows_page_number ... ok
test renderer::layout::integration_tests::tests::test_639_aift_page2_cover_style_no_page_number ... ok
test renderer::layout::integration_tests::tests::test_639_aift_page3_cover_style_no_page_number ... ok
test renderer::layout::integration_tests::tests::test_639_aift_page1_shows_page_number ... ok
test renderer::layout::integration_tests::tests::test_639_aift_page74_tac_true_table_shows_page_number ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 1136 filtered out
```

**5건 모두 PASS** — RED → GREEN 전환 성공.

## 5. 전체 cargo test sweep — 회귀 0

```
$ cargo test --release 2>&1 | grep "test result:"
test result: ok. 1139 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out  # lib (기본)
test result: ok. 14 passed; 0 failed; 0 ignored
test result: ok. 25 passed; 0 failed; 0 ignored
... (다수 테스트 크레이트, 모두 0 failed)
test result: ok. 6 passed; 0 failed; 0 ignored
test result: ok. 1 passed; 0 failed; 0 ignored
```

전체 **1139 + 76+ 통합 테스트 모두 PASS, 0 failed**. Stage 2 fix 회귀 0 확정.

| 분류 | baseline (Stage 1 시점) | Stage 2 적용 후 | 변화 |
|------|----------------------|---------------|------|
| 라이브러리 lib 테스트 | 1136 통과 / 3 fail (RED 의도) | **1139 통과 / 0 fail** | RED 3 → GREEN 3 |
| 외부 통합 테스트 | 모두 PASS | 모두 PASS | 0 |

**clippy**: warning 0.

## 6. Edge case 검증 (구현계획서 4 절)

| 케이스 | 검증 |
|--------|------|
| `paragraphs.get(para_idx)` None | `let Some(...) else { return false; }` |
| `para.controls.get(ctrl_idx)` None | 동일 패턴 |
| `controls[ctrl_idx]` 가 Table 가 아님 | `let Some(Control::Table(t)) else { return false; }` |
| 다단 페이지 (column_contents.len() > 1) | `if != 1 return false` |
| PartialTable | `PageItem::Table` 만 매칭 (PartialTable 자동 제외) |
| Shape, Picture | 동일 (PageItem::Table 만 매칭) |
| 기존 PageHide 컨트롤 있음 | `page.page_hide.is_none()` 가드 — 기존 PageHide 우선 |
| 페이지 1, 6, 74 | Stage 1 회귀 가드 PASS 확인 |

## 7. 메모리 룰 준수 재확인

- **rule_not_heuristic**: 룰이 `treat_as_char=false + items=1 + 단일 단 + 완전 Table` 명시
  비트/카운트 조합. 휴리스틱 임계값 미도입.
- **essential_fix_regression_risk**: cargo test sweep 전체 PASS + Stage 1 5건 GREEN 전환.
  Task #637 의 174 샘플 전수 조사로 사전 확인된 회귀 위험과 일관 (영향 페이지 2 만).
- **pdf_not_authoritative**: 검증은 IR 기반 (page_hide 자동 설정 → SVG footer 글리프
  카운트 = 0). PDF 측정 불요.

## 8. Stage 3 진입 준비

Stage 3 에서:
- 174 샘플 dump-pages page_hide diff sweep (베이스라인 vs fix 적용)
- 주요 샘플 SVG export 비교
- 최종 보고서 작성

---

**Stage 2 결과**: cover-style 자동 미표시 룰 두 페이지네이션 경로 (Paginator + TypesetEngine)
모두 적용. Stage 1 RED → GREEN 전환 5건. cargo test sweep 회귀 0. clippy warning 0.

승인 후 Stage 3 (광범위 회귀 검증 + 최종 보고서) 진입.
