# Task #352 Stage 5 완료보고서: dash leader elastic Justify 분배

> 2026-04-28 | Branch: `local/task352`

---

## 배경

Stage 2~3 의 좁은 dash advance 적용 후 작업지시자 피드백:
> "5, 6 페이지 32 item 이후부터는 폭이 2배 커짐"
> "폰트가 달라져서 발생하는 현상인가?"
> "pdf 출력도 폰트가 아니고 직접 그린거지?"

**원인 분석**:
- dash advance 가 좁아지자 dash leader 가 있는 라인의 자연 폭이 짧아짐
- Justify Branch A 가 발생한 슬랙을 단어 사이 공백에 균등 분배 → 단어 공백이 자연 폭의 2~3 배로 팽창
- PDF 는 dash leader 자체를 path/graphic 으로 elastic 하게 그려 슬랙을 흡수 (pdftotext 가 dash 를 단어로 추출하지 못함이 증거)

**예시 (p6 Q33 라인 y=414)**:
| 측정 | 13 글자 폭 | 비고 |
|------|----------|------|
| Baseline (수정 전) | 83.6 px | 자연 폭 압축 (-2.55 px/공백) |
| Stage 3 (0.5 em) | 125.7 px | **공백 +14 px 팽창** ← 사용자가 본 "2배" |
| **Stage 5 (elastic)** | **95.6 px** | 자연 폭 (압축 없음) ✓ |

---

## 변경 사항

### `src/renderer/mod.rs`
TextStyle 에 신규 필드 추가:
```rust
/// Task #352: dash leader 글자당 추가 간격 (px). Justify 슬랙을
/// dash leader 가 흡수하여 공백 분배 부담을 줄임.
pub extra_dash_advance: f64,
```
Default = 0.0. `style_resolver`/`text_measurement::resolved_to_text_style` 의 초기값에도 0.0 추가.

### `src/renderer/layout/text_measurement.rs`
세 char_width 클로저 (`estimate_text_width`, `compute_char_positions`, `estimate_text_width_unrounded`) 에 적용:
```rust
let is_leader = is_dash_leader_run(&chars, i);
let base_w = if is_leader {
    base_w_raw.min(font_size * 0.3)  // 좁은 base (Stage 2 의 0.5 em 에서 0.3 em 으로 회수, elastic 으로 보충)
} else {
    base_w_raw
};
let mut w = base_w * ratio + style.letter_spacing + style.extra_char_spacing;
if c == ' ' { w += style.extra_word_spacing; }
if is_leader { w += style.extra_dash_advance; }
```

### `src/renderer/layout/paragraph_layout.rs`
Justify 분기에 `count_dash_leaders` 헬퍼 + leader-aware 분배 추가:
```rust
let leader_dashes = count_dash_leaders(&all_chars[..visible_count]);
if interior_spaces > 0 {
    let slack = available_width - effective_used;
    if leader_dashes > 0 && slack > 0.0 {
        // dash leader 가 슬랙 흡수
        (0.0, 0.0, slack / leader_dashes as f64)
    } else {
        // 기존: 공백에 분배
        ...
    }
}
```

`extra_dash_sp` 출력값을 `text_style.extra_dash_advance` 에 전달.

세 분기 (Justify Branch A, Branch B, Distribute, overflow, cell underflow) 의 반환 튜플을 `(extra_word_sp, extra_char_sp, extra_dash_sp)` 3-tuple 로 확장. dash leader 검출은 Justify Branch A/B 에만 적용 (다른 분기는 0.0 유지).

---

## 검증

### Q32 (s0 p221 L6) 디버그 로그

```
[#352-S5] s0p221 L6 leader_dashes=29 interior_sp=3 text_w=337.00
          effective_used=330.00 avail=408.21 slack=78.21 per_dash_extra=2.697
```

per_dash_extra = 78.21 / 29 = 2.697 px ✓
각 dash advance = 0.3 × 15.31 × 0.95 + 2.70 = 4.36 + 2.70 = **7.06 px**
29 dashes 폭 = 29 × 7.06 = **204.7 px**

SVG underline 측정: x1=597.12 → x2=801.84, **width = 204.72 px** ✓

PDF 측정 218 px 의 93.9% (이전 Stage 3 의 96.7% 보다 약간 감소했으나, 단어 공백이 자연 폭으로 회복된 게 더 중요).

### p6 Q33 라인 (y=414, 단어 공백 정상화) — 핵심 회귀 해소

| 글자 | Baseline | Stage 3 (0.5em) | Stage 5 |
|------|---------|----------------|---------|
| 't' | 132.24 | 132.24 | 132.24 |
| 'o' | 136.48 | 136.48 | 136.48 |
| 'b' | 147.79 | **161.84** (+14) | **151.79** (+4) |
| 'e' | 155.44 | 169.49 | 159.44 |
| 'f' | 165.88 | **193.99** (+24) | **173.88** (+8) |
| 마지막 'n' | 215.80 | 257.96 (+42) | **227.80** (+12) |

Stage 5 의 단어 공백은 Baseline 대비 +3~4 px 만 차이 (Branch A 압축 해제 결과 — 자연 폭). Stage 3 의 +14~24 px 폭증은 **완전히 해소**.

### cargo test --release

전 테스트 통과: 1023 + 50+ 통합 = 회귀 없음.

---

## 작업지시자 질문 답변

### "pdf 출력도 폰트가 아니고 직접 그린거지?"

**Yes — 사용자 추정 정확**. `pdftotext -bbox-layout` 으로 PDF p5 Q32 / p6 Q34 를 추출한 결과:

```
Q34 PDF 추출:
  "34. Centralized, formal rules can"  (xMax=265.02 pt)
  "."                                    (xMin=402.24, 137 pt 공백)
  ─ 그 사이 137 pt 영역에 단어 0개
```

dash 글리프가 텍스트로 출력되었다면 pdftotext 가 추출했을 것. 단어 부재는 PDF 가 dashes 를 path/line graphic 으로 직접 그린 것을 강하게 시사. Stage 3 의 SVG `<line>` 대체와 동일 접근.

### PDF 의 dash advance 가변성

| 라인 | dash 수 | 영역 폭 | per dash | em |
|------|---------|---------|---------|-----|
| Q32 | 29 | 218 px | 7.49 px | 0.49 |
| Q34 | 40 | 183 px | 4.57 px | 0.30 |

PDF 는 dash leader 를 **고정 폭이 아니라 라인 슬랙에 따라 가변** 으로 그림. 우리 Stage 5 알고리즘 (`slack / leader_dashes`) 이 동일 동작.

---

## 다음 단계

- 최종 보고서 갱신 (Stage 5 결과 반영)
- orders/20260428.md 의 #352 항목 갱신
- 통합 커밋 + PR-ready 상태
