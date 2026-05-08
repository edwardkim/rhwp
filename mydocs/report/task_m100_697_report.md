# Task #697 최종 결과 보고서

큰 셀 cross-page split 결함 정정 — `inner-table-01.hwp` (한글 2022 PDF mismatch)

- 이슈: [#697](https://github.com/edwardkim/rhwp/issues/697)
- 마일스톤: v1.0.0 (M100)
- 브랜치: `local/task697`
- 상태: **부분 정합 완료** — 본 task 종결, 후속 작업은 [#700](https://github.com/edwardkim/rhwp/issues/700) 분리

## 1. 결함 요약

`samples/inner-table-01.hwp` (8행×2열 표) 의 행 6 셀[11] (`사업개요`, 26 paragraphs, h=48776 HU≈677px) 이 본문 영역(877px) 절반을 초과한다. 한글 2022 PDF 는 셀 내부 vpos 리셋 위치(p[19]→p[20], 33120 HU = 459.7px)에서 페이지 분할하지만 rhwp 는 cell 안 26 paragraphs 모두를 1페이지에 그려 압축/오버플로우 발생.

| 페이지 | 변경 전 RMSE | 변경 후 RMSE |
|---|---|---|
| p1 | 26.41% | **26.22%** (-0.19%) |
| p2 | 22.57% | **22.48%** (-0.09%) |

## 2. 정정 내용

### 2.1 핵심 변경

`src/renderer/layout/table_layout.rs::compute_cell_line_ranges` — paragraph 진입 시 LINE_SEG.vpos 리셋 검출 후 cum 강제 진행:

```rust
if pi > 0 && has_limit && cum < abs_limit {
    let prev_para = &cell.paragraphs[pi - 1];
    let prev_end_vpos_hu = prev_para.line_segs.last()
        .map(|s| s.vertical_pos + s.line_height)
        .unwrap_or(0);
    let cur_first_vpos_hu = para.line_segs.first().map(|s| s.vertical_pos).unwrap_or(0);
    if prev_end_vpos_hu > 0 && cur_first_vpos_hu < prev_end_vpos_hu {
        // 한컴이 셀 내부 페이지 분할 위치에서 LINE_SEG.vpos 를 0 으로 리셋한 신호
        cum = abs_limit;
    }
}
```

### 2.2 line_ranges 산출 정합

| | inner-table-01 cell[11] |
|---|---|
| 변경 전 | 26 paras 모두 visible — 한컴 정합과 어긋남 |
| 변경 후 | 20 visible (p[0..19]) + 6 skip (p[20..25]) — 한컴 정합과 일치 |

### 2.3 시도했으나 보류 — Stage 3-2 광범위 정합

| 시도 | 결과 |
|---|---|
| `compute_cell_line_ranges` 정상 vpos delta 보정 | form-002.hwpx p1 회귀 (마지막 paragraph 누락) |
| `table_partial.rs` paragraph 시작 y 의 vpos 보정 | 위 변경 A 회귀에 종속 — 함께 revert |

→ paragraph y 시각 배치 정합은 셀별 가드와 광범위 회귀 검증이 필요. **후속 이슈 [#700](https://github.com/edwardkim/rhwp/issues/700) 으로 분리**.

## 3. 회귀 검증

| Fixture | 결과 |
|---|---|
| `cargo test --release` 전체 (21 그룹) | ✅ 0 failures |
| `tests/svg_snapshot.rs` (form-002 포함, 7 tests) | ✅ pass |
| `renderer::layout::*` 104 tests | ✅ pass |
| `samples/hwpx/form-002.hwpx` p1 PartialTable | ✅ paragraph 누락 없음 |

### 3.1 광범위 RMSE 비교 (변경 전/후 동일 baseline)

| Fixture | 페이지 매핑 | 변경 전 avg RMSE | 변경 후 avg RMSE |
|---|---|---|---|
| `samples/inner-table-01.hwp` | 2/2 | 24.49% | **24.35%** |
| `samples/k-water-rfp.hwp` | 18/27 | 22.87% | 22.87% |
| `samples/issue_265.hwp` | 7/16 | 22.00% | 22.00% |
| `samples/hwp3-sample.hwp` | 7/16 | 22.00% | 22.00% |

→ inner-table-01 외 fixture **변화 없음** (회귀 없음). 22-23% baseline 은 Linux 환경 폰트 폴백 (Noto Sans 등) 차이.

## 4. 단계별 진행 결과

| 단계 | 산출물 | 상태 |
|---|---|---|
| Stage 1 | `mydocs/working/task_m100_697_stage1.md` (분석) | ✅ |
| Stage 2 | `mydocs/plans/task_m100_697_impl.md` (구현 계획서) | ✅ |
| Stage 3-1 | `mydocs/working/task_m100_697_stage3_1.md` (vpos 리셋 검출 적용) | ✅ |
| Stage 3-2 | `mydocs/working/task_m100_697_stage3_2.md` (광범위 정합 보류) | ✅ |
| Stage 4 | 본 보고서 + 후속 이슈 #700 | ✅ |

## 5. 커밋 이력 (`local/task697`)

| 커밋 | 내용 |
|---|---|
| `76ec93e8` | Stage 1: 수행계획서 + 분석 보고서 |
| `51140755` | Stage 2: 구현 계획서 |
| `3cb5ac50` | Stage 3-1: compute_cell_line_ranges vpos 동기화 + 리셋 검출 |
| `7c8b84a9` | Stage 3-2 후속: vpos delta 보정 제거 (form-002 회귀 가드) |
| `5e76020c` | Stage 3-2 보고서 |

## 6. 남은 작업

본 task 에서 정정 완료한 영역:
- ✅ `compute_cell_line_ranges` 의 line_ranges 산출 — vpos 리셋 검출로 한컴 정합 일치

본 task 에서 다루지 못한 영역 (→ #700):
- paragraph y 시각 배치 metric 의 vpos 정합화 (셀별 가드 + 광범위 회귀 검증 필요)
- 폰트 폴백 RMSE baseline (22-23%) — 별 영역, Linux 환경 한컴 호환 폰트 부재

## 7. 결론

본 task 의 1차 목표 — `compute_cell_line_ranges` 의 vpos 리셋 정합화 — 는 완수했다. 이는 후속 paragraph y 정합 작업의 전제 조건으로 작동한다. 시각 RMSE 의 결정적 개선은 paragraph y 정합 (#700) 과 함께 이뤄질 영역.

**회귀 없음**, **본 변경은 안전**, **후속 작업은 별 이슈로 분리**.

---

승인 요청: 본 최종 보고서 검토 후 issue #697 close 진행해도 되는지 확인 부탁드립니다 (작업지시자 승인 후 close 수행).
