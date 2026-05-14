# Task #901 Stage 2+3 보고서 — typeset wrap_anchor cs_only_match 추가

**Stage**: 2+3 / 5
**상태**: 부분 정합 — paragraph 1 fix 성공, paragraph 7+ 본문 wrap 미정합

## 1. 진단 — paragraph 1 "대한민국" 좌측 정렬 원인

pic2.hwp paragraph 1 line_segs:

| field | 값 |
|-------|----|
| text | " 대한민국" (5 chars) |
| controls | (없음) |
| ls[0] cs | 24470 |
| ls[0] sw | 18050 |

anchor (paragraph 0 의 그림) 의 wrap zone 등록 상태:

| field | 값 |
|-------|----|
| wrap_around_cs | 24470 |
| wrap_around_sw | 2570 (좁은 영역) |
| wrap_around_any_seg | true |

### 1.1 기존 매칭 분기 결과 (`src/renderer/typeset.rs:617`)

```rust
if (para_cs == st.wrap_around_cs && para_sw == st.wrap_around_sw)
    || (any_seg_matches && (is_empty_para || st.wrap_around_any_seg))
    || sw0_match
    || anchor_image_match {
```

- `para_cs == wrap_around_cs (24470) && para_sw == wrap_around_sw` → sw 다름 (18050 vs 2570) → **fail**
- `any_seg_matches` → fail (ls[0] sw=18050 not in wrap_around_sw=2570)
- `sw0_match` → fail
- `anchor_image_match` → fail (wrap_around_cs!=0)
- **결과**: wrap_anchors 미등록 → paragraph 1 좌측 (x=132) 그려짐

## 2. Fix — cs_only_match 분기 추가

`src/renderer/typeset.rs:617-625`:

```rust
// [Task #901] cs 일치 + 합리적 sw 매칭 (anchor 의 wrap zone region 다양성).
let cs_only_match = st.wrap_around_any_seg
    && para_cs == st.wrap_around_cs
    && para_sw > 0;
if (para_cs == st.wrap_around_cs && para_sw == st.wrap_around_sw)
    || (any_seg_matches && (is_empty_para || st.wrap_around_any_seg))
    || sw0_match
    || anchor_image_match
    || cs_only_match {
```

### 2.1 Why

anchor 의 wrap zone 이 여러 region 으로 분할 (그림 사이/주위) 된 경우, 후속 paragraph 의 line_seg 는 anchor 의 정확한 sw 와 일치하지 않을 수 있다. 그러나 cs 일치 + `wrap_around_any_seg` 활성 + `para_sw > 0` 이면 paragraph 가 wrap zone 내부에 있음이 확실하므로 등록.

### 2.2 Fix 결과

paragraph 1 "대한민국" SVG x:
- 이전: x=132 (좌측 margin)
- **이후: x=458~567 (우측, 한컴 정합)** ✅

## 3. 회귀 검증

- [ ] `cargo test --release --all-targets` (진행 중)
- [ ] 모든 sample 페이지 수 동일
- [ ] golden SVG 회귀 없음

## 4. 잔존 차이 — paragraph 7+ 본문 wrap zone

paragraph 7 ("SK하이닉스가 역대급 성과급을...") 162 chars:
- line_segs: ls[0~3] cs=0 sw=42520 (전체 폭, **wrap zone 미인코딩**)
- controls: 무용수 그림 57.8×84mm Square wrap @ paper(122.2mm, 164.0mm)
- ROOT CAUSE: HWP file 이 paragraph 7 line_seg 에 wrap zone 을 사전 인코딩하지 않음 → 한컴 뷰어가 런타임 계산. rhwp 는 line_seg 의 cs/sw 만 사용 → 본문이 그림 우측 영역까지 모두 침범.

Stage 4 (A-1) 에서 composer + paragraph_layout 변경 시도 예정.
