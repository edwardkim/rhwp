# Task #770 최종 결과 보고서

**제목**: shortcut.hwp 페이지 2~7 헤더 TAC 1x1 표 후속 spacing 누락 (~13-33px 압축)
**Issue**: [#770](https://github.com/edwardkim/rhwp/issues/770)
**브랜치**: `local/task770` (stream/devel 베이스)
**작업 기간**: 2026-05-10 (단일 세션)
**최종 상태**: ✅ closes #770

---

## 1. 결함 요약

`samples/basic/shortcut.hwp` 페이지 2~7 의 헤더 zone (1x1 TAC 표 + 후속 PartialParagraph) 이 PDF 권위(한글 2022) 대비 13~33 px 짧음. 본문이 위쪽으로 압축.

**PDF 권위**: 페이지 2 헤더 ('파일') ~ 본문 ('새 문서') 거리 ≈ 60 px.
**rhwp (Before)**: 동일 거리 ≈ 21 px → 40 px 압축.
**rhwp (After)**: ≈ 47 px → PDF 정합 (hwp_used 47.1 일치).

## 2. Root cause 분석

### Stage 2 instrument trace

```
TASK770_TBL: pi=1  y_in=99.36  y_out=130.45 advance=31.09  (정합)
TASK770_TBL: pi=29 y_in=410.45 y_out=441.55 advance=31.09  (정합)
TASK770_TBL: pi=36 y_in=56.69  y_out=87.79  advance=31.09  ← 결함 (47.1 정합 필요)
TASK770_TBL: pi=81 y_in=56.69  y_out=94.19  advance=37.49  ← 결함 (53.5 정합 필요)
```

### 핵심 차이 — LINE_SEG 개수

| paragraph | LINE_SEGs | first.lh | last.lh | vpos_end (px) | advance (px) |
|-----------|-----------|----------|---------|---------------|------------|
| pi=1 | **1** | 2332 | 2332 | 31.1 | 31.09 ✓ |
| pi=29 | 1 | 2332 | 2332 | 31.1 | 31.09 ✓ |
| **pi=36** | **2** | 1200 | 2332 | **47.1** | **31.09 ❌** |
| **pi=81** | **2** | 1200 (ls=480) | 2332 (ls=480) | **53.5** | **37.49 ❌** |

→ `tac_seg_applied` 분기 (`layout.rs:2585+`) 가 `line_segs.get(control_index)` 만 처리. **다중 LINE_SEG paragraph 의 후속 line 의 lh+ls 가 advance 에 누락**.

### 식별된 패턴

- shortcut pi=36: ColumnDef + 1x1 TAC 헤더 표 + 후속 빈 line. **ls[0].lh < ls[1].lh** (16 < 31)
- Textmail pi=0: ColumnDef + 11x3 표 + 2x1 표. **ls[0].lh > ls[1].lh** (744 > 126)

→ `last.lh > first.lh` 가드 가 헤더 패턴 단독 식별자.

## 3. 정정 (`src/renderer/layout.rs:2599-2628`)

```rust
// [Task #770] ColumnDef + TAC 1x1 헤더 표 + 후속 빈 라인 패턴의
// paragraph (예: shortcut.hwp pi=36 페이지 2 "파일") 처리 시
// 후속 LINE_SEG 의 lh+ls 가 advance 에 누락되는 결함 정정.
let has_column_def = para.controls.iter().any(|c|
    matches!(c, Control::ColumnDef(_)));
if has_column_def {
    if let (Some(first_seg), Some(last_seg)) = (
        para.line_segs.first(),
        para.line_segs.last(),
    ) {
        if last_seg.vertical_pos > first_seg.vertical_pos
            && last_seg.line_height > first_seg.line_height
        {
            let para_vpos_end = last_seg.vertical_pos
                + last_seg.line_height
                + last_seg.line_spacing.max(0);
            let target_y = para_y_for_table
                + hwpunit_to_px(para_vpos_end, self.dpi);
            if target_y > y_offset {
                y_offset = target_y;
            }
        }
    }
}
```

### 3중 가드

1. **has_column_def**: ColumnDef 동반 (다단 영역 진입 paragraph)
2. **last.vpos > first.vpos**: 다중 LINE_SEG
3. **last.lh > first.lh**: 헤더 line 0 < 후속 line 1 패턴 (헤더 표 단독 식별)

가드 통과 시 paragraph_vpos_end 까지 advance 보장.

## 4. 검증 결과

### RED → GREEN

```
$ cargo test --test issue_770 -- --nocapture

[issue_770] page_index=1 body_y=56.69 pi=37_y=103.79 offset=47.09 (expected_min=40)
test issue_770_page2_body_paragraph_below_header_zone ... ok
```

→ pi=37 ('새 문서') offset 31.09 → **47.09** (hwp_used 47.1 정합 ✓).

### 회귀 검증

```
$ cargo test --release
test result: ok. 1217 passed; 0 failed; 2 ignored;
```

- 통합 테스트 회귀 0
- 골든 SVG 7개 PASS
- test_539 / test_548 / test_exam_math_page_count PASS

### 광범위 (205 샘플)

| 메트릭 | Before | After | Δ |
|--------|--------|-------|---|
| 샘플 수 | 205 | 205 | — |
| `LAYOUT_OVERFLOW_DRAW` 총 | 226 | 228 | +2 (shortcut만) |
| `LAYOUT_OVERFLOW` 총 | 354 | 358 | +4 (shortcut만) |

샘플별 변경:
- `shortcut.hwp`: DRAW 12→14 (+2), FLOW 13→17 (+4) — 본문 정합 후 column 잔여 변화 (의도된 차이)
- 그 외 204 샘플: 변동 없음 ✓

→ **shortcut.hwp 단독 영향**. 다른 샘플 (Textmail.hwp 등 다중 LINE_SEG TAC 표 보유) 회귀 0.

### 시각 정합 (PDF 비교)

PDF 페이지 2: 헤더 ('파일') ~ 본문 ('새 문서') ≈ 60 px
rhwp (Before): 21 px (40 px 압축)
rhwp (After): **~47 px** (PDF 정합 근접, hwp_used 와 일치)

## 5. 영향 분석

### 본 정정의 효과

1. **다중 LINE_SEG 헤더 paragraph 의 advance 정합** — line 0 (표) + line 1 (후속) 모두 반영
2. shortcut.hwp 페이지 2/3 의 본문 위치 PDF 정합
3. ColumnDef + Table 동반 다단 영역 진입 paragraph 의 vpos_end 정합

### Newspaper 다단 보호

- Textmail.hwp pi=0 (ls[0].lh=55830 > ls[1].lh=9460): 가드 3 차단 ✓
- 일반 TAC 표 (ColumnDef 미동반): 가드 1 차단 ✓
- 단일 LINE_SEG paragraph: 가드 2 차단 ✓

→ shortcut.hwp 의 페이지 2/3 헤더만 영향.

### 잔존 영역 (본 task 비범위)

페이지 4/5/7 의 단일 paragraph 헤더 (pi=94 등 FullParagraph) 는 본 결함과 별개. 본 정정으로 영향 없음.

## 6. 단계별 산출물

| Stage | 커밋 | 산출물 |
|-------|------|--------|
| 0 | `0d2cf5b0` | 수행 + 구현 계획서 |
| 1 (RED) | `5f9892ce` | tests/issue_770.rs + FAIL 확인 |
| 2-3 (분석+GREEN) | `f12565cb` | instrument 측정 + 가드 적용 + RED PASS |
| 4-5 (회귀+광범위) | (보고서 통합) | cargo test 0 failed + 205 샘플 1건 정정 |
| 6 (최종) | (본 커밋) | 최종 보고서 + closes #770 |

## 7. PR 정보

- 브랜치 (origin push 예정): `pr-task770` (stream/devel 베이스)
- conflict 점검: `git merge-tree --write-tree origin/stream/devel...HEAD`
- PR 본문 작성 후 작업지시자 검토 → 승인 시 PR 생성

## 8. 학습 / 노트

### 가드 좁히기의 가치

1차 가드 (ColumnDef + 다중 LINE_SEG): 회귀 14건
2차 가드 (+ last.lh > first.lh): 회귀 0건

→ 단순 패턴 식별 시 회귀 광범위. **한 가지 더 좁은 식별자 추가**가 정확한 정정의 결정자.

### 다중 LINE_SEG 의미

paragraph 의 LINE_SEG 가 다중인 경우:
- vpos=0 → vpos=N (line 0 끝) → vpos=N+ls (line 1 시작)
- 각 line 이 paragraph 안의 별도 영역 (표 본체 / 후속 빈 라인 등)
- layout 시 각 line 의 lh+ls 합으로 paragraph 전체 advance 산출

기존 코드는 `line[control_index]` 만 처리 → control 다음의 후속 line 누락. 본 정정으로 paragraph_vpos_end 까지 보장.

## 9. 관련 자료

- 수행 계획서: `mydocs/plans/task_m100_770.md`
- 구현 계획서: `mydocs/plans/task_m100_770_impl.md`
- Stage 보고서: `mydocs/working/task_m100_770_stage{1,2,3}.md`
- 회귀 테스트: `tests/issue_770.rs`
- 정정 위치: `src/renderer/layout.rs:2599-2628`
- 관련 task: Task #9 (TAC 표 fix_overlay), Task #716 (빈 paragraph fix_overlay skip), Task #768 (Distribute 다단 column-break)
