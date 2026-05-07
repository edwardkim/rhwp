# PR #621 처리 보고서

## 1. 처리 결과

| 항목 | 값 |
|------|-----|
| PR | #621 — Task #617 표 셀 padding shrink 휴리스틱 다중 줄 가드 |
| 컨트리뷰터 | @planet6897 (Jaeuk Ryu / Jaeook Ryu) — 5번째 사이클 PR (PR #629/#620/#578/#670 직후) |
| 연결 이슈 | #617 (closed) |
| 처리 옵션 | 옵션 B — src + 회귀 차단 가드만 cherry-pick + 거버넌스 산출물 영역 본 환경 명명 규약 정합 |
| devel commits | `2f28866` (본질) + `ab146d6` (거버넌스) + `b23468c` (rhwp-studio public) |
| 처리 일자 | 2026-05-07 |

## 2. cherry-pick 결과

### 본질 commit (`2f28866`)
- `src/renderer/layout/table_layout.rs` (+19/-1) — `shrink_cell_padding_for_overflow` 다중 줄 가드 + 함수 시그니처 (`paragraphs: &[Paragraph]` 추가) + docs
- `src/renderer/layout/table_cell_content.rs` (+1/-1) — 호출 인자
- `src/renderer/layout/table_partial.rs` (+1/-1) — 호출 인자
- `tests/svg_snapshot.rs` (+8) — 신규 회귀 테스트 `issue_617_exam_kor_page5`
- `tests/golden_svg/issue-617/exam-kor-page5.svg` (+1893) — 신규 골든
- author Jaeook Ryu (jaeook.ryu@gmail.com), committer edward

### 거버넌스 commit (`ab146d6`)
- `mydocs/plans/task_m100_617.md` (수행 계획서, 76 LOC) — `task_m07_617` → `task_m100_617` 명명 규약 정합
- `mydocs/plans/task_m100_617_impl.md` (구현 계획서, 128 LOC)
- `mydocs/working/task_m100_617_stage1.md` (단계별 보고서, 84 LOC)
- `mydocs/report/task_m100_617_report.md` (최종 보고서, 107 LOC, PR 두 번째 commit `79ed72b` 영역에서 추출)
- committer edward

### public commit (`b23468c`)
- `rhwp-studio/public/rhwp.js` (갱신) — vite dev server 영역 web 환경 시각 판정 영역
- committer edward

## 3. 본질 정정 영역

### 다중 줄 가드 (단일 룰, 구조적 판단)
```rust
let any_multiline_distributed = paragraphs.iter()
    .any(|p| p.line_segs.len() >= 2);
if any_multiline_distributed {
    return (pad_left, pad_right);
}
```

**본질**: 다중 줄 단락이 있는 셀은 HWP 가 가용 폭에 자간 분배 / 줄바꿈을 마친 상태이므로 자연 폭 추정 (`estimate_text_width`) 으로 다시 깎으면 오버 페인팅 → padding 850 HU 가 1 px 까지 축소되어 본문이 셀 테두리에 닿는 시각 오류. 단일 줄 좁은 셀 (table-text / form-002 수치 셀 등 오버플로우 가능성) 은 종전 휴리스틱 유지.

### `feedback_hancom_compat_specific_over_general` 권위 사례 강화
- 다중 줄 가드는 **구조적 판단 영역** — `line_segs.len() >= 2` 는 측정값 의존 없음, paragraph IR 의 직접 영역
- 임계값 완화 (1.15 → 1.30) / 최소 padding 30% 하한 / segment_width 기반 비교 모두 시도 후 회귀 발견 → 다중 줄 셀에서만 shrink skip 으로 좁혀 적용
- 메모리 룰 본질 영역의 권위 케이스

## 4. 본 환경 결정적 검증

| 항목 | 결과 |
|------|------|
| `cargo build --release` | ✅ |
| `cargo test --lib --release` | ✅ **1155 passed** (회귀 0) |
| `cargo test --test svg_snapshot --release` | ✅ **7/7** (issue_617 신규 포함) |
| `cargo test --test issue_546 --release` | ✅ 1/1 |
| `cargo test --test issue_554 --release` | ✅ 12/12 (`task554_no_regression_exam_kor` 포함) |
| `cargo test --test issue_501 --release` | ✅ 1/1 (Task #501 vertical padding 영역 회귀 0) |
| `cargo clippy --lib -- -D warnings` | ✅ 0 |
| Docker WASM 빌드 | ✅ **4,606,564 bytes** (PR #609 baseline 4,598,892 +7,672) |
| `rhwp-studio npm run build` | ✅ TypeScript 타입 체크 + dist 빌드 |

## 5. 메인테이너 시각 판정 ★ 통과

권위 영역 — `samples/exam_kor.hwp` (한컴 2010 + 한컴 2022 편집기 권위 정답지, `reference_authoritative_hancom`):

| 페이지 | 박스 | 영역 |
|--------|------|------|
| 6 | 16번 보기 | 본문 좌측 padding 약 3mm 복원 |
| 9 | 27번 보기 | 좌·우 padding 약 3mm 복원 |
| 17 | 36번 자료 | 본문 좌측 padding 약 3mm 복원 |

**회귀 영역 점검 통과**: 다른 페이지 단일 줄 좁은 셀 (form-002 / table-text 영역) 미회귀 — 종전 휴리스틱 유지.

작업지시자 평가: "시각판정 통과입니다."

## 6. cherry-pick simulation 영역의 학습 영역

본 PR 처리 영역에서 발견한 영역 — `git checkout` 영역 vs `git cherry-pick` 영역의 차이:
- `/tmp/pr621_test` 임시 clone 영역의 cherry-pick simulation: 통과 (1155 passed)
- 본 환경의 `git checkout local/pr621 -- src/...` 영역: **컴파일 에러 6 건** (E0061, paragraph_layout.rs:634 16→17 인자 + render_tree.rs:771 3→4 인자 시그니처 영역의 PR base 시점 이후 변경 영역)
- 정정: `git cherry-pick 5e9d996` 으로 재진행 → 3-way merge 영역으로 base 시점 이후 영역 변경 흡수 → 컴파일 통과

**학습 영역**: 외부 PR 처리 영역에서 src 영역 전달 영역 시 항상 **`git cherry-pick`** 영역 사용 (`git checkout` 영역 회피) — base 시점 이후 영역의 변경 영역 흡수 영역 정합.

## 7. devel 머지 + push

### 진행
1. `git cherry-pick 5e9d996` (3-way auto-merge 통과, 충돌 0)
2. cherry-pick 결과의 `task_m07_617` 영역 → `task_m100_617` 영역 git mv + 본문 영역 m07 → m100 일괄 치환 (`sed -i`)
3. PR 두 번째 commit `79ed72b` 영역의 `task_m07_617_report.md` 영역 추출 + `task_m100_617_report.md` 영역 신규 영역 추가
4. 거버넌스 commit `ab146d6` 분리
5. `pkg/rhwp.js` → `rhwp-studio/public/rhwp.js` 영역 갱신 → public commit `b23468c`
6. devel ← local/devel ff merge → push 완료 (`08972a8..b23468c`)

### 분기 처리
- 본 cherry-pick 시점 origin/devel 분기 0 — `feedback_release_sync_check` 정합

## 8. PR / Issue close

- PR #621: 한글 댓글 등록 + close (`gh pr close 621`)
- Issue #617: 한글 댓글 등록 + close (`gh issue close 617`)

> `closes #617` 키워드 영역은 cherry-pick merge 영역의 hash 재생성으로 자동 처리 안 됨 — 수동 close 진행. `feedback_close_issue_verify_merged` 정합.

## 9. 주요 정정 영역

### 9.1 다중 줄 가드 영역 (Task #617 본질)
- 함수 시그니처: `paragraphs: &[Paragraph]` 인자 추가
- 호출처 4곳 동시 수정 (`table_cell_content.rs`, `table_partial.rs`, `table_layout.rs` ×2)
- 단일 룰: `any(|p| p.line_segs.len() >= 2)` → padding 보존

### 9.2 회귀 차단 가드 영구 보존
- `tests/svg_snapshot.rs::issue_617_exam_kor_page5` 신규 영구 보존
- `tests/golden_svg/issue-617/exam-kor-page5.svg` (1893 LOC) 신규 영구 보존
- 향후 동일 결함 영역 회귀 시 즉시 검출 영역

### 9.3 거버넌스 산출물 영역의 본 환경 명명 규약 정합
- `task_m07_617` (PR 본문) → `task_m100_617` (본 환경)
- m07 = v0.7.x 약어 영역 → m100 = v1.0.0 마일스톤 정합 영역
- PR #629 / PR #668 패턴 정합 — 외부 컨트리뷰터의 거버넌스 산출물 영역 + 본 환경 명명 규약 영역의 정합 영역

## 10. 메모리 룰 적용

- `feedback_hancom_compat_specific_over_general` — **권위 사례 강화**. 본 PR 의 다중 줄 가드는 구조적 판단 (측정 의존 없음, line_segs 개수만) → 메모리 룰 본질 영역의 권위 케이스
- `reference_authoritative_hancom` — 한컴 2010 + 한컴 2022 편집기 권위 정답지 비교 영역
- `feedback_close_issue_verify_merged` — Issue #617 close 시 본 PR 머지 검증 + 수동 close
- `feedback_assign_issue_before_work` — Issue #617 assignee 미지정 영역 (외부 컨트리뷰터 자기 등록 사례)
- PR #629 / PR #668 본 사이클 패턴 정합 — `mydocs/` 거버넌스 산출물 영역의 외부 / 내부 분리 영역의 권위 적용 케이스

## 11. 다음 사이클 영역

- v0.7.11 PATCH 릴리즈 후보 — 본 사이클 (5/7) 처리 누적 PR (#578/#629/#611/#620/#642/#601/#609/#670/#621 = 9건) 흡수 결과 묶음 가능성
- 본 PR 의 후속 영역 부재 — 단일 룰 가드 영역 정합

## 12. 본 사이클 (5/7) PR 처리 누적

| # | PR | Task / Issue | 결과 |
|---|-----|--------------|------|
| 1 | PR #620 | Task #618 (Picture flip/rotation) | 시각 판정 ★ + close |
| 2 | PR #642 | Task #598 (각주 마커) | 시각 판정 ★ + close |
| 3 | PR #601 | Task #594 (복수 제목행) | 옵션 A-2 + close + Issue #652 신규 |
| 4 | PR #659 | Task #653 (ir-diff 표 속성) | 시각 판정 ★ + close |
| 5 | PR #602 | Issue #449 (rhwpDev) | close + Issue #449 reopen |
| 6 | PR #668 | Task #660 (Neumann ingest) | 첫 PR + 시각 판정 ★ + close + 후속 #665/#666/#667 |
| 7 | PR #609 | Task #604 (Document IR) | 11 commits 단계별 + 시각 판정 ★ + close |
| 8 | PR #670 | (이슈 미연결) 한글 2022 PDF 199 | 옵션 D 변형 + 메모리 룰 갱신 + close |
| 9 | **PR #621** | **Task #617 (표 셀 padding)** | **옵션 B + 시각 판정 ★ + close** |

본 PR 의 **단일 룰 구조적 가드 + 회귀 차단 가드 영구 보존 + 본 환경 명명 규약 정합 + 컨트리뷰터의 5번째 사이클 PR + 메인테이너 시각 판정 ★ 통과 + `feedback_hancom_compat_specific_over_general` 권위 사례 강화 패턴 모두 정합**.
