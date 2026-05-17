# Task #957 Stage 1 — Cursor over-advance source 정확 식별

## 1. RHWP_DEBUG_TAC_CURSOR 추적 결과

```
TAC_CURSOR FullPara pi=394 y_in=255.8 y_out=767.3 dy=511.5 ✓ 정상 (paragraph height)
TAC_CURSOR Shape pi=394 ci=1 y_in=767.3 y_out=1197.9 dy=430.6 ← 회귀 source
TAC_CURSOR FullPara pi=395 y_in=1197.9 ...
```

→ **Shape pi=394 ci=1 의 layout_shape_item 이 +430.6px 추가 advance**.

비교 — pi=393 (InFrontOfText wrap picture):
```
TAC_CURSOR Shape pi=393 ci=0 y_in=257.3 y_out=257.3 dy=0.0 ← 정상
```

→ InFrontOfText 는 OK. **TopAndBottom wrap + caption 보유** 시에만 회귀.

## 2. Code path 추적

위치: `src/renderer/layout.rs:3243` `layout_shape_item` → `if let Control::Picture(pic) = ctrl` → `if pic.common.treat_as_char` 분기.

### 2.1 pic_y 계산 (라인 3284)
```rust
let pic_y = para_start_y.get(&para_index).copied().unwrap_or(y_offset);
```

`para_start_y[394]` 가 has_prior_tac_in_para 조건 (라인 3267-3273) 으로 인해 갱신됨:
- 초기값: pi=394 FullPara 진입 시 y_offset = 255.8
- 갱신: `needs_update = y_offset > existing + 1.0` → 767.3 > 255.8+1.0 → true → para_start_y[394] = 767.3

→ pic_y = 767.3 (실제 image SVG y=298.99 와 불일치).

### 2.2 Caption advance (라인 3428-3476)

pi=394 ci=1 picture 의 `pic.caption`:
- `dump samples/hwp3-sample16.hwp -s 0 -p 394` 결과:
  ```
  [1] 그림: ... caption: dir=Bottom width=0 paras=1 text=""
  ```
- caption 존재 (Some) 이지만 text="" (빈 caption)
- direction = Bottom

라인 3440-3475 의 caption 계산:
```rust
let image_bottom = pic_y + baseline_px.max(pic_h);
// = 767.3 + max(2.87, 411.89) = 767.3 + 411.89 = 1179.19
let cap_y = image_bottom + caption_spacing;
// = 1179.19 + 0 = 1179.19
let caption_h = self.calculate_caption_height(&pic.caption, styles);
// = ~18.7 (empty paragraph default line height)
if matches!(caption.direction, CaptionDirection::Bottom) {
    let cap_bottom = cap_y + caption_h;
    // = 1179.19 + 18.7 = 1197.89
    if cap_bottom > result_y {
        result_y = cap_bottom;  // 767.3 → 1197.89
    }
}
```

→ **빈 caption 의 phantom 영역 할당** + **잘못된 pic_y 사용** 의 복합 결함.

## 3. Visual 영향

- "나." caption (pi=394 **ci=2** Table, 별개 control) 은 y=755 에 정상 emit (image_bottom + spacing).
- pi=394 ci=1.caption (text="") 은 SVG 에 invisible 하지만 **layout 만 cap_bottom=1197.89 까지 진행**.
- pi=395 (○ 통합모델...) 가 1197.89 부터 시작 → body 외 영역 emit.

## 4. Fix 후보

### A. 빈 caption (text="" + paragraphs 모두 empty) 시 result_y advance skip

**위치**: 라인 3470-3475 의 caption.direction == Bottom 분기.

**변경**:
```rust
// 빈 caption (text 없음 + controls 없음) 은 layout 공간 차지하지 않음
let caption_is_empty = caption.paragraphs.iter().all(|p|
    p.text.chars().all(|c| c <= '\u{001F}' || c == '\u{FFFC}')
    && p.controls.is_empty()
);
if !caption_is_empty && matches!(caption.direction, CaptionDirection::Bottom) {
    let cap_bottom = cap_y + caption_h;
    if cap_bottom > result_y {
        result_y = cap_bottom;
    }
}
```

- 위험: **중** — caption 이 의도적으로 빈 frame 인 case 영향
- 정밀도: 본 sample16 영역 한정

### B. `already_registered` 시 pic_y 를 actual position (tree.get_inline_shape_position) 으로 교정

**위치**: 라인 3284 의 pic_y 결정.

**변경**:
```rust
let pic_y = if already_registered {
    tree.get_inline_shape_position(
        page_content.section_index, para_index, control_index, None
    ).map(|(_, y)| y).unwrap_or(y_offset)
} else {
    para_start_y.get(&para_index).copied().unwrap_or(y_offset)
};
```

- 위험: **고** — already_registered 다른 case 모두 영향
- 정밀도: 광범위

### C. 위 A + B 결합

본 회귀의 양쪽 결함 모두 정정. 위험 가장 큼.

## 5. 권장 fix

**Option A** — 가장 정밀 + 회귀 위험 최소.

빈 caption 은 layout 공간을 차지하지 않아야 하는 명백한 본질 (HWP3 의 caption frame 이 frame structure 만 보유). 본 fix 는 빈 caption 의 phantom advance 만 차단하고, 정상 caption case (text 보유) 는 영향 없음.

## 6. 후속 (Stage 2)

- Stage 2: 구현 계획 V2 — Fix Option A 의 안전한 구현 + 위험 평가
- Stage 3: 구현 + sample16 page 18 검증
- Stage 4: 다중 sample 회귀 검증 (특히 empty caption 보유 다른 sample)
