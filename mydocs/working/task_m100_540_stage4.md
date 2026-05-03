# Task #540 Stage 4 완료 보고서

**제목**: 후속 회귀 정정 — passage 글상자 (paragraph border) 시프트 누락
**브랜치**: `local/task540`
**이슈**: https://github.com/edwardkim/rhwp/issues/540 (closed, Stage 4 후속 정정)

---

## 1. 발견된 회귀

Stage 3 merge 직후 작업지시자 시각 검증에서 21_언어_기출 의 passage 글상자 9곳
모두 박스 안 위쪽 여백이 늘어난 회귀 발견:

> "박스가 조금 위쪽으로 밀려서 그려지는 것 같음 — [4~6], [7~9], [10~12], [13~15],
> [16~18], [19~21], [22~24], [25~27], [28~30] 모두 동일 현상"

| 사례 | 빈 paragraph ls | 박스 위쪽 여백 증가 |
|------|----------------|------------------|
| 9개 passage 글상자 (ps_id=11, border_fill_id=4) | -56 ~ -440 | +0.75 ~ +5.87 px |

### 1.1 원인 분석

페이지 2 col 2 [4~6] 박스 좌표 (BEFORE / AFTER Stage 3):

```
BEFORE:  box top y=224.4    AFTER:  box top y=224.4 (변동 없음)
         box bot y=1429.2           box bot y=1435.1 (+5.87 px)
         첫 텍스트 y=255.2          첫 텍스트 y=261.1 (+5.87 px)
```

박스 top 은 시프트 안 되고 텍스트만 시프트 → 박스 안 위쪽 여백이 +5.87 px 늘어남.

**근본 원인**: Task #540 Stage 2/3 의 cumulative `vpos_neg_ls_floor_total` 보정은
vpos correction 단계에 적용되어 paragraph 본문 텍스트 y 는 시프트되지만,
**paragraph border 그리는 코드 경로는 영향받지 않음**.

`src/renderer/layout/paragraph_layout.rs:786`:
```rust
let bg_y_start = if para_border_fill_id > 0 {
    y_start  // ← 빈 paragraph (pi=45) 의 y_start = unshifted (cumulative 0)
} else { y };
```

빈 paragraph (pi=45) 가 paragraph border (border_fill_id=7) 를 가지고 있어 push
되고, 다음 paragraph (pi=46, border_fill_id=4) 와 stroke_sig 동일하면 merge →
group top = pi=45 의 unshifted y_start. pi=46 시작은 cumulative 시프트 되지만
group top 은 그대로 → 위쪽 여백 증가.

## 2. 정정

A안 채택: floor 대상 paragraph (text=∅ + controls=∅ + 음수 ls) 의
border push 자체를 skip 하여 group top 이 다음 paragraph 의 shifted 시작
위치로 잡히게 한다.

### 2.1 변경 (paragraph_layout.rs:2665)

```rust
let is_540_floor_target = para.map(|p|
    p.text.is_empty()
        && p.controls.is_empty()
        && p.line_segs.iter().any(|s| s.line_spacing < 0)
).unwrap_or(false);
if para_border_fill_id > 0 && cell_ctx.is_none() && !is_540_floor_target {
    // ... border push ...
}
```

### 2.2 변경 (layout.rs:1807)

A안만 적용 시 cross-column/page 연속 검출 로직이 floor 대상 paragraph 의
stroke_sig 를 인식하여 partial_start = true 로 잘못 설정 → 박스 top edge 누락
회귀 추가 발견. floor 대상 paragraph 를 sig 매칭에서도 제외:

```rust
let is_540_floor = |pi: usize| -> bool {
    paragraphs.get(pi).map(|p|
        p.text.is_empty()
            && p.controls.is_empty()
            && p.line_segs.iter().any(|s| s.line_spacing < 0)
    ).unwrap_or(false)
};

if !g.7 && first_pi > 0 && !is_540_floor(first_pi - 1) {
    let prev_sig = stroke_sig(para_bf(first_pi - 1));
    if prev_sig.is_some() && prev_sig == group_sig { g.7 = true; }
}

if !g.8 && !is_540_floor(last_pi + 1) {
    let next_sig = stroke_sig(para_bf(last_pi + 1));
    if next_sig.is_some() && next_sig == group_sig { g.8 = true; }
}
```

## 3. 검증

### 3.1 단위 테스트

```
test result: ok. 1120 passed; 0 failed; 1 ignored
test_540_empty_paragraph_negative_ls_floor ... ok (gap 38.88 px ✓)
```

### 3.2 박스 위치 검증

페이지 2 col 1 [4~6] 박스:
| | BEFORE Stage 1 | AFTER Stage 4 | 시프트 |
|---|---|---|---|
| 박스 top y | 224.4 | **246.6** | +22.2 px |
| 박스 bottom y | 1429.2 | 1435.1 | +5.87 px |
| 박스 outline lines | 4 (top/right/bottom/left) | 4 (모두 그려짐) | ✓ |
| 첫 텍스트 baseline | 255.2 | 261.1 | +5.87 px |

박스 top 시프트 +22.2 px 의 분해:
- pi=45 (빈) advance 흡수 (+14.67 px) — push skip 으로 group 첫 항목이 pi=46
- cumulative comp (+5.87 px) — pi=46 시작 위치 시프트
- + small inset adjustment

박스 안 텍스트 위치와 박스 top 간격이 정합 유지 (BEFORE 30.8 px → AFTER 14.5 px,
이전엔 pi=45 의 advance 가 박스 안 빈 공간으로 보였으나 이제는 박스 외부로 이동).

### 3.3 광범위 회귀 검증 (Stage 3 와 동일)

```
=== synam: 0 pages w/ diffs, NEG: 0 ===
=== 21: 3 pages w/ diffs, NEG: 0 ===  (페이지 1, 2, 14)
=== math: 6 pages w/ diffs, NEG: 0 ===
=== eng: 0 pages w/ diffs, NEG: 0 ===
=== kor: 0 pages w/ diffs, NEG: 0 ===
=== sci: 0 pages w/ diffs, NEG: 0 ===
```

음수 시프트 (회귀) 0건. Stage 3 결과와 동일.

페이지 1 col 0 박스: top 551.9 → 566.6 (+14.7 px)
페이지 14 col 1 박스: top 224.4 → 246.6 (+22.2 px)

## 4. 산출물

| 파일 | 변경 |
|------|------|
| `src/renderer/layout.rs` | sig 검사 가드 (+10 LOC) |
| `src/renderer/layout/paragraph_layout.rs` | border push skip 가드 (+12 LOC) |
| `mydocs/working/task_m100_540_stage4.md` | 본 보고서 |

## 5. 회고

Stage 3 광범위 회귀 검증은 텍스트 좌표만 비교하여 paragraph border 의 시프트
누락을 검출하지 못했다. 향후 paragraph border 가 영향받는 변경에서는 박스
outline 좌표도 회귀 검증 대상에 포함해야 한다.

「본질 정정 회귀 위험」 [feedback_essential_fix_regression_risk] 메모리 룰에
따라 본 정정도 광범위 검증 시 시각적 회귀 (박스 위치/edge) 까지 포함해야
한다는 교훈.

## 6. 승인 요청

Stage 4 완료. 커밋 + merge + PR 업데이트 진행 승인 요청.
