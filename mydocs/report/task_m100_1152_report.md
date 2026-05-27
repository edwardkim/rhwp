# Task #1152 최종 결과 보고서 — 호스트 문단 내 TAC 표 intra-paragraph vpos-reset 가드

- 이슈: [#1152](https://github.com/edwardkim/rhwp/issues/1152) — 본문 — 호스트 문단 내 TAC 표 line_seg vpos=0 (intra-paragraph reset) 페이지 분할 미적용
- 브랜치: `local/task1152` (base: `local/devel`)
- 마일스톤: v1.0.0
- 작업기간: 2026-05-28

## 1. 요약

- `samples/2022년 국립국어원 업무계획.hwp` 페이지 32 하단 `별첨 \| 국립국어원 일반현황` 박스가 한컴 PDF 와 다르게 페이지 32 에 배치되던 문제 해결.
- 원인: `TypesetEngine` 의 TAC 표 배치가 같은 문단 내 line_seg vpos=0 신호 (intra-paragraph reset) 를 무시.
- 수정: `src/renderer/typeset.rs:typeset_tac_table()` 진입부에 보수적 가드 (empty-text host + N:N 매핑 + 매핑 line_seg vpos==0) 추가.
- 영향: page 32 → 33 정합 한컴 PDF 와 일치. 인접 4 sample 페이지 수 변동 0. 신규 회귀 0.

## 2. Root cause

pi=586 호스트 문단 (`samples/2022년 국립국어원 업무계획.hwp`):

| 컨트롤 | 표 종류 | line_seg |
|--------|---------|----------|
| ci=0 | 12×5 본문 표 (wrap=위아래, 비-TAC, RowBreak) | ls[0] vpos=69196, lh=3480 |
| ci=1 | 1×3 별첨 박스 (wrap=위아래, **TAC**) | ls[1] **vpos=0**, lh=3480, ls=780 |

`ls[1].vertical_pos == 0` 은 한컴이 명시한 "새 페이지 상단부터" 신호 (intra-paragraph vpos-reset). `dump-pages` 의 `[vpos-reset@line1]` 라벨이 확인.

기존 코드:
- `TypesetEngine` 은 inter-paragraph vpos-reset (Task #321, #724) 만 처리.
- `typeset_tac_table()` 의 fit 검사는 잔여 영역(46.5px) ≥ 박스 크기(38.9px) → fit 통과 → 같은 페이지 배치.
- `--respect-vpos-reset` 옵션은 옛 Paginator (`pagination/engine.rs`) 에만 적용, 기본 엔진에 효과 없음.

## 3. 변경

### 코드 패치 (`src/renderer/typeset.rs:2244-2263`, +20 줄)

```rust
// [Task #1152] 호스트 문단의 intra-paragraph vpos-reset 가드.
// empty-text host paragraph 가 N controls + N line_segs 1:1 매핑이고,
// 현재 TAC 표의 매핑 line_seg(ctrl_idx>0) 의 vpos==0 이면 HWP 가 "이 표를
// 새 페이지 상단부터" 라고 명시한 신호. fit 검사는 표 크기가 잔여 영역에
// 들어가면 통과시키지만 명시 신호를 존중하려면 fit 이전에 advance.
// 케이스: 2022년 국립국어원 업무계획.hwp pi=586 ci=1 (별첨 박스).
if !st.current_items.is_empty()
    && ctrl_idx > 0
    && para.text.is_empty()
    && para.line_segs.len() == para.controls.len()
    && para
        .line_segs
        .get(ctrl_idx)
        .map(|s| s.vertical_pos)
        .unwrap_or(-1)
        == 0
{
    st.advance_column_or_new_page();
}
```

### 회귀 테스트 (`tests/issue_1152_intra_para_vpos_reset.rs`, +79 줄)

- `samples/2022년 국립국어원 업무계획.hwp` 로 page 32 / 33 dump 검증.
- page 32 에 `pi=586 ci=1` 없음, page 33 에 있음 확인.

### 문서

- `mydocs/plans/task_m100_1152.md` — 수행계획서
- `mydocs/plans/task_m100_1152_impl.md` — 구현계획서
- `mydocs/working/task_m100_1152_stage1.md` — 진단
- `mydocs/working/task_m100_1152_stage2.md` — 구현 + 단위
- `mydocs/working/task_m100_1152_stage3.md` — 회귀 + clippy
- `mydocs/working/task_m100_1152_stage4.md` — 시각 검증
- `mydocs/report/task_m100_1152_report.md` (본 문서)

## 4. 검증 결과

### 4-1. 단위 검증 (`rhwp dump-pages`)

| 페이지 | 패치 전 | 패치 후 (= 한컴 PDF 정합) |
|--------|---------|---------|
| page 32 items | 2 | **1** (PartialTable만) |
| page 32 used | 926.8px | 915.1px |
| page 33 첫 항목 | `(빈) 문단` | **`pi=586 ci=1` 별첨 박스** |
| page 33 hwp_used diff | -48.5px | **+7.6px** |

### 4-2. 인접 케이스 페이지 수 (`samples/`)

| sample | 패치 전 | 패치 후 | Δ |
|--------|---------|---------|---|
| `2022년 국립국어원 업무계획.hwp` | 35 | 35 | 0 |
| `kps-ai.hwp` | 80 | 80 | 0 |
| `2025년 기부·답례품 실적 지자체 보고서_양식.hwpx` | 30 | 30 | 0 |
| (비공개 sample A) | 185 | 185 | 0 |

→ 페이지 수 변동 0. 페이지 32 → 33 의 콘텐츠 재배치만 발생, 신규 페이지 생성/삭제 없음.

### 4-3. 전체 테스트 (`cargo test --release --no-fail-fast`)

- 신규 실패: **0**
- 사전 실패 5건: 본 패치와 무관 (stash 검증으로 확인)
  - `issue_598_footnote_marker_nav` 2건 (좌표 hit-test)
  - `svg_snapshot` 3건 (golden mismatch: issue-267, issue-617, issue-677)

### 4-4. clippy

- `cargo clippy --release --lib --no-deps -- -D warnings` ✅
- `cargo clippy --release --test issue_1152_intra_para_vpos_reset --no-deps -- -D warnings` ✅

### 4-5. fmt

- 본 패치가 추가한 라인 (typeset.rs:2244-2263) 드리프트 0.
- 사전 드리프트 (다른 파일 + typeset.rs:2059/2687/2847/2868) 는 본 브랜치 범위 외 — CLAUDE.md 규칙 준수.

### 4-6. SVG 시각 검증

- `별`, `첨` 글자 SVG 검색: page 32 에 0회, page 33 에 각 2회.
- 페이지 33 콘텐츠 순서 (1×3 박스 → 빈 문단 → 1×2 표 → "□ 연 혁" → 연표 → "□ 임 무") 한컴 PDF (`pdf/2022년 국립국어원 업무계획-2022.pdf`) 페이지 31 과 정합.

## 5. 위험 / 한계

| 위험 | 평가 |
|------|------|
| 정상 케이스 false positive 페이지 분할 | 보수적 가드 (empty-text + N:N 매핑) 로 협소 적용 → 인접 4 sample 변동 0 으로 확인 |
| 비-TAC float 표 동일 패턴 | 본 이슈 범위 외 — `typeset_block_table` 에 같은 가드 필요 시 별도 이슈 |
| HWPX 변종 | 검증된 HWPX 인접 3 sample 변동 0 — 일반화 가능 |

## 6. 결론

- 한컴 한글 2022 PDF 와 정합 회복.
- 인접 케이스 회귀 0.
- 본 가드는 strictly additive: 기존 자연 fit-실패 케이스는 영향 없고, 명시 신호만 새로 advance.

## 7. 다음 단계

- 본 브랜치 (`local/task1152`) → `local/devel` merge.
- `local/devel` → `devel` push 는 다른 작업과 묶어 메인테이너 판단.
