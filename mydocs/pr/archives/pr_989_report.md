# PR #989 최종 보고 — HWPX preset lineSegArray line_spacing double-count 해소

## 1. 결정

**merge** — root cause 정밀 + 전 검증 + 시각 판정 통과. HWPX 한정 격리.

| 항목 | 값 |
|------|-----|
| 번호 | #989 |
| 제목 | fix: HWPX preset lineSegArray line_spacing double-count 해소 |
| 작성자 | jangster77 (Taesup Jang) — 기존 컨트리뷰터 |
| base ← head | `devel` ← `local/task969` |
| 연결 이슈 | closes #969, partial #942 (잔존 +7 → #988 분리) |
| 처리 | cherry-pick (`912e945c` → 최신 local/devel) |

## 2. 검증 결과

cherry-pick `631cf977`. 충돌 없음 (orders append, 코드 무충돌).

| 항목 | 결과 |
|------|------|
| `cargo build --release` | ✅ Finished |
| 전체 `cargo test` | ✅ 1487 passed, 0 failed (골든 svg_snapshot 포함) |
| `cargo clippy -- -D warnings` | ✅ 0 warnings |
| `cargo fmt --all -- --check` | ✅ 위반 0건 |
| WASM 빌드 (Docker) | ✅ 성공 (typeset core WASM 호환) |
| sample16-hwp5.hwpx 페이지 수 | ✅ 71 (PR 주장 72→71 정확) |
| hwpx-02.hwpx 페이지 수 | ✅ 5 (PR 주장 6→5 side effect 정확) |
| **작업지시자 시각 판정** | ✅ **통과** (sample16-hwp5 + hwpx-02 회귀 없음) |
| CI | ✅ 전부 pass |

## 3. 평가 요약

### 강점
- **root cause 정밀**: HWPX preset(`<hp:linesegarray>`)의
  vertsize(font)+spacing(extra)이 ParaShape `line=160%`와 결합 시,
  `format_paragraph` composed branch 의 recompute 가 ls 60% 를 lh 에
  흡수하는데 `line_spacing` 별도 가산 → 220% double-count.
  HWP5(preset 없음)는 composer 가 line_spacing=0 → 비대칭. 정확히 짚음.
- **격리 정확**: `recompute_lh == false`(HWP5 등)는 기존
  `hwpunit_to_px(line.line_spacing)` 그대로 — 동작 불변. recompute
  분기에서만 0 → HWPX preset 한정, HWP5 회귀 면 없음.
- 골든 svg_snapshot 변동 없이 페이지 수만 정확히 감소
  (sample16-hwp5 72→71, hwpx-02 6→5 side effect).
- 잔존 +7 페이지를 #988 로 정직하게 분리 — 과대 주장 없음.

### 트러블슈팅 정합 (feedback_search_troubleshootings_first)
`line_spacing_lineseg_sync.md`, `hwpx_lineseg_reflow_trap.md` 2건
확인. PR 진단이 ParaShape↔LineSeg 이중 저장 / HWPX lineSegArray
처리 함정과 정합 — 과거 함정 재현 아닌 해소 방향.

### 시각 판정 (feedback_visual_regression_grows)
페이지 수 변동 동반이라 페이지 수 비교만으로 회귀 검출 불가.
작업지시자 시각 판정으로 sample16-hwp5 + hwpx-02 회귀 없음 확인.

## 4. 처리

- cherry-pick → 검증 + 시각 판정 통과 → `local/devel` merge
- PR #989 close (cherry-pick 반영 명시) + 이슈 #969 close
- `pr_989_review.md` / `pr_989_report.md` → `pr/archives/`
- 잔존 +7 페이지 후속: #988 (별도 task)
