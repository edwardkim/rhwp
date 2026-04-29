# Task #445 Stage 3 — 페이지 번호 박스 위치 보정

**브랜치**: `local/task445`
**작성일**: 2026-04-29

---

## 1. 추가 발견

Stage 1+2 (paragraph border 클램프) 완료 후 사용자 피드백:

> "1 페이지 하단에 '1/20' 페이지 번호 표시가 줄에 붙어서 따라 올라감. PDF 와 SVG 가 모양이 다름"
> "이것이 근본 문제임"

추가 진단 결과 — **페이지 번호 박스 위치가 PDF 대비 4.3mm 위쪽에 렌더되어 column line 끝점과 박스 top 이 만남(붙어 보임).**

PDF 측정값 (`hancomdocs-exam_kor.pdf` A3 200dpi):
- 페이지 번호 박스 top y: 380.6mm
- column line end y: 376.3mm
- 둘 사이 갭: 4.3mm

기존 SVG:
- 박스 top: 376.4mm (= body_bottom)
- line end: 376.4mm
- 갭: 0mm (붙음)

## 2. 원인 확정

페이지 번호 박스는 **꼬리말 paragraph 의 1×1 표** 로 그려짐. 이 표는:
- `wrap=TopAndBottom`
- `vert=Para` / `vert_offset=0`
- `vert_align=Top`

꼬리말 paragraph 의 line_seg:
- `vpos=0, lh=2480 HU (8.7mm), th=2480 HU, bl=2108 HU, ls=688 HU`

기존 코드 (`compute_table_y_position`) 는 vert=Para + vert_align=Top 일 때 `ref_y = anchor_y` 를 사용 → 표 top 이 paragraph top 에 정렬. 즉 footer_area.y (= body_bottom).

**HWP 의 실제 동작**: vert=Para + wrap=TopAndBottom 표는 첫 라인의 **line_height/2** 만큼 아래에 anchor 됨.

수치 검증:
- line_height/2 = 2480/2 HU = 16.5px ≈ PDF 측정 갭(16.0px)

## 3. 수정 내용

`src/renderer/layout.rs:layout_header_footer_paragraphs` 에서 머리말/꼬리말의 첫 paragraph 가 wrap=TopAndBottom + vert=Para 표를 가질 때, 표 배치 y_offset 에 `line_height / 2` 를 더한다.

```rust
let line_anchor_offset = if matches!(t.common.text_wrap, TextWrap::TopAndBottom)
    && matches!(t.common.vert_rel_to, VertRelTo::Para)
    && i == 0
{
    let lh_hu = para.line_segs.first().map(|ls| ls.line_height as i32).unwrap_or(0);
    hwpunit_to_px(lh_hu, self.dpi) / 2.0
} else {
    0.0
};
let table_y = y_offset + line_anchor_offset;
```

## 4. 검증 결과

### 페이지별 (exam_kor.hwp)

| 페이지 | column line end | 박스 top | 갭 | PDF 갭 |
|--------|-----------------|----------|------|--------|
| p1 | 1422.43 (376.4mm) | 1439.47 (380.7mm) | 17.04 px | 16.0 px |
| p2~p20 | 1423.17 (376.5mm) | 1439.47 (380.7mm) | 16.29 px | 16.0 px |

PDF 와 거의 정확히 일치 (px 단위 0.3 ~ 1.0 px 차이는 폴리곤 종류(body vs master) 의 정의 길이 차이 + 반올림).

### column line 길이

| 페이지 | 우리 SVG | PDF |
|--------|----------|-----|
| p1 | 1131px (299.3mm) | 299.6mm |
| p2+ | 1226px (324.4mm) | 324.6mm |

PDF 자체도 p1 vs p2 가 25mm 길이 차이 — 이는 page 1 에 본문 내 타이틀 컨텐츠가 있어 폴리곤 시작 y 가 더 아래쪽이기 때문 (의도된 design). 우리 SVG 도 동일하게 재현.

### 회귀

- `cargo test --release`: 1117 passed / 0 failed / 1 ignored
- `samples/exam_eng.hwp, exam_math.hwp, exam_science.hwp, k-water-rfp.hwp, KTX.hwp, aift.hwp` 등 다른 샘플에서 새로운 가시 회귀 없음 (pages-past-bottom line 검출: 0)

## 5. 산출물

- 코드 변경: `src/renderer/layout.rs`
  - `layout_header_footer_paragraphs` 의 표 배치에 `line_anchor_offset` 추가
- (Stage 3 시도 중 도입했다가 PDF 비교 후 제거된 코드: `normalize_column_divider_shapes`, `clamp_master_page_separators`, column-separator 콘텐츠 클램프 — 모두 PDF 와 부적합하여 폐기)

## 6. 산출 요약

Task #445 의 두 시각적 결함이 모두 해결됨:
1. ✅ paragraph border 가 col_bottom 을 넘어 페이지 바깥까지 그려지는 문제 (Stage 1+2)
2. ✅ 페이지 번호 박스가 column line 에 붙어 보이는 문제 (Stage 3)

다음 단계: Stage 4 — 최종 결과 보고서 + orders 갱신.
