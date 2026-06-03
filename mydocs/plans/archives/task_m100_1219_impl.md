# 구현 계획서 — Task #1219

**제목**: 수식 포함 줄 본문 한글 압축 — 측정 루프 TAC 소스를 줄-경계 규칙(`line_tac_offsets`)으로 통일
**이슈**: [#1219](https://github.com/edwardkim/rhwp/issues/1219)
**브랜치**: `local/task1219`
**수행계획서**: `mydocs/plans/task_m100_1219.md` (승인됨)
**작성일**: 2026-06-01

---

## 설계 결론

`src/renderer/layout/paragraph_layout.rs`에는 **줄별 TAC 배정의 정규 함수**가 이미 존재한다.

```rust
fn composed_line_char_end(comp, line_idx) -> usize   // 다음 줄 char_start (= 줄 끝, exclusive)
fn char_pos_in_line(pos, start, end) -> bool         // end>start 면 pos>=start && pos<end (엄격 미만)
fn tac_offsets_for_line(comp, tac_offsets_px, line_idx) -> Vec<...>  // 위 규칙으로 줄별 TAC 필터
```

렌더 경로는 `let line_tac_offsets = tac_offsets_for_line(...)` (라인 1429)로 줄별 TAC를 얻어
그리기 때문에 **줄 끝 위치(`pos == 다음 줄 시작`)의 수식은 다음 줄에 배정**된다.

그러나 폭 **측정 경로**는 같은 줄 루프 안에서 전역 `tac_offsets_px`를 자체 필터로 재해석한다:

- **(A)** `est_x` 측정 루프 (라인 1784-1788): run 범위 + `is_last_run_est_tac && pos == run_char_end_est`
  → 줄 끝 == 다음 줄 선두 수식을 **현재 줄에 포함** (문26 라인0에 `a₁=b₁=1` 55px 오포함 → 주 원인).
- **(B)** `total_tac_width_in_line` (라인 1905-1918): `pos >= line_start && pos <= line_end`
  → `<= line_end` 로 경계 수식 포함 (동일 결함, 현재 fallback 미발동이나 잠재 버그).

**수정 방침**: (A)(B)의 TAC 소스를 이미 계산된 **`line_tac_offsets`** 로 교체하여
측정과 렌더가 동일한 줄-경계 규칙을 공유하도록 한다. 줄 끝 경계 수식 오포함이 구조적으로 사라진다.

---

## 단계별 구현 (3단계)

### Stage 1 — 측정 루프 TAC 소스 통일 + 1차 시각 검증

**대상**: `src/renderer/layout/paragraph_layout.rs`

1. **(A) est_x 측정 루프 (1784)**: 필터 소스를 `tac_offsets_px` → `line_tac_offsets`로 교체.
   - `line_tac_offsets`는 이미 줄-경계로 필터링됨 → run 범위 필터(`pos >= run_char_pos_est
     && pos < run_char_end_est`)만 유지, 경계 포함 클러스 `is_last_run_est_tac && pos == end`는
     무해해지나 의미 명확화를 위해 정리 검토 (마지막 run 내 줄-끝 TAC는 `line_tac_offsets`에
     이미 포함/제외 결정됨).
2. **(B) total_tac_width_in_line (1905)**: 소스를 `line_tac_offsets`로 교체.
   `line_tac_offsets`가 줄-범위 집합이므로 폭 합산만 수행 (경계 조건 `<= line_end` 제거).
3. `cargo build --release`.
4. `export-svg -p 5` → 문26 첫 줄 한글 advance ≈ 12px (겹침 해소) 확인.
   PDF(`pdf/3-09월_교육_통합_2023.pdf` 6쪽) 시각 비교.
5. 동일 페이지 문24/27/30 동반 개선 여부 측정(advance 비율 ≈ 1.0).

→ **Stage 1 완료보고서** (`working/task_m100_1219_stage1.md`) + 소스 커밋.

### Stage 2 — 회귀 검증 (전체 테스트 + 경계 케이스)

1. `cargo test` 전체 통과 확인 (특히 인라인 TAC / auto_tab_right / 목차 / 우측탭 관련).
2. 경계 케이스 시각 회귀 점검:
   - **마지막 줄 끝에 수식이 오는 문서** (경계 포함 규칙이 도입된 원래 의도) — 회귀 없음 확인.
   - 인라인 그림/도형(TAC) 포함 문단, 셀 내부 TAC.
3. 회귀 발견 시 원인 분석 후 보정 (필요 시 `_v2` 또는 Stage 추가).

→ **Stage 2 완료보고서** (`working/task_m100_1219_stage2.md`) + (수정 있으면) 커밋.

### Stage 3 — 회귀 테스트 추가 + 최종 보고서

1. 회귀 테스트 추가: 문26 문단(또는 축약 픽스처)에서 수식 줄 본문 한글 advance가
   압축되지 않음(≈ font_size)을 검증하는 단위/통합 테스트.
   - 기존 테스트 위치 관례(`src/renderer/layout/` 내 tests, 또는 golden SVG) 따름.
2. 변경 파일 한정 `rustfmt` 정리 (전체 fmt 금지).
3. **최종 보고서** (`report/task_m100_1219_report.md`) 작성 + 오늘할일 갱신은
   별도 지시 없으면 생략(`orders/` 변경 금지 규칙) — 보고서/테스트 커밋.

---

## 변경 파일 (예상)

| 파일 | 변경 |
|------|------|
| `src/renderer/layout/paragraph_layout.rs` | 측정 루프 (A)(B) TAC 소스를 `line_tac_offsets`로 통일 |
| `src/renderer/layout/` (테스트) | 회귀 테스트 추가 (Stage 3) |
| `mydocs/working/task_m100_1219_stage{1,2}.md` | 단계 완료보고서 |
| `mydocs/report/task_m100_1219_report.md` | 최종 보고서 |

## 검증 기준 (완료 정의)

- [ ] 문26 첫 줄 한글 advance ≈ 12px, 시각 겹침 해소 (PDF 정합)
- [ ] 문24/27/30 동반 정상화
- [ ] `cargo test` 전체 통과 (회귀 0)
- [ ] 마지막 줄 끝 수식 케이스 회귀 없음
- [ ] 회귀 테스트 추가

## 리스크 / 완화

- **경계 포함 규칙의 원래 의도(마지막 줄 끝 TAC 포함)** 회귀 → `line_tac_offsets`가
  `composed_line_char_end`의 마지막-줄 분기(`+ has_line_break`)로 이미 해당 케이스를 포함하므로,
  렌더와 동일 동작 보장. Stage 2에서 명시적 회귀 케이스로 검증.
- 측정/렌더 통일로 다른 문단 폭이 미세 변동 가능 → `cargo test` golden 비교로 포착.

---

**승인 요청**: 위 구현 계획서 승인 시 Stage 1부터 진행합니다.
