---
PR: #707
제목: Task #703 — BehindText/InFrontOfText 표 본문 흐름 누락 정정
컨트리뷰터: @planet6897 (Jaeuk Ryu) — 30+ 사이클 핵심 컨트리뷰터
처리: 옵션 A — 3 commits 단계별 보존 cherry-pick + no-ff merge
처리일: 2026-05-09
머지 commit: e3484101
---

# PR #707 처리 보고서

## 1. 처리 결과

✅ **머지 완료** — 옵션 A (3 commits 단계별 보존 cherry-pick + no-ff merge `e3484101`)

| 항목 | 값 |
|------|-----|
| 머지 commit | `e3484101` (--no-ff merge) |
| Issue #703 | close 자동 정합 (closes #703) |
| 시각 판정 | 게이트 면제 (페이지 수 정합 직접 측정 + sweep 통과) |
| 자기 검증 | lib **1167** + 통합 ALL GREEN + issue_703 1/3 + form-002 보존 + clippy clean |

## 2. 정정 본질

### 2.1 결함 본질 (Issue #703)

`pagination/engine.rs:976-981` 에는 BehindText/InFrontOfText (글뒤로/글앞으로) 표 가드가 있으나, 메인 pagination `typeset.rs::typeset_table_paragraph` 분기에 미반영. → `place_table_with_text` 의 `cur_h += pre_height + table_total_height` 가 BehindText/InFrontOfText 표에도 적용되어 본문 흐름 누적 발생 → trailing 항목 다음 페이지 밀림.

### 2.2 정정 (`src/renderer/typeset.rs:1403`, +13 LOC)

```rust
match ctrl {
    Control::Table(table) => {
        // [Issue #703] 글앞으로 / 글뒤로 표는 Shape처럼 취급 — 본문 흐름 공간 차지 없음
        if matches!(
            table.common.text_wrap,
            TextWrap::InFrontOfText | TextWrap::BehindText
        ) {
            st.current_items.push(PageItem::Shape { ... });
            continue;
        }
        // ... 기존 분기
    }
}
```

`pagination/engine.rs:976-981` 와 의미 정합. `text_wrap` 두 변형만 가드 (영향 좁힘, `feedback_hancom_compat_specific_over_general` 정합).

## 3. 본 환경 cherry-pick + 검증

### 3.1 cherry-pick (3 commits)
```
afa70578 Task #703 Stage 1: TDD RED — BehindText/InFrontOfText 표 본문 흐름 누락 결함 검증 테스트
a759a1c2 Task #703 Stage 2: GREEN — typeset.rs 에 BehindText/InFrontOfText 표 가드 추가
fcc37cf8 Task #703 Stage 3: 광범위 회귀 검증 + 최종 보고서 (closes #703)
```
충돌 0건.

### 3.2 결정적 검증

| 검증 | 결과 |
|------|------|
| `cargo build --release` | ✅ 통과 (28.72s) |
| `cargo test --release --test issue_703` | ✅ **1 PASS + 2 ignored** (Issue #704 별개) |
| `cargo test --release --test svg_snapshot form_002_page_0` | ✅ PASS (PR #706 영역 보존) |
| `cargo test --release` | ✅ lib **1167** + 통합 ALL GREEN, failed 0 |
| `cargo clippy --release --all-targets` | ✅ 신규 경고 0 |

### 3.3 본 환경 직접 페이지 수 측정 (PR 본문 정합 입증)

| 파일 | rhwp BEFORE | PDF 권위 | rhwp AFTER | 효과 |
|------|-------------|---------|------------|------|
| `samples/basic/calendar_year.hwp` | 2 페이지 | 1 페이지 | **1 페이지** | ✅ 정합 회복 ★ |
| `samples/table-ipc.hwp` | 11 페이지 | 10 페이지 | **10 페이지** | ✅ 정합 회복 (부수 효과) |

→ 본 환경 직접 측정 영역 PR 본문 정합 영역 정확 입증. lib 테스트 1166 → **1167** (+1 단위 테스트, paginator vs typeset 동치성).

### 3.4 광범위 sweep (PR 본문)
- 196 샘플 SVG/PDF 페이지 수 비교 — 회귀 0, 정합 +2 (calendar_year + table-ipc)
- baseline / after TSV 영구 보존 (`mydocs/report/svg_vs_pdf_diff_20260508{,_after}.tsv`)

### 3.5 머지 commit
`e3484101` — `git merge --no-ff local/task707` 단일 머지 commit. PR #694/#693/#695/#699/#706 패턴 일관.

### 3.6 시각 판정 게이트 면제
- 결정적 검증 (CI ALL SUCCESS + 196 샘플 sweep + 본 환경 직접 페이지 수 측정) 모두 통과
- 페이지 분할 영향 영역 (UI/렌더링 영역 부분 영향 부재 영역)
- `feedback_visual_judgment_authority` 권위 룰 정합 — 결정적 검증 + sweep 통과 영역 의 면제 합리

## 4. 분리된 후속 — Issue #704

PR 본문 명시:
> 통합재정통계 (2010.11/2011.10) 페이지 분할 결함은 다른 본질 영역 — TopAndBottom TAC 1×1 + 각주 환경 0.84 px borderline. Issue #704 별도 분리. tests/issue_703.rs 의 해당 2 케이스는 `#[ignore]` 처리.

→ scope 정확 분리 정합. Issue #704 OPEN 유지.

## 5. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @planet6897 30+ 사이클 핵심 컨트리뷰터 (누적 23 머지) 정확 표현 |
| `feedback_hancom_compat_specific_over_general` | text_wrap 두 변형만 가드 (영향 좁힘) — 일반화 회피 |
| `feedback_process_must_follow` | TDD Stage 1 RED → Stage 2 GREEN → Stage 3 sweep 절차 정합 + 후속 Issue #704 분리 |
| `feedback_image_renderer_paths_separate` | typeset.rs 분기 + pagination/engine.rs 분기 동기 — 두 경로 동치성 단위 테스트 명시 |
| `feedback_visual_judgment_authority` | 결정적 검증 (CI + sweep + 직접 측정) 통과 → 시각 판정 게이트 면제 정합 |
| `feedback_assign_issue_before_work` | Issue #703/#704 컨트리뷰터 self-등록 패턴 (assignee 부재 영역) |

## 6. 잔존 후속

- **Issue #704** OPEN 유지 (TopAndBottom TAC + 각주 환경 0.84 px borderline 결함, 별개 본질) — 후속 PR 처리 가능
- 본 PR 본질 정정 영역 의 잔존 결함 부재

---

작성: 2026-05-09
