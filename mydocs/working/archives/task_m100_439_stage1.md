# Task #439 Stage 1 — 베이스라인 측정 + 진단

> Square wrap 표 직후 col 0 over-fill (exam_kor 페이지 14 단 0 1225>1211)

- **이슈**: [#439](https://github.com/edwardkim/rhwp/issues/439)
- **브랜치**: `local/task439`
- **단계**: Stage 1 (진단, 코드 변경 없음)

---

## 1. 베이스라인 (현 상태)

```
$ ./target/release/rhwp dump-pages samples/exam_kor.hwp -p 13
=== 페이지 14 (global_idx=13, section=1, page_num=14) ===
  body_area: x=117.2 y=211.7 w=888.2 h=1211.3
  단 0 (items=25, used=1225.8px)             # ← 본문 1211.3 초과 +14.5
  단 1 (items=2,  used=64.3px, hwp_used≈1189.0px, diff=-1124.7px)
```

이슈 명세와 일치. 본문 높이 1211.3 px 를 14.5 px 초과, col 1 은 거의 비어있다.

### 문서 페이지 수
- rhwp: **22 페이지**
- HWP: **21 페이지** (이슈 명세)

## 2. 페이지 14 col 0 구성 (25 items)

```
 1. FullParagraph    pi=28  h=55.1
 2. FullParagraph    pi=29  h=15.3
 3. FullParagraph    pi=30  h=30.7
 4. FullParagraph    pi=31  h=15.3
 5. FullParagraph    pi=32  h=46.0
 6. PartialParagraph pi=33  lines=0..4   ← Square wrap host text
 7. Table            pi=33  3x2 75.6px   wrap=어울림 vert=문단(1.4mm)
 8. FullParagraph    pi=34  h=30.7
 9. FullParagraph    pi=35  h=30.7
10. FullParagraph    pi=36  h=15.3
11. PartialParagraph pi=37  lines=0..3   ← Square wrap host text
12. Table            pi=37  3x2 51.4px   wrap=어울림 vert=문단(1.4mm)
13. FullParagraph    pi=38  h=46.0
14. FullParagraph    pi=39  h=46.0
15. PartialParagraph pi=40  lines=0..2   ← Square wrap host text
16. Table            pi=40  3x2 26.4px   wrap=어울림 vert=문단(1.4mm)
17. FullParagraph    pi=41  h=46.0
18. FullParagraph    pi=42  h=15.3
19. FullParagraph    pi=43  h=46.0
20. FullParagraph    pi=44  h=46.0
21. FullParagraph    pi=45  h=34.2
22. Shape            pi=45  wrap=TopAndBottom tac=true
23. FullParagraph    pi=46  h=15.3
24. PartialParagraph pi=47  lines=0..3   ← Square wrap host text
25. Table            pi=47  3x2 51.4px   wrap=어울림 vert=문단(1.4mm)
```

페이지 14 col 1: pi=48 (h=15.3) + pi=49 (h=30.7) = **64.3 px** 만 사용.
나머지 (pi=50 ~) 는 페이지 15 로 밀림.

## 3. 핵심 발견 — **이슈 가설의 코드 위치는 fallback 경로**

이슈 본문은 `src/renderer/pagination/engine.rs:702-711` 의 `prev_is_table` 분기 (trailing line_spacing 제외 로직) 를 원인으로 추정했다.

**그러나 활성 페이지네이션 엔진은 `engine.rs::Paginator` 가 아니다.**

### 3.1 활성 엔진 확인

`src/document_core/queries/rendering.rs:882-908`:

```rust
// TypesetEngine을 main pagination으로 사용. RHWP_USE_PAGINATOR=1 로 fallback 가능.
let use_paginator = std::env::var("RHWP_USE_PAGINATOR").map(|v| v == "1").unwrap_or(false);
let mut result = if use_paginator {
    paginator.paginate_with_measured_opts(...)
} else {
    use crate::renderer::typeset::TypesetEngine;
    let typesetter = TypesetEngine::new(self.dpi);
    typesetter.typeset_section(...)
};
```

기본값은 `TypesetEngine` (`src/renderer/typeset.rs`). `engine.rs::Paginator` 는 환경변수로만 활성화되는 **legacy fallback** 이다.

### 3.2 검증

`engine.rs::paginate_table_control` 과 `place_table_fits` 에 진단용 `eprintln!` 을 임시 삽입하고 실행 → **stderr 출력 0 줄** (호출되지 않음 확인). 디버그 코드는 모두 revert.

`RHWP_USE_PAGINATOR=1` 로 실행 시 페이지 수가 **22 → 25 페이지** 로 증가하며 페이지 구성도 완전히 다름 (engine.rs 는 별개의 회귀 케이스 보유).

## 4. 실제 버그 위치 — `typeset.rs::place_table_with_text`

`src/renderer/typeset.rs:1400-1467` 의 `place_table_with_text` 가 Square wrap 표를 잘못 처리한다.

### 4.1 typeset.rs 의 코드 (현재)

```rust
fn place_table_with_text(...) {
    let vertical_offset = Self::get_table_vertical_offset(table);
    let total_lines = fmt.line_heights.len();
    let pre_table_end_line = if vertical_offset > 0 && !para.text.is_empty() {
        total_lines
    } else {
        0
    };

    // pre-table 텍스트 (첫 번째 표에서만)
    let is_first_table = !para.controls.iter().take(ctrl_idx)
        .any(|c| matches!(c, Control::Table(_)));
    if pre_table_end_line > 0 && is_first_table {              // ← Square wrap 가드 없음!
        let pre_height: f64 = fmt.line_advances_sum(0..pre_table_end_line);
        st.current_items.push(PageItem::PartialParagraph { ... });
        st.current_height += pre_height;
    }

    // 표 배치
    st.current_items.push(PageItem::Table { ... });
    st.current_height += table_total_height;
    ...
}
```

### 4.2 engine.rs 의 동일 위치 (legacy, 가드 보유)

`src/renderer/pagination/engine.rs:1349`:

```rust
if pre_table_end_line > 0 && is_first_table && !is_wrap_around_table {  // ← 가드 있음
    ...
}
```

`engine.rs:1422`:

```rust
if is_last_table && tac_table_count <= 1 && !para.text.is_empty()
    && total_lines > post_table_start && !is_wrap_around_table && !pre_text_exists {
    ...
}
```

`is_wrap_around_table` 정의 (`engine.rs:1328`):

```rust
let is_wrap_around_table = !table.common.treat_as_char
    && matches!(table.common.text_wrap, crate::model::shape::TextWrap::Square);
```

**typeset.rs 는 이 가드를 보유하지 않음**. 따라서 Square wrap 호스트 문단의 host 텍스트가 PartialParagraph 로 emit 되며 그 높이가 `current_height` 에 누적된다.

### 4.3 Square wrap 호스트 문단 4 개의 영향

|     pi | 호스트 줄 수 | pre_height (≈) | table 높이 | 합산 누적 |
|-------:|------------:|---------------:|----------:|---------:|
|  pi=33 |           4 |          ~98.0 |     75.6 |   ~173.6 |
|  pi=37 |           3 |          ~73.5 |     51.4 |   ~124.9 |
|  pi=40 |           2 |          ~49.0 |     26.4 |    ~75.4 |
|  pi=47 |           3 |          ~73.5 |     51.4 |   ~124.9 |

(pre_height 는 `line_advances_sum` = sum of (line_height + line_spacing). 1 줄 ≈ 24.5 px, 4 줄 ≈ 98.0 px)

이론상 호스트 텍스트 높이를 추가로 카운트하므로 **약 ~290 px 의 누적 과다** 가 예상되지만 실제 col 0 over-fill 은 **+14.5 px** 에 불과하다. 어딘가에서 보정/상쇄가 일어나고 있음을 의미하며 Stage 2 에서 정밀 추적이 필요하다.

## 5. 호스트 문단 (pi=33) 상세

```
$ ./target/release/rhwp dump samples/exam_kor.hwp -s 1 -p 33
--- 문단 1.33 --- cc=111, text_len=102, controls=1
  텍스트: "학생 3 :-아, 그래? 그런 가사 내용이 ..."
  ls[0]: vpos=17916, lh=1150, th=1150, ls=688, sw=28039
  ls[1]: vpos=19754, lh=1150, th=1150, ls=688, sw=28039
  ls[2]: vpos=21592, lh=1150, th=1150, ls=688, sw=28039
  ls[3]: vpos=23430, lh=1150, th=1150, ls=688, sw=28039
  [0] 표: 3행×2열, padding=(0,0,0,0)
  [0]   [common] treat_as_char=false, wrap=어울림, vert=문단(396=1.4mm), horz=단(708=2.5mm)
  [0]   [common] size=1722×5667(6.1×20.0mm), valign=Top, halign=Right
  [0]   [outer_margin] left=1.5mm(425) right=0.0mm(0) top=0.0mm(0) bottom=0.0mm(0)
```

특징:
- `wrap=어울림` (Square wrap), `vertical_offset=396` HU (= 1.4mm = 5.3 px)
- 표 크기 6.1×20.0mm — 좁고 긴 형태 (객관식 보기 박스 [A], [B], ...)
- 호스트 본문 `sw=28039` (단 폭 전체) — 표 옆으로 좁혀지지 않고 본문은 단 전체 폭 사용
- 표는 단 우측 (halign=Right) 에 outer_margin 1.5mm + 자체 vert_offset 1.4mm 로 배치

다른 Square wrap 표 (pi=37, 40, 47) 도 동일한 패턴 확인 (wrap=어울림, vert=문단, halign=Right).

## 6. 가설 갱신

**이슈 본문의 원인 분석은 잘못된 코드 위치를 지목**하고 있었으나, **현상은 정확히 재현됨**.

### 가설 (Stage 2 검증 대상)

- **A (확정 직전)**: typeset.rs::place_table_with_text 의 Square wrap 가드 누락. Square wrap 호스트의 pre-text 가 PartialParagraph 로 추가되어 height 가 누적됨.
- **B**: typeset.rs 어딘가에 보정 로직이 있어 ~290 px 의 과다 누적이 ~14.5 px 까지 줄어듦. 이 보정이 정확히 어디서 일어나는지 추적 필요.
- **C**: Square wrap 의 PartialParagraph 자체는 layout 측에서도 필요한 emit 일 수 있음 (호스트 텍스트 위치 정보 보존). 단순 가드 추가 시 텍스트가 사라질 가능성 — 별도 검증 필요.

### Stage 2 작업 목록 (예고)

1. typeset.rs 의 wrap_around 처리 코드 (461-486 의 `current_column_wrap_around_paras` 분기) 가 어디서 height 보정에 영향을 주는지 확인
2. layout 단에서 Square wrap host text 가 어떻게 그려지는지 확인 (PartialParagraph 가 필요한가? 아니면 wrap_around_paras 만으로 충분한가?)
3. 14.5 px over-fill 의 정확한 원인 식별 — pre-text 누적인가, trailing ls 인가, 다른 누락인가
4. 회귀 우려 케이스 평가 (hwpspec, basic 의 Square wrap 사용 문서)
5. 수정안 후보 도출:
   - (A) typeset.rs 에 `!is_wrap_around_table` 가드 추가 + layout 측 wrap_around_paras 활용 검증
   - (B) Square wrap 시 host pre_height 만 추가하고 table_total_height 는 추가하지 않음 (max 처리)
   - (C) trailing line_spacing 분기를 typeset.rs 에 도입

## 7. 회귀 베이스라인 (Stage 4 회귀 테스트 기준)

```
exam_kor.hwp:    22 페이지 (목표: 21)
hwpspec.hwp:     별도 측정 필요
basic.hwp:       별도 측정 필요
```

(Stage 4 시작 시 측정값 기록 예정)

## 8. 코드 변경 내역

**없음**. 본 단계는 진단만 수행. 임시 삽입된 디버그 출력은 모두 revert.

```
$ git diff --stat
(empty)
```

---

## 산출물

- `mydocs/working/task_m100_439_stage1.md` (본 문서)

## Stage 2 진입 승인 요청

다음 단계는 typeset.rs 의 보정 로직 추적 + 정확한 14.5 px over-fill 원인 식별 + 수정안 도출 + `mydocs/plans/task_m100_439_impl.md` 작성. 코드 변경 없음.

진행 승인을 요청드립니다. 특히:
1. 이슈 가설이 잘못된 코드 위치 (engine.rs) 를 지목한 점 — 이슈 코멘트 등으로 이 사실을 기록할지 여부
2. 가설 A/B/C 중 우선 검증할 순서 (현재는 B → A → C 권장)
