# Task #697 Stage 3-1 — vpos 리셋 검출 구현 결과 보고

- 단계: Stage 3-1 (구현 1차)
- 변경 파일: `src/renderer/layout/table_layout.rs`
- 빌드: ✅ 통과 (cargo build --release)
- 테스트: ✅ `renderer::layout` 모듈 104 tests passed, 0 failed (회귀 없음)

## 1. 적용 변경

`compute_cell_line_ranges` 의 paragraph 루프 진입부에 다음 분기 추가:

```rust
if pi > 0 {
    let prev_para = &cell.paragraphs[pi - 1];
    let prev_end_vpos_hu = prev_para.line_segs.last()
        .map(|s| s.vertical_pos + s.line_height)
        .unwrap_or(0);
    let cur_first_vpos_hu = para.line_segs.first().map(|s| s.vertical_pos).unwrap_or(0);
    if prev_end_vpos_hu > 0 && cur_first_vpos_hu < prev_end_vpos_hu {
        // vpos 리셋 — page-break 신호
        if has_limit && cum < abs_limit { cum = abs_limit; }
    } else if prev_end_vpos_hu > 0 && cur_first_vpos_hu > prev_end_vpos_hu {
        // vpos 정상 누적 — paragraph 사이 spacing 차분만큼 cum 보정
        let vpos_delta_hu = cur_first_vpos_hu - prev_end_vpos_hu;
        cum += hwpunit_to_px(vpos_delta_hu, self.dpi);
    }
}
```

## 2. line_ranges 산출 검증 — `inner-table-01.hwp` cell[11]

변경 전:
```
[(0, 1), (0, 1), ..., (0, 1)]   (26 paragraphs 모두 visible)
```

변경 후:
```
[(0, 1)×20, (0, 0)×6]
        ↑ p[0..19] visible, p[20..25] skip (vpos 리셋 검출 후 마킹)
```

→ paragraph 단위 line_ranges 산출은 **PDF 정합과 일치**.

## 3. layout 분기 정상 작동 확인

`table_partial.rs` L558 `else if !has_nested_table` 분기에서 `(0, 0)` paragraph 들을 정상 skip (continue). debug 로그로 검증:
```
skip cell r=6 c=1 cp_idx=20  (start_line=0 end_line=0)
skip cell r=6 c=1 cp_idx=21  (start_line=0 end_line=0)
... (p[20..25] 모두 skip 분기 진입)
```

## 4. RMSE 측정

| 페이지 | 변경 전 | 변경 후 | Δ |
|---|---|---|---|
| p1 | 26.41% | **26.18%** | -0.23% (개선) |
| p2 | 22.57% | 22.66% | +0.09% (미세 회귀) |

→ **변화 미미** — 본 변경만으로 결정적 개선 안 됨.

## 5. 잔존 결함 분석 — paragraph y 시각 배치 mismatch

SVG cell[11] 영역 텍스트 y 분포:

| 항목 | 값 |
|---|---|
| 첫 텍스트 y (cell area 시작 545.64 + first paragraph) | 562.25 |
| 마지막 visible 텍스트 y (p[19] 끝) | 969.19 |
| 분포 폭 | ~407px |
| 한컴 vpos 기반 cell 분포 폭 (p[19] 끝 = 33120 HU) | 459.7px |
| 차이 | **~52px** (rhwp 가 paragraph 들을 더 좁게 그림) |

→ **layout 단의 paragraph y 배치 metric 이 line_height+line_spacing 누적을 사용**하므로 한컴 vpos 와 어긋남. PDF 가 마지막 visible paragraph 를 y≈1005 까지 spaced out 하게 그리는 반면 rhwp 는 y=969 까지 compact 하게 그림. 모든 paragraph 의 시각 위치가 PDF 와 다름 → RMSE 차이의 주된 원인.

이 정정은 `compute_cell_line_ranges` (산출) 가 아니라 `layout_composed_paragraph` 호출처 (`table_partial.rs` L660 등) 의 **paragraph 사이 y 진행 metric 변경** 이 필요. 영향 범위가 큼.

## 6. 추가 발견

### 6.1 PDF 측 cut 위치 정밀 분석

PDF p1 cell[11] 의 visible 끝 paragraph 는 약 **p[17]** 부근 (`- 전사 데이터 수집/유통체계 구축`). p[17] 끝 vpos = 27920+1300 = 29220 HU = 405.83px. 이는 abs_limit (459.72) 보다 작은 위치에서 cut 했음을 의미.

→ 한컴은 단순 `vpos > abs_limit` 비교가 아니라 추가 마진/spacing 고려. 정밀 정합화는 후속 분석 필요.

### 6.2 본 변경의 가치

- `line_ranges` 산출의 정합성은 명확히 개선 (이전: 26 paras visible / 변경: 20 paras visible — 한컴 vpos 단위와 1:1 매칭).
- 그러나 paragraph y 시각 배치 정합 없이는 시각 RMSE 가 거의 변화 없음.
- 후속 단계에서 paragraph y 배치 정합이 함께 이뤄지면 본 변경이 전제 조건으로 작동.

## 7. 권고 — 진행 옵션

| 옵션 | 설명 | 영향 |
|---|---|---|
| (A) | 본 변경 유지 + 단위 테스트 추가 + 다음 sub-stage 로 진행 (paragraph y 배치 정합) | 본 task 범위 확장 — Stage 2 plan 갱신 필요 |
| (B) | 본 변경 유지 + 단위 테스트 추가 + Stage 4 검증 종료 (현 결함 부분 정합으로 확정) | 시각 RMSE 개선 미미 — task 목표 (시각 정합) 미달 |
| (C) | 본 변경 revert + Task #697 재계획 | paragraph y 배치까지 포함한 통합 변경으로 재설계 |

## 8. 권고 의견

옵션 **(A)** 채택 권고. 이유:
- 본 변경은 line_ranges 정합화로서 명확한 가치 — paragraph y 배치 정합과 결합 시 전제 조건
- 회귀 없음 (104 tests pass)
- paragraph y 배치 정합은 본 task 범위 안에서 추가 sub-stage 로 진행 가능
- 옵션 (C) 의 revert 는 정합 작업의 기초를 잃음

---

승인 요청: 옵션 (A) 채택 + Stage 2 plan 에 sub-stage 추가 (paragraph y 배치 정합) + Stage 3-2 진행, 또는 (B)/(C) 중 선택해 주시면 그에 따라 진행하겠습니다.
