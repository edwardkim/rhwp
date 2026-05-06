# Task #634 Stage 2 — Fix 적용

**브랜치**: `local/task634`
**이슈**: https://github.com/edwardkim/edwardkim/rhwp/issues/634
**구현계획서**: `mydocs/plans/task_m100_634_impl.md`
**Stage 1 보고서**: `mydocs/working/task_m100_634_stage1.md`
**진행 시점**: 2026-05-06

---

## 1. 변경 파일 (9개)

| 파일 | 변경 내용 | LOC |
|------|----------|-----|
| `src/renderer/page_number.rs` | `numbering_started: bool` 필드 + `new_with_started()` 생성자 + `show_for_last_page()` / `numbering_started()` 액세서 + 기존 mk_page 헬퍼 갱신 | +35 |
| `src/renderer/pagination.rs` | `PageContent.show_page_number: bool` (default `true`) + `PaginationOpts.numbering_started_initial: bool` | +9 |
| `src/renderer/pagination/state.rs` | `new_page_content` 초기화에 `show_page_number: true` 추가 | +1 |
| `src/renderer/pagination/engine.rs` | `finalize_pages()` 시그니처에 `numbering_started_initial: bool` 추가, `assigner.show_for_last_page()` 로 page.show_page_number 설정 | +6 |
| `src/renderer/typeset.rs` | `typeset_section_with_started()` 신규, `finalize_pages()` 시그니처 + 초기화 정합 | +30 |
| `src/document_core/queries/rendering.rs` | 전체 문서 `any_newnumber_in_doc` 사전 스캔 + `carry_numbering_started` carry, `paginate_with_measured_opts` / `typeset_section_with_started` 호출 부 갱신 | +15 |
| `src/renderer/layout.rs` | `build_page_number` 에 `if !page_content.show_page_number { return; }` 가드 | +6 |
| `src/renderer/layout/tests.rs` | mk_page 6곳 `show_page_number: true` 추가 | +6 (replace_all) |
| `src/renderer/page_number.rs` mk_page (테스트) | `show_page_number: true` 추가 | +1 |

총 약 100 LOC 증가.

## 2. 핵심 로직

### 2.1 PageNumberAssigner.numbering_started

```rust
pub fn new(new_page_numbers: &'a [(usize, u16)], initial: u32) -> Self {
    let numbering_started = new_page_numbers.is_empty();  // 단일 호출 시 A안
    Self { ..., numbering_started }
}

pub fn new_with_started(new_page_numbers, initial, prev_started: bool) -> Self {
    Self { ..., numbering_started: prev_started }  // 호출 측 결정 그대로 (휴리스틱 없음)
}

pub fn assign(&mut self, page) -> u32 {
    for ... NewNumber ... {
        ...
        self.numbering_started = true;  // 발화 시 영구 true
    }
    ...
}

pub fn show_for_last_page(&self) -> bool { self.numbering_started }
```

### 2.2 rendering.rs 구역간 carry

```rust
// 전체 문서 NewNumber 존재 여부 사전 스캔
let any_newnumber_in_doc = self.document.sections.iter().any(...);
let mut carry_numbering_started: bool = !any_newnumber_in_doc;  // A안 (관대)

for (idx, section) in ... {
    // dirty 아닌 구역도 carry 업데이트
    if !self.dirty_sections[idx] {
        if last.show_page_number { carry_numbering_started = true; }
        ...
    }

    // paginate 호출 시 carry 전달
    typesetter.typeset_section_with_started(..., carry_numbering_started)

    // 처리 후 carry 업데이트
    if let Some(last) = result.pages.last() {
        if last.show_page_number { carry_numbering_started = true; }
    }
}
```

### 2.3 build_page_number 가드

```rust
fn build_page_number(...) {
    if let Some(ref ph) = page_content.page_hide {
        if ph.hide_page_num { return; }
    }
    // Task #634: NewNumber 게이팅
    if !page_content.show_page_number { return; }
    if let Some(pnp) = &page_content.page_number_pos {
        if pnp.position == 0 { return; }
        ...
    }
}
```

## 3. 검증 결과

### 3.1 Task #634 통합 테스트 (5건)

```
test test_634_no_newnumber_doc_shows_page_numbers_from_page1 ... ok
test test_634_gukrip_page2_no_page_number_before_new_number ... ok
test test_634_gukrip_page3_shows_page_number_after_new_number ... ok
test test_634_aift_page2_no_page_number_before_new_number ... ok
test test_634_aift_page7_shows_page_number_after_new_number ... ok

test result: ok. 5 passed; 0 failed
```

**Stage 1 RED 2건 → GREEN 전환 확인.**

### 3.2 전체 단위 테스트

```
test result: ok. 1124 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out
```

**기존 1119 + 신규 5 = 1124 모두 통과. 회귀 0.**

## 4. 일시적 시행 착오 기록

첫 빌드 후 테스트 시 aift_page2 만 fail. 원인:
- `new_with_started(prev_started=false, new_page_numbers=[])` 가 `prev_started || empty` 로직에서 `true` 반환
- aift 의 구역 0 은 NewNumber 없으나 (구역 2 에 있음), 본 구역 기준으로만 보면 empty=true 적용 → 잘못 표시

**수정**: `new_with_started` 의 empty 검사 제거. 호출 측 (rendering.rs) 가 전체 문서 기준으로
판단한 `carry_numbering_started` 를 그대로 사용. **휴리스틱 회피, 룰 단순화**.

## 5. 다음 단계

Stage 3 (광범위 회귀 검증 + 최종 보고서):
1. 11개 HWP+PDF 짝 샘플 + pgnp-only 8건 SVG 출력 → 쪽번호 표시/미표시 매트릭스 검증
2. 한컴 PDF 와 footer 텍스트 op 카운트 비교 (가능한 PDF 짝)
3. 최종 보고서 작성 + closes #634

승인 후 진행.
