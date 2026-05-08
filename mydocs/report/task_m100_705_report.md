# Task #705 최종 결과 보고서

## 한 줄 요약

한컴 호환 셀 안 PageHide 컨트롤의 본 환경 페이지네이션·렌더러·dump 적용 결함 3건을 본질 정정. PR #638 (Task #634) close 시 메인테이너 발견의 후속 본질 PR.

## 배경

PR #638 (Task #634) close 시 메인테이너 (@edwardkim) 가 본 환경 페이지네이션의 본질 결함 발견:

- aift.hwp page 2 의 셀[167] paragraph[3] 에 PageHide 컨트롤 6 필드 모두 true 로 인코딩
- 본 환경 파서는 정확 인식, 그러나 페이지네이션·렌더러·dump 3 곳에 결함

PR #641 (Task #639, cover-style 휴리스틱) 의 우회 접근 폐기 후 본 task 로 본질 정정.

## 정정한 결함 3건

| # | 위치 | 결함 |
|---|------|------|
| 1 | `pagination/engine.rs:519-544` + `typeset.rs:2120` | `page_hides` 수집이 본문 paragraph 만 → 셀 안 paragraph 의 PageHide 무시 |
| 2 | `layout.rs:411-422` | `build_page_background()` + `build_page_borders()` 호출에 `hide_fill`/`hide_border` 가드 부재 |
| 3 | `main.rs:1665-1670` (dump) | 셀 안 controls 매칭에서 PageHide 분기 부재 (디버깅 한계) |

## 핵심 발견 — 두 페이지네이션 경로

Stage 2 GREEN 시도 시 engine.rs 만 수정 → RED 유지 → typeset.rs 도 동일 결함 발견:

| 경로 | 위치 | 호출 시점 |
|------|------|----------|
| **TypesetEngine::typeset_section** | `typeset.rs:2120` | **main path** (`render_page_svg_native` → `build_page_tree` → `find_page`) |
| Paginator::paginate_with_measured | `engine.rs:504` | `RHWP_USE_PAGINATOR=1` fallback |

PR #641 description 의 "두 경로 양분" 진단 정합. 두 경로 동시 정정.

## 코드 변경

### `engine.rs` + `typeset.rs` (헬퍼 함수 동일 패턴)

```rust
// 본문 paragraph 순회 안:
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
                    Control::PageHide(ph) => page_hides.push((pi, ph.clone())),
                    Control::Table(inner) => Self::collect_pagehide_in_table(inner, pi, page_hides),
                    _ => {}
                }
            }
        }
    }
}
```

외부 paragraph index `pi` 사용으로 페이지 매핑 정합 유지. 중첩 표 (depth 2+) 재귀 보존 (실측 0건이지만 미래 케이스 대비).

### `layout.rs` (가드 추가)

```rust
let hide_fill = page_content.page_hide.as_ref()
    .map(|ph| ph.hide_fill).unwrap_or(false);
if !hide_fill {
    self.build_page_background(...);
}

let hide_border = page_content.page_hide.as_ref()
    .map(|ph| ph.hide_border).unwrap_or(false);
if !hide_border {
    self.build_page_borders(...);
}
```

기존 `hide_master` (`:417`) + `hide_header` (`:427`) 패턴과 동일.

### `main.rs` (dump 분기 추가)

```rust
Control::PageHide(ph) => {
    println!("{}    ctrl[{}] PageHide: header={} footer={} master={} border={} fill={} page_num={}",
        indent, ci,
        ph.hide_header, ph.hide_footer, ph.hide_master_page,
        ph.hide_border, ph.hide_fill, ph.hide_page_num);
}
```

## 검증 결과

### 1. 본질 정정 RED → GREEN (6건)

```
running 6 tests
test test_705_aift_page2_cell_pagehide_collected ... ok
test test_705_aift_page2_cell_pagehide_six_fields ... ok
test test_705_aift_page3_cell_pagehide_collected ... ok
test test_705_aift_cell_pagehides_total_count ... ok
test test_705_kor2022_cell_pagehide_collected ... ok
test test_705_ktx_cell_pagehide_collected ... ok

test result: ok. 6 passed; 0 failed
```

### 2. 전체 회귀 sweep

- `cargo test --release --lib`: **1123 passed, 0 failed, 1 ignored**
- `cargo clippy --release --lib`: **0 warning**

### 3. 198 sample sweep — 분포 무변화

- 본문 PageHide: 95 (Stage 0 == Stage 5)
- 셀 안 PageHide: 13 (분포 무변화)
- 영향 샘플: 6 (aift.hwp, 2022 국립국어원, KTX, kps-ai, tac-img-02 x 2)

### 4. aift.hwp 페이지 카운트

- 77 페이지 (Stage 0 == Stage 5, 무변화)

### 5. SVG smoke check

| 페이지 | 분류 | bg rect | footer | 정합 |
|--------|------|---------|--------|------|
| 1 (정상) | page_hide=None | 1 | 3 | ✓ |
| **2 (감추기 6필드)** | 셀[167] full6 | **0** | (본문 표 영역) | ✓ (`hide_fill` 가드) |
| 4 (목차 본문 PageHide) | s2/p[34] | 1 | **0** | ✓ |
| 5 (별첨 본문 PageHide) | s2/p[54] | 1 | **0** | ✓ |
| 6 (본문 시작) | page_hide=None | 1 | 3 | ✓ |

### 6. dump 검증 (메인테이너 권위 측정 일치)

```
$ rhwp dump samples/aift.hwp -s 0 -p 1 | grep "셀\[167\]\|PageHide"

[0]   셀[167] r=34,c=0 ... text="...|       년        월        일|..."
[0]       ctrl[0] PageHide: header=true footer=true master=true border=true fill=true page_num=true
```

## 메모리 룰 정합

- `pdf_not_authoritative`: IR 기반 검증 (page.page_hide + dump + SVG bg_rect 카운트)
- `rule_not_heuristic`: 본질 정정 (휴리스틱 cover-style 폐기, PR #641 의 `is_cover_style_page` 도입 안 함)
- `essential_fix_regression_risk`: 198 sample sweep + cargo test 1123 passed

## 영향 범위

### 기능적 영향

aift.hwp page 2: PageHide 6 필드 모두 적용 → 한컴 호환 정합
- header/footer/master/border/fill/page_num 모두 미렌더

aift.hwp page 3, 2022 국립국어원, KTX, kps-ai 목차 페이지: hide_page_num=true 적용 → page_num 미표시

tac-img-02.hwp/.hwpx 4 페이지: 다양한 패턴 적용 (header / page_num)

### 무영향

- 다른 173 샘플 — 셀 안 PageHide 0건이므로 본 정정 영향 없음
- 페이지 카운트, 본문 글리프, 표 셀 컨텐츠 등 — 본 정정은 PageHide 의 적용 영역만 변경

## 산출물

### 코드
- `src/renderer/pagination/engine.rs` (+25)
- `src/renderer/typeset.rs` (+27)
- `src/renderer/layout.rs` (+14, -4)
- `src/main.rs` (+5)
- `src/renderer/layout/integration_tests.rs` (+121)
- `examples/inspect_705.rs` (신규, +85)
- `examples/scan_cell_pagehide.rs` (신규, +144)

### 문서
- `mydocs/plans/task_m100_705.md` (수행 계획서)
- `mydocs/plans/task_m100_705_impl.md` (구현 계획서)
- `mydocs/working/task_m100_705_stage{0..5}.md` (단계별 보고서 6건)
- `mydocs/report/task_m100_705_report.md` (본 최종 보고서)

## 관련

- Issue #705 (본 task, OPEN — close 예정): https://github.com/edwardkim/rhwp/issues/705
- PR #638 (CLOSED) — Task #634, 메인테이너 본질 결함 발견
- PR #640 (CLOSED) — Task #637, H2 가설 잘못 기각 (본 환경 결함 #1 으로 인한 측정 누락)
- PR #641 (CLOSED) — Task #639, cover-style 휴리스틱 폐기
- Issue #639 (CLOSED) — cover-style 자동 인식 → 본 task 로 대체

## PR 제출 안내

본 task 의 코드 변경은 `local/task705` 브랜치 6 커밋:
- d299f460 Stage 0 (사전 측정 + 198 샘플 재조사)
- 1be31129 Stage 1 RED (통합 테스트 4건)
- 280acb99 Stage 2 GREEN #1 (engine.rs + typeset.rs)
- 24524da8 Stage 3 GREEN #2 (layout.rs 가드)
- 71071183 Stage 4 #3 (main.rs dump)
- (Stage 5 커밋 예정 — 본 보고서 + 회귀 가드 2건 추가)

`local/devel` merge → `devel` push 후 PR 생성 가능.
