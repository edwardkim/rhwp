# Task #639 구현 계획서

**제목**: 한컴 호환 — cover-style 페이지 자동 쪽번호 미표시 fix 구현
**브랜치**: `local/task639` (pr-task637 base)
**상태**: Stage 0b (구현계획서 작성) — 승인 대기
**작성일**: 2026-05-06

---

## 1. 변경 개요

| 파일 | 변경 종류 | 변경 LOC (예상) |
|------|----------|---------------|
| `src/renderer/pagination/engine.rs` | 함수 시그니처 + 신규 헬퍼 + finalize_pages 분기 | +30~40 |
| `src/renderer/layout/integration_tests.rs` | 통합 테스트 5건 신규 | +120~150 |
| **합계 (코드)** | | **+150~190** |

src/ 코드 변경 자체는 작음 (~30 LOC). 테스트가 비중 큼.

## 2. 핵심 변경: `finalize_pages` 시그니처 + cover-style 룰

### 2.1 현재 코드 (engine.rs:1891)

```rust
fn finalize_pages(
    pages: &mut [PageContent],
    hf_entries: &[(usize, HeaderFooterRef, bool, HeaderFooterApply)],
    page_number_pos: &Option<crate::model::control::PageNumberPos>,
    page_hides: &[(usize, crate::model::control::PageHide)],
    new_page_numbers: &[(usize, u16)],
    _section_index: usize,
) {
    // ... 기존 로직 ...
    // PageHide: 해당 문단이 이 페이지에서 **처음** 시작하는 경우만 적용
    for (ph_para, ph) in page_hides {
        if Self::para_starts_in_page(page, *ph_para) {
            page.page_hide = Some(ph.clone());
            break;
        }
    }
    // ← 여기에 cover-style 룰 추가
}
```

### 2.2 변경 후

**(a)** `finalize_pages` 에 `paragraphs: &[Paragraph]` 추가:

```rust
fn finalize_pages(
    pages: &mut [PageContent],
    paragraphs: &[Paragraph],   // ← 신규 파라미터
    hf_entries: &[(usize, HeaderFooterRef, bool, HeaderFooterApply)],
    page_number_pos: &Option<crate::model::control::PageNumberPos>,
    page_hides: &[(usize, crate::model::control::PageHide)],
    new_page_numbers: &[(usize, u16)],
    _section_index: usize,
) {
```

**(b)** 호출 측 (engine.rs:498) 수정:

```rust
Self::finalize_pages(
    &mut st.pages,
    paragraphs,             // ← 추가
    &hf_entries,
    &page_number_pos,
    &page_hides,
    &new_page_numbers,
    section_index,
);
```

**(c)** 기존 PageHide 적용 직후 cover-style 룰 추가 (engine.rs:1986 부근):

```rust
// PageHide: 해당 문단이 이 페이지에서 **처음** 시작하는 경우만 적용
for (ph_para, ph) in page_hides {
    if Self::para_starts_in_page(page, *ph_para) {
        page.page_hide = Some(ph.clone());
        break;
    }
}

// [Issue #639] cover-style 페이지 자동 쪽번호 미표시
// 룰: items=1 + 단일 단 + 완전한 Table (PartialTable 아님) + tac=false
// → page_hide.hide_page_num=true 자동 설정
if page.page_hide.is_none() && Self::is_cover_style_page(page, paragraphs) {
    page.page_hide = Some(crate::model::control::PageHide {
        hide_page_num: true,
        ..Default::default()
    });
}
```

**(d)** 신규 헬퍼 함수 (engine.rs 의 `para_starts_in_page` 근처):

```rust
/// [Issue #639] cover-style 페이지 판정.
///
/// 룰: 단일 단(column=1) + 단일 항목(item=1) + 완전한 Table (PartialTable 아님) +
/// `treat_as_char=false` 일 때 한컴은 쪽번호 표시 안 함. Task #637 분석 결과
/// 174 샘플 중 aift.hwp 페이지 2, 3 만 매칭 (한컴 미표시와 일치).
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

## 3. 테스트 (Stage 1 — TDD RED)

### 3.1 추가 위치

`src/renderer/layout/integration_tests.rs` 모듈 끝 (line 1221 직전).

### 3.2 테스트 5건

```rust
// ─── Issue #639: cover-style 페이지 자동 쪽번호 미표시 ───

#[test]
fn test_639_aift_page2_cover_style_no_page_number() {
    let Some(core) = load_document("samples/aift.hwp") else { return; };
    let svg = core.render_page_svg_native(1).unwrap_or_default();  // 0-based: page 2 = 1
    assert!(!svg.is_empty(), "aift.hwp 페이지 2 SVG 가 비어있음");
    // page_number_pos.dash_char = '-' 이므로 형식은 "- N -"
    // 페이지 2 가 cover-style 이면 쪽번호 텍스트 "- 2 -" 가 footer 영역에 없어야 함
    assert!(
        !svg.contains(">- 2 -<"),
        "Issue #639: aift.hwp 페이지 2 (cover-style: items=1 + Table 35×27 tac=false) 의 \
         쪽번호 '- 2 -' 가 SVG 에 표시됨. 한컴 PDF 미표시와 불일치."
    );
}

#[test]
fn test_639_aift_page3_cover_style_no_page_number() {
    let Some(core) = load_document("samples/aift.hwp") else { return; };
    let svg = core.render_page_svg_native(2).unwrap_or_default();  // 0-based: page 3 = 2
    assert!(!svg.is_empty(), "aift.hwp 페이지 3 SVG 가 비어있음");
    assert!(
        !svg.contains(">- 3 -<"),
        "Issue #639: aift.hwp 페이지 3 (cover-style: items=1 + Table 14×17 tac=false) 의 \
         쪽번호 '- 3 -' 가 SVG 에 표시됨. 한컴 PDF 미표시와 불일치."
    );
}

#[test]
fn test_639_aift_page1_shows_page_number() {
    // 회귀 가드: 페이지 1 (items=2: tac=true Table + PartialPara) 은 표시 유지
    let Some(core) = load_document("samples/aift.hwp") else { return; };
    let svg = core.render_page_svg_native(0).unwrap_or_default();
    assert!(!svg.is_empty(), "aift.hwp 페이지 1 SVG 가 비어있음");
    assert!(
        svg.contains(">- 1 -<"),
        "Issue #639 회귀: aift.hwp 페이지 1 (items=2 — cover-style 미해당) 의 쪽번호 \
         '- 1 -' 가 표시되어야 함. SVG 에서 누락됨."
    );
}

#[test]
fn test_639_aift_page6_shows_page_number() {
    // 회귀 가드: 페이지 6 (items=18: 작은 Table tac=false + 17 paragraph) 는 표시 유지
    let Some(core) = load_document("samples/aift.hwp") else { return; };
    let svg = core.render_page_svg_native(5).unwrap_or_default();  // 0-based: page 6 = 5
    assert!(!svg.is_empty(), "aift.hwp 페이지 6 SVG 가 비어있음");
    // 페이지 6 의 page_number 는 본문 시작 페이지로 NewNumber 영향 받음 (rhwp 기준 값 사용)
    // SVG 에 footer 쪽번호가 어떤 형태로든 존재하는지 검증 — "- N -" 패턴 자체 존재 여부
    let has_any_footer_page_num = (0..100).any(|n| svg.contains(&format!(">- {} -<", n)));
    assert!(
        has_any_footer_page_num,
        "Issue #639 회귀: aift.hwp 페이지 6 (items=18 — cover-style 미해당) 의 쪽번호 \
         footer 가 SVG 에 표시되어야 함."
    );
}

#[test]
fn test_639_aift_page74_tac_true_table_shows_page_number() {
    // 회귀 가드: 페이지 74 (items=1 Table tac=true) 는 cover-style 룰 미매칭 → 표시 유지
    // Task #637 분석: tac=true 가 결정적 분리자 (한컴 PDF 표시 확정)
    let Some(core) = load_document("samples/aift.hwp") else { return; };
    let total = core.page_count();
    if total < 74 { return; }
    let svg = core.render_page_svg_native(73).unwrap_or_default();  // 0-based: page 74 = 73
    assert!(!svg.is_empty(), "aift.hwp 페이지 74 SVG 가 비어있음");
    let has_any_footer_page_num = (0..200).any(|n| svg.contains(&format!(">- {} -<", n)));
    assert!(
        has_any_footer_page_num,
        "Issue #639 회귀: aift.hwp 페이지 74 (items=1 + Table 2×2 tac=true) 의 쪽번호 \
         footer 가 SVG 에 표시되어야 함. tac=true 는 cover-style 룰 미매칭."
    );
}
```

### 3.3 Stage 1 RED 시나리오

1. fix 미적용 상태 (현재 코드) 에서 테스트 실행:
   - `test_639_aift_page2_cover_style_no_page_number` → **FAIL** (svg 에 "- 2 -" 포함)
   - `test_639_aift_page3_cover_style_no_page_number` → **FAIL**
   - `test_639_aift_page1_shows_page_number` → PASS (이미 표시됨)
   - `test_639_aift_page6_shows_page_number` → PASS
   - `test_639_aift_page74_tac_true_table_shows_page_number` → PASS

2. fix 적용 후 (Stage 2):
   - 5건 모두 PASS

## 4. Edge case 검토

| 케이스 | 처리 |
|--------|------|
| `paragraphs.get(para_idx)` 가 None | early `return false` |
| `para.controls.get(ctrl_idx)` 가 None | early `return false` |
| `controls[ctrl_idx]` 가 Table 가 아님 | 매치 실패 시 `return false` |
| 다단 페이지 (column_contents.len() > 1) | `return false` (cover-style 정의상 단일 단) |
| PartialTable | `PageItem::Table` 만 매칭하므로 자동 제외 |
| Shape, Picture | `PageItem::Table` 만 매칭하므로 자동 제외 |
| `page.page_hide.is_some()` (PageHide 컨트롤 이미 있음) | 룰 적용 안 함 (기존 PageHide 우선) |
| 페이지 0 (첫 페이지) | PageNumberPos 등록 후 첫 매칭 시 정상 동작. PageNumberAssigner 단조 증가 보장은 page_number 자체 (page_hide 와 무관) |

## 5. PageNumberAssigner 와 상호작용

룰 적용 시 `page.page_hide` 가 `hide_page_num=true` 로 설정되지만 `page.page_number` 자체는 변경되지 않음. PageNumberAssigner 는 page_hide 무관하게 page_number 를 단조 증가로 할당. **즉 cover-style 페이지는 번호를 가지지만 표시만 안 함** (Task #634 의 페이지 4, 5 PageHide 와 동일한 의미).

이 동작은 한컴과 일치 (Task #637 분석에서 확인 — 페이지 6 page_num=4 → "- 4 -" 표시, 페이지 5 page_num=2 → 미표시이지만 카운트됨).

## 6. 회귀 검증 전략 (Stage 3)

### 6.1 cargo test sweep

```bash
cargo test --release 2>&1 | tail -10
```

기존 통과 수 + 신규 5건 모두 통과 확인.

### 6.2 174 샘플 page_hide diff sweep

Python script (Stage 3 작성):
1. fix 미적용 baseline: `dump-pages` 출력 저장
2. fix 적용 후: 모든 샘플 dump-pages 재실행
3. diff 비교: aift.hwp 페이지 2, 3 외 변경 0건 확인

### 6.3 광범위 SVG 회귀

```bash
# 주요 샘플 SVG 출력 비교 (변경 없을 것)
for f in samples/synam-001.hwp samples/exam_*.hwp samples/2010-01-06.hwp ...; do
  ./target/release/rhwp export-svg "$f" -o /tmp/before-$(basename "$f")
done
# fix 적용 후 동일 명령 + diff
```

## 7. 메모리 룰 준수 재확인

- **rule_not_heuristic**: 룰이 `treat_as_char=false + items=1 + 단일 단 + 완전 Table` 명시
  비트/카운트 조합. 휴리스틱 임계값 (예: 표 비율) 도입하지 않음.
- **essential_fix_regression_risk**: Stage 3 의 174 샘플 sweep + cargo test sweep 으로
  회귀 0 확정. Task #637 의 사전 조사 (174 샘플 중 영향 페이지 2 만) 와 일관.

## 8. 마이그레이션 / 호환성

- 기존 PageHide 컨트롤 동작 변경 없음 (`page.page_hide.is_none()` 가드)
- 기존 PageNumberAssigner 동작 변경 없음
- HWPX/HWP3 파서 변경 없음 (pagination/engine 만 영향)
- 라운드트립 영향 없음 (page_hide 자동 설정은 렌더 시점 derived state)

## 9. 단계별 일정 (재확인)

| Stage | 작업 | 산출물 | 다음 승인 시점 |
|-------|------|--------|--------------|
| 1 | TDD RED 통합 테스트 5건 추가 | `task_m100_639_stage1.md`, `integration_tests.rs` (+150 LOC) | Stage 1 RED 검증 후 |
| 2 | engine.rs cover-style 룰 fix | `task_m100_639_stage2.md`, `engine.rs` (+30 LOC) | Stage 2 GREEN 검증 후 |
| 3 | 광범위 회귀 검증 + 최종 보고서 | `task_m100_639_stage3.md`, `task_m100_639_report.md` | 종료 |

---

**현재 상태**: 본 구현계획서 승인 대기.

승인 후 Stage 1 (TDD RED 통합 테스트 추가) 진입.
