---
PR: #723
제목: Task #722 — hwp3-sample5.hwp wrap=Square 그림 paragraph 시각 정합 (closes #722)
컨트리뷰터: @jangster77 (Taesup Jang) — 15+ 사이클 핵심 컨트리뷰터 (HWP 3.0 파서 영역)
base / head: devel / local/task722
mergeStateStatus: BEHIND
mergeable: MERGEABLE — 충돌 0건
CI: ALL SUCCESS
변경 규모: +1459 / -3, 16 files (소스 3 + 보고서 11 + plans 2)
검토일: 2026-05-10
---

# PR #723 검토

## 1. 메타

| 항목 | 값 |
|------|-----|
| PR 번호 | #723 |
| 제목 | Task #722 — hwp3-sample5.hwp wrap=Square 그림 paragraph 시각 정합 |
| 컨트리뷰터 | @jangster77 (Taesup Jang) — 15+ 사이클 핵심 컨트리뷰터 (HWP 3.0 파서 영역, 누적 1 머지 + 다회 close cherry-pick) |
| base / head | devel / local/task722 |
| mergeStateStatus | BEHIND, mergeable: MERGEABLE — `git merge-tree` 충돌 0건 |
| CI | ALL SUCCESS |
| 변경 규모 | +1459 / -3, 16 files |
| 커밋 수 | 8 (1 본질 + 7 devel merge 영역) |
| closes | #722 |
| 분리된 후속 | Issue #732 (Task #724 — HWP5 변환본 paragraph 441 영역, 별도 등록) |

## 2. 결함 본질 (Issue #722)

### 2.1 결함 영역 — 3 paragraph

`samples/hwp3-sample5.hwp` 영역 의 wrap=Square 그림 paragraph 영역 (페이지 8/27/48):

| paragraph | 페이지 | LINE_SEG | 결함 |
|-----------|--------|----------|------|
| **175** ("아래에 디렉토리 트리...") | 8 | 2 | image 영역 침범 → image z-order 후 가려짐 |
| **779** ("Figure 4-4. 마운트된...") | 27 | 1 | caption-style anchor host wrap zone 매칭 (회귀) |
| **1394** ("접근 제어") | 48 | 1 | image 영역 침범 + image 가 텍스트 가려짐 |

### 2.2 inter-image-text gap 미적용
모든 paragraph 영역 의 image outer margin (3mm = 852 HU) 미반영 영역.

## 3. PR 의 정정 — 3 영역

### 3.1 `src/renderer/pagination.rs` (+6 LOC)

`WrapAnchorRef` 영역 의 `anchor_image_margin_right: i32` 필드 추가:
```rust
pub struct WrapAnchorRef {
    pub anchor_para_index: usize,
    pub anchor_cs: i32,
    pub anchor_sw: i32,
    /// [Task #722] anchor image 의 outer margin_right (HWPUNIT)
    pub anchor_image_margin_right: i32,
}
```

### 3.2 `src/renderer/typeset.rs` (+56 LOC)

#### A. anchor 다음 paragraph 등록 영역 의 margin_right 추출
```rust
// Picture/Shape/ShapeObject::Picture 영역 의 image margin.right 추출
let anchor_margin_right = paragraphs.get(st.wrap_around_table_para)
    .and_then(|p| p.controls.iter().find_map(|c| {
        let cm = match c {
            Control::Picture(pic) => Some(&pic.common),
            Control::Shape(s) => if let ShapeObject::Picture(pic) = s.as_ref() {
                Some(&pic.common)
            } else { None },
            _ => None,
        };
        cm.filter(|cm| !cm.treat_as_char && matches!(cm.text_wrap, TextWrap::Square))
            .map(|cm| cm.margin.right as i32)
    })).unwrap_or(0);
st.current_column_wrap_anchors.insert(
    para_idx,
    WrapAnchorRef { ..., anchor_image_margin_right: anchor_margin_right },
);
```

#### B. anchor host paragraph 자체 self-register (case 가드)
```rust
// Case 가드 (Stage 3~5 진단):
//   - LINE_SEG ≥ 2 → wrap zone (multi-line, 강제 wrap)
//   - LINE_SEG 1 + caption_room ≤ line_height → wrap zone (image 위 caption 영역 부재)
//   - LINE_SEG 1 + caption_room > line_height → caption-style (자기 미등록, 자유 영역)
let body_top_hu = page_def.margin_top as i32;
let line_height_hu = para.line_segs.first().map(|s| s.line_height as i32).unwrap_or(900);
let (image_voff_hu, image_margin_right_hu) = para.controls.iter().find_map(...);
let caption_room_hu = image_voff_hu - body_top_hu;
let is_caption_style = para.line_segs.len() == 1 && caption_room_hu > line_height_hu;
if !is_caption_style {
    // host paragraph 자체도 wrap zone 등록 (image 우측 layout)
    st.current_column_wrap_anchors.insert(para_idx, WrapAnchorRef { ... });
}
```

### 3.3 `src/renderer/layout/paragraph_layout.rs` (+10/-3)

wrap_anchor 영역 의 cs/sw 영역 inter-image-text gap 보정:
```rust
let (line_cs_offset, line_avail_w_override) = if let Some(anchor) = wrap_anchor {
    let cs = seg.column_start;
    let sw = seg.segment_width;
    let mr = anchor.anchor_image_margin_right;
    // [Task #722] cs += margin_right_px / sw -= margin_right_px (3mm gap 정합)
    let cs_px = hwpunit_to_px(cs + mr, self.dpi);
    let sw_px = if sw > 0 {
        Some(hwpunit_to_px((sw - mr).max(0), self.dpi))
    } else { None };
    (cs_px, sw_px)
} else { (0.0, None) };
```

## 4. Case 가드 정합 (3 paragraph)

| paragraph | LINE_SEG | caption_room | line_height | 분류 | 처리 |
|-----------|----------|--------------|-------------|------|------|
| 175 | 2 | — | — | wrap zone (multi-line) | host self-register + gap 3mm |
| 779 | 1 | 9720 | 900 | **caption-style** | host 미등록 (col 전체 폭) |
| 1394 | 1 | -12 | 900 | wrap zone | host self-register + gap 3mm |

→ caption_room 가드 영역 영역 (1394 영역 의 -12 < 900) 영역 의 정확한 분기 영역. 회귀 (779 영역 의 caption-style 영역 의 wrap zone 매칭) 영역 차단 영역.

## 5. 본 환경 점검

- fixture: `samples/hwp3-sample5.hwp` 존재 ✓
- PDF 권위본: `pdf/hwp3-sample5-2022.pdf` 존재 ✓
- 본 환경 rhwp = PDF 권위 = **64 페이지** 정합 ✓
- 충돌 0건

## 6. 영향 범위

### 6.1 변경 영역
- HWP3 변환본 영역 의 wrap=Square 그림 paragraph 영역 의 시각 정합
- inter-image-text gap (3mm) 영역 적용 영역

### 6.2 무변경 영역
- treat_as_char=true 영역 무관
- 비-Square wrap (TopAndBottom / BehindText / InFrontOfText) 영역 무관
- LINE_SEG 1 + caption-style 영역 (자유 영역 보존)
- HWP5 변환본 (hwp3-sample5-hwp5.hwp) — 별건 (Issue #732, Task #724)

### 6.3 위험 영역
- `caption_room` 임계값 영역 의 line_height 영역 영역 — 실 케이스 영역 의 재현 가능성 영역
- Stage 8~9 시도 (`anchor_full_width_match` 가드) 영역 회귀 발견 → rollback (PR 본문 명시) 영역 정합

## 7. 충돌 / mergeable

- merge-base = `c9dd6f9c` (5/9 PR #719 후속 시점) — **devel HEAD 와 매우 가까움**
- `git merge-tree --write-tree` 실측: **CONFLICT 0건**
- 8 commits 영역 의 7개 영역 devel merge 영역 영역 의 정합 영역 (작업지시자 영역 의 다른 PR 머지 영역 자동 영역 영역 따라감 영역)

## 8. 처리 옵션

### 옵션 A — 1 commit cherry-pick + no-ff merge (추천)

PR 영역 의 8 commits 영역 중 7개 영역 devel merge 영역 영역 의 본질 commit 1개만 영역 cherry-pick. PR #694~#720 패턴 일관.

```bash
git branch local/task723 c9dd6f9c
git checkout local/task723
git cherry-pick ab910d65
git checkout local/devel
git merge --no-ff local/task723
```

→ **옵션 A 추천**.

## 9. 검증 게이트 (구현 단계 DoD)

### 자기 검증
- [ ] `cargo build --release` 통과
- [ ] `cargo test --release` — 전체 lib + 통합 ALL GREEN
- [ ] `cargo clippy --release --all-targets` clean
- [ ] 광범위 sweep — 7 fixture / 170 페이지 회귀 0
- [ ] form-002 / test_634 / issue_712/713/716 회귀 가드 영역 보존

### 시각 판정 게이트 (필수)
- 본 PR 영역 의 본질 = **시각 정합 영역** (PDF 권위 영역 정합) 영역
- PR 본문 명시: `rsvg-convert PNG 시각 판정: 페이지 8/27/48 PDF 권위 자료 정합 ★`
- PR Test plan 영역: `메인테이너 시각 판정 게이트` (체크 부재 영역 — 본 메인테이너 영역 영역 의 직접 시각 판정 영역 권장)
- 점검 영역:
  - 페이지 8 paragraph 175 (디렉토리 트리 설명) — image 침범 부재 + 3mm gap
  - 페이지 27 paragraph 779 (Figure 4-4 caption) — caption-style 영역 자유 영역 보존
  - 페이지 48 paragraph 1394 (접근 제어) — image 침범 부재 + 3mm gap
  - PDF 권위본 (`pdf/hwp3-sample5-2022.pdf`) 정합

→ `feedback_visual_judgment_authority` 정합 (시각 판정 권위).

## 10. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @jangster77 15+ 사이클 핵심 컨트리뷰터 (HWP 3.0 파서 영역) |
| `feedback_hancom_compat_specific_over_general` | LINE_SEG + caption_room 영역 case 가드 영역 의 영향 좁힘 — caption-style 영역 보존 + Stage 8~9 회귀 영역 rollback 정합 |
| `feedback_process_must_follow` | TDD Stage 1~9 절차 정합 + 후속 분리 (Issue #732 → Task #724) |
| `feedback_visual_judgment_authority` | 시각 정합 영역 본질 영역 → 작업지시자 시각 판정 게이트 권장 |
| `feedback_image_renderer_paths_separate` | typeset.rs (anchor 등록) + paragraph_layout.rs (cs/sw 보정) + pagination.rs (필드 추가) 세 영역 동기 정정 |

## 11. 처리 순서 (승인 후)

1. `local/devel` 에서 1 commit cherry-pick (옵션 A)
2. 자기 검증 (cargo test/build/clippy + 광범위 sweep)
3. WASM 빌드 + 작업지시자 시각 판정 (페이지 8/27/48 PDF 정합)
4. 시각 판정 통과 → no-ff merge + push + archives 이동 + 5/10 orders 갱신
5. PR #723 close (closes #722 자동 정합)

---

작성: 2026-05-10
