# Task #688 3단계 완료보고서 — height_measurer 누락 수정 + PDF 시각 정합 + 보조 관찰

## 단계 3 도중 발견된 추가 결함 (Stage 3 fix)

단계 1 의 layout_table 만 수정해서는 외부 표가 시각적으로 그려지지 않는 결함이 잔존. SVG → PNG 변환 시 페이지 5의 4그룹 + 12 추진과제 그리드 영역이 **텅 비어 있음**.

### 진단 (디버그 로깅 trace)

`measured_table.is_some() == true` 이면 `resolve_row_heights` 가 라인 586-590 에서 early return. 그 measured_table 이 `[57.72px]` (= nested 1×1 헤더 셀 height) 였음.

`src/renderer/height_measurer.rs::measure_table_impl()` 라인 457-468 에 layout_table 과 **완전히 동일한 결함의 1×1 unwrap 코드** 잔존:

```rust
// 결함 코드
if let Some(nested) = cell.paragraphs.iter()
    .flat_map(|p| p.controls.iter())
    .find_map(|c| if let Control::Table(t) = c { Some(t.as_ref()) } else { None })
{
    return self.measure_table_impl(nested, ..., depth + 1);
}
```

외부 표 측정 시 nested 1×1 헤더로 unwrap → measured_table.row_heights = [4329HU=57.72px] → layout 단계에서 외부 표 권위 cell.height(50720HU=676px) 무시 → cell-clip 이 57.72px 로 잡혀 nested 11×3 그리드(y=295~)가 모두 클립 밖.

### 수정

layout_table 과 동일한 4가지 조건 (paragraphs.len()==1 + control 정확히 1개의 nested table + visible text 없음) 으로 좁힘. commit `7d8cca27`.

## 시각 정합 검증 (DoD 1, 2, 3, 4 최종 확인)

`rsvg-convert` 로 SVG → PNG 변환:

| 시점 | PNG 크기 |
|------|----------|
| 단계 1 만 (Stage 1 commit) | 26 KB |
| 단계 1 + 단계 3 fix | **142 KB** (×5.5) |

페이지 5 시각 정합 (PDF 권위본 vs SVG/PNG):

✅ 참고 + 정부혁신 비전 및 추진전략
✅ 빨간 박스: 국민이 주도하고 AI가 뒷받침하는 국민주권정부
✅ 파란 박스: 정부혁신 4대 추진전략, 12대 추진과제
✅ 1 참여소통 그룹 + 추진과제 ① ② ③
✅ 2 기본사회 그룹 + 추진과제 ④ ⑤ ⑥
✅ 3 공직혁신 그룹 + 추진과제 ⑦ ⑧ ⑨
✅ 4 공공 AX 그룹 + 추진과제 3개
✅ 페이지 번호 - 5 -

## 광범위 회귀 재검증 (Stage 1 + Stage 3 fix 통합)

```
TOTAL: pages=1502 same=1499 diff=3 samples=159
diff list:
  exam_social__hwp/exam_social_001.svg     (자연 해소, Stage 2 와 동일)
  table-vpos-01__hwp/table-vpos-01_005.svg (의도된 수정)
  table-vpos-01__hwpx/table-vpos-01_005.svg (의도된 수정)
```

Stage 3 fix (height_measurer 수정) 가 **추가 회귀를 일으키지 않음**. Stage 2 결과와 동일.

`cargo test` 1192+ 테스트 전부 통과 (재실행 확인).

## 보조 관찰 측정

### 페이지 1 LAYOUT_OVERFLOW 4.1px — **본 수정의 효과 아님 (정정)**

단계 1 보고서에서 "본 수정으로 자연 해소" 라 적었으나 정정 필요.

base ref(`local/devel`, HEAD~3) 상태에서 직접 `export-svg` 실행 시 **LAYOUT_OVERFLOW 미발생**. 즉 base 자체에서 발생하지 않았으며, 첫 분석 시 발견된 4.1px overflow 는 작업 진입 직전의 `pr-task677` 브랜치 환경 차이로 인한 것이지 본 수정의 효과가 아님.

→ 결론: 본 타스크 수정은 LAYOUT_OVERFLOW 에 영향이 없음. 단계 1 보고서의 해당 항목은 본 보고서에서 정정한다.

### 페이지 2 / 페이지 3 hwp_used diff — 별개 결함, 본 수정 무관

`dump-pages samples/table-vpos-01.hwpx` 결과:

| 페이지 | used | hwp_used | diff | 본 수정 후 변화 |
|--------|------|----------|------|----------------|
| 1 | 913.8 | 863.4 | +50.4px | 변화 없음 |
| 2 | 930.1 | 1722.0 | -791.9px | 변화 없음 |
| 3 | 911.2 | 2569.5 | -1658.3px | 변화 없음 |

본 타스크 수정으로 변화 없음. 별개 결함으로 분류, 후속 이슈 분리 권장.

## 산출물

- 코드 수정: `src/renderer/height_measurer.rs` (+13 / -9 라인) — commit `7d8cca27`
- 검증 SVG: `/tmp/tvpos01_fix4/table-vpos-01_005.svg` (132,896 bytes)
- 검증 PNG: `/tmp/tvpos01_png/p5_after4.png` (142,274 bytes)
- 회귀 검증 결과: `/tmp/svg_diff_full_*` (1502 페이지)

## DoD 최종 충족 현황

| DoD | 항목 | 상태 |
|-----|------|------|
| 1 | pi=34 외부 표 외곽 그려짐 | ✅ — 권위 778.8px 영역에 nested 11×3 그리드 외곽선 정합 |
| 2 | nested 1×1 헤더 텍스트 | ✅ |
| 3 | nested 11×3 그리드 4그룹 + 12 추진과제 텍스트 | ✅ — 시각적으로 표시 (단계 1 만으로는 클립으로 가려졌음) |
| 4 | PDF 권위본과 시각 정합 | ✅ — `rsvg-convert` 변환 결과 PDF 와 동등 |
| 5 | 회귀 없음 | ✅ — 1502페이지 중 의도 변경 2건 + 자연 해소 1건 |
| 6 | `cargo test` 통과 | ✅ — 1192+ 테스트 전부 |

## 결론

3단계로 본 타스크의 모든 DoD 시각/정량 충족. 최종 결과보고서 작성으로 진행.
