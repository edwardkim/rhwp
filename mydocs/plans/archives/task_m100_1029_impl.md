# Task #1029 구현 계획서

**Issue**: [#1029 HWP3 외곽선 paper-edge 정합 회귀](https://github.com/edwardkim/rhwp/issues/1029)
**Branch**: `local/task1029`
**Base**: `local/devel` (= `upstream/devel` = `65c8e693`)

---

## 1. 사전 단언 (Stage 1 시작 시 verify)

| 항목 | 단언 |
|------|------|
| `PageBorderBasis` enum 존재 | `src/model/page.rs:74` ✓ (확인됨) |
| `PageBorderFill::basis` 필드 존재 | `src/model/page.rs:69` ✓ (확인됨) |
| HWP3 parser `basis=PaperBased` 주입 | `src/parser/hwp3/mod.rs:2840` ✓ (확인됨) |
| HWP5 parser `basis=PaperBased` 주입 | `src/parser/body_text.rs:869` ✓ (확인됨) |
| HWPX parser `basis=PaperBased` 주입 | `src/parser/hwpx/section.rs:527` ✓ (확인됨) |

**결론**: parser 측 PR #1011 의 변경은 모두 살아있음. **회귀는 renderer (`layout.rs`) 한정** — fix 도 layout.rs 한정.

---

## 2. PR #1003 revert 의 net 영향 (단언)

PR #1003 (c2024ec9) merge 결과로 `src/renderer/layout.rs` 에서 다음 PR #1011 의 변경이 모두 revert 됨:

### 2.1 변경 A — `page_number_baseline_y()` (line ~947)

```rust
// 현재 (회귀):
let paper_based = (pbf.attr & 0x01) != 0;

// PR #1011 (복원 목표):
use crate::model::page::PageBorderBasis;
let paper_based = matches!(pbf.basis, PageBorderBasis::PaperBased);
```

### 2.2 변경 B — `build_page_borders()` paper_based 산출 (line ~976) + 주석 + 로그

```rust
// 현재 (회귀):
// [Task #987] #952 의 "sample16 = paper 정합" 은 ...
//   ... HWP3 는 파서가 attr=0(body) 주입 (CLAUDE.md HWP3 격리 규칙).
let paper_based = (pbf.attr & 0x01) != 0;
if std::env::var("RHWP_DEBUG_PAGE_BORDER").is_ok() {
    eprintln!(
        "PAGE_BORDER: attr=0x{:08x} bit0={} paper_based={} bfid={} ...",
        pbf.attr, pbf.attr & 0x01, paper_based, ...
    );
}

// PR #1011 (복원 목표):
// 정답 (#1006): 포맷별 분리 (PageBorderFill.basis) + 모두 PaperBased.
// 작업지시자 Hancom Office close-up 시각 판정: HWP3/HWP5/HWPX 모두
// logo 가 outline 내부 top-left 위치 → 세 포맷 모두 PaperBased contract.
// 또한 머리말 conditional clip 제거 (그림 이동 시 외곽선 shrink 회귀 해소),
// 꼬리말 clip 은 유지 (페이지 번호 외곽선 안쪽 회귀 해소 — PR #1011).
use crate::model::page::PageBorderBasis;
let paper_based = matches!(pbf.basis, PageBorderBasis::PaperBased);
let footer_inside = (pbf.attr & 0x04) != 0;
if std::env::var("RHWP_DEBUG_PAGE_BORDER").is_ok() {
    eprintln!(
        "PAGE_BORDER: attr=0x{:08x} bit0={} bit1={} bit2={} paper_based={} footer_inside={} bfid={} ...",
        pbf.attr, pbf.attr & 0x01, (pbf.attr >> 1) & 0x01, (pbf.attr >> 2) & 0x01,
        paper_based, footer_inside, ...
    );
}
```

### 2.3 변경 C — `footer_inside` clip 복원 (line ~1015)

```rust
// 현재 (회귀): clip 없음
let (bx, by, bw, bh) = if paper_based { ... } else { ... };
// (clip 블록 부재)
let borders = &bs.borders;

// PR #1011 (복원 목표):
let (bx, mut by, bw, mut bh) = if paper_based { ... } else { ... };
// [Task #1006 part 2] header_inside 는 clip 미적용 (cover logo
// 가 외곽선 내부 top-left 위치), footer_inside 만 clip 적용
// (페이지 번호가 외곽선 바깥 위치 — 한컴 viewer 정합 — PR #1011).
// attr bit 1 (header) 무시, bit 2 (footer) 존중.
if !footer_inside {
    let footer_top = layout.body_area.y + layout.body_area.height;
    if by + bh > footer_top {
        bh = footer_top - by;
    }
}
let _ = &mut by;
let borders = &bs.borders;
```

### 2.4 변경 D — 변수 mutability

`let (bx, by, bw, bh) = ...` → `let (bx, mut by, bw, mut bh) = ...`

(footer clip 의 `by`/`bh` 수정 위해 필요)

---

## 3. Stage 진행 (수행계획서의 3 단계 정밀화)

### Stage 1 — Fix 적용

**구체 작업**:
1. `src/renderer/layout.rs` 에서 4 항목 (변경 A/B/C/D) 모두 PR #1011 상태로 복원.
2. 추가 단언 (사전):
   - `cargo build --release` 컴파일 통과
   - `grep -n "paper_based\|basis" src/renderer/layout.rs` 로 복원 단언

**파일**: `src/renderer/layout.rs` (단일)

**커밋**: "Task #1029: HWP3 외곽선 paper-edge 정합 회귀 정정 — PR #1011 layout.rs 복원"

### Stage 2 — 검증

**Step 2.1 — 단일 포맷 단언** (`RHWP_DEBUG_PAGE_BORDER=1`):
```
HWP3 native   : paper_based=true (basis 기반) → border_top y=17.88
HWP5 변환본   : paper_based=true (basis 기반) → border_top y=17.88 (현재값 유지)
HWPX 변환본   : paper_based=true (basis 기반) → border_top y=17.88 (현재값 유지)
```

**Step 2.2 — SVG 시각 비교** (`rsvg-convert` 셀프):
```bash
for f in samples/hwp3-sample16{.hwp,-hwp5.hwp,-hwp5.hwpx}; do
  ./target/release/rhwp export-svg "$f" -p 0 -o /tmp/sweep/
done
```
- hwp3-sample16: 외곽선 paper-edge 복원 (border_top y=17.88 단언)
- HWP5/HWPX 변환본: 무변동 단언

**Step 2.3 — 회귀 sweep**:
- `cargo test --release --lib` (1307+ passed 단언)
- `cargo clippy --release -- -D warnings` (clean)
- `cargo fmt --check` (clean)
- 페이지 수 sweep: hwp3-sample10/11/13/14/16, exam_kor/eng/math, aift, biz_plan, hy-001 (PR #1003 의 Task #990 fixture)
- golden SVG snapshot diff 확인 (hwp3-sample16 외곽선 + footer clip 적용된 일부 fixture 만 — 의도)

**Step 2.4 — Task #990 효과 보존 단언**:
- `tests/issue_table_vpos_01_page5_cell_hit_test.rs` 13 passed
- hy-001 / aift 페이지 수 diff = 0 (PR #1003 의 의도된 변경 보존)

**커밋**: "Task #1029: 검증 + 회귀 sweep — paper-edge 복원, Task #990 보존"

### Stage 3 — 최종 보고 + PR

**문서 작업**:
1. `mydocs/working/task_m100_1029_stage1.md` — Stage 1 결과
2. `mydocs/working/task_m100_1029_stage2.md` — Stage 2 검증 결과
3. `mydocs/report/task_m100_1029_report.md` — 최종 보고서
4. `mydocs/orders/20260520.md` — Task #1029 status 갱신

**PR 생성** (작업지시자 승인 후):
- title: "Task #1029: HWP3 외곽선 paper-edge 정합 회귀 정정 (closes #1029)"
- base: devel
- body: 본 issue 요약 + Bisect 단언 + Fix 본질 + Test plan

---

## 4. 위험 분석 + 완화 (수행계획서 §6 정밀화)

| 위험 | 완화 (Stage) |
|------|-------------|
| `PageBorderBasis` import 위치/path 변경 | Stage 1 시작 시 `grep PageBorderBasis src/renderer/layout.rs` 로 import 형태 단언 |
| `footer_inside` clip 복원이 시험지 페이지 번호 위치 회귀 | Stage 2 step 2.3 sweep 시 exam_kor 페이지 번호 위치 시각 단언 |
| Task #990 의 의도된 ire 좌표 변경 (pi=34 30.84px) 가 본 변경으로 영향 | Stage 2 step 2.4 issue_table_vpos_01 hit_test 13 passed 단언 |
| golden SVG snapshot 회귀 (외곽선 외 다른 fixture) | Stage 2 step 2.3 snapshot diff 분류 — paper_based 변경 외 0 단언 |

---

## 5. 비대상 (수행계획서 §8 재확인)

- PR #1003 의 Task #990 본질 (treat-as-char advance 이중 가산 정정) — 보존
- HWP3/HWP5/HWPX parser 의 `basis` 필드 setter — 변경 없음 (이미 PR #1011 상태)
- WASM 빌드 — Stage 3 최종 1회

---

## 6. 검증 명령 모음 (참고)

```bash
# Build
cargo build --release

# 단일 페이지 단언
for f in samples/hwp3-sample16{.hwp,-hwp5.hwp,-hwp5.hwpx}; do
  echo "=== $f ==="
  RHWP_DEBUG_PAGE_BORDER=1 ./target/release/rhwp export-svg "$f" -p 0 -o /tmp/dbg/ 2>&1 | grep PAGE_BORDER
done

# 회귀 sweep
cargo test --release --lib 2>&1 | tail -10
cargo clippy --release --all-targets -- -D warnings 2>&1 | tail -10
cargo fmt --check 2>&1 | tail -5

# 페이지 수 sweep
for f in samples/hwp3-sample{10,11,13,14,16}.hwp samples/hwp3-sample16-hwp5.hwp samples/exam_{kor,eng,math}.hwp samples/aift.hwp samples/biz_plan.hwp; do
  echo -n "$f: "
  ./target/release/rhwp dump-pages "$f" 2>&1 | grep -c "^=== 페이지"
done
```
