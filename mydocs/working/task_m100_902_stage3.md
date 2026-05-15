# Task #902 Stage 3 보고서 — fix3: DX byte-aware indexing + absolute X

**Stage**: 3 / 9 (v2)
**상태**: 완료

## 1. 변경 영역

`src/wmf/converter/svg/mod.rs:915~935` — EXTTEXTOUT 의 SVG tspan 생성 로직 변경

## 2. ROOT CAUSE — DX byte index 미인식 버그

### 2.1 WMF spec 의 EXTTEXTOUT DX 배열

[MS-WMF] §2.3.3.5 `META_EXTTEXTOUT`:
> Dx (variable): An array of 16-bit signed integers that indicate the distance between origins of adjacent character cells.

**핵심**: 문자열은 MBCS (CP949 Korean) 인코딩으로 저장. Korean wide char = 2 byte → DX 2 entry 차지 (실제 advance + 0).

### 2.2 기존 코드 버그

```rust
for (i, s) in text_content.graphemes(true).enumerate() {
    let dx = *record.dx.get(i - 1).unwrap_or(&0);  // grapheme i → dx[i-1]
    // ...
}
```

`text_content` 는 이미 decoded Unicode (Rust String). grapheme index 로 DX 접근 시 wide char 마다 매 둘째 entry (=0) 접근 → 잘못된 advance.

### 2.3 검증 (수정 전 SVG 추출)

`/tmp/task902_fix2/wmf_decoded.svg`:
```xml
<tspan>주</tspan>
<tspan x="291">전</tspan>   ← dx[0]=117 ✓
<tspan x="291">산</tspan>   ← dx[1]=0 (WRONG: should advance by 117)
<tspan x="408">센</tspan>   ← dx[2]=117 ✓
<tspan x="408">타</tspan>   ← dx[3]=0 (WRONG)
```

## 3. 수정 내용

```rust
// [Task #902] WMF EXTTEXTOUT 의 DX 배열은 MBCS byte index — Korean
// wide char 는 2 byte 이므로 DX 2 entry (실제 advance + 0) 차지.
// grapheme index 로 접근하면 wide char 마다 매 둘째 dx=0 으로 잘못
// 산출되어 글자 겹침. unicode_width 의 s.width() (Korean=2, ASCII=1)
// 로 byte advance 후 합산. absolute x 로 폰트 metric 독립 위치 정합.
let mut acc_x: i32 = i32::from(point.x);
let mut dx_idx: usize = 0;
for (i, s) in text_content.graphemes(true).enumerate() {
    let mut tspan = Node::new("tspan").add(Node::new_text(s));
    if i > 0 {
        tspan = tspan.set("x", acc_x);
    }
    let width = s.width().max(1);
    let advance: i32 = (0..width)
        .map(|k| i32::from(*record.dx.get(dx_idx + k).unwrap_or(&0)))
        .sum();
    acc_x += advance;
    dx_idx += width;
    text = text.add(tspan);
}
```

**핵심 변경**:
1. tspan 의 relative `dx` → absolute `x` (폰트 metric 독립 위치 정합)
2. `excess_dx = font.height/2 * width` 휴리스틱 제거
3. `unicode_width::s.width()` 로 byte 폭 인식 + DX 배열을 byte-aware 인덱스로 합산

## 4. 검증 결과

### 4.1 빌드

```
cargo build --release
   Finished `release` profile [optimized] target(s) in 1m 24s
```

### 4.2 회귀 테스트

```
cargo test --release --all-targets
Total passed: 1412 / failed: 0
```

기존 1411+ 기준 유지 (회귀 없음).

### 4.3 SVG 출력 검증 (수정 후)

`/tmp/task902_fix3/wmf_decoded.svg`:
```xml
<tspan>주</tspan>           ← x=174 (parent)
<tspan x="291">전</tspan>   ← +117 ✓
<tspan x="408">산</tspan>   ← +117 ✓
<tspan x="525">센</tspan>   ← +117 ✓
<tspan x="641">타</tspan>   ← +116 ✓
```

x 값 monotonic 정합 — Korean wide char 별로 정확한 advance.

### 4.4 시각 비교

rhwp-studio + 한컴 viewer 비교 (작업지시자 스샷 첨부):
- 텍스트 위치 호전 — fix3 가 fix1/fix2 대비 한컴 참조와 구조 정합
- 잔존 차이 (폰트 weight, glyph 너비) 는 Stage 4~7 영역 (viewport / EXTTEXTOUT flags / 폰트 metric)

## 5. 산출물

- 소스 수정: `src/wmf/converter/svg/mod.rs`
- 본 보고서: `mydocs/working/task_m100_902_stage3.md`
- 검증 SVG: `/tmp/task902_fix3/hwp3-sample16_018.svg`

## 6. 다음 단계

Stage 4: META_SETVIEWPORTEXT/ORG 구현 + MM_ANISOTROPIC ratio 정합
