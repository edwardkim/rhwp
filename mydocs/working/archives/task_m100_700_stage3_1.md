# Task #700 Stage 3-1 보고서 — cum 절대 동기화 구현 결과

- 단계: Stage 3-1 (구현)
- 파일: `src/renderer/layout/table_layout.rs`
- 빌드: ✅ 통과
- 테스트: ✅ `cargo test --release` 21 그룹 0 failures (svg_snapshot 7 tests 포함)

## 1. 변경 적용

`compute_cell_line_ranges` paragraph 루프 진입부에 다음 분기 추가:

```rust
let cell_first_vpos = cell.paragraphs.first()
    .and_then(|p| p.line_segs.first().map(|s| s.vertical_pos))
    .unwrap_or(-1);

for (pi, (comp, para)) in composed_paras.iter().zip(cell.paragraphs.iter()).enumerate() {
    if pi > 0 && cell_first_vpos == 0 {
        let prev_para = &cell.paragraphs[pi - 1];
        let prev_end_vpos = prev_para.line_segs.last()
            .map(|s| s.vertical_pos + s.line_height)
            .unwrap_or(-1);
        let cur_first_vpos = para.line_segs.first().map(|s| s.vertical_pos).unwrap_or(-1);
        if cur_first_vpos >= 0 && prev_end_vpos > 0 {
            if cur_first_vpos < prev_end_vpos {
                // vpos 리셋 — page-break (Task #697)
                if has_limit && cum < abs_limit { cum = abs_limit; }
            } else {
                // 정상 누적 — vpos 절대 동기화
                let target_cum = hwpunit_to_px(cur_first_vpos, self.dpi);
                if target_cum > cum { cum = target_cum; }
            }
        }
    }
    // ... 기존 로직 ...
}
```

## 2. 시각 정합 검증

`samples/inner-table-01.hwp` p2 첫 줄:

| | 결과 |
|---|---|
| 변경 전 | `- 생성형 AI 기반 ...` (`p[18]`) — `p[17]` 누락 |
| 변경 후 | **`- 전사 데이터 수집/유통체계 구축`** (`p[17]`) ✓ |
| PDF 정합 | `p[17]` (vpos=27920) — **일치** |

## 3. RMSE 비교

| Fixture | Baseline (stream/devel) | Stage 3-1 |
|---|---|---|
| `inner-table-01.hwp` (2p) | 24.49% | **24.36%** (-0.13%) |
| `k-water-rfp.hwp` (18p 매핑) | 22.87% | 22.87% |
| `issue_265.hwp` (7p) | 22.00% | 22.00% |
| `hwp3-sample.hwp` (7p) | 22.00% | 22.00% |

→ inner-table-01 외 회귀 없음. baseline 22-23% 는 폰트 폴백 (Linux Noto Sans) noise.

## 4. 회귀 fixture 검증 ✅

| 검증 영역 | 결과 |
|---|---|
| `cargo test --release` 전체 (21 그룹) | 0 failures |
| `tests/svg_snapshot.rs` (form-002 포함, 7 tests) | ✅ pass |
| `renderer::layout::*` 104 tests | ✅ pass |
| `samples/hwpx/form-002.hwpx` p1 (PartialTable 26x27, cell[73] paras=29) | ✅ paragraph 누락 없음 |

핵심 가드의 효과:
- `cell_first_vpos == 0` — 한컴 정상 인코딩 케이스만 적용
- `target_cum > cum` — cum 만 전진 (감소 금지) — line metric > vpos 인 paragraph 영향 차단
- 차분 누적 (delta) 대신 절대 동기화 — form-002 회귀 가드 성공

## 5. 단위 테스트

합성 Cell/Paragraph fixture 구성이 매우 복잡 (LINE_SEG, ParaShape, ComposedParagraph 의존성). 본 변경의 정합성은 **통합 테스트 (svg_snapshot)** 와 회귀 fixture RMSE 비교로 충분히 검증 가능. 별도 단위 테스트는 추가하지 않음.

## 6. 잔존 결함

`사업개요` 라벨 정렬 — PR #701 의 결함 2 정정 (`cell_was_split` valign 가드) 이 본 task 의 base (stream/devel) 에 없음. PR #701 merge 후 자동 정합. 또는 PR #701 의 변경을 본 task 에 합치는 옵션도 있으나 별 PR 분리 유지 (중복 작업 회피).

## 7. Stage 4 진행

Stage 3-1 만으로 본 task 의 핵심 결함 (p[17] 누락) 정정 완료. 추가 sub-stage 불필요.

→ 바로 Stage 4 (최종 보고서) 진입.

---

승인 요청: Stage 4 진행 승인 부탁드립니다.
