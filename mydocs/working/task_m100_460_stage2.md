---
타스크: #460 HWP3 파서 렌더러 중립 재구현
문서: Stage 2 완료 보고서
브랜치: local/task460
작성일: 2026-04-29
---

# Stage 2 완료 보고서

## 수행 내용

### `src/parser/hwp3/mod.rs` — 혼합 단락 LINE_SEG 높이 보정

#### 호출 삽입 (기존 후처리 블록 앞)

```rust
// HWP3 혼합 단락: Para-relative TopAndBottom 그림 구역 내 줄을 그림 하단 아래로 재배치
fixup_hwp3_mixed_para_line_segs(&mut para);
```

#### 헬퍼 함수 추가 (파일 하단, 테스트 모듈 앞)

```rust
fn fixup_hwp3_mixed_para_line_segs(para: &mut Paragraph) {
    // Para-relative TopAndBottom 비-TAC 그림 구역 탐색 → (fig_top_hu, fig_bottom_hu)
    // LINE_SEG 누적 위치 계산 → 마지막 그림-위쪽 seg 탐색 (split_idx)
    // seg[split_idx].line_height = fig_bottom_hu - pos  (그림 하단까지 확장)
    // seg[split_idx].text_height = 0   → advance = lh+ls 보장
    // seg[split_idx].line_spacing = 0
}
```

### 동작 원리

1. pi=76 에서 Para-relative TopAndBottom non-TAC 그림 구역 `[fig_top, fig_bottom]` 탐색
2. LINE_SEG 누적 위치(pos)를 document.rs advance 공식과 동일하게 계산
3. `pos < fig_top && pos + advance > fig_top` → `split_idx` (마지막 그림-위쪽 seg)
4. `seg[split_idx].line_height = fig_bottom - pos` (예: 44700 - 8000 = 36700 HU = 489.3px)
5. compose_paragraph → ComposedLine.line_height 복사 → 렌더러 `y += 489.3px` → 그림 구역 점프

### 수치 검증 (pi=76)

```
before: seg[5] line_height=1000, text_height=1000, line_spacing=600
after:  seg[5] line_height=36700, text_height=0, line_spacing=0

렌더러 sequential y:
  줄 5: y_start+106.7px, advance=489.3px → 줄 6: y_start+596px (그림 하단) ✓

document.rs advance:
  seg[5]: th=0 → lh+ls = 36700+0 = 36700 HU ✓
  총 advance: 8000+36700+11200 = 55900 HU → 올바른 단락 높이 ✓
```

## 검증 결과

```
cargo test --lib
test result: ok. 1068 passed; 0 failed; 1 ignored; 0 measured
```

기존 통과 수(1068) 유지, HWP5/HWPX 회귀 0 ✓
