# Task #332 Stage 4b — clamp pile 제거 + typeset 측 마진 보강 — 완료보고서

- **계획서**: `mydocs/plans/task_m100_332_impl.md`
- **브랜치**: `task332`
- **작성일**: 2026-04-25

---

## 변경 사항

### 코드 1 — `src/renderer/layout/paragraph_layout.rs`

기존 `text_y = (col_bottom - line_height).max(col_area.y)` 클램프(piling 의 원인)를 제거하고 overflow line 을 원래 y 좌표 그대로 그린다. piling 자체가 발생하지 않으며 콘텐츠 손실도 없다.

```diff
-            let text_y = if cell_ctx.is_none() && text_y + line_height > col_bottom + 0.5 {
-                let clamped = (col_bottom - line_height).max(col_area.y);
-                y = clamped;
-                clamped
-            } else {
-                text_y
-            };
+            if cell_ctx.is_none() && text_y + line_height > col_bottom + 0.5 {
+                eprintln!("LAYOUT_OVERFLOW_DRAW: ... overflow={:.1}px", ...);
+            }
```

`paragraph_layout.rs:2533-2542` 의 두 번째 클램프(빠른 경로)도 동일 정책으로 변경.

### 코드 2 — `src/renderer/typeset.rs`

partial split 의 `avail_for_lines` 에 layout drift 안전 마진 차감 추가. Stage 4a 의 typeset_paragraph 진입 시 마진과 합쳐 10px 적용.

```diff
-            let avail_for_lines = (page_avail - sp_b).max(0.0);
+            let avail_for_lines = (page_avail - sp_b - LAYOUT_DRIFT_SAFETY_PX).max(0.0);
```

또한 Stage 4a 의 마진을 30 → 10 으로 조정 (큰 마진은 페이지 수 증가 회귀를 유발).

## 정책 결정 과정 (참고)

본 단계 진행 중 다음 두 가지를 시도했으나 부적절하다고 판단해 폐기:

1. **stop drawing (overflow line skip)**: 글자 겹침은 차단하나 콘텐츠 손실 발생. 21_언어 pi=10 line 1, hwp-multi-002 pi=68 line 0, aift pi=222 line 3 모두 손실. 폐기.
2. **`detect_column_breaks_in_paragraph` 후처리로 마지막 1 줄 차감**: 21_언어 pi=10 케이스는 LINE_SEG vpos 가 column break 를 만들지 않아 적용 안 됨. 다른 multicolumn 정상 케이스 회귀 우려도 있어 폐기.

최종적으로 **clamp 제거 + 그대로 그림** 정책이 가장 균형. piling 차단 + 손실 없음. 시각적으로 col 경계를 약간 넘김 허용 (수~수십 px).

## 검증 결과

### 자동 테스트

```
cargo test --lib                  → 992 passed
cargo test --test '*' (기타)      → 모두 passed
cargo test --test svg_snapshot    → 4 passed, 2 FAILED (Stage 2 와 동일 baseline 차이)
```

### 21_언어 page 0

```
LAYOUT_OVERFLOW_DRAW: section=0 pi=10 line=1 y=1451.7 col_bottom=1436.2 overflow=15.5px
LAYOUT_OVERFLOW: page=0, col=0, para=10, type=PartialParagraph, y=1461.2, bottom=1436.2, overflow=25.1px
```

clamp pile 차단됨. line 1 이 col_bottom 을 15.5px 넘어 그려지지만 piling 없음, 손실 없음. drift 의 본질적 해결은 Stage 5 의 header (표/Shape) 측정 통합으로 넘김.

### 다른 샘플 회귀

| 샘플 | OVERFLOW | 비고 |
|------|----------|------|
| form-01 | 0 | 정상 |
| hwp-multi-002 | page 2 Table 31.3px (pre-existing), pi=68 line 0 44.7px DRAW | 손실/piling 없음 |
| multi-table-001 | 0 | 정상 |
| lseg-06-multisize | 0 | 정상 |
| aift | Table 2건 (pre-existing), pi=222 line 3 8.6px DRAW | 손실/piling 없음 |

회귀 (콘텐츠 손실/글자 겹침) 없음.

## 알려진 한계

- 21_언어 pi=10 line 1 의 15.5px col 경계 넘김: layout 의 partial 시작 y_offset 이 typeset cur_h 보다 ~48.9px 앞서 있는 본질적 drift. 표 + Shape 영역의 측정 다중성에서 기인.
- Stage 5 (header 측정 통합) 후 drift 자체가 줄어들어야 정상 위치 도달.

## 다음 단계

Stage 5: HeightMeasurer 와 typeset 의 header (표/Shape) 영역 측정 정합. 21_언어 page 0 의 단 0 시작 시점 drift (~17px) 와 partial 진입 시점 drift (~48.9px) 의 원인 추적 후 measurer/typeset 의 동일 데이터 사용 보장.
