# Task #700 Stage 1 — 정밀 진단 보고서

- 단계: Stage 1 (분석, 소스 무변경)
- 브랜치: `local/task700`

## 1. mismatch 산술 본질

`samples/inner-table-01.hwp` cell[11] (사업개요, 26 paras):

| paragraph 끝 위치 | line_h+ls+spacing 누적 (rhwp) | LINE_SEG.vpos 누적 (한컴) | 차이 |
|---|---|---|---|
| p[0] 끝 | 21.67px | 18.06px | +3.61 |
| p[7] 끝 | ~119px | 142px (10220 HU) | −23 |
| p[17] 끝 | ~373px | 405.8px (29220 HU) | −33 |
| p[19] 끝 | **406.93px** | **459.7px** (33120 HU) | **−52.8** |

abs_limit (effective_limit) = 459.7px (한컴 vpos 단위로 산출됨, pagination engine 기준).

→ rhwp 의 cum < abs_limit 이라 p[19] 까지 visible 처리. 한컴은 vpos 누적이 abs_limit 와 정확히 일치하므로 p[19] 까지가 cell area 끝까지 차지하지만, **PDF 결과는 p[16] 까지 visible**.

## 2. 한컴 PDF cut 위치 정밀 분석

PDF p1 cell[11] 의 마지막 visible paragraph: `- OA망 내 계측 데이터 DB 인프라 이전을 위한 실시간 데이터 구축` = `p[16]` (text_len=42, vpos=26620, 끝 27920 HU = 387.78px).

PDF p2 cell[11] 의 첫 visible paragraph: `- 전사 데이터 수집/유통체계 구축` = `p[17]` (vpos=27920).

→ **한컴 cut 위치 = 387.78px** (p[17] 시작 = p[16] 끝). abs_limit (459.7px) 보다 **72px 더 일찍 cut**.

이는 pagination engine 의 `split_end_content_limit` 산출이 **이미 잘못**임을 시사. abs_limit 가 한컴 단위 459.7 px 가 맞다면 한컴은 그 위치까지 visible 해야 정합. 그러나 한컴은 387.78 까지 → engine 산출이 한컴보다 72px 큼.

## 3. pagination engine 의 산출 metric

`src/renderer/height_measurer.rs::MeasuredCell` (L800-887):
- `line_heights`: paragraph 별 line 의 `line_height + line_spacing + spacing_before + spacing_after`
- `total_content_height`: `line_heights.iter().sum()` = line metric 누적

`pagination/engine.rs::find_break_row` 등이 `MeasuredTable.remaining_content_for_row` → `total_content_height` 를 사용. **line metric 기반**.

→ pagination 산출도 line metric 단위. 즉 abs_limit=459.7 은 line metric 기준 cell content height 의 절반에 해당. 한컴 vpos 단위 459.7 와는 다른 의미.

다시 검증: cell[11] line_heights 합 = ~573.5px (대략, 26 paras 의 line metric). 한컴 cell h = 48776 HU = 677.4px (cell area, padding 제외). engine 측은 `line_sum < capped` 이면 line_sum 우선 사용. cell.h 기반 cap = 677.4 - padding(2*141 HU = 3.92px) = 673.5. 즉 line_sum 573.5 보다 작으므로 cap 적용 안 됨.

split_end_content_limit 산출 = `avail_content - row_cs - padding`. avail_content 는 본 page 의 행 별 가용 content 영역. cell[11] 행 6 의 avail_content_for_r ≈ avail_for_rows - prev_rows_total - row_cs - padding. 만약 page available_for_rows = 877px - prev_rows(432) = 445px → 약 459.7 (산출).

즉 engine 은 "이 페이지에 459.7px 의 content 가 들어갈 수 있다" 고 판단. line metric 기준으로 459.7px 의 paragraph = p[0..19] (= 406.93px 누적, 마진 약간 있음).

한컴은 이 paragraph 들의 시각적 분포가 vpos 단위로 459.7 + ~50px 더 차지함을 알고 있어, 더 일찍 cut → p[16] 까지만.

## 4. form-002 회귀 분석

`samples/hwpx/form-002.hwpx` cell[73] (행 19, paras=29):

| paragraph | vpos | line_h + ls (HU) |
|---|---|---|
| p[0] vpos=0 lh=1400 | 끝 1400 | 1400+492 = 1892 (line metric) |
| p[1] vpos=1892 lh=200 | 끝 1892+200 = 2092 | 200+72 = 272 |
| p[2] vpos=2164 ... | | |
| p[14] vpos=31646 lh=200 | 끝 31846 | |
| **p[15] vpos=0 (RESET)** | | |
| p[28] vpos=25376 lh=4768 | 끝 30144 | |

- line metric 누적 끝 (cell 마지막) ≈ ?
- vpos 누적 끝 (한컴): 30144 HU = 418.7px (after RESET 영역 7000~30144)

**Stage 3-2 시도의 회귀 원인 (가설)**:

이전 시도: `cum += vpos_delta` (정상 누적). cell[73] 의 paragraph 사이 vpos delta 가 line_h+ls 와 다른 spacing 가짐:
- p[0] line metric end = 1400 + 492 (ls) + spacing_after = ?
- p[1] start vpos = 1892 → vpos_delta = 1892 - 1400 = 492 HU = 6.83px

ParaShape ps_id=59 의 spacing_before 가 정확히 0 이라면 line metric end 는 단지 p[0] line_h+ls = 1892 HU (정확). 그런데 cell 마지막이 아니라 line_spacing 포함 → end_pos = 1400 + 492 = 1892 HU. delta 보정 시 cum += (1892-1892) = 0. mismatch 없음.

하지만 paragraph 별로 spacing_after 가 적용되는데 line metric 은 항상 line spacing 까지만, vpos 는 paragraph 사이 추가 spacing 도 포함할 수 있다. ParaShape 마다 다른 결과.

실증: form-002 회귀 발생 시 마지막 paragraph 누락 — cum 이 abs_limit (443.0) 에 너무 빨리 도달. cell[73] 의 paragraph 사이 spacing 보정이 누적되어 cum > abs_limit.

해법:
1. 보정 분기에 셀별 가드 추가 — paragraph 사이 spacing 차분이 양수인 경우만? (이미 그러함, 제거된 분기)
2. 또는 cum 산출을 paragraph 진입 시 vpos 절대값으로 동기화 (셀 첫 paragraph vpos 가 0 인 경우만)

## 5. 정정 방향 옵션

| 옵션 | 변경 영역 | 위험 | 효과 |
|---|---|---|---|
| **A. height_measurer 의 line_heights/total_content_height 를 vpos 기반으로 전환** | `src/renderer/height_measurer.rs` MeasuredCell 산출 (L800-887) | **매우 큼** — 모든 표 pagination 영향 | pagination + layout 모두 root 정합 |
| **B. layout 만 paragraph y 보정 (Stage 3-2 시도)** | `src/renderer/layout/table_partial.rs` paragraph 루프 | 중간 — 셀별 가드 필요 (form-002 회귀) | 시각 정합 (compact 해결) |
| **C. compute_cell_line_ranges cum 동기화 + 셀별 가드** | `src/renderer/layout/table_layout.rs::compute_cell_line_ranges` | 중간 — 가드 정합 | line_ranges 산출 정합 (paragraph cut 위치만) |

### 권고

**옵션 A** 가 root cause 정정. 그러나 변경 영향이 매우 커서 회귀 위험. 단계적 진행:
1. **Stage 3-1: 옵션 A 시도** — 모든 표 fixture RMSE 비교 (변경 전/후) + 회귀 검출
2. **회귀 발생 시 Stage 3-2: 옵션 B + C 의 결합**, 셀별 가드 정합화
3. **최종**: 가장 안전한 변경 채택

또는:
**옵션 C 우선** — line_ranges 산출 정합만 하고 paragraph y 시각 배치는 추후 별 task. 본 결함의 핵심 (`p[17]` skip 처리되도록) 만 해결.

## 6. 옵션 C 상세 — cum 절대 동기화 (셀별 가드)

```rust
// compute_cell_line_ranges paragraph 루프 진입부
if pi > 0 {
    let cur_first_vpos = para.line_segs.first().map(|s| s.vertical_pos).unwrap_or(-1);
    let prev_para = &cell.paragraphs[pi - 1];
    let prev_end_vpos = prev_para.line_segs.last()
        .map(|s| s.vertical_pos + s.line_height)
        .unwrap_or(-1);
    let cell_first_vpos = cell.paragraphs.first()
        .and_then(|p| p.line_segs.first().map(|s| s.vertical_pos))
        .unwrap_or(-1);

    // 셀별 가드: 첫 paragraph vpos == 0 (한컴 정상 인코딩)
    if cell_first_vpos == 0 && cur_first_vpos >= 0 && prev_end_vpos > 0 {
        if cur_first_vpos < prev_end_vpos {
            // vpos 리셋 — page-break 신호 (기존 분기 유지)
            if has_limit && cum < abs_limit { cum = abs_limit; }
        } else {
            // 정상 누적: cum 을 vpos 절대값으로 동기화
            let target_cum = hwpunit_to_px(cur_first_vpos, self.dpi);
            if target_cum > cum {
                cum = target_cum;
            }
        }
    }
}
```

이 방식의 장점:
- cell first vpos == 0 가드로 form-002 같은 paragraph 들이 영향 없는 케이스 회피 (form-002 cell[73] 첫 vpos = 0 이라 영향 받음 — 검증 필요)
- cum 절대 동기화로 누적 mismatch 방지 (delta 보정의 차분 누적 문제 회피)
- `target_cum > cum` 조건으로 cum 만 전진 (감소 방지)

검증 대상: form-002 cell[73], cell[11] 등 모든 영향 셀에서 cum 변화 확인.

## 7. Stage 2 진입 옵션

| 결정 | 진행 |
|---|---|
| 옵션 A | Stage 2 구현 계획서 — height_measurer 변경 광범위 |
| 옵션 C | Stage 2 구현 계획서 — cum 절대 동기화 (가드 정합) |
| 옵션 B + C | Stage 2 구현 계획서 — paragraph y 보정 + cum 동기화 결합 |

**권고**: **옵션 C 우선** + 광범위 회귀 fixture 검증 (Stage 3 의 sub-stage 로 옵션 A 보강 검토).

근거:
- 옵션 C 는 `compute_cell_line_ranges` 한 함수 내 변경, 회귀 영향 작음
- form-002 회귀 가드 (cell first vpos == 0) 실효 검증 가능
- 시각 정합 (paragraph y 배치) 은 추후 별 task 또는 본 task 의 후속 sub-stage

---

승인 요청: 정정 방향 옵션 (A / B / C / 결합) 선택 + Stage 2 구현 계획서 작성 진행 부탁드립니다.
