# Task #639 최종 보고서

**제목**: 한컴 호환 — cover-style 페이지 (items=1 + 완전한 Table + tac=false) 자동 쪽번호 미표시
**브랜치**: `local/task639` (pr-task637 base)
**이슈**: https://github.com/edwardkim/rhwp/issues/639
**Milestone**: M100 (v1.0.0)
**상태**: **완료 (Stage 0 → 1 → 2 → 3)**
**완료 시점**: 2026-05-06

---

## 1. 배경

Task #637 (분석, closes #637) 의 결정적 룰을 코드에 구현.

**룰**:
> 페이지가 `items=1` 인 단일 완전한 Table (PartialTable 아님) 을 포함하고
> 그 Table 의 `treat_as_char=false` 일 때 한컴은 쪽번호 표시 안 함.

`samples/aift.pdf` 페이지 2 (사업계획서 표지, 35×27 표) 와 페이지 3 (요약문, 14×17 표) 이 매칭.

## 2. 결과

### 2.1 코드 변경

| 파일 | 변경 |
|------|------|
| `src/renderer/typeset.rs` | finalize_pages 시그니처 + cover-style 룰 + is_cover_style_page 헬퍼 (+29 LOC) |
| `src/renderer/pagination/engine.rs` | 동일 (+29 LOC, fallback 경로 일관성) |
| `src/renderer/layout/integration_tests.rs` | Task #639 통합 테스트 5건 (+99 LOC) |

**총 코드 추가**: +157 LOC. 테스트 +99, 코어 fix +58.

### 2.2 핵심 발견 — 두 페이지네이션 경로

rhwp 의 페이지네이션은 두 경로 존재:
- **`TypesetEngine::typeset_section`** (기본 main path, `src/renderer/typeset.rs`)
- **`Paginator::paginate_with_measured`** (`RHWP_USE_PAGINATOR=1` fallback, `src/renderer/pagination/engine.rs`)

`render_page_svg_native` → `build_page_tree` → `find_page` → `self.pagination` 경로는
TypesetEngine 사용. 두 경로 모두에 동일 fix 적용.

### 2.3 구현 룰

```rust
// finalize_pages 내부, PageHide 적용 직후
if page.page_hide.is_none() && Self::is_cover_style_page(page, paragraphs) {
    page.page_hide = Some(crate::model::control::PageHide {
        hide_page_num: true,
        ..Default::default()
    });
}

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

`page.page_hide.is_none()` 가드로 기존 PageHide 컨트롤 우선. 매칭 시 derived
PageHide 인스턴스로 hide_page_num=true 만 설정 (header/footer/master 등은 false).

## 3. 검증

### 3.1 통합 테스트 (5건 신규)

| 테스트 | 기대 | 결과 |
|--------|------|------|
| `test_639_aift_page2_cover_style_no_page_number` | footer 글리프 0 | ✓ PASS |
| `test_639_aift_page3_cover_style_no_page_number` | footer 글리프 0 | ✓ PASS |
| `test_639_aift_page1_shows_page_number` | footer 글리프 ≥ 3 (회귀 가드) | ✓ PASS |
| `test_639_aift_page6_shows_page_number` | footer 글리프 ≥ 3 (회귀 가드) | ✓ PASS |
| `test_639_aift_page74_tac_true_table_shows_page_number` | footer 글리프 ≥ 3 (tac=true 분리자) | ✓ PASS |

검출 패턴: SVG 의 `<text y="1079.16" font-size="10">` 글리프 카운트.

### 3.2 174 샘플 룰 매칭 재확인

Python 분석 스크립트 결과:
```
Total pages matching rule (items=1 + Table + tac=false): 2
  aift.hwp page 2
  aift.hwp page 3
```

Task #637 분석과 정확 일치. **174 샘플 중 영향 페이지 2 만**.

### 3.3 cargo test 전수 검증

```
test result: ok. 1139 passed; 0 failed; 2 ignored          # lib (기본)
test result: ok. 14 passed; 0 failed; 0 ignored             # outline_numbering
test result: ok. 25 passed; 0 failed; 0 ignored             # issue_369_master_page
... (다수 테스트 크레이트, 모두 0 failed)
```

**총 1139 + 76+ 통합 테스트 PASS, 0 failed**. 회귀 0.

### 3.4 clippy

```
$ cargo clippy --release --lib
(warning, error 0)
```

### 3.5 페이지 카운트 무변화

aift.hwp 페이지 수 77 유지.

## 4. 4단계 진행 요약

| Stage | 상태 | 산출물 |
|-------|------|--------|
| 0a | 완료 | `task_m100_639.md` (수행계획서) |
| 0b | 완료 | `task_m100_639_impl.md` (구현계획서) |
| 1 | 완료 | `task_m100_639_stage1.md` + integration_tests.rs (+99 LOC, RED) |
| 2 | 완료 | `task_m100_639_stage2.md` + engine.rs/typeset.rs (+58 LOC, GREEN) |
| 3 | 완료 | `task_m100_639_stage3.md` (회귀 검증) |

## 5. 회귀 위험 평가 (최종)

| 항목 | 영향 | 검증 |
|------|------|------|
| 라운드트립 | 0 | page_hide 는 렌더 시점 derived state |
| HWPX 호환성 | 0 | 파서 변경 없음 |
| 페이지 분할 | 0 | aift.hwp 페이지 카운트 77 유지 |
| header/footer | 0 | page_hide.hide_page_num 만 설정 |
| PageNumberAssigner | 0 | page_hide 와 page_number 별도 처리 |
| 다른 173 샘플 | 0 | 174 샘플 전수 조사로 룰 매칭 0건 확인 |

**회귀 위험 0 최종 확정**.

## 6. 메모리 룰 준수

- **[feedback_pdf_not_authoritative]**: 검증은 IR 기반 (page_hide → SVG footer 글리프
  카운트). PDF 측정은 Task #637 에서 완료.
- **[feedback_rule_not_heuristic]**: 룰이 `treat_as_char=false + items=1 + 단일 단 +
  완전 Table` 명시 비트/카운트 조합. 휴리스틱 임계값 미도입. 174 샘플 매칭 정확도로
  결정성 재확정.
- **[feedback_essential_fix_regression_risk]**: Task #637 의 174 샘플 사전 조사 +
  Stage 3 의 cargo test sweep + SVG 비교 검증으로 회귀 0 확정. Task #634 의 잘못된
  가설 H1'' (NewNumber 게이팅) 시도와 다르게 결정적 룰만 정확히 정정.

## 7. 학습한 교훈

1. **두 페이지네이션 경로 존재**: rhwp 의 페이지네이션은 `TypesetEngine` (기본) +
   `Paginator` (RHWP_USE_PAGINATOR=1 fallback) 양분. 페이지 단위 derived state
   (page_hide, page_number_pos 등) 변경 시 양쪽 동일하게 적용 필수.
2. **derived state 의 위치**: page_hide 와 같이 "렌더 시점에 결정되는 상태" 는 paginate
   직후의 finalize_pages 가 적합. PageNumberAssigner 단계에서 결정하면 page_number
   할당 로직과 결합되어 회귀 위험 증가.
3. **TDD 의 검출 패턴 사전 조사 가치**: `examples/probe_637.rs` 로 SVG 출력 형식 사전
   파악 (footer 쪽번호가 글자별 분리 `<text>` + y="1079.16" + font-size="10") 후에야
   결정적 검출 패턴 도출. 첫 시도 (`>- N -<` 단일 문자열 검색) 는 SVG 구조 가정
   잘못으로 실패.

## 8. Issue 처리

본 task 완료 후:
- **#639 close-as-fixed** (closes #639)
- **#637 (analysis) 은 이미 close** (closed-as-analysis-complete, 2026-05-06)

## 9. 산출물

| 파일 | 설명 |
|------|------|
| `mydocs/plans/task_m100_639.md` | 수행 계획서 |
| `mydocs/plans/task_m100_639_impl.md` | 구현 계획서 |
| `mydocs/working/task_m100_639_stage1.md` | Stage 1 RED |
| `mydocs/working/task_m100_639_stage2.md` | Stage 2 GREEN (fix 구현) |
| `mydocs/working/task_m100_639_stage3.md` | Stage 3 회귀 검증 |
| `mydocs/report/task_m100_639_report.md` | 본 최종 보고서 |
| `src/renderer/typeset.rs` | TypesetEngine cover-style 룰 (+29 LOC) |
| `src/renderer/pagination/engine.rs` | Paginator cover-style 룰 (+29 LOC) |
| `src/renderer/layout/integration_tests.rs` | Task #639 통합 테스트 5건 (+99 LOC) |

## 10. 다음 단계

작업지시자 승인 후:
1. orders 갱신 (#639 완료)
2. GitHub issue #639 close (closes #639)
3. PR 생성: `local/task639` → `edwardkim/rhwp:devel`

---

**완료 상태**: 결정적 룰 fix 구현 완료. Stage 1 RED → Stage 2 GREEN → Stage 3 회귀 0
확정. 한컴 호환 개선 — aift.hwp cover-style 페이지 2 페이지 정합성 회복.
