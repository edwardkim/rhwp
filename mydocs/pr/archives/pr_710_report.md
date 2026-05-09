---
PR: #710
제목: Task #702 — shortcut.hwp 다단 정의 후속 갱신 누락 정정
컨트리뷰터: @planet6897 (Jaeuk Ryu) — 30+ 사이클 핵심 컨트리뷰터
처리: 옵션 A — 2 commits 단계별 보존 cherry-pick + no-ff merge
처리일: 2026-05-09
머지 commit: 63d38737
---

# PR #710 처리 보고서

## 1. 처리 결과

✅ **머지 완료** — 옵션 A (2 commits 단계별 보존 cherry-pick + no-ff merge `63d38737`)

| 항목 | 값 |
|------|-----|
| 머지 commit | `63d38737` (--no-ff merge) |
| Issue #702 | close 자동 정합 (closes #702) |
| 시각 판정 | **면제** (작업지시자 결정 — 잔존 결함 분리 정합 정합) |
| 자기 검증 | lib **1167** + 통합 ALL GREEN + issue_702 2/2 + 회귀 가드 + 광범위 sweep 회귀 0 |

## 2. 정정 본질 (2 영역)

### 2.1 Distribute 다단 vpos-reset 임계값 완화

`src/renderer/typeset.rs:430-446` — `pv > 5000` 임계값이 짧은 Distribute (배분) 컬럼 (예: 지우기 3+3 분배, vpos=3000) 에서 미달 → column-advance 미발동 → 6항목 1단 적층.

```rust
let is_distribute = st.col_count > 1
    && matches!(st.current_zone_column_type, ColumnType::Distribute);
let trigger = if st.col_count > 1 {
    if is_distribute { cv < pv && pv > 0 }       // [Task #702] Distribute 한정 완화
    else { cv < pv && pv > 5000 }                  // Normal 다단 — 기존 유지
} else {
    cv == 0 && pv > 5000                            // 단일 단 — 기존 유지
};
```

### 2.2 Page/Column break + 새 ColumnDef 미적용

shortcut.hwp p2 의 파일/미리보기/편집 sections — `[쪽나누기]+1단` / `[단나누기]+2단 배분` 패턴에서 새 ColumnDef 미적용 → `col_count` 이전 zone 값 유지 → 페이지 분기 폭주.

정정: Page/Column break + 새 ColumnDef 검출 후 zone 재정의 (process_multicolumn_break 또는 force_new_page + ColumnDef 적용).

### 2.3 ColumnType 추적
```rust
struct TypesetState {
    current_zone_column_type: ColumnType,  // 신규 필드
    ...
}
```

`process_multicolumn_break` 내부 ColumnDef 매칭 시 `current_zone_column_type` 갱신.

## 3. 본 환경 cherry-pick + 검증

### 3.1 cherry-pick (2 commits)
```
507b5136 Task #702: shortcut.hwp 다단 정의 후속 갱신 누락 정정 (closes #702)
f6c4725e Task #702: 거버넌스 산출물 (수행/구현 계획서 + 단계별 보고서 + 최종 보고서)
```
충돌 0건 (auto-merging typeset.rs).

### 3.2 결정적 검증

| 검증 | 결과 |
|------|------|
| `cargo build --release` | ✅ 통과 (27.72s) |
| `cargo test --release --test issue_702` | ✅ **2 PASS** (회귀 가드) |
| `cargo test --release --test exam_eng_multicolumn` | ✅ PASS (Normal 다단 회귀 차단) |
| `cargo test --release --test issue_418` | ✅ PASS (Task #321/#418 회귀 차단) |
| `cargo test --release --test svg_snapshot` | ✅ **8/8** (form-002 PR #706 영역 보존) |
| `cargo test --release` | ✅ lib **1167** + 통합 ALL GREEN, failed 0 |
| `cargo clippy --release --all-targets` | ✅ 신규 경고 0 |
| 광범위 sweep | ✅ 7 fixture / **170 페이지 / 회귀 0** |
| WASM 빌드 (Docker) | ✅ 4,604,562 bytes |

### 3.3 본 환경 직접 페이지 수 측정 (PR 본문 정합 입증)

| 파일 | rhwp BEFORE | PDF 권위 | rhwp AFTER | 효과 |
|------|-------------|---------|------------|------|
| `samples/basic/shortcut.hwp` | 10 페이지 | 7 페이지 | **8 페이지** | ✅ PDF +1 정합 (Issue #708 잔존, 별개) |

### 3.4 머지 commit
`63d38737` — `git merge --no-ff local/task710` 단일 머지 commit. PR #694/#693/#695/#699/#706/#707 패턴 일관.

### 3.5 시각 판정 게이트 면제

작업지시자 결정 (2026-05-09):
> "컨트리뷰터가 잔존 결함은 별도 이슈로 분리했으니 PR 체리픽해서 마무리하면 됩니다."

→ 결정적 검증 (CI ALL SUCCESS + 회귀 가드 2/2 + 광범위 sweep 7 fixture / 170 페이지 / 회귀 0) + 컨트리뷰터의 후속 결함 분리 (Issue #708/#709) 정합으로 시각 판정 면제. `feedback_visual_judgment_authority` 권위 룰 정합.

## 4. 분리된 후속

- **Issue #708** OPEN (1쪽 시프트 잔존) — pi=94 bare `[단나누기]` at last col. 컨트리뷰터 fix 시도 시 `test_539_partial_paragraph_after_overlay_shape` / `test_548_cell_inline_shape_first_line_indent_p8` / `test_exam_math_page_count` 회귀 발견 → rollback. 별도 task 분리.
- **Issue #709** OPEN (부수 시각 결함 4건) — PUA 글자 / 탭 leader / 바탕쪽 자동번호 / 우측 정렬.

→ scope 정확 분리 정합 (`feedback_process_must_follow` 정합).

## 5. 영향 범위

### 5.1 무변경 영역
- Normal (NEWSPAPER) 다단 → 기존 `pv > 5000` 임계값 유지 → Task #321/#418/#470 회귀 차단
- 단일 단 → 기존 `cv == 0 && pv > 5000` 유지
- 광범위 sweep 회귀 0 (7 fixture / 170 페이지)

### 5.2 변경 영역
- Distribute 다단 짧은 컬럼 vpos-reset 검출 (`pv > 0` 영역)
- Page/Column break + 새 ColumnDef 차이 시 zone 재정의

→ **위험 매우 낮음**. 영향 좁힘 + 회귀 가드 + 다른 PR (PR #644/#679/#707) 영향 부재.

## 6. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @planet6897 30+ 사이클 핵심 컨트리뷰터 정확 표현 |
| `feedback_hancom_compat_specific_over_general` | Distribute 한정 임계값 완화 (영향 좁힘) — Normal/단단 영역 보존 |
| `feedback_process_must_follow` | 후속 Issue #708/#709 분리 — fix 시도 회귀 발견 시 rollback + 별도 task 분리 정합 |
| `feedback_visual_judgment_authority` | 결정적 검증 (CI + 회귀 가드 + 광범위 sweep) + 컨트리뷰터 후속 결함 분리 → 시각 판정 면제 정합 |
| `feedback_assign_issue_before_work` | Issue #702/#708/#709 컨트리뷰터 self-등록 패턴 (assignee 부재) |

## 7. 잔존 후속

- Issue #708 / #709 OPEN — 후속 PR 처리 가능 (작업지시자 결정)
- 본 PR 본질 정정 영역 의 잔존 결함 부재

---

작성: 2026-05-09
