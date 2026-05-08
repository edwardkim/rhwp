# Task #634 구현 계획서

**제목**: 한컴 호환 — 첫 NewNumber Page 컨트롤 발화 전 페이지의 쪽번호 미표시
**브랜치**: `local/task634`
**이슈**: https://github.com/edwardkim/rhwp/issues/634
**수행계획서**: `mydocs/plans/task_m100_634.md`
**Stage 0 보고서**: `mydocs/working/task_m100_634_stage0.md`

---

## 1. Stage 0 결정 사항 요약

### 가설 H1'' (A안 관대 모드) 채택

> **문서에 `NewNumber Page` 컨트롤이 하나라도 존재하는 경우에만**, 첫 NewNumber 발화 전
> 페이지의 쪽번호를 감춘다. NewNumber 가 전혀 없는 문서는 PageNumberPos 등록 시점부터
> 모든 페이지에 표시 (현재 동작 보존).

### 수정 범위 (9개 파일)

| 파일 | 변경 |
|------|------|
| `src/renderer/page_number.rs` | `numbering_started: bool` + `show_for_last_page() -> bool` |
| `src/renderer/pagination.rs` | `PageContent.show_page_number: bool` (default `true`) |
| `src/renderer/pagination/state.rs:232` | 초기화에 `show_page_number: true` 추가 |
| `src/renderer/pagination/engine.rs:1979` | finalize 후 `page.show_page_number = ...` 설정 |
| `src/renderer/typeset.rs:2223` | 동일 |
| `src/document_core/queries/rendering.rs` | 구역간 `carry_numbering_started` 추가 |
| `src/renderer/layout.rs:1102` | `if !page_content.show_page_number { return; }` 가드 |
| `src/renderer/page_number.rs` 기존 5건 테스트 | mk_page 갱신 |
| `src/renderer/layout/tests.rs` 기존 6건 mk_page | `show_page_number: true` 추가 |

---

## 2. 단계 분할

### Stage 1 — TDD RED 통합 테스트 (코드 수정 없음)

목적: 한컴 호환 동작을 명세하는 통합 테스트 추가. **본질 코드 수정 없음, 회귀 위험 0**.

#### 1.1 테스트 추가 위치

`src/renderer/layout/integration_tests.rs` 하단에 추가.

#### 1.2 테스트 케이스 (RED 예상)

```rust
/// Task #634: aift.hwp 페이지 2 (사업계획서 표지) 는 NewNumber 발화 전이므로
/// 한컴은 쪽번호 미표시. rhwp 도 동일해야 함.
#[test]
fn test_634_aift_page2_no_page_number_before_new_number() {
    let bytes = std::fs::read("samples/aift.hwp").expect("aift.hwp not found");
    let mut core = DocumentCore::from_bytes(&bytes).expect("parse");
    let svg = core.render_page_svg_native(1).unwrap_or_default();  // page index 1 = page 2
    // 페이지 2 는 NewNumber (구역 2 문단 79) 발화 전이므로 쪽번호 미표시
    let page_num_text = "y=\"1079.16\"";  // footer y 좌표
    assert!(!svg.contains(page_num_text),
        "Page 2 should not have page number (before first NewNumber). Found: {} occurrences",
        svg.matches(page_num_text).count());
}

/// Task #634: aift.hwp 페이지 7 (□ 배경, NewNumber 발화) 부터는 쪽번호 표시.
#[test]
fn test_634_aift_page7_shows_page_number_after_new_number() {
    let bytes = std::fs::read("samples/aift.hwp").expect("aift.hwp not found");
    let mut core = DocumentCore::from_bytes(&bytes).expect("parse");
    let svg = core.render_page_svg_native(6).unwrap_or_default();  // page index 6 = page 7
    let page_num_text = "y=\"1079.16\"";
    assert!(svg.contains(page_num_text),
        "Page 7 should have page number (NewNumber fires here)");
    // "1" 자가 있는지 확인 (- 1 -)
    assert!(svg.contains(">1<") || svg.contains("\">1\""),
        "Page 7 page number should contain '1'");
}

/// Task #634: 2022년 국립국어원 페이지 1 (표지) 는 NewNumber 발화 전 → 미표시.
#[test]
fn test_634_gukrip_page1_no_page_number() {
    let bytes = std::fs::read("samples/2022년 국립국어원 업무계획.hwp").expect("not found");
    let mut core = DocumentCore::from_bytes(&bytes).expect("parse");
    let svg = core.render_page_svg_native(0).unwrap_or_default();  // page 1
    // 페이지 1 은 표지 — NewNumber 발화 전. 한컴 PDF 도 쪽번호 미표시 (footer y=42.6 op 0개).
    // rhwp footer y 좌표는 다를 수 있으므로 "바탕" 폰트 (page number font) 의 footer 영역 텍스트로 검증
    let lower_y_threshold = 1000.0;  // 페이지 하단 100px
    let count = count_text_below_y(&svg, lower_y_threshold);
    assert_eq!(count, 0,
        "Page 1 should have no text near footer (no page number). Found {} text ops below y={}",
        count, lower_y_threshold);
}

/// Task #634: 2022년 국립국어원 페이지 3 (NewNumber 발화) 부터 쪽번호 표시.
#[test]
fn test_634_gukrip_page3_shows_page_number() {
    let bytes = std::fs::read("samples/2022년 국립국어원 업무계획.hwp").expect("not found");
    let mut core = DocumentCore::from_bytes(&bytes).expect("parse");
    let svg = core.render_page_svg_native(2).unwrap_or_default();  // page 3
    let count = count_text_below_y(&svg, 1000.0);
    assert!(count >= 3, "Page 3 should have at least 3 text ops in footer (= '- 1 -')");
}

/// Task #634: A안 (관대) — NewNumber 가 없는 문서는 모든 페이지에 표시 (회귀 방지).
#[test]
fn test_634_no_newnumber_doc_shows_page_numbers() {
    // hwp3-sample 또는 issue_265 등 pgnp만 있고 nn 0건인 샘플
    let bytes = std::fs::read("samples/hwp3-sample.hwp").expect("not found");
    let mut core = DocumentCore::from_bytes(&bytes).expect("parse");
    let svg = core.render_page_svg_native(0).unwrap_or_default();
    let count = count_text_below_y(&svg, 1000.0);
    assert!(count > 0,
        "Doc with no NewNumber should show page number from page 1 (A안 관대 default)");
}
```

#### 1.3 헬퍼 함수

```rust
/// SVG 에서 y 값이 `threshold` 이상인 <text> 요소 수를 센다 (footer 영역 추정용).
fn count_text_below_y(svg: &str, threshold: f64) -> usize {
    use regex::Regex;
    let re = Regex::new(r#"<text[^>]*y="([0-9.]+)""#).unwrap();
    re.captures_iter(svg)
        .filter_map(|cap| cap[1].parse::<f64>().ok())
        .filter(|y| *y >= threshold)
        .count()
}
```

(또는 정규식 없이 grep-style 매칭)

#### 1.4 RED 확인

```bash
cargo build --release
cargo test --release --lib test_634 2>&1 | tail -20
```

예상 결과:
- `test_634_aift_page2_no_page_number_before_new_number`: **FAIL (현재 표시함)**
- `test_634_aift_page7_shows_page_number_after_new_number`: PASS (현재도 표시함)
- `test_634_gukrip_page1_no_page_number`: **FAIL (현재 표시함)**
- `test_634_gukrip_page3_shows_page_number`: PASS (현재도 표시함)
- `test_634_no_newnumber_doc_shows_page_numbers`: PASS (회귀 방지, 사전 확인)

#### 1.5 Stage 1 산출물

- `src/renderer/layout/integration_tests.rs` 에 5개 test 추가 (+200 LOC)
- `mydocs/working/task_m100_634_stage1.md` Stage 1 보고서

#### 1.6 Stage 1 커밋

```
Task #634 Stage 1: TDD RED 통합 테스트

aift / 국립국어원 의 NewNumber 발화 전 페이지 쪽번호 미표시 명세.
A안 (관대) — NewNumber 미존재 문서는 즉시 표시 회귀 방지 테스트 포함.

5건 테스트 중 2건 RED (aift p2, gukrip p1) 확인. 나머지는 PASS (회귀 미발생).
```

---

### Stage 2 — Fix 적용

#### 2.1 PageNumberAssigner (page_number.rs)

```rust
pub(crate) struct PageNumberAssigner<'a> {
    new_page_numbers: &'a [(usize, u16)],
    consumed: HashSet<usize>,
    counter: u32,
    /// 첫 NewNumber 발화 후 true. 이후 영구 true.
    /// new_page_numbers 가 empty 이면 생성 시 즉시 true (A안 관대 모드).
    numbering_started: bool,
}

impl<'a> PageNumberAssigner<'a> {
    pub fn new(new_page_numbers: &'a [(usize, u16)], initial: u32) -> Self {
        // A안: NewNumber 가 전혀 없으면 즉시 표시 가능
        let numbering_started = new_page_numbers.is_empty();
        Self { new_page_numbers, consumed: HashSet::new(), counter: initial, numbering_started }
    }

    /// 이전 구역에서 numbering_started 가 true 였다면 그 상태로 시작.
    pub fn new_with_started(
        new_page_numbers: &'a [(usize, u16)],
        initial: u32,
        prev_started: bool,
    ) -> Self {
        let numbering_started = prev_started || new_page_numbers.is_empty();
        Self { new_page_numbers, consumed: HashSet::new(), counter: initial, numbering_started }
    }

    pub fn assign(&mut self, page: &PageContent) -> u32 {
        for (idx, &(nn_pi, nn_num)) in self.new_page_numbers.iter().enumerate() {
            if self.consumed.contains(&idx) { continue; }
            if Self::para_first_appears(page, nn_pi) {
                self.counter = nn_num as u32;
                self.consumed.insert(idx);
                self.numbering_started = true;  // 발화 페이지부터 표시
            }
        }
        let assigned = self.counter;
        self.counter += 1;
        assigned
    }

    /// 직전 assign() 의 페이지가 쪽번호를 표시해야 하는지 반환.
    pub fn show_for_last_page(&self) -> bool {
        self.numbering_started
    }

    pub fn next_counter(&self) -> u32 { self.counter }

    /// 구역간 carry 용 — 누적 numbering_started 상태.
    pub fn numbering_started(&self) -> bool { self.numbering_started }
}
```

#### 2.2 PageContent (pagination.rs)

```rust
pub struct PageContent {
    ...
    pub page_number_pos: Option<crate::model::control::PageNumberPos>,
    pub page_hide: Option<crate::model::control::PageHide>,
    /// Task #634: 쪽번호 표시 여부 (NewNumber 게이팅).
    /// default `true` (현재 동작 보존). false 인 페이지는 build_page_number 가 skip.
    pub show_page_number: bool,
    ...
}
```

#### 2.3 finalize_pages 두 경로 (engine.rs / typeset.rs)

```rust
let page_num = assigner.assign(page);
page.page_number = page_num;
page.show_page_number = assigner.show_for_last_page();  // ← 신규
...
page.page_number_pos = page_number_pos.clone();
```

#### 2.4 구역간 carry (rendering.rs)

기존 `carry_last_page_number` 옆에 추가:

```rust
let mut carry_numbering_started: bool = false;
...
for (idx, section) in self.document.sections.iter().enumerate() {
    ...
    // finalize_pages 호출 부에서 carry 반영
    // (engine/typeset 시그니처에 carry_numbering_started 인자 추가, 또는
    //  finalize 후 carry 값으로 page.show_page_number 보정)
    ...
    // carry 업데이트 (구역 마지막 페이지 기준)
    if let Some(last) = result.pages.last() {
        if last.show_page_number {
            carry_numbering_started = true;
        }
    }
}
```

#### 2.5 build_page_number (layout.rs)

```rust
fn build_page_number(...) {
    if let Some(ref ph) = page_content.page_hide {
        if ph.hide_page_num { return; }
    }
    if !page_content.show_page_number {
        return;  // ← 신규 가드
    }
    if let Some(pnp) = &page_content.page_number_pos {
        if pnp.position == 0 { return; }
        ...
    }
}
```

#### 2.6 테스트 mk_page 헬퍼 갱신

`page_number.rs:103-125` 의 mk_page 와 `layout/tests.rs` 의 mk_page (6곳) 에
`show_page_number: true` 추가.

#### 2.7 검증

```bash
cargo build --release
cargo test --release --lib  # 전체 테스트 통과
cargo test --release --lib test_634  # Stage 1 RED → GREEN 전환 확인
```

#### 2.8 Stage 2 커밋

```
Task #634 Stage 2: NewNumber 발화 전 쪽번호 미표시 fix

PageNumberAssigner 에 numbering_started 상태 추가 (A안 관대 모드).
PageContent.show_page_number 필드 + build_page_number 가드.
구역간 carry 로 NewNumber 발화 후 후속 구역도 표시 유지.

Stage 1 5건 테스트 모두 GREEN.
```

---

### Stage 3 — 광범위 회귀 검증 + 최종 보고서

#### 3.1 검증 매트릭스

| 분류 | 샘플 | 기대 동작 | 검증 |
|------|------|----------|------|
| NewNumber + pgnp (검증된 정정 케이스) | aift.hwp, 국립국어원.hwp | NewNumber 발화 전 미표시, 후 표시 | RED → GREEN |
| pgnp만 (NewNumber 없음) | hwp3-sample, issue_265, pic-in-head-02 | 모든 페이지 표시 (A안 default) | 회귀 0 |
| pgnp + page_num PageHide | KTX.hwp | PageHide 적용 페이지 미표시, 외 표시 | 회귀 0 |
| pgnp + 일반 PageHide | endnote-01, footnote-01, hwp-multi-001, table-vpos-01 | 회귀 0 | 동작 동일 |
| pgnp/nn 모두 없음 | 21_언어_기출, exam_eng/math, equation-lim 등 | 쪽번호 0 | 회귀 0 |

#### 3.2 검증 명령

```bash
cargo build --release
cargo test --release --lib  # 전체 1120+건 통과

# SVG 회귀 검증
for f in samples/aift.hwp samples/2022년*.hwp samples/KTX.hwp \
         samples/hwp3-sample.hwp samples/issue_265.hwp \
         samples/pic-in-head-02.hwp samples/endnote-01.hwp \
         samples/footnote-01.hwp samples/hwp-multi-001.hwp \
         samples/table-vpos-01.hwp samples/exam_*.hwp \
         samples/21_언어_기출_편집가능본.hwp; do
  ./target/release/rhwp export-svg "$f" -o /tmp/diag634/
done

# 페이지별 쪽번호 텍스트 등장 카운트 (footer y 좌표 기반)
python3 mydocs/scripts/check_page_number_visibility.py /tmp/diag634/
```

#### 3.3 회귀 발견 시

- 가드 정밀화: `numbering_started` 초기화 조건 추가/축소
- 본 task 정정 보류 + 별도 보완 task 분리

#### 3.4 Stage 3 산출물

- `mydocs/working/task_m100_634_stage3.md` 단계별 보고서
- `mydocs/report/task_m100_634_report.md` 최종 보고서
- `mydocs/orders/yyyymmdd.md` 갱신 (해당일 작업 추가)

#### 3.5 Stage 3 커밋

```
Task #634 Stage 3: 광범위 회귀 검증 + 최종 보고서

closes #634
```

---

## 3. 위험 및 완화

| 위험 | 영향 | 완화 |
|------|------|------|
| 가설 H1'' (A안) 이 일부 NewNumber 있는 문서에서 잘못된 결과 | 중간 | 검증 케이스 2건 (aift, 국립국어원) 모두 일치. 추가 회귀 검증으로 안전 확인 |
| 기존 테스트 mk_page 헬퍼 갱신 누락으로 컴파일 에러 | 작음 | `show_page_number: true` default 추가. 컴파일 에러 즉시 표시 |
| HWPX/HWP3 동작 분기 회귀 | 중간 | IR-level 수정이라 자동 적용. 별도 HWPX/HWP3 샘플 회귀 검증 |
| rhwp-studio 편집기 시각 회귀 | 중간 | 편집기는 native render 호출 — 자동 적용. 사용자 의도와 동일 (한컴 호환) |

---

## 4. 검증 명령 (요약)

```bash
# Stage 1 RED 확인
cargo test --release --lib test_634 2>&1 | tail -10

# Stage 2 GREEN 확인 + 전체 회귀
cargo test --release --lib 2>&1 | tail -10

# Stage 3 광범위 SVG 회귀
for f in samples/*.hwp; do
  ./target/release/rhwp export-svg "$f" -o /tmp/diag634/ 2>/dev/null
done
```

---

## 5. 커밋 단위 (요약)

- Stage 1: "Task #634 Stage 1: TDD RED 통합 테스트"
- Stage 2: "Task #634 Stage 2: NewNumber 발화 전 쪽번호 미표시 fix"
- Stage 3: "Task #634 Stage 3: 광범위 회귀 검증 + 최종 보고서"

`closes #634` 는 Stage 3 마지막 커밋.

---

승인 후 Stage 1 (TDD RED) 부터 시작합니다.
