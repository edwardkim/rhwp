# PR #621 1차 검토 보고서

## 1. PR 메타 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | #621 |
| 제목 | Task #617: 표 셀 padding shrink 휴리스틱 다중 줄 가드 (closes #617) |
| 컨트리뷰터 | @planet6897 (Jaeuk Ryu / Jaeook Ryu) — 5번째 사이클 PR (PR #629/#620/#578/#670 직후) |
| base / head | `devel` ← `planet6897:feature/task-617-cell-padding` |
| state / mergeable | OPEN / **CONFLICTING** / **DIRTY** (PR base 71 commits 뒤) |
| 변경 | 10 files, +2,324 / -3 |
| commits | 2 (`5e9d996` 본질 + `79ed72b` 보고서) |
| labels / milestone | 없음 / 미지정 |
| 연결 이슈 | **closes #617** (PR 본문 명시) |
| 작성일 / 갱신 | 2026-05-06 00:17 / 01:08 |

### CI 상태 (모두 통과)
- Build & Test ✅ (5/6 두 번 실행, 모두 SUCCESS)
- Analyze (rust / python / javascript-typescript) ✅
- Canvas visual diff ✅
- CodeQL ✅
- WASM Build SKIPPED

### 댓글 영역
- 댓글 없음 (작업지시자 댓글 / 컨트리뷰터 추가 댓글 부재)

---

## 2. Issue #617 권위 영역

### 결함
`exam_kor.hwp` 의 16/27/36번 `<보기>` / `<자료>` 글상자에서 본문과 셀 테두리 사이 좌·우 padding 이 거의 0 으로 표시됨.

### 원인 (Issue 본문 분석)
`src/renderer/layout/table_layout.rs::shrink_cell_padding_for_overflow` 가 양쪽 정렬 한국어 본문의 자연 폭을 과대 추정하여 정상 padding (=850 HU ≈ 3 mm) 을 거의 0 까지 축소.

- `resolve_cell_padding` 은 cell.padding (850) 정상 채택
- 직후 `shrink_cell_padding_for_overflow` 가 `estimate_text_width` (음수 letter_spacing 0 으로 clamp) 결과로 1.15× 임계 초과 판단
- CharShape `ratio=95%` / `spacing=-5%` 와 HWP 가 이미 line_segs 로 분배해 둔 자간이 추정에 충분히 반영되지 않음

### 권위 증거
같은 표 구조 / `pad=(850,850,992,708)` / aim=false 인데:
- 25번 박스 (text_len=639, 8 lines): 정상
- 27번 박스 (text_len=681, 9 lines): 깨짐

→ 텍스트 분량 차이로 임계 초과 여부만 갈리는 것이 결정적 증거.

### 영향 샘플
- `samples/exam_kor.hwp` 페이지 6 (16번), 페이지 9 (27번), 페이지 17 (36번)

### Issue #617 assignee 영역
- **assignee 미지정** — 컨트리뷰터 (@planet6897) 가 직접 자기 등록 후 작업 진입 영역. `feedback_assign_issue_before_work` 일차 방어선 부재 사례.

---

## 3. 본 환경 정합 상태 점검

### 본 환경 devel 의 관련 영역
- `src/renderer/layout/table_layout.rs:860 shrink_cell_padding_for_overflow` 영역 잔존 (Task #347 cont 영역)
- Task #501 머지 영역 (`f294cb6` "비정상 cell.padding 한컴 방어 로직 모방 정정") — **vertical padding 영역** (top/bottom) 정정, 본 PR 의 horizontal padding (left/right) shrink 영역과 별개 영역
- Task #347 (`72d8245` "cell padding 부적절 축소 + cell.padding/table.padding 우선순위 보정") — 본 PR 가 정정하려는 영역의 직전 영역

### 본 환경 정합 영역
- `feedback_hancom_compat_specific_over_general` 메모리 룰 영역과 정합 — 본 PR 의 다중 줄 가드 (`paragraphs.iter().any(|p| p.line_segs.len() >= 2)`) 는 **구조적 판단 영역** (측정 의존 없음)

### 마일스톤 명명 영역
- 본 PR 의 문서 영역 명명: `task_m07_617.md` / `task_m07_617_impl.md` / `task_m07_617_stage1.md` / `task_m07_617_report.md`
- 본 환경 CLAUDE.md 영역 명명 규약: `task_m{숫자}_{이슈번호}.md` (예: `task_m100_71.md`)
- **`m07` 영역은 본 환경 명명 규약 영역의 변형 영역** — `m100` (v1.0.0) / `m05x` (v0.5.x) 영역의 패턴 영역. `m07` 은 v0.7.x 의 약어 영역 추정 → 본 환경 명명 규약 영역의 **불일치 영역**

---

## 4. PR 의 본질 정정 영역

### 4.1 본질 정정 (단일 룰 + 구조적 가드)

**다중 줄 가드 추가**:
```rust
let any_multiline_distributed = paragraphs.iter()
    .any(|p| p.line_segs.len() >= 2);
if any_multiline_distributed {
    return (pad_left, pad_right);
}
```

다중 줄 단락이 있는 셀은 HWP 가 자간 분배 / 줄바꿈을 마친 상태이므로 자연 폭 추정으로 다시 깎으면 오버 페인팅. 단일 줄 좁은 셀 (table-text / form-002 수치 셀 등 오버플로우 가능성 있는 케이스) 은 종전 휴리스틱 유지로 회귀 0.

### 4.2 함수 시그니처 변경
함수 시그니처에 `paragraphs: &[Paragraph]` 인자 추가, 호출처 4곳 동시 수정:
- `table_cell_content.rs` (1곳)
- `table_partial.rs` (1곳)
- `table_layout.rs` (2곳, ×2)

### 4.3 시도하고 채택하지 않은 변경 (PR 본문)
- 임계 1.15 → 1.30 완화: form-002 / table-text 골든 회귀
- 최소 padding 30% 하한: 위 골든 시각 회귀
- segment_width 기반 비교: 단일 줄 셀에서 HWP segment_width 가 inner_w 이내였으나 실 렌더에서 외곽선 넘는 케이스 발견

→ **다중 줄 셀에서만 shrink skip** 으로 좁혀 적용 (`feedback_hancom_compat_specific_over_general` 정합).

### 4.4 회귀 차단 가드
- `tests/svg_snapshot.rs::issue_617_exam_kor_page5` 신규 회귀 테스트
- 골든 SVG: `tests/golden_svg/issue-617/exam-kor-page5.svg` (+1893 LOC)
- 기존 골든 모두 통과 (변경 0): form_002_page_0 / table_text_page_0 / issue_157_page_1 / issue_267_ktx_toc_page / issue_147_aift_page3 / render_is_deterministic_within_process

---

## 5. 본 환경 cherry-pick simulation 결과

본 환경 임시 clone (`/tmp/pr621_test`) 에서 cherry-pick simulation 진행:

### cherry-pick
- `5e9d996` (본질 commit) cherry-pick 통과 — **충돌 0** (auto-merge `table_layout.rs` + `table_partial.rs` 통과)
- 본 환경 devel 보다 71 commits 뒤 영역에서도 src 영역 충돌 없음

### 결정적 검증 결과 (모두 통과)

| 항목 | 결과 |
|------|------|
| `cargo test --lib --release` | ✅ **1155 passed** (회귀 0) |
| `cargo test --test svg_snapshot --release` | ✅ **7/7** (issue_617 신규 포함) |
| `cargo test --test issue_546 --release` | ✅ 1/1 |
| `cargo test --test issue_554 --release` | ✅ **12/12** (`task554_no_regression_exam_kor` 통과 — exam_kor 영역 회귀 0) |
| `cargo test --test issue_501 --release` | ✅ 1/1 (Task #501 vertical padding 영역 회귀 0) |
| `cargo clippy --lib --release -- -D warnings` | ✅ 0 warning |
| `cargo build --release` | ✅ |

### 본 PR 의 본질 정합 영역
- 본 PR 의 정정 영역과 본 환경 직전 정정 영역 (Task #501 vertical padding) 의 **별개 영역** 정합 — 충돌 영역 부재
- exam_kor 영역의 다중 페이지 (페이지 6/9/17) 회귀 영역 부재 (`task554_no_regression_exam_kor` 통과)
- 신규 회귀 차단 가드 (issue_617 svg_snapshot) 정합 영역

---

## 6. 옵션 분류

본 환경 cherry-pick simulation 결과 + 본 환경 정합 영역 점검 결과 기반:

### 옵션 A — 전체 cherry-pick (2 commits)
**진행 영역**:
```bash
git checkout local/devel
git cherry-pick 5e9d996 79ed72b
```

**장점**:
- src 영역 + 회귀 차단 가드 (svg_snapshot + golden) + 거버넌스 산출물 (계획서 + 보고서) 모두 보존
- 컨트리뷰터의 단계별 작업 영역 (수행 계획 + 구현 계획 + Stage 1 보고서 + 최종 보고서) 영구 보존

**잠재 위험**:
- 거버넌스 산출물 영역의 명명 영역 (`task_m07_617`) 이 본 환경 명명 규약 (`task_m100_{이슈번호}`) 과 불일치 영역 → 본 환경 거버넌스 영역 영구 부담 영역
- PR #629 / PR #668 패턴 정합 — `mydocs/` 거버넌스 산출물 영역 cherry-pick 제외 영역 정합 영역

### 옵션 B — 본질 cherry-pick + 거버넌스 영역 제외 (PR #629 / #668 패턴 정합)
**진행 영역**:
```bash
git checkout local/devel
git checkout local/pr621 -- src/ tests/svg_snapshot.rs tests/golden_svg/issue-617/
git add src/ tests/
git commit --author="Jaeook Ryu <jaeook.ryu@gmail.com>" \
  -m "Task #617: 표 셀 padding shrink 휴리스틱 다중 줄 가드 (closes #617)"
```

**장점**:
- src 영역 + 회귀 차단 가드 (svg_snapshot + golden) 만 영구 보존
- 거버넌스 산출물 영역 (`task_m07_617*` 5 파일) cherry-pick 제외 — 본 환경 명명 규약 영역 충돌 영역 회피
- PR #629 / PR #668 의 본 환경 패턴 정합 (`mydocs/` 거버넌스 산출물 영역의 외부/내부 분리 영역 정합)

**잠재 위험**:
- 컨트리뷰터의 거버넌스 산출물 영역의 작업 영역 일부 손실 — 단 본 환경 명명 규약 영역 외 영역으로 영구 보존 영역 부담 회피
- 단일 squash commit 영역 → 컨트리뷰터 author 보존 영역 정합 (committer edward)

### 옵션 C — close + 컨트리뷰터에게 base 동기화 + 거버넌스 영역 명명 규약 정정 권유
**진행 영역**:
컨트리뷰터에게 PR close + base 동기화 (71 commits 뒤) + 거버넌스 산출물 명명 영역 정정 (`task_m07_617` → `task_m100_617`) 후 재제출 권유.

**잠재 위험**:
- 컨트리뷰터 부담 영역 (이미 작업 + CI 통과 영역의 close)
- 본 환경 결정적 검증 (cargo test 1155 + svg_snapshot 7/7 + clippy 0) 모두 통과 영역 — close 사유 영역 부재

### 옵션 D — 옵션 B 진행 + 본 환경 정합 보강 영역 (CLAUDE.md 명명 규약 영역의 외부 컨트리뷰터 영역 명시)
**진행 영역**:
1. 옵션 B 진행 (src + 회귀 차단 가드만 cherry-pick)
2. 본 환경 정합 보강 commit — 외부 컨트리뷰터의 거버넌스 산출물 영역 명명 규약 영역의 분리 영역 명시 영역 (선택)

### 옵션 영역 요약 표

| 옵션 | 진행 가능 | Issue #617 정정 | 결정적 검증 | 거버넌스 영역 | 권장 |
|------|----------|----------------|------------|--------------|------|
| **A** (전체 2 commits) | ✅ 충돌 0 | ✅ Stage 가드 포함 | ✅ 1155/7/12 | `task_m07_617*` 5 파일 영역 영구 보존 (명명 규약 충돌 영역) | ⚠️ |
| **B** (src + 가드만) | ✅ 충돌 0 | ✅ Stage 가드 포함 | ✅ 1155/7/12 | `mydocs/` 영역 cherry-pick 제외 (PR #629/#668 패턴 정합) | ⭐ |
| **C** (close + 재제출) | — | — | — | 컨트리뷰터 부담 + close 사유 부재 | ❌ |
| **D** (옵션 B + 본 환경 보강) | ✅ | ✅ | ✅ | 옵션 B + 명명 규약 영역의 외부 컨트리뷰터 영역 명시 | ⭐ |

### 권장 영역 — 옵션 B (PR #629 / #668 패턴 정합)

**사유**:
1. **본 환경 결정적 검증 모두 통과** — cargo test 1155 / svg_snapshot 7/7 (issue_617 신규 포함) / clippy 0 / exam_kor 영역 회귀 0
2. **본질 정정 영역의 정합성** — `feedback_hancom_compat_specific_over_general` 메모리 룰 영역 정합 (다중 줄 = 구조적 가드, 측정 의존 없음)
3. **본 환경 패턴 정합** — PR #629 (외부 컨트리뷰터의 src + .claude/skills 영역만 cherry-pick + `mydocs/` 거버넌스 산출물 영역 제외) + PR #668 (동일 패턴) 영역의 본 환경 권위 적용 케이스 영역
4. **명명 규약 영역의 영구 영역 충돌 회피** — `task_m07_617` 영역은 본 환경 `task_m100_{이슈번호}` 영역의 변형 영역 → cherry-pick 제외로 영구 영역 부담 영역 회피
5. **회귀 차단 가드 영구 보존** — issue_617 svg_snapshot 테스트 + 골든 SVG (1893 LOC) 영구 보존

---

## 7. 잠정 결정

### 권장 결정
- **옵션 B 진행** — 본 PR 의 src 영역 + 회귀 차단 가드 (svg_snapshot + golden SVG) 만 cherry-pick + 거버넌스 산출물 영역 (`mydocs/` 5 파일) cherry-pick 제외
- **squash 단일 commit** — 본질 commit `5e9d996` + 보고서 commit `79ed72b` 의 src 영역 만 합쳐 단일 commit + author Jaeook Ryu 보존
- 본 환경 결정적 검증 진행 후 머지 + push + PR/Issue close

### 후속 영역 (PR 본문)
- 단일 줄 좁은 수치 셀 영역 (table-text / form-002) 영역의 본질 영역 — 종전 휴리스틱 유지, 본 PR 영역 외
- **`feedback_hancom_compat_specific_over_general` 권위 사례 강화** — 본 PR 의 다중 줄 가드 (구조적 판단 영역) 가 메모리 룰 영역의 권위 케이스 영역

### 검증 영역 (옵션 B 진행 시 본 환경 직접 점검)
1. `cargo test --lib --release` 1155 passed 정합 (회귀 0) — 본 환경 simulation 결과 정합
2. `cargo test --test svg_snapshot --release` 7/7 (issue_617 신규 포함)
3. `cargo test --test issue_546` / `issue_554` / `issue_501` 통과
4. `cargo clippy --lib -- -D warnings` 0
5. Docker WASM 빌드 + byte 측정
6. `rhwp-studio npm run build` (영향 0 영역)
7. **시각 판정 ★** — `samples/exam_kor.hwp` 페이지 6 (16번 보기) / 페이지 9 (27번 보기) / 페이지 17 (36번 자료) 작업지시자 시각 판정 (한컴 2010 + 한컴 2022 편집기 권위 정답지)

---

## 8. 메모리 룰 관점

본 PR 검토에 적용되는 메모리 룰:
- `feedback_hancom_compat_specific_over_general` — **본 PR 의 다중 줄 가드는 권위 사례 강화 영역**. `paragraphs.iter().any(|p| p.line_segs.len() >= 2)` 는 **구조적 판단** (측정 의존 없음, line_segs 개수만) → 메모리 룰 본질 영역 정합
- `reference_authoritative_hancom` — exam_kor 영역의 한컴 편집기 권위 정답지 영역 (한컴 2010 + 2022) — 작업지시자 시각 판정 영역의 권위 영역
- `feedback_assign_issue_before_work` — Issue #617 assignee 미지정 영역 (외부 컨트리뷰터 자기 등록 영역, 일차 방어선 부재 사례)
- `feedback_close_issue_verify_merged` — Issue #617 close 시 본 PR 머지 검증 + 수동 close (closes #617 키워드는 cherry-pick merge 로 자동 처리 안 될 가능성)
- PR #629 / PR #668 본 사이클 패턴 정합 영역 — `mydocs/` 거버넌스 산출물 영역의 외부 / 내부 분리 영역의 본 PR 적용 케이스 영역

---

## 9. 다음 단계 (CLAUDE.md PR 처리 4단계)

1. (완료) PR 정보 + Issue 연결 + base/head + mergeable + CI 상태 확인
2. (현재) `pr_621_review.md` 작성 → 승인 요청
3. (필요 시) `pr_621_review_impl.md` 작성 → 승인 요청
4. 검증 (빌드/테스트/clippy + 시각 판정 ★) + 판단 → `pr_621_report.md` 작성

### 작업지시자 결정 요청
1. **옵션 결정** — 옵션 A (전체) / 옵션 B (src + 가드만, 권장) / 옵션 D (옵션 B + 본 환경 보강) 결정
2. **거버넌스 영역 처리** — `task_m07_617*` 5 파일 영역 cherry-pick 제외 가/부 (PR #629/#668 패턴 정합)
3. **시각 판정 권위 영역** — exam_kor.hwp 페이지 6/9/17 의 16/27/36번 보기·자료 박스 영역 작업지시자 직접 시각 판정 영역 진행 가/부

결정 후 본 환경 직접 결정적 검증 (이미 simulation 통과) + WASM 빌드 + 작업지시자 시각 판정 ★ + `pr_621_report.md` 작성.
