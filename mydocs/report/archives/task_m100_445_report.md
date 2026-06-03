# Task #445 최종 결과 보고서

**이슈**: [#445](https://github.com/edwardkim/rhwp/issues/445) — 지문 박스(border_fill_id) 문단이 페이지 분할 시 세로 테두리가 col_bottom 을 초과해 그려짐
**브랜치**: `local/task445` (분기점: `local/devel`)
**마일스톤**: M100 (v1.0.0)
**작성일**: 2026-04-29

---

## 1. 작업 개요

`exam_kor.hwp` SVG 출력에서 사용자가 보고한 시각적 결함 2가지를 해결:

1. **Paragraph border 페이지 바깥 침범**: 본문 하단의 지문 박스 세로선이 col_bottom 을 넘어 페이지 영역 자체(1587px) 까지도 초과
2. **페이지 번호 박스가 column divider line 에 붙음**: 본문 가운데 세로선과 페이지 번호 박스(예: "1/20") 사이에 갭이 없어 시각적으로 부자연스러움

PDF (`samples/hancomdocs-exam_kor.pdf` 포함 한컴 PDF 3종) 와 비교하며 두 가지 모두 PDF 와 일치하도록 수정.

## 2. 진단 결과

### 2.1 결함 1: Paragraph border 페이지 바깥 침범

| 페이지 | 수정 전 세로선 끝 y | col_bottom (1422.93) 대비 | 페이지(1587) 대비 |
|--------|------|------|------|
| 2 | 1452.7 | +30 px 초과 | OK |
| 5 | 1506.5 | +84 px 초과 | OK |
| 8 | **1671.3** | +248 px 초과 | **+84 px 페이지 바깥** |
| 15 | 1595.8 | +173 px 초과 | +9 px 페이지 바깥 |

직접 원인:
- 페이지네이션이 `respect_vpos_reset=false` (기본값) 로 동작 → vpos-reset 가 있는 paragraph 가 FullParagraph 로 처리되며 분할 라인이 col_bottom 너머에 layout 됨 (page 8 의 pi=297 가 91.7px 오버슈트)
- 그 결과 다음 PartialParagraph (pi=298) 의 layout y_start 가 이미 col_bottom 너머
- `layout.rs:expand_clip` 이 자식 bbox 의 bottom 으로 body-clip 을 확장만 하므로(축소 없음) clip 이 페이지 바깥까지 늘어남
- paragraph border 가 그 영역에 그대로 그려짐

### 2.2 결함 2: 페이지 번호 박스 위치

PDF 측정 (`hancomdocs-exam_kor.pdf` A3 200dpi):
- 페이지 번호 박스 top y = **380.6 mm** (= body_bottom + 4.3mm)
- column line 과 박스 갭 = **4.3 mm (16.0 px)**

기존 SVG:
- 박스 top y = 376.4 mm (= body_bottom)
- 갭 = 0 mm (붙음)

직접 원인:
- 페이지 번호 박스는 꼬리말 paragraph 의 1×1 표 (`wrap=TopAndBottom`, `vert=Para/off=0`, `vert_align=Top`) 로 그려짐
- 기존 `compute_table_y_position` 은 vert=Para + vert_align=Top 일 때 표 top 을 paragraph top 에 정렬
- HWP 의 실제 동작: 첫 라인의 line_height/2 만큼 아래에 anchor (line center 기준)
- 꼬리말 paragraph 의 line_height = 2480 HU = **16.5 px** ≈ PDF 측정 갭 16.0 px

## 3. 수정 내용

### 3.1 Stage 1+2 — Paragraph border 클램프

**위치**: `src/renderer/layout.rs::build_single_column`

paragraph border 의 merge 그룹을 col_area 바닥/꼭대기로 클램프 (border 만 클램프, 텍스트 자체의 overflow 처리는 별도 이슈로 분리).

```rust
let col_top = col_area.y;
let col_bot = col_area.y + col_area.height;
for g in groups.iter_mut() {
    if g.2 < col_top { g.2 = col_top; }
    if g.4 > col_bot { g.4 = col_bot; }
}
groups.retain(|g| g.4 > g.2);
```

### 3.2 Stage 3 — 꼬리말 표 line center anchor

**위치**: `src/renderer/layout.rs::layout_header_footer_paragraphs`

머리말/꼬리말의 첫 paragraph 가 wrap=TopAndBottom + vert=Para 표를 가질 때, 표 배치 y_offset 에 `line_height / 2` 를 더한다.

```rust
let line_anchor_offset = if matches!(t.common.text_wrap, TextWrap::TopAndBottom)
    && matches!(t.common.vert_rel_to, VertRelTo::Para)
    && i == 0
{
    let lh_hu = para.line_segs.first().map(|ls| ls.line_height as i32).unwrap_or(0);
    hwpunit_to_px(lh_hu, self.dpi) / 2.0
} else { 0.0 };
let table_y = y_offset + line_anchor_offset;
```

### 3.3 폐기된 시도 (커밋되지 않음)

진단 도중 도입했다가 PDF 측정 후 부적합으로 제거한 시도:
- `clamp_master_page_separators` (master-page polygon 강제 클램프) — PDF 는 폴리곤을 자연 길이대로 그림
- `normalize_column_divider_shapes` (폴리곤을 body_top..body_bottom 으로 클램프) — PDF 는 폴리곤 시작 y 가 페이지마다 다름 (body 내 타이틀 유무에 따라)
- `build_column_separators` 의 콘텐츠-기반 클램프 — exam_kor 은 HWP 내장 separator 를 쓰지 않음

## 4. 검증

### 4.1 페이지별 결과 (exam_kor.hwp 20p)

| 페이지 | 수정 전 line end y | 수정 후 line end y | 박스 top y | line-박스 갭 |
|--------|------|------|------|------|
| 1 | 1422.43 | 1422.43 | 1439.47 | **17.04 px** |
| 2 | 1452.73 | 1423.17 | 1439.47 | **16.29 px** |
| 5 | 1506.45 | 1423.17 | 1439.47 | 16.29 px |
| 8 | **1671.28** | 1423.17 | 1439.47 | 16.29 px |
| 15 | 1595.83 | 1422.93 | 1439.47 | 16.49 px |

PDF 비교:
- PDF column line end (모든 페이지): **376.3-376.4 mm** = 1422-1423 px ✓ 일치
- PDF 박스 top: **380.6 mm** = 1438.9 px ✓ (우리 1439.47 — 0.6 px 차)
- PDF 갭: **4.3 mm** = 16.0 px ✓ (우리 16.3-17.0 px)

### 4.2 column line 길이 (PDF 자연 차이 재현)

| 페이지 | 우리 SVG 길이 | PDF 길이 |
|--------|---------------|---------|
| p1 | 1131 px (299.3mm) | 299.6mm |
| p2~p20 | 1226 px (324.4mm) | 324.6mm |

p1 vs p2+ 의 25mm 길이 차이는 PDF 도 동일 — 페이지 1 은 본문 내 타이틀(제 1 교시, 국어 영역) 이 있어 폴리곤 시작 y 가 더 아래임 (의도된 design).

### 4.3 회귀 테스트

| 항목 | 결과 |
|------|------|
| `cargo test --release` | **1117 passed / 0 failed / 1 ignored** |
| svg_snapshot 갱신 | 1건 (`tests/golden_svg/issue-267/ktx-toc-page.svg`, invisible 구조 rect height 5.34px 변경, 가시 변화 없음) |
| exam_eng / exam_math / exam_science / k-water-rfp / KTX / aift | 새로운 line-past-page-bottom 검출 0건 |

## 5. 산출물

### 코드 변경

- `src/renderer/layout.rs`:
  - `build_single_column` 의 paragraph border merge 후 col_area 클램프 (5줄 추가)
  - `layout_header_footer_paragraphs` 의 머리말/꼬리말 표에 line_anchor_offset 추가 (15줄 추가)

### 문서

- `mydocs/plans/task_m100_445.md` — 수행계획서
- `mydocs/plans/task_m100_445_impl.md` — 구현계획서
- `mydocs/working/task_m100_445_stage1.md` — Stage 1 (진단)
- `mydocs/working/task_m100_445_stage2.md` — Stage 2 (paragraph border 클램프)
- `mydocs/working/task_m100_445_stage3.md` — Stage 3 (footer 표 anchor 보정)
- `mydocs/report/task_m100_445_report.md` — 본 문서

### 테스트 골든

- `tests/golden_svg/issue-267/ktx-toc-page.svg` 갱신

### 커밋 (`local/task445`)

```
10d8709 Task #445 Stage 3: 머리말/꼬리말 wrap=TopAndBottom 표 anchor 위치 보정
c151156 Task #445 Stage 1+2: paragraph border 가 col_bottom 너머로 그려지는 문제 수정
```

## 6. 본 작업 범위 외 발견 사항

다음은 본 작업 진단 중 확인했으나 이슈 #445 범위를 벗어나는 별도 이슈:

- **respect_vpos_reset 정책**: 페이지네이션의 vpos-reset 미존중이 paragraph 의 col_bottom 너머 layout 의 진짜 원인. 본 작업의 클램프는 시각적 증상만 가림. 텍스트 자체의 overflow (예: page 8 col 0 의 일부 텍스트가 col_bottom 너머 그려짐) 는 남음. 별도 이슈로 분리 검토 필요.

## 7. 결론

이슈 #445 의 두 가지 시각적 결함 (paragraph border 페이지 바깥 침범 + 페이지 번호 박스 line 에 붙음) 모두 PDF 와 일치하도록 수정 완료. 회귀 0건. 본 task 종료 후 `local/devel` 머지 권장.

---

승인 후 `local/devel` 머지 + 이슈 #445 close 진행.
