# Task #296 Stage 1 보고서 — 조사·실증

## 목표

WASM Canvas 렌더 경로가 inline_tabs 를 무시하여 `exam_math.hwp` p.7 #18 "수열" 문항이 우측으로 밀리는 버그의 원인 실측.

## 진단 방법

`src/renderer/layout/text_measurement.rs` 의 두 측정기(`EmbeddedTextMeasurer`, `WasmTextMeasurer`)에 환경변수 `RHWP_TAB296=1` 시 로그 출력:
- `estimate_text_width`: 3개 분기(inline/custom/default) 각각
- `compute_char_positions`: inline 분기

네이티브 실행: `RHWP_TAB296=1 cargo run ... export-svg samples/exam_math.hwp -p 6 ...`

## 실측 결과

문단 0.144 `"18.\t\t\t수열 이 모든 자연수 에 대하여"` 의 inline_tabs 3개:

| i (char idx) | ext[0] (HU) | ext[2] | tab_width_px | total 변화 |
|--------------|-------------|--------|--------------|------------|
| 3 (첫 `\t`) | 132 | 256 (0x0100) | 1.76 | 26.48 → 28.24 |
| 4 (둘째 `\t`) | 671 | 256 (0x0100) | 8.95 | 28.24 → 37.19 |
| 5 (셋째 `\t`) | 79 | 256 (0x0100) | 1.05 | 37.19 → 38.24 |

- `ext[2] = 256 = 0x0100` → **high=1 (LEFT), low=0** (PR #292 트러블슈팅 문서에서 실증된 포맷과 일치)
- 3개 탭 합계 폭 ≈ 12px → "수열"이 x≈38 (PDF 일치)

## 구조 비교 (두 측정기)

### `EmbeddedTextMeasurer` (네이티브, L170~305)

**inline_tabs 분기 존재**:
```rust
if tab_char_idx < style.inline_tabs.len() {
    let ext = &style.inline_tabs[tab_char_idx];
    let tab_width_px = ext[0] as f64 * 96.0 / 7200.0;
    let tab_type = ext[2];                         // ⚠️ 전체 u16 사용
    // match { 1 => RIGHT, 2 => CENTER, _ => LEFT }
    tab_char_idx += 1;
}
```

**버그**: `tab_type = ext[2]` 전체 u16. 실제 HWP 값 `0x0100 = 256` 이 `1`, `2` 와 match 실패 → 전부 `_ =>` LEFT 처리. **우연히 이번 exam_math #18 케이스는 LEFT 가 맞아서 증상 노출 안 됨**.

### `WasmTextMeasurer` (WASM, L520~680)

**inline_tabs 분기 자체가 부재**:
```rust
if c == '\t' {
    if has_custom_tabs {
        let (tab_pos, tab_type, _) = find_next_tab_stop(...);  // TabDef만
        // ...
    }
}
```

→ `style.inline_tabs` 완전 무시. TabDef 만 사용 → `abs_x > tab_stops` 모두 지나면 `auto_tab_right=true` 폴스루 → **RIGHT 오판 → 다음 run 우측 밀림** (x≈290.91).

## 진단 결과

- **근본 원인**: WasmTextMeasurer 에 inline_tabs 처리 부재
- **수정 방향**: WasmTextMeasurer 에 EmbeddedTextMeasurer 와 동등한 inline_tabs 분기 추가. 단 `tab_type = (ext[2] >> 8) & 0xFF` 로 고바이트 판정 (PR #292 실증 포맷).
- **네이티브 측 수정 여부**: 동일 버그가 네이티브에도 있으나 기존 골든 SVG (issue-147, issue-267)가 "우연한 LEFT 폴백"에 의존 중 → Task #296 범위 외, 별도 이슈로 분리.

## 결론

수행계획서 **옵션 A (WASM만 수정, 네이티브는 주석으로 이슈화)** 방향 확정. Stage 2 구현계획서 작성 진행.
