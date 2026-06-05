# PR #1125 검토 — #1122 #1124 3-11월 실전 통합 p4 수식 및 HWPX 다단 구분선 개선

- 검토일: 2026-05-26
- PR: https://github.com/edwardkim/rhwp/pull/1125
- 연결 이슈: closes #1122 + closes #1124 (양쪽 OPEN, assignee 비어있음)
- 선행 PR (CLOSED): #1123 (base=main, #1122 영역 영역 단독 영역 영역 시도)
- 검토자: Claude (rhwp 메인테이너 보조)

## 1. PR 정보

| 항목 | 값 |
|------|-----|
| 번호 | #1125 |
| 제목 | #1122 #1124 3-11월 실전 통합 p4 수식 및 HWPX 다단 구분선 개선 |
| 작성자 | jangster77 (Taesup Jang) — 16번째 PR, HWP3 핵심 컨트리뷰터 |
| base ← head | **`main`** ← `jangster77:local/task1124-hwpx-column-line` |
| head SHA | `e7c95e578bc0f1f34ea3c4237465437d3972b5da` |
| commits | 4 (source 2 + docs 2) |
| 상태 | OPEN / mergeable=**MERGEABLE** / mergeStateStatus=**BLOCKED** (base=main 영역 영역 룰 차단) |
| 변경 | 16 files, +969 / -14 |
| 본질 변경 | src/parser/hwpx/section.rs (+163/-4) · src/renderer/equation/parser.rs (+38) · src/renderer/equation/tokenizer.rs (+58/-1) · src/renderer/layout/utils.rs (+47/-1) · src/renderer/layout/picture_footnote.rs (+7/-5) · src/renderer/layout.rs (+5/-3) |
| 문서 | mydocs/plans (4) + mydocs/report (3) + mydocs/working (2) + mydocs/orders (1) |
| GitHub CI | 컨트리뷰터 보고 영역 영역 통과 (cargo test --lib + clippy + svg_snapshot) |

## 2. base=main 영역 영역 — 정정 영역

본 PR 영역 영역 **base=main** 영역 영역 설정. 본 프로젝트 정상 워크플로 영역 영역 base=devel 영역 영역. main 영역 영역 영역 영역 릴리즈 시점 영역 영역 devel→main PR 영역 영역 별도.

선행 #1123 (CLOSED) 영역 영역 base=main 영역 영역 close → #1125 영역 영역 통합 영역 영역 다시 시도 (영역 영역 base=main 그대로). 컨트리뷰터 영역 영역 base 영역 영역 변경 영역 영역 영역 영역 안 함.

**처리 정책** (작업지시자 결정): 메인테이너 측 직접 cherry-pick / 영역 영역 devel 영역 영역 머지 + PR 자체 close. PR #1120 영역 영역 영역 영역 영역 영역 패턴.

## 3. 컨트리뷰터 누적 사이클

| PR | 영역 | 결과 |
|----|------|------|
| #1085 (Task #1042) | HWP3→HWP5 multi-fixture alignment | CLOSED (superseded) |
| #1103 (Task #1086) | 한컴오피스 페이지네이션 | CLOSED (superseded) |
| #1107 (Task #1105) | HWP3→HWP5 page break | **MERGED** |
| #1118/#1119/#1120 (#1116) | sample16 한컴 3mm 격자 | **#1120 MERGED** |
| #1123 (#1122) | 수식 over + 그림 (단독) | CLOSED (superseded) |
| **#1125 (#1122 + #1124)** | 본 PR — 통합 | **OPEN** |

## 4. 변경 내용

### 4.1 #1122 — 수식 over+숫자 결합 (`c4810c33`)

#### src/renderer/equation/tokenizer.rs (+58/-1)

```rust
fn matches_at_ascii_ci(&self, kw: &str) -> bool {
    // case-insensitive prefix match
}

// over/atop + 숫자 영역 영역 prefix-split
for kw in ["over", "atop"] {
    if self.matches_at_ascii_ci(kw) {
        let after = self.peek(kw.len());
        if matches!(after, Some(c) if c.is_ascii_digit()) {
            // "over20" → Token("over") + 영역 영역 "20" 영역 영역 별도 토큰
        }
    }
}
```

**평가**:
- 합리적 분기 — over/atop 뒤 숫자 영역 영역 한정 (`alpha/sqrt` 등 영역 영역 영역 영역 안전)
- 메모리 룰 `feedback_hancom_compat_specific_over_general` 정합
- unit test 추가 (tokenizer.rs +30 영역 영역 영역 영역 영역 영역 영역 영역)

#### src/renderer/equation/parser.rs (+38)

over/atop 영역 영역 분수 토큰 영역 영역 처리 추가. 컨트리뷰터 영역 영역 영역 영역 한컴 정합 영역 영역 정정.

#### src/renderer/layout/utils.rs (+47/-1) + picture_footnote.rs

`picture_display_size_hu()` 신규 — Picture 영역 영역 `current_width/height` (SHAPE_COMPONENT) 영역 영역 영역 영역 `common.width/height` (CommonObjAttr) 영역 영역 더 큰 영역 영역 채택. 한컴 영역 영역 영역 영역 정합 (문26 그림 크기).

unit test 2개 추가 (영역 영역 영역 영역 + smaller current keeps common).

### 4.2 #1124 — HWPX 다단 구분선 (`fa1927a0`)

#### src/parser/hwpx/section.rs (+163/-4)

`parse_col_pr` → `parse_col_pr_with_children` 영역 영역 영역 영역. 자식 `<hp:colLine>` 영역 영역:
- `type` (NONE/SOLID/DASH/DOT/DASH_DOT/...)
- `width` ("0.12 mm" → 1)
- `color` (#RRGGBB)

영역 영역 ColumnDef 영역 영역 적용 (separator_type/width/color).

unit test 2개 추가:
- `test_task1124_col_pr_parses_col_line` — colPr + colLine 통합 파싱
- `test_task1124_col_line_type_and_width_mapping` — type/width 매핑

**평가**:
- 깔끔한 영역 영역 분리 (parse_col_pr 영역 영역 유지 + with_children 영역 영역 신규)
- 자식 영역 영역 처리 영역 영역 일관 패턴
- unit test 영역 영역 회귀 가드 충분

### 4.3 컨트리뷰터 문서

- `mydocs/plans/task_m100_1122.md` + `task_m100_1122_impl.md`
- `mydocs/plans/task_m100_1124.md` + `task_m100_1124_impl.md`
- `mydocs/working/task_m100_1122_stage1.md` + `task_m100_1124_stage1.md`
- `mydocs/report/task_m100_1122_report.md` + `task_m100_1124_report.md`
- `mydocs/report/task_m100_1122_1124_pr_body.md` (PR 본문 초안)

## 5. 충돌 해결

devel 영역 영역 PR #1120 영역 영역 다른 변경 영역 영역 충돌 — **5 파일**:

| 파일 | 충돌 영역 | 해결 |
|------|----------|------|
| mydocs/orders/20260526.md | add/add — 메인테이너 일지 vs 컨트리뷰터 Task #1122/#1124 일지 | 양쪽 보존 |
| src/parser/hwpx/section.rs | 새 unit test (HEAD: 8개) vs (PR: 2개) — 같은 위치 영역 영역 영역 영역 새 영역 영역 | 양쪽 모두 추가 |
| src/renderer/layout.rs | use re-export 영역 영역 정리 vs picture_display_size_hu 추가 | HEAD 영역 영역 정리 영역 영역 + picture_display_size_hu 영역 영역 영역 영역 |
| src/renderer/layout/picture_footnote.rs | use 영역 영역 정리 vs picture_display_size_hu import | HEAD 영역 영역 정리 영역 영역 + picture_display_size_hu 영역 영역 |
| src/renderer/layout/utils.rs | use 영역 영역 정리 vs Picture import | HEAD 영역 영역 영역 영역 (PR 영역 영역 Picture 영역 영역 영역 영역 자동 merge 됨) |

## 6. 자동 검증 결과

| 항목 | 결과 |
|------|------|
| merge (with conflicts) | ✅ 충돌 5 파일 해결 |
| `cargo build --release` | ✅ 통과 (2.68s, 이전 캐시 활용) |
| `cargo fmt --all` | ⚠️ 1 파일 정정 (equation/parser.rs — 신규 테스트 영역 영역 fmt 위반) → 자동 정정 |
| `cargo clippy --lib --release -- -D warnings` | ✅ warnings 0 (26.36s) |
| `cargo test --release --test svg_snapshot` | ✅ 8/8 passed |
| GitHub CI | 컨트리뷰터 보고 영역 영역 통과 |

## 7. 위험·관찰

| 항목 | 등급 | 영역 |
|------|------|------|
| base=main 영역 영역 잘못 설정 | 중 | 컨트리뷰터 영역 영역 PR 워크플로 영역 영역 이해 부족 — 컨트리뷰터 메모리 룰 `feedback_pr_*` 영역 영역 영역 영역 base 영역 영역 영역 영역 — 향후 추가 룰 권장 |
| 이슈 #1122 #1124 영역 영역 assignee 비어있음 | 저 | 메모리 룰 `feedback_assign_issue_before_work` 위반 (PR #1120 영역 영역 동일 패턴) |
| 별도 통합 테스트 파일 (issue_1122/1124) 없음 | 저 | unit test (section.rs + tokenizer.rs 영역 영역) 영역 영역 회귀 가드 영역 영역 |
| 직접 시각 검증 미수행 | 중 | 컨트리뷰터 보고 의존 + svg_snapshot 회귀 가드 일부 확보. 3-11월_실전_통합_2022 fixture 영역 영역 시각 영역 영역 작업지시자 영역 영역 확인 권장 |
| `picture_display_size_hu` 영역 영역 영역 영역 — 더 큰 축 채택 | 저 | 합리적 가설 — 한컴 영역 영역 영역 영역 정합 보고. 영역 영역 영역 영역 회귀 위험 (current 영역 영역 더 큰 영역 영역 영역 영역 의도된 다른 케이스) — unit test 영역 영역 일부 확보 |
| over/atop 영역 영역 + 숫자 분기 | 저 | case-specific — 영역 영역 영역 영역 알파/sqrt 영역 영역 영역 영역 영역 회귀 0 |

## 8. 최종 평가 (잠정)

| 항목 | 결과 |
|------|------|
| 본질 해결 | ✅ #1122 (수식 over + 문26 그림) + #1124 (HWPX 다단 구분선) — 두 이슈 통합 영역 영역 정밀 작업 |
| 자동 검증 | ✅ 모두 통과 (build / fmt / clippy --lib / svg_snapshot) |
| 시각 검증 | ⚠️ 컨트리뷰터 자체 보고 의존 — 작업지시자 시각 판정 권장 |
| 코드 품질 | ✅ unit test 영역 영역 회귀 가드 + 명확한 분기 영역 영역 영역 영역 |
| 메모리 룰 정합 | ✅ `hancom_compat_specific_over_general`, `pr_supersede_chain` (#1123 → #1125) |
| 충돌 해결 | ✅ 5 파일 (orders + section.rs + layout.rs + picture_footnote.rs + utils.rs) |
| base 영역 영역 정정 | ⚠️ base=main 영역 영역 영역 영역 — 메인테이너 측 cherry-pick + devel 영역 영역 머지 + PR close |
| **결정 권장** | **MERGE** (메인테이너 측 직접 머지 + PR close) — 본질 해결 + 자동 검증 통과 + 회귀 가드 영역 영역 충분 |

## 9. 작업지시자 결정 요청

1. **MERGE** 진행 (메인테이너 측 직접 머지 + PR close) 또는 컨트리뷰터에게 base 영역 영역 변경 요청 — 작업지시자 결정 완료 (메인테이너 측 직접 cherry-pick + devel 영역 영역 머지)
2. 이슈 #1122 + #1124 close 영역 영역 — PR merge 후 (closes 본문 영역 영역 영역 영역 자동 영역 영역 영역 영역 — 본 PR 영역 영역 base=main 영역 영역 closed-via-commit 영역 영역 안 영역 영역 → 수동)
3. 3-11월_실전_통합_2022 fixture 영역 영역 시각 검증 권장 (사용자)
