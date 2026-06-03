# Task #705 Stage 2 — GREEN 결함 #1 정정 (engine.rs + typeset.rs)

## 산출물

- `src/renderer/pagination/engine.rs` — `collect_header_footer_controls` + 헬퍼 추가 (+25 line)
- `src/renderer/typeset.rs` — 동일 패턴 (+27 line)
- `mydocs/working/task_m100_705_stage2.md` (본 보고서)

## 핵심 발견 — 두 페이지네이션 경로

Stage 2 시작 시 engine.rs 만 수정 → 4건 RED 유지. 측정 결과 **`TypesetEngine::typeset_section`** (typeset.rs:360) 가 main path:

| 경로 | 위치 | 호출 시점 |
|------|------|----------|
| **TypesetEngine::typeset_section** | `src/renderer/typeset.rs:2120` | 기본 main path (`render_page_svg_native` → `build_page_tree` → `find_page`) |
| Paginator::paginate_with_measured | `src/renderer/pagination/engine.rs:504` | `RHWP_USE_PAGINATOR=1` fallback |

PR #641 description 의 "두 페이지네이션 경로 양분" 진단 정합. 본 결함 #1 정정도 양쪽 모두 필요.

## 코드 변경

### engine.rs (`collect_header_footer_controls`)

```rust
// 추가:
Control::Table(table) => {
    Self::collect_pagehide_in_table(table, pi, &mut page_hides);
}

// 신규 헬퍼:
fn collect_pagehide_in_table(
    table: &crate::model::table::Table,
    pi: usize,
    page_hides: &mut Vec<(usize, crate::model::control::PageHide)>,
) {
    for cell in &table.cells {
        for cp in &cell.paragraphs {
            for ctrl in &cp.controls {
                match ctrl {
                    Control::PageHide(ph) => {
                        page_hides.push((pi, ph.clone()));
                    }
                    Control::Table(inner) => {
                        Self::collect_pagehide_in_table(inner, pi, page_hides);
                    }
                    _ => {}
                }
            }
        }
    }
}
```

외부 paragraph index `pi` 를 그대로 사용 — 페이지 매핑 정합성 유지 (셀 안 PageHide 가 외부 paragraph 의 페이지에 적용).

### typeset.rs (`collect_header_footer_controls` at :2120)

동일 패턴 적용 (impl 분리되어 있어 별도 헬퍼).

## 회귀 위험 평가

### 본질 정정 RED → GREEN

```
running 4 tests
test test_705_aift_page2_cell_pagehide_six_fields ... ok
test test_705_aift_page3_cell_pagehide_collected ... ok
test test_705_aift_cell_pagehides_total_count ... ok
test test_705_aift_page2_cell_pagehide_collected ... ok

test result: ok. 4 passed; 0 failed
```

### 전체 회귀 sweep

```
cargo test --release --lib
test result: ok. 1123 passed; 0 failed; 1 ignored
```

→ **0 fail** (기존 1119 + 신규 4 = 1123). 회귀 0.

## 중첩 표 (depth 2+) 처리

Stage 0 의 198 샘플 sweep 결과 실측 0 건. 그러나 `collect_pagehide_in_table` 의 `Control::Table(inner) => Self::collect_pagehide_in_table(inner, pi, page_hides)` 재귀 호출로 미래 케이스 대비 보존.

## 결함 #2 (border/fill 가드) 미정정 영향

현재 결함 #1 정정으로 `page.page_hide` 6 필드가 정확히 채워짐. 그러나 `layout.rs:411,414` 의 `build_page_background()` + `build_page_borders()` 호출에 `hide_fill`/`hide_border` 가드 없음. 셀 안 PageHide 의 6 필드 중 page_num/header/footer/master 는 적용되나 fill/border 는 미적용 상태.

→ Stage 3 에서 layout.rs 가드 추가.

## 영향 매트릭스 (예상)

aift.hwp:
- page 2 (s0/p[1]) — page_hide 6 필드 모두 true 로 채워짐
  - hide_page_num = true → page_number 미렌더 (이전: 표시) ✓
  - hide_header/footer = true → 머리말/꼬리말 미렌더 (이전: 표시) ✓
  - hide_master_page = true → 바탕쪽 미렌더 (이전: 표시) ✓
  - hide_border/fill = true → **layout.rs 가드 미적용 → 여전히 렌더** ✗ (Stage 3)
- page 3 (s1/p[0]) — hide_page_num=true → page_number 미렌더 ✓

기타 영향 샘플 (Stage 0 측정):
- 2022년 국립국어원 업무계획.hwp 목차 페이지 — hide_page_num=true → 미렌더
- KTX.hwp 목차 페이지 — 동일
- kps-ai.hwp 목차 페이지 — 동일
- tac-img-02.hwp/.hwpx — 다양한 패턴 적용

## Stage 3 진입 결정

**Stage 3 (GREEN 결함 #2 정정)** 진입 가능:

1. `src/renderer/layout.rs:411,414` 의 `build_page_background()` + `build_page_borders()` 호출에 `hide_fill`/`hide_border` 가드 추가
2. 회귀 sweep (PR #638 회귀 가드는 본 브랜치에 미존재 — Stage 5 에서 174 sample sweep)

## 관련

- 수행 계획서: `mydocs/plans/task_m100_705.md`
- 구현 계획서: `mydocs/plans/task_m100_705_impl.md`
- Stage 0 보고서: `mydocs/working/task_m100_705_stage0.md`
- Stage 1 보고서: `mydocs/working/task_m100_705_stage1.md`
- 본 보고서: `mydocs/working/task_m100_705_stage2.md`
