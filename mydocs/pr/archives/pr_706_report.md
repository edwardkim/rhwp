---
PR: #706
제목: Task #700 — 셀 paragraph cut 위치 vpos 정합 (compute_cell_line_ranges cum 절대 동기화)
컨트리뷰터: @planet6897 (Jaeuk Ryu) — 30+ 사이클 핵심 컨트리뷰터
처리: 옵션 A — 5 commits 단계별 보존 cherry-pick + golden SVG 갱신 + no-ff merge
처리일: 2026-05-09
머지 commit: fce2d870
---

# PR #706 처리 보고서

## 1. 처리 결과

✅ **머지 완료** — 옵션 A (5 commits cherry-pick + golden SVG 갱신 + no-ff merge `fce2d870`)

| 항목 | 값 |
|------|-----|
| 머지 commit | `fce2d870` (--no-ff merge) |
| Issue #700 / #697 | close 자동 정합 (closes #700, closes #697) |
| 시각 판정 | ★ **통과 (작업지시자 직접, 정답지 정합 확정)** |
| 자기 검증 | lib 1166 + 통합 ALL GREEN + svg_snapshot 8/8 (form-002 갱신 후 통과) + clippy clean |

## 2. 정정 본질

### 2.1 Issue #700 — cum 누적 metric 부정합

`samples/inner-table-01.hwp` cell[11] (사업개요, 26 paras) cell-internal split 시:
- cum 누적 metric (line_height + line_spacing + spacing) 이 한컴 LINE_SEG.vpos 누적과 ~50px 어긋남
- abs_limit (한컴 vpos 단위) 와 비교 시 paragraph cut 위치 부정합
- p[17] `- 전사 데이터 수집/유통체계 구축` 누락 → rhwp 가 p[18] 부터 표시

### 2.2 정정 (`compute_cell_line_ranges`)

**Task #697 영역** (vpos 리셋 검출):
```rust
if cur_first_vpos < prev_end_vpos {
    // page-break 신호 — cum 을 abs_limit 까지 강제 진행
    if has_limit && cum < abs_limit { cum = abs_limit; }
}
```

**Task #700 영역** (정상 누적 시 절대 동기화):
```rust
} else {
    let target_cum = hwpunit_to_px(cur_first_vpos, self.dpi);
    if target_cum > cum { cum = target_cum; }   // 전진 보장, 감소 금지
}
```

**가드**:
- `cell_first_vpos == 0` — 한컴 정상 인코딩 케이스만
- `target_cum > cum` — 전진만 허용
- 차분 누적 (delta) 대신 절대 동기화 — paragraph 사이 spacing mismatch 누적 방지

### 2.3 Task #697 후속 (table_partial.rs)

split row 미분할 cell 의 valign 보존:
- `is_in_split_row` → `is_in_split_row && cell_was_split` 가드 좁힘
- inner-table-01 cell[10] '사업개요' 라벨 중앙 정렬 정합

## 3. PR supersede 영역

PR #701 (Task #697) close → PR #706 (Task #697 + #700 통합) supersede.
컨트리뷰터 자체 결정 (PR #701 close 댓글 명시).

## 4. 작업지시자 시각 판정 ★ 통과

### 4.1 시각 판정 결과 (2026-05-09)

| 영역 | BEFORE (devel) | AFTER (PR #706) |
|------|----------------|-----------------|
| `samples/hwpx/form-002.hwpx` page 0 | 26 글자 ("ㅇPFC 나노산소운반체의 최적제조공정개발 및 GMP실증") 표시 | **26 글자 누락** ★ |
| 한컴 PDF 권위본 (`pdf/hwpx/form-002-2022.pdf` page 1) | — | **AFTER 정합** |
| `samples/inner-table-01.hwp` p2 cell[11] | p[18] 부터 시작 | **p[17] 부터 정상 시작** ★ |

→ **PR #706 이 한컴 정답지 정합** 확정 (작업지시자 직접 시각 판정).

### 4.2 부수 사실 — PR #662 (Task #656) 의 form-002 정정 본질 부정합

PR #662 본문 명시 영역의 "form-002 page 0 마지막 visible 줄 26 글자 클립 해소" 가 **사실은 한컴 권위와 부정합** 영역. 26 글자가 한컴 PDF page 1 의 본문 영역 외부 영역에 위치 (한컴 권위 영역 — 표시되지 않는 영역이 정합).

→ PR #662 (Task #656) 영역 의 form-002 정정 본질 자체 영역이 잘못된 영역. 후속 별도 영역 분석 가능성 (작업지시자 결정).

## 5. golden SVG 갱신

| 파일 | 변경 |
|------|------|
| `tests/golden_svg/form-002/page-0.svg` | md5 `12a6cbcc...` → **`672c78c6...`** (26 글자 삭제, -26 LOC) |

작업지시자 시각 판정 권위 영역 commit (`78e38e51`) — 한컴 정답지 정합 영역 보존.

## 6. 자기 검증

| 검증 | 결과 |
|------|------|
| `cargo build --release` | ✅ 통과 (27.61s) |
| `cargo test --release --test svg_snapshot form_002_page_0` | ✅ PASS (golden 갱신 후) |
| `cargo test --release` | ✅ lib **1166** + 통합 ALL GREEN, failed 0 |
| `cargo clippy --release --all-targets` | ✅ 신규 경고 0 |
| WASM 빌드 (Docker) | ✅ 4,596,712 bytes |

## 7. 본 환경 cherry-pick + 머지

### 7.1 cherry-pick (5 commits)
```
7d1c2c52 Task #700 Stage 1: 수행 계획서 + 정밀 진단 보고서
5431ff41 Task #700 Stage 2: 구현 계획서 (옵션 C)
63250e0e Task #700 Stage 3-1: compute_cell_line_ranges cum 절대 동기화
0b31c324 Task #700 Stage 3-1 보고서 + Stage 4 최종 보고서
84f67fcf Task #697 후속: split row 미분할 cell 의 valign 보존
```
충돌 0건.

### 7.2 golden SVG 갱신 commit (메인테이너 추가)
```
78e38e51 golden_svg: form-002/page-0 갱신 — PR #706 (Task #700) 정정 결과 한컴 정답지 정합
```

### 7.3 머지 commit
`fce2d870` — `git merge --no-ff local/task706` 단일 머지 commit. PR #694/#693/#695/#699 패턴 일관.

## 8. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @planet6897 30+ 사이클 핵심 컨트리뷰터 정확 표현 |
| `feedback_visual_judgment_authority` | 결정적 검증 (CI FAILURE) 와 시각 판정 (작업지시자 ★ 통과) 영역의 **충돌 영역 신규 사례** — 시각 판정이 권위 영역 |
| `feedback_pr_supersede_chain` | PR #701 (Task #697) close → PR #706 (Task #697 + #700 통합) supersede. 컨트리뷰터 자체 결정 신규 패턴 |
| `feedback_visual_regression_grows` | golden SVG 가 회귀 가드 영역 영역 부정확 (PR #662 의 잘못된 정정 영역 보존) — golden 갱신 신규 사례 |
| `feedback_close_issue_verify_merged` | Issue #700 / #697 컨트리뷰터 자체 close 영역 의 PR 머지 영역 정합 회복 |
| `feedback_image_renderer_paths_separate` | table_layout.rs (cum 동기화) + table_partial.rs (valign 보존) 두 경로 영역 정정 |

## 9. 잔존 후속

- **PR #662 (Task #656) form-002 정정 본질 영역 부정확** — 작업지시자 결정 영역 의 후속 분석 가능성. 본 PR 머지 영역에서 부분 해소 (form-002 page 0 영역) 영역 + 다른 영역 (synam-001 영역) 영역 별도 점검 가능성.
- 본 PR 본질 정정 영역 의 잔존 결함 부재.

---

작성: 2026-05-09
