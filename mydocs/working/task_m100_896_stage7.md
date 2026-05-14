# Task #896 Stage 7 보고서 — HWPX 변환본 페이지 외곽선 미표시 Fix

**Stage**: 7 / 8 (HWPX page border)
**상태**: ✅ 완료

## 1. 진단

### 1.1 HWPX section XML

`hwp3-sample16-hwp5.hwpx` 의 `Contents/section0.xml` 의 `<hp:secPr>` 안에 page border 3 elements:

```xml
<hp:pageBorderFill type="BOTH" borderFillIDRef="2" fillArea="PAPER">
    <hp:offset left="1420" right="1420" top="1420" bottom="1420"/>
</hp:pageBorderFill>
<hp:pageBorderFill type="EVEN" borderFillIDRef="1" fillArea="PAPER">
    <hp:offset left="1417" right="1417" top="1417" bottom="1417"/>
</hp:pageBorderFill>
<hp:pageBorderFill type="ODD" borderFillIDRef="1" fillArea="PAPER">
    <hp:offset left="1417" right="1417" top="1417" bottom="1417"/>
</hp:pageBorderFill>
```

3 types: BOTH (양쪽), EVEN (짝수), ODD (홀수). 각각 borderFillIDRef + offset.

### 1.2 ROOT CAUSE

`src/parser/hwpx/section.rs` 의 `parse_sec_pr_children` (secPr 자식 처리) 에서 **`<hp:pageBorderFill>` 처리 누락**. SectionDef.page_border_fill 미설정 → renderer 가 외곽선 그리지 않음.

대조: HWP5 파서 (`body_text.rs:573`) 에는 `parse_page_border_fill` 있음. HWPX 만 누락.

## 2. Fix

`src/parser/hwpx/section.rs`:

```rust
b"pageBorderFill" => {
    let (ty, pbf) = parse_page_border_fill_element(e, reader)?;
    match ty.as_str() {
        "BOTH" => pbf_both = Some(pbf),
        "ODD" => pbf_odd = Some(pbf),
        "EVEN" => pbf_even = Some(pbf),
        _ => {}
    }
}

// 우선순위 BOTH > ODD > EVEN
if let Some(pbf) = pbf_both { sec_def.page_border_fill = pbf; }
else if let Some(pbf) = pbf_odd { sec_def.page_border_fill = pbf; }
else if let Some(pbf) = pbf_even { sec_def.page_border_fill = pbf; }
```

`parse_page_border_fill_element` 함수 신규:
- attribute: `type`, `borderFillIDRef`, `fillArea`, `headerInside`, `footerInside`
- 자식 `<hp:offset>`: left, right, top, bottom → spacing_* 설정
- `fillArea="PAPER"` → attr bit 0 = 1 (paper_based)

## 3. 결과

### 3.1 SVG 변화

sample16-hwp5.hwpx page 1 SVG:

이전: page border line element **없음**

이후: 4개 line element (top + bottom + left + right) emit:
```xml
<line x1="18.93" y1="17.88" x2="774.77" y2="17.88" stroke="#000000"/>
<line x1="18.93" y1="1102.52" x2="774.77" y2="1102.52" stroke="#000000"/>
...
```

paper width 793.7 px 의 5mm 안쪽 외곽선 정합.

### 3.2 회귀 점검

- `cargo test --release --lib`: **1250 passed**, 0 failed
- HWP3 sample 6종 페이지 수 회귀 없음
- HWPX sample 페이지 수 회귀 없음

## 4. 커밋

- `(다음 commit)` — Task #896 Stage 7: HWPX pageBorderFill parsing
