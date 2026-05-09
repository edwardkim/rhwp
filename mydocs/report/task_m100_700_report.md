# Task #700 최종 결과 보고서

셀 paragraph y 시각 배치 vpos 정합 — Task #697 후속 (form-002 회귀 가드 필요)

- 이슈: [#700](https://github.com/edwardkim/rhwp/issues/700)
- 마일스톤: v1.0.0 (M100)
- 브랜치: `local/task700` (← `stream/devel @ 2fe386c4`)
- 상태: **완료**

## 1. 결함 요약

`samples/inner-table-01.hwp` cell[11] (사업개요, 26 paras) 의 cell-internal split 처리에서:
- p2 첫 줄에 `- 전사 데이터 수집/유통체계 구축` (`p[17]`) 가 누락 (rhwp 가 `p[18]` 부터 표시)
- 원인: `compute_cell_line_ranges` 의 `cum` 누적 metric (`line_height + line_spacing + spacing_before/after`) 이 한컴 LINE_SEG.vpos 누적과 ~50px 어긋남 → `abs_limit` (한컴 vpos 단위) 와 비교 시 더 많은 paragraph 가 visible 처리됨

## 2. 정정 내용

### 2.1 핵심 변경

`src/renderer/layout/table_layout.rs::compute_cell_line_ranges` — paragraph 진입 시 `cum` 을 LINE_SEG.vpos 절대값으로 동기화:

```rust
if pi > 0 && cell_first_vpos == 0 {
    if cur_first_vpos < prev_end_vpos {
        // vpos 리셋 — page-break 신호
        if has_limit && cum < abs_limit { cum = abs_limit; }
    } else {
        // 정상 누적 — vpos 절대 동기화 (전진만)
        let target_cum = hwpunit_to_px(cur_first_vpos, self.dpi);
        if target_cum > cum { cum = target_cum; }
    }
}
```

### 2.2 핵심 가드

- **`cell_first_vpos == 0`** — 한컴 정상 인코딩 케이스만 적용 (다른 케이스 회피, 회귀 방지)
- **`target_cum > cum`** — cum 만 전진 (감소 금지) — line metric 가 vpos 보다 큰 paragraph 영향 차단
- **차분 누적 대신 절대 동기화** — Task #697 Stage 3-2 시도의 form-002 회귀 (paragraph 사이 spacing mismatch 누적) 회피

### 2.3 Task #697 vpos 리셋 검출 통합

본 task 의 base 가 `stream/devel` 이라 Task #697 의 vpos 리셋 검출 변경이 없는 상태. 본 변경은 Task #697 의 vpos 리셋 검출을 함께 포함 (옵션 C).

## 3. 시각 정합 검증

`samples/inner-table-01.hwp`:

| 페이지 | 변경 전 | 변경 후 | PDF 정합 |
|---|---|---|---|
| p1 cell[11] 마지막 visible | `p[19]` 끝 (`SaaS 갤러리...`) | **`p[16]` 끝 (`- OA망 내 계측 데이터 ...`)** | ✓ p[16] 끝 |
| p2 cell[11] 첫 visible | `p[18]` (`- 생성형 AI 기반...`) | **`p[17]` (`- 전사 데이터 수집/유통체계 구축`)** | ✓ p[17] |

## 4. RMSE 비교

| Fixture | Baseline | 변경 후 | Δ |
|---|---|---|---|
| `inner-table-01.hwp` (2p) | 24.49% | **24.36%** | -0.13% |
| `k-water-rfp.hwp` (18p) | 22.87% | 22.87% | ±0 |
| `issue_265.hwp` (7p) | 22.00% | 22.00% | ±0 |
| `hwp3-sample.hwp` (7p) | 22.00% | 22.00% | ±0 |

→ inner-table-01 외 회귀 없음. RMSE 22-23% baseline 은 폰트 폴백 (Linux Noto Sans vs 한컴 맑은 고딕) noise — 시각 정합과 무관.

## 5. 회귀 검증

| 검증 영역 | 결과 |
|---|---|
| `cargo test --release` 전체 (21 그룹) | ✅ 0 failures |
| `tests/svg_snapshot.rs` (form-002 포함, 7 tests) | ✅ pass |
| `renderer::layout::*` 104 tests | ✅ pass |
| `samples/hwpx/form-002.hwpx` p1 (cell[73] paras=29, vpos 리셋 1회) | ✅ paragraph 누락 없음 |

## 6. 진행 단계 결과

| 단계 | 산출물 | 상태 |
|---|---|---|
| Stage 1 | `mydocs/working/task_m100_700_stage1.md` (정밀 진단) | ✅ |
| Stage 2 | `mydocs/plans/task_m100_700_impl.md` (옵션 C 채택) | ✅ |
| Stage 3-1 | `mydocs/working/task_m100_700_stage3_1.md` (구현) | ✅ |
| Stage 4 | 본 보고서 | ✅ |

## 7. 커밋 이력 (`local/task700`)

| 커밋 | 내용 |
|---|---|
| Stage 1: 수행 계획서 + 정밀 진단 보고서 |
| Stage 2: 구현 계획서 (옵션 C) |
| Stage 3-1: cum 절대 동기화 구현 |
| Stage 4: 최종 보고서 (본 문서) |

## 8. 잔존 작업

- `사업개요` 라벨 정렬 (cell_was_split valign 가드) — PR #701 (Task #697) 에 정정 포함. PR #701 merge 후 자동 정합.
- pagination engine 의 `split_end_content_limit` 산출 정합화 (line metric → vpos) — 별 task 로 분리 가능 (현재 결함 영향 미미)
- 폰트 폴백 RMSE baseline (22-23%) — Linux 환경 한컴 호환 폰트 부재. 별 영역.

## 9. 결론

본 task 의 핵심 결함 (cell-internal split 시 paragraph cut 위치 mismatch) 정정 완료. cum 절대 동기화 + `cell_first_vpos == 0` 가드로 form-002 등 다양한 표 fixture 에서 회귀 없이 안전하게 적용.

**회귀 없음**, **시각 정합 정확**, **변경 영역 좁음** (한 함수 내 분기 추가).

---

승인 요청: 본 최종 보고서 검토 후 issue #700 close + PR 생성 진행해도 되는지 확인 부탁드립니다.
