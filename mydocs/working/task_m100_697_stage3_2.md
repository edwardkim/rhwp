# Task #697 Stage 3-2 — paragraph y 배치 vpos 정합 시도 + 회귀 가드

- 단계: Stage 3-2 (구현 2차)
- 결과: **부분 성공** — 광범위 정합은 회귀 발생으로 보류
- 빌드: ✅ 통과
- 테스트: ✅ 전체 통과 (svg_snapshot 7 passed, layout 모듈 104 passed, total ~21 그룹)

## 1. 두 변경 시도

### 1.1 변경 A — `compute_cell_line_ranges` 정상 vpos delta 보정 (Stage 3-1 추가분)

paragraph 진입 시 직전 paragraph 끝 vpos 와 현재 first vpos 의 차분만큼 cum 보정.

**결과**: form-002.hwpx p1 (PartialTable rows=0..20, split_end=443px) 에서 마지막 paragraph(`-나노산소운반체의수동 ...암표적지향`) 누락 회귀.

**원인 추정**: form-002 의 split row 의 어느 셀에서 vpos delta 가 line_h+ls 보다 커서 cum 이 더 빨리 abs_limit 도달 → 마지막 paragraph 가 limit 초과 영역으로 잘못 산정.

→ 즉 vpos delta 보정은 **paragraph 사이 spacing 산출 방식 차이**가 셀별로 다르므로 일반화 불가. 안전하게 적용하려면 셀별 가드 필요.

### 1.2 변경 B — `table_partial.rs` paragraph 시작 y 의 vpos 기반 정합

split row paragraph 들에 대해 paragraph 시작 y 를 baseline + (현재 vpos - baseline vpos) px 로 보정.

**결과**: inner-table-01 p1 시각적 정합 ✅ (PDF p1 과 거의 동일), 그러나 RMSE 변화 미미 (26.18% → 26.16%) — 폰트 폴백 baseline noise 가 26% 의 대부분.

그러나 변경 A 회귀가 selecting 해야 했으므로 (변경 A 가 변경 B 의 전제) 함께 revert.

## 2. 최종 채택

**변경 A 의 단순 리셋 검출 부분만 유지** (Stage 3-1 보고서의 핵심 개선):

```rust
if pi > 0 && has_limit && cum < abs_limit {
    let prev_para = &cell.paragraphs[pi - 1];
    let prev_end_vpos_hu = prev_para.line_segs.last()
        .map(|s| s.vertical_pos + s.line_height)
        .unwrap_or(0);
    let cur_first_vpos_hu = para.line_segs.first().map(|s| s.vertical_pos).unwrap_or(0);
    if prev_end_vpos_hu > 0 && cur_first_vpos_hu < prev_end_vpos_hu {
        cum = abs_limit;
    }
}
```

가드 조건 추가:
- `has_limit && cum < abs_limit` 미리 체크 (이전엔 안쪽에 있던 가드를 진입 조건으로 끌어올림)
- 정상 vpos delta 보정 분기 삭제 (회귀 원인)

## 3. 회귀 fixture 검증

| fixture | 결과 |
|---|---|
| `samples/hwpx/form-002.hwpx` p1 (PartialTable 26x27, split_end=443) | ✅ 통과 |
| svg_snapshot 7 tests | ✅ 모두 통과 |
| renderer::layout 104 tests | ✅ 모두 통과 |
| 전체 cargo test | ✅ 0 failures |

## 4. inner-table-01.hwp 결과

| | 변경 전 | 변경 후 (현재) |
|---|---|---|
| p1 RMSE | 26.41% | **26.22%** |
| p2 RMSE | 22.57% | **22.48%** |
| line_ranges 산출 | 26 paras visible | 20 paras visible + 6 skip ✅ |

**시각적 정합 (옵션 B 변경 시)**: 매우 우수 (사업개요 cell 의 paragraph 들이 PDF 와 동일 위치 + 분할), 그러나 폰트 폴백 차이 (Linux Noto Sans vs Hangul 맑은 고딕) 가 RMSE 의 대부분이라 측정값 개선 미미.

## 5. 잔존 결함 / 후속 작업

본 task 에서 완전 정합 불가능. 후속 이슈 분리 권고:

| 항목 | 설명 |
|---|---|
| paragraph y 시각 배치 vpos 정합 | 안전한 가드 필요 — split row 일반화 시 form-002 회귀. paragraph 사이 spacing 산출 방식 정합 작업이 본질. |
| 폰트 폴백 RMSE baseline | Linux 환경 폰트 차이 — 별 영역 |

## 6. Stage 3 결과 요약

- Stage 3-1 변경 (vpos 리셋 검출만, 좁힌 가드) 채택 — 안전, 회귀 없음
- Stage 3-2 광범위 정합 (vpos delta 보정 + paragraph y 보정) 보류 — 회귀 위험
- inner-table-01 line_ranges 산출은 PDF 정합과 일치 (20 visible + 6 skip)
- 시각 RMSE 개선은 미미 (폰트 폴백 baseline)

## 7. Stage 4 제안

본 변경의 영향 fixture 를 추가 검증 + 최종 보고서 작성 → 본 task 종결.

후속:
- 새 이슈 등록 — paragraph y 시각 배치 vpos 정합 (form-002 회귀 가드 포함)

---

승인 요청: Stage 4 (검증 + 최종 보고서) 진행 후 본 #697 task 종결, paragraph y 정합 후속 이슈 등록 으로 진행해도 되는지 확인 부탁드립니다.
