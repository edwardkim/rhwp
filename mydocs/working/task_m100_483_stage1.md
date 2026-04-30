# Task M100 #483 Stage 1 완료 보고서

## 1. 코드 추적 결과

### 핵심 발견

`src/renderer/layout/picture_footnote.rs:702`의 `layout_footnote_paragraph_with_number` 가 paragraph 마지막 line 처리에서 `y += line_height`만 누적 — **trailing line_spacing 누락**.

비교: 같은 `paragraph_layout.rs:2560`의 `layout_composed_paragraph`는 `y += line_height + line_spacing_px` 로 정확히 적용.

각주의 첫 paragraph는 `layout_footnote_paragraph_with_number`(번호 표시)로, 두 번째 이후는 `layout_composed_paragraph`로 처리되는 비대칭 구조가 multi-paragraph 각주에서만 시각 차이로 드러남.

## 2. 수정 적용

`src/renderer/layout/picture_footnote.rs:702`:

```rust
parent.children.push(line_node);
// [Issue #483] trailing line_spacing 추가 — layout_composed_paragraph:2560 과 정합.
let line_spacing_px = hwpunit_to_px(comp_line.line_spacing, self.dpi);
y += line_height + line_spacing_px;
```

## 3. 핵심 회귀 검증

### 각주 1) 안 paragraph 간 gap (samples/2010-01-06.hwp 페이지 1)

| transition | Before | After | HWP vpos delta |
|-----------|--------|-------|---------------|
| p[0]→p[1] | 16.0px | **20.8px** ✓ | 20.8px |
| p[1]→p[2] | 20.8px | 20.8px | 20.8px |

각주 1) 안 줄 간격이 다른 paragraph 사이 간격과 일치하게 정정됨.

### 단일 paragraph 각주 사이 gap (부수 영향)

| transition | Before | After | 차이 |
|-----------|--------|-------|------|
| 1)→2) | 28.4px | 28.4px | 0 |
| 2)→3) | 23.6px | **33.2px** | +9.6 |
| 3)→4) | 23.6px | 33.2px | +9.6 |
| 4)→5) | 23.6px | 33.2px | +9.6 |

각주 2)~5)는 단일 paragraph (ParaShape 5, ls=720 HU=9.6px). trailing line_spacing이 추가되며 각주 사이 gap이 +9.6px 증가. 이는 코드 일관성 정정의 결과 — `layout_composed_paragraph`와 동일 동작.

## 4. 회귀 검증

### 단위/통합 테스트
```
cargo test --release
test result: ok. 1078 passed; 0 failed; (lib + 모든 통합)
```

### 골든 SVG
```
test result: ok. 6 passed; 0 failed; (svg_snapshot)
```

### footnote-01.hwp 시각 비교
| 페이지 | baseline | fix | 차이 |
|-------|----------|-----|------|
| 1 | 107912 bytes | 107703 bytes | 시각 거의 동일 (-209 byte 미세 차이) |
| 2 | 137794 | 137684 | 시각 거의 동일 |
| 3 | 135518 | 135342 | 시각 거의 동일 |
| 4-6 | identical | identical | 변경 없음 |

footnote-01.hwp는 단일 paragraph 각주 위주라 시각적 차이 미미. multi-paragraph 각주가 있는 케이스만 명확한 시각 정정.

## 5. 다음 단계

Stage 2/3 — 최종 보고서 + 커밋 + PR.
