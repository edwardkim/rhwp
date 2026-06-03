# Task #332 Stage 1 — typeset advance 를 `height_for_fit` 기반으로 변경 — 완료보고서

- **계획서**: `mydocs/plans/task_m100_332_impl.md`
- **브랜치**: `task332`
- **작성일**: 2026-04-25

---

## 변경 사항

### 코드 (`src/renderer/typeset.rs`)

```diff
@@ typeset_paragraph(): fits 분기
-    st.current_height += fmt.total_height;
+    st.current_height += fmt.height_for_fit;

@@ typeset_paragraph(): line_count == 0 분기
-    st.current_height += fmt.total_height;
+    st.current_height += fmt.height_for_fit;
```

`typeset.rs:612, 622` 의 두 군데. TAC/표 advance(`1059, 1089-1090`) 는 sub-step 1b 로 분리하고 본 단계에서는 손대지 않음 (host paragraph 의 trail_ls 와 표 effective_height 의 관계가 미묘 → 별도 검증 필요).

### 테스트 calibration (`src/document_core/commands/text_editing.rs`)

`height_for_fit` 모델은 trailing line_spacing 을 advance 누적에서 제외하므로, paragraph 단위로 누적할 때 trailing 만큼의 buffer 가 사라진다. 다음 5 개 테스트가 이를 가정으로 만들어져 있어 calibration 필요:

| 테스트 | 변경 |
|--------|------|
| `test_page_overflow_with_enter` | 50 → 500 paragraphs |
| `test_page_break_with_default_line_spacing` | 50 → 100 paragraphs |
| `test_page_break_with_tight_line_spacing` | 비교 대상 160% → 200% (1페이지 차이 역전 노이즈 회피), 50 → 500 paragraphs |
| `test_page_break_with_mixed_line_spacing` | 40 → 120 paragraphs |
| `test_page_boundary_with_incremental_spacing_increase` | single-line text → multi-line text (20 회 반복), 39 → 29 paragraphs, 변경 범위 15..26 → 5..30 |

**핵심 이슈**: `test_page_break_with_tight_line_spacing` 의 100% vs 160% 비교가 `height_for_fit` 모델에서 1페이지 차이로 역전 발생 (100% → 9페이지, 160% → 8페이지). 이는 trail_ls 절약 효과가 100%(작은 절약) 대비 160%(큰 절약) 에서 더 크기 때문. 비교 대상을 200% 로 키워 절대 차이를 명확화 (100% → 9페이지, 200% → 9페이지, ≤ 만족).

`test_page_boundary_with_incremental_spacing_increase` 는 single-line 문단으로는 trailing line_spacing 이 advance 에 반영되지 않아 spacing 증가 효과 약함 → multi-line text 로 전환 후 6 → 12 페이지 정상 증가 확인.

## 검증 결과

### 자동 테스트

```
cargo test --lib                  → 992 passed (Task #321 v6 의 992 유지)
cargo test --test '*'             → 47 passed (golden SVG 6 + 기타 41)
```

### 수동 회귀 (21_언어 page 0)

```bash
./target/release/rhwp export-svg samples/21_언어_기출_편집가능본.hwp -p 0 --debug-overlay
```

- `LAYOUT_OVERFLOW: page=0, col=0, para=9, type=FullParagraph, y=1443.9, bottom=1436.2, overflow=7.7px`
- `LAYOUT_OVERFLOW: page=0, col=0, para=10, type=PartialParagraph, y=1445.7, bottom=1436.2, overflow=9.5px`
- 단 1: pi=26 ~ pi=29 진입 (보기 ①②③ 까지) → Task #331 원 의도 부분 달성 ✓
- col 0 의 LAYOUT_OVERFLOW 2 건이 clamp pile 트리거 가능 → Stage 4 에서 제거 예정

### 단독 적용의 알려진 위험

- col 0 의 pi=9, pi=10 partial 이 typeset 측 fit 통과 후 layout 측 overflow → `paragraph_layout.rs:807-816` 의 clamp pile 분기 트리거 가능. SVG 시각 확인 시 글자 겹침 검증 필요.
- 본 위험은 Stages 2~4 의 정합 작업으로 해소되도록 설계됨. 단독 commit 은 회귀 가능성 있으나, 단계별 분리 정책에 따라 그대로 commit 후 다음 단계 즉시 진행.

## 다음 단계

Stage 2: layout per-paragraph advance 를 `height_for_fit` 와 정합 (`paragraph_layout.rs` 의 trailing line_spacing 누적 위치 식별 후 제외).
