# Stage 6 보고서 — Task #1082: 본문+미주 컬럼 누적 vpos-absolute 정합 — 성공

- 브랜치: `local/task1082`
- 수정: `src/renderer/typeset.rs` + 회귀 가드 `tests/issue_1082_*.rs`

## 구현 — Stage 5 의 두 문제 동시 해결
Stage 5 시도(vpos-delta)가 net 부정이었던 원인 정정:
1. **`.max(fmt.height_for_fit)` 안전 floor 누락** (#1062 회귀) → 복원.
2. **body→endnote 전환 시 base 불일치** → `TypesetState.prev_body_bottom_vpos` 추가, 본문
   FullParagraph 배치 시 갱신, `flush_column` 시 리셋. 미주 vpos-delta 누적의 초기 base 로 시드.

핵심 코드 (typeset.rs:1418~1530):
- `prev_en_bottom_vpos: Option<i32> = st.prev_body_bottom_vpos;` (body 후 시드)
- 각 미주 para: `compute_en_metrics(prev)` 클로저 — `advance_px = px(this.bottom_offset − prev_or_self_first)`, `.max(height_for_fit)` floor.
- fit check → 필요시 advance(prev_en 리셋) → 재평가 acc → push → prev_en 갱신.
- 단단(col_count==1)은 종전(`fmt.total_height` 누적) 유지.

## 검증
**C군 5파일 max overflow (baseline → Stage 6)**:
| 파일 | baseline | Stage 6 | 개선 |
|------|----------|---------|------|
| 3-09'23 hwp | 626.9 | **24.1** | -603 |
| 3-09'23 hwpx | 626.9 | **24.1** | -603 |
| 3-09'22 hwp | 277.0 | **25.7** | -251 |
| 3-10'22 hwp | 158.5 | **16.9** | -142 |
| 3-11'22 hwp | 561 | **8.9** | -552 |

잔여 ~25px = 본문 fmt 누적의 작은 드리프트(trailing_ls overcount) 잔영. 실용 한계 내.

**전수 sweep** (devel fbfcf682 기준):
- baseline 1156 lines / 46163px / 97파일 → **1024 / 17386 / 97파일** (-132 lines, **-62% px**)
- **신규 회귀 0, 악화 파일 0** (per-file diff 검증). 모든 변화 = 개선/불변.

**테스트**:
- 골든 SVG **8/8**, `cargo test` lib **1336** + 통합 0 failed.
- 회귀 가드 4 신규(`tests/issue_1082_*`, 4파일 doc_total_overflow ≤ 60px) 통과.
- clippy clean, fmt clean.

## 결론
C군(다단 미주 누적 vpos 간격 누락)을 본문+미주 통일 vpos-delta 누적으로 해소. PR 준비 완료.
