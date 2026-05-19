# PR #989 검토 — HWPX preset lineSegArray line_spacing double-count 해소

## 1. PR 정보

| 항목 | 값 |
|------|-----|
| 번호 | #989 |
| 제목 | fix: HWPX preset lineSegArray line_spacing double-count 해소 — sample16-hwp5 +1 페이지 감소 |
| 작성자 | jangster77 (Taesup Jang) — 기존 컨트리뷰터 |
| base ← head | `devel` ← `local/task969` |
| 연결 이슈 | closes #969, partial #942 (잔존 +7 → #988 분리), assignee 본인 지정 |
| mergeable | MERGEABLE / BEHIND (cherry-pick 으로 해소) |
| CI | Build & Test ✅ / Analyze ✅ / Canvas diff ✅ / CodeQL ✅ |
| 커밋 | 2 (912e945c fix + 42af7aea devel merge) |

## 2. 배경 (이슈 #969)

`samples/hwp3-sample16-hwp5.hwpx` HWPX 파서가 `<hp:linesegarray>`
preset 을 IR LineSeg 로 emit. `format_paragraph` composed branch 에서:

1. `raw_lh = 17.32px` (vertsize 1299 HU = 13pt font)
2. `max_fs = 17.33px` → `raw_lh < max_fs` (1 HU 미만 차) → recompute
3. `lh = max_fs * 1.6 = 27.7px` (ls 60% 흡수)
4. **그러나** `line_spacing = 10.4px` 별도 가산 → **220% double-count**

HWP5(preset 없음)는 composer 가 `line_spacing=0` → double-count
없음. HWPX 만 inflate → 불일치. sample16-hwp5 +8 페이지 중 1개 원인.

## 3. 변경 내용

**`src/renderer/typeset.rs` 단일 코드 파일** (composed branch) + 문서 8.

```rust
let recompute_lh = max_fs > 0.0 && raw_lh < max_fs;
let lh = if recompute_lh { /* ls_type 기반 재계산, ls 흡수 */ } else { raw_lh };
// [Task #969] recompute 시 line_spacing 은 이미 lh 에 흡수 → 별도 가산 금지
let line_spacing_px = if recompute_lh { 0.0 } else {
    hwpunit_to_px(line.line_spacing, self.dpi)
};
```

## 4. 검토 의견

### 4.1 트러블슈팅 사전 검색 (feedback_search_troubleshootings_first)

관련 문서 2건 확인:
- `line_spacing_lineseg_sync.md`: ParaShape ↔ LineSeg 이중 저장,
  compose_lines 가 LineSeg 값 그대로 사용 — 본 PR 이 다루는 구조.
- `hwpx_lineseg_reflow_trap.md`: HWPX lineSegArray vertsize/spacing
  처리 함정 — 동일 영역.

→ PR 의 진단(preset ls 가 recompute 된 lh 에 이미 흡수)이 이
구조적 함정과 정합. 과거 함정 재현 아닌 해소 방향.

### 4.2 강점

- **root cause 정밀**: HWPX preset(linesegArray) vs HWP5(동적
  생성) 의 line_spacing 처리 비대칭을 정확히 짚음. recompute 시
  ls_type 기반 재계산이 60% extra 를 이미 lh 에 흡수하므로
  line_spacing 별도 가산은 명백한 double-count.
- **격리 정확**: `recompute_lh == false` (HWP5 경로 등) 는 기존
  `hwpunit_to_px(line.line_spacing)` 그대로 — 동작 불변. recompute
  분기에서만 0 → HWPX preset 한정 수정, HWP5 회귀 면 없음.
- 페이지 수: sample16-hwp5.hwpx 72→71, hwpx-02.hwpx 6→5
  (side effect, PDF 권위 없음 명시). 골든 SVG 변동 없음.
- `cargo test` 전체 + `cargo fmt --check` 통과 (PR 본문).
- 잔존 +7 페이지를 #988 로 정직하게 분리 — 과대 주장 없음.

### 4.3 검토 포인트

- **페이지 수 변동 = 시각 본질** (feedback_visual_regression_grows):
  페이지 수 비교만으로 회귀 검출 불가. sample16-hwp5 +
  hwpx-02 의 시각 회귀 없음을 작업지시자 시각 판정으로 확인 필요.
- typeset.rs composed branch 는 렌더링 핵심. 회귀 가드(svg_snapshot
  골든, 전체 test) 통과 확인이 수용 전제.
- side effect(hwpx-02 6→5)는 PDF 권위 없는 샘플 — 시각 판정 시
  함께 확인 권고.

## 5. 검증 결과 (cherry-pick `631cf977`)

| 항목 | 결과 |
|------|------|
| cherry-pick | ✅ 충돌 없음 (orders append, 코드 무충돌) |
| `cargo build --release` | ✅ Finished |
| 전체 `cargo test` | ✅ 1487 passed, 0 failed (골든 svg_snapshot 포함) |
| `cargo clippy -- -D warnings` | ✅ 0 warnings |
| `cargo fmt --all -- --check` | ✅ 위반 0건 |
| WASM 빌드 (Docker) | ✅ 성공 (typeset core WASM 호환) |
| sample16-hwp5.hwpx 페이지 수 | ✅ **71** (PR 주장 72→71 정확) |
| hwpx-02.hwpx 페이지 수 | ✅ **5** (PR 주장 6→5 side effect 정확) |

산출물: `output/poc/pr989/sample16-hwp5/` (71 SVG),
`output/poc/pr989/hwpx-02/` (5 SVG).
정답지: `pdf/hwp3-sample16-hwp5-2022.pdf` (참고). hwpx-02 PDF 권위 없음.

- [ ] **작업지시자 시각 판정**: sample16-hwp5 + hwpx-02 시각 회귀
      없음 (페이지 수 변동 동반 — 시각 게이트 필수)

## 6. 판단 (잠정)

root cause 정밀 + HWPX preset 한정 격리(HWP5 무영향) + 트러블슈팅
함정과 정합. 단 **페이지 수 변동 동반이라 시각 판정이 핵심 게이트**.
검증 + 시각 판정 통과 시 수용 권고.

검증 결과에 따라 `pr_989_report.md` 작성.
