# Task #332 Stage 3a — vpos_end 에서 trail_ls 제외 — 완료보고서

- **계획서**: `mydocs/plans/task_m100_332_impl.md`
- **브랜치**: `task332`
- **작성일**: 2026-04-25

---

## 변경 사항

### 코드 (`src/renderer/layout.rs:1367`)

```diff
-let vpos_end = seg.vertical_pos + seg.line_height + seg.line_spacing;
+// Task #332: typeset/layout 의 height_for_fit 모델과 정합 —
+// vpos_end 의 trailing line_spacing 을 제외해 다음 문단의 시작 y_offset
+// 이 trail_ls 만큼 일찍 보정되도록 한다.
+let vpos_end = seg.vertical_pos + seg.line_height;
```

vpos correction 이 trigger 될 때 다음 문단 시작 y_offset 을 계산할 때 trailing line_spacing 을 더하지 않도록 한다. typeset 의 `height_for_fit` 모델과 일관됨.

## 검증 결과

### 자동 테스트

```
cargo test --lib                  → 992 passed
cargo test --test '*' (기타)      → 모두 passed
cargo test --test svg_snapshot    → 4 passed, 2 FAILED (Stage 2 와 동일 baseline 차이, 추가 변동 없음)
```

### 수동 회귀 (21_언어 page 0)

```bash
./target/release/rhwp export-svg samples/21_언어_기출_편집가능본.hwp -p 0 --debug-overlay
```

| 단계 | LAYOUT_OVERFLOW |
|------|-----------------|
| Stage 2 | col=0 pi=10 partial (9.5px) — 1 건 |
| Stage 3a | col=0 pi=10 partial (9.5px) — **1 건 (변동 없음)** |

pi=10 partial 의 overflow 가 본 단계로는 해소되지 않음. vpos correction 이 trigger 되는 조건(prev_layout_para 와 segment_width 일치 등)이 col 0 의 pi=9 → pi=10 partial 경계에서 만족되지 않아 보정 자체가 적용되지 않는 듯. 본질적으로 typeset 이 pi=10 partial 을 col 0 끝까지 배치한 결과를 layout 이 그대로 따라가다 overflow 하는 케이스이므로:

- Stage 3b: 양방향 보정 + collapse 가드 → 보정 적용 범위 확대로 일부 해소 가능
- Stage 4: clamp pile 제거 → overflow 라인 stop drawing 으로 글자 겹침 자체 차단

본 단계 단독의 효과는 미미하나, 다음 문단 시작 y_offset 이 trail_ls 만큼 위로 당겨지는 정합 효과는 다른 케이스(특히 form-002, multi-table) 에서 누적적으로 작용. 회귀 없음.

## 다음 단계

Stage 3b: `layout.rs:1392` 의 `end_y >= y_offset - 1.0` 단방향 가드 → 양방향 + collapse 가드.
