# Task #1029 최종 결과 보고서

**Issue**: [#1029 HWP3 외곽선 paper-edge 정합 회귀 (PR #1003 cherry-pick `--theirs` 충돌 해소 사고)](https://github.com/edwardkim/regression-rhwp/issues/1029)
**Branch**: `local/task1029`
**Milestone**: v1.0.0
**관련**: PR #1011 (closes #1006) 회귀 정정, PR #1003 (closes #990) 부수효과

---

## 1. 결과 요약

PR #1003 의 cherry-pick `--theirs` 자동 충돌 해소가 PR #1011 의 `PageBorderBasis` 통합 contract 를 PR #987 시절 `attr & 0x01` 비트 로직으로 무심코 revert 했던 시각 회귀를 정정. HWP3 native 의 외곽선이 paper-edge 로 복원되어 PR #1011 baseline 과 byte-for-byte 정합.

**변경 범위**: `src/renderer/layout.rs` 단일 파일 4 hunk (parser/model 무수정, 시그니처 무변경).

---

## 2. 회귀 메커니즘 (단언)

### 2.1 Bisect 결과 (border_top y)

| Merge | PR | border_top y | 상태 |
|-------|-----|--------------|------|
| 850cfb54 | PR #1011 | 17.88 | ✓ paper-edge (PR #1011 baseline) |
| 71aedda9 | PR #1015 | 17.88 | ✓ 무관 |
| 84246b2a | PR #1018 | 17.88 | ✓ 무관 |
| 27c05d53 | PR #1020 | 17.88 | ✓ 무관 |
| b5d38346 | PR #1021 | 17.88 | ✓ 무관 |
| **c2024ec9** | **PR #1003 (Task #990)** | **55.64** | **❌ 회귀 도입** |
| 77a25471 | PR #1004 | 55.64 | (회귀 유지) |
| 65c8e693 | devel HEAD | 55.64 | (회귀 유지) |

작업지시자의 초기 추정 (PR #1015 가 회귀 도입) 은 bisect 결과 PR #1003 으로 정정.

### 2.2 Root cause

PR #1003 의 merge commit (c2024ec9) 본문에 명시:
> "옵션 A: 3 본질 커밋 cherry-pick (**d7663dd4 Stage 2 layout.rs PR #1005 영역 `--theirs` 충돌 해소** ...)"

`d7663dd4` (Task #990 Stage 2) cherry-pick 시 PR #1011 영역과 충돌. `--theirs` 자동 해소로 PR #1011 의 `PageBorderBasis` 기반 로직이 PR #987 시절 `attr & 0x01` 비트 로직으로 revert. PR #1003 merge 시점에 sweep 로그에는 "sample16-hwp5/hwp3 외곽선만 (텍스트 무변동)" 으로 기록되었으나 비공개 샘플 시각 판정 생략 수용으로 시각 회귀가 인지되지 못했음.

### 2.3 포맷별 attr 단언 (RHWP_DEBUG_PAGE_BORDER=1)

| 포맷 | attr | bit0 | basis | paper_based (회귀 로직) | 외곽선 |
|------|------|------|-------|-------------------------|--------|
| HWP3 native | 0x00000000 | 0 | PaperBased | **false** (revert 로직) | **body-edge (회귀)** |
| HWP5 변환본 | 0x00000001 | 1 | PaperBased | true | paper-edge (OK) |
| HWPX 변환본 | 0x00000041 | 1 | PaperBased | true | paper-edge (OK) |

PR #1011 의 `basis` 필드 접근은 모든 포맷 parser 가 `PaperBased` 로 주입했기 때문에 attr 비트 0 과 무관했으나, revert 로 HWP3 native (attr=0) 만 body-edge 로 분기.

---

## 3. Fix 본질

`src/renderer/layout.rs` 4 hunk 복원 (parser 측 `basis=PaperBased` 주입은 이미 보존됨):

| Hunk | 위치 | 변경 |
|------|------|------|
| A | `page_number_baseline_y()` (line ~947) | `(pbf.attr & 0x01) != 0` → `matches!(pbf.basis, PageBorderBasis::PaperBased)` |
| B | `build_page_borders()` (line ~976) | A 와 동일 + 주석 history 정정 + 디버그 로그 포맷 (`bit1`/`bit2`/`footer_inside` 추가) |
| C | line ~1015 | `footer_inside` clip 블록 복원 — 페이지 번호 외곽선 바깥 위치 정합 |
| D | line ~1001 | `(bx, by, bw, bh)` → `(bx, mut by, bw, mut bh)` — footer clip 의 `by`/`bh` 수정 |

**변경량**: `1 file changed, 31 insertions(+), 12 deletions(-)`

---

## 4. 검증

### 4.1 단일 페이지 단언 (Stage 1)

```
HWP3 native   : paper_based=true (basis 기반) → border_top y=17.88 ✓ 복원
HWP5 변환본   : paper_based=true (basis 기반) → border_top y=17.88 ✓ 유지
HWPX 변환본   : paper_based=true (basis 기반) → border_top y=17.88 ✓ 유지
```

### 4.2 회귀 sweep (Stage 2)

| 항목 | 결과 |
|------|------|
| `cargo build --release` | ✓ warning 0 |
| `cargo clippy --release --lib -- -D warnings` | ✓ clean |
| `cargo fmt --check` | ✓ clean |
| `cargo test --release --lib` | ✓ 1307 passed; 0 failed |
| `cargo test --release --tests` (전체 integration) | ✓ 68 passed; 0 failed |
| `issue_table_vpos_01_page5_cell_hit_test` (Task #990 보존) | ✓ 13 passed |

### 4.3 페이지 수 sweep (12 fixture)

| Sample | 페이지 | 비고 |
|--------|--------|------|
| hwp3-sample10 | 763 | ✓ |
| hwp3-sample11 | 151 | ✓ |
| hwp3-sample13 | 3 | ✓ |
| hwp3-sample14 | 11 | ✓ |
| hwp3-sample16 | 64 | ✓ |
| hwp3-sample16-hwp5 (.hwp / .hwpx) | 64 / 71 | ✓ |
| exam_kor / exam_eng / exam_math | 20 / 8 / 20 | ✓ |
| aift | 74 | ✓ (Task #990 의도된 효과 보존) |
| biz_plan | 6 | ✓ |

### 4.4 시각 판정

작업지시자 한컴 viewer 비교 시각 검증 통과 — HWP3 cover paper-edge 정합 복원 확인.

---

## 5. 성공 기준 충족

| 조건 | 기준 | 결과 |
|------|------|------|
| C1 | HWP3 sample16 border_top y = 17.88 (paper-edge 복원) | ✓ |
| C2 | HWP5 변환본 border_top y = 17.88 유지 | ✓ |
| C3 | HWPX 변환본 border_top y = 17.88 유지 | ✓ |
| C4 | PR #1003 Task #990 효과 보존 | ✓ |
| C5 | 시험지 회귀 0 | ✓ |
| C6 | `cargo test --release --lib` 1307+ passed | ✓ |
| C7 | 작업지시자 한컴 viewer 시각 검증 | ✓ |

---

## 6. 본 task 의 교훈 (재발 방지)

본 회귀의 본질은 cherry-pick `--theirs` 자동 충돌 해소가 PR 간 source-of-truth 우선순위 판단을 우회한 것. 특히 `src/renderer/layout.rs` 같은 high-impact 파일에서 자동 해소는 시각 회귀를 야기할 수 있음.

권고:
- `--theirs`/`--ours` 자동 해소 후 영향 받은 hunk 의 line-by-line 검토
- 시각 회귀 sweep 을 비공개 샘플 외에 **공개 fixture** (예: `hwp3-sample16`) 로 사전 단언 의무화
- PR merge 직전에 `RHWP_DEBUG_PAGE_BORDER=1` 같은 진단 hook 으로 외곽선 좌표 baseline diff 자동 단언

별도 `mydocs/troubleshootings/` 문서로 정리 가능 (선택, 본 task 범위 외).

---

## 7. 커밋 history

| 커밋 | 단계 |
|------|------|
| `944ddff2` | Stage 1: layout.rs 4 hunk 복원 + 수행/구현 계획서 + Stage 1 보고서 |
| `9bd851f4` | Stage 2: 회귀 sweep 검증 통과 보고서 |
| (Stage 3 commit) | 최종 보고서 + orders 갱신 |

---

## 8. 본 task 범위 외 (PR 후속)

- PR #1003 의 Task #990 본질 (treat-as-char advance 이중 가산 정정) — 보존
- HWPX 변환본 페이지 inflate (별도 issue #895, #1000)
- WASM 빌드 — PR merge 후 별도 ops

---

closes #1029
