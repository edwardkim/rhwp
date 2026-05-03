# PR #507 검토 문서

**PR**: [#507 fix: CASES+EQALIGN 중첩 토폴로지 분수 분실 정정 (#505 / #175 후속)](https://github.com/edwardkim/rhwp/pull/507)
**작성자**: @cskwork (Agentic-Worker) — 외부 컨트리뷰터 첫 PR
**Base / Head**: `devel` ← `feature/issue-505-cases-eqalign-fraction`
**Linked Issue**: [#505](https://github.com/edwardkim/rhwp/issues/505) (OPEN, bug 라벨, milestone 미지정, assignees 없음)
**상태**: OPEN, MERGEABLE, mergeStateStatus = **BEHIND** (devel 진행으로 뒤처짐 — 시각 판정 후 cherry-pick 또는 head-merge 로 해소)
**CI**: ALL SUCCESS (Build & Test + CodeQL × 3 + Canvas visual diff; WASM Build SKIPPED)
**작성일**: 2026-05-01
**최초 검토일**: 2026-05-02
**재검토일**: 2026-05-03 — fixture HWP 추가 (4b1feeac) 후 머지 차단 사유 해소

---

## 1. 개요

### 1.1 본질

PR #396 (Task #175/#174, EQALIGN 영역 구현) 이 다루지 못한 **CASES+EQALIGN 중첩 토폴로지의 분수 분실 결함** 정정. 미적분03.hwp p5 의

```
g(x)= {cases{{1} over {2} x ^{2} & (0 LEQ x LEQ 2) # eqalign{# x} & ...}}
```

에서 `{1} over {2}` 분수가 squashed (`12r²` 시각 결함) 되어 출력되던 결함 해소.

### 1.2 근본 원인

`src/renderer/equation/parser.rs:201-203` 의 `parse_command` 가 OVER/ATOP 단독 호출 시 `EqNode::Empty` 로 폐기. OVER/ATOP 의 중위 연산자 처리는 `parse_expression` (line 92) + `parse_group` (line 446) 에만 존재하나, `parse_cases` / `parse_pile` / `parse_eqalign` 은 `parse_element` 직접 호출 → OVER 분실.

### 1.3 정정

**`try_consume_infix_over_atop()` 헬퍼 추출 (DRY)** → 5개 호출지점 (parse_expression / parse_group / parse_cases / parse_pile / parse_eqalign × 2) 통합.

추가: `tokenizer.rs::skip_spaces` 에 `\n`, `\r` 추가 — HWP 수식 스크립트는 `#`/`&` 으로 명시적 행/탭 구분.

---

## 2. 변경 정합

| 파일 | 변경 | 비고 |
|------|------|------|
| `src/renderer/equation/parser.rs` | +37 / -36 | 헬퍼 추출 + 5 호출지점 통합 (DRY, net +1 line) |
| `src/renderer/equation/tokenizer.rs` | +3 / -1 | `skip_spaces` 에 `\n`/`\r` 추가 |
| `tests/issue_505.rs` (신규) | +292 | 영구 회귀 테스트 9건 (PR 본문 4건 + 5건 추가) |
| 문서 (mydocs/) | 9 파일 | 계획서 / 단계별 보고서 / 최종 보고서 / 전략 문서 |

**소스**: 2 파일 (+40 / -37) — 매우 작은 변경.
**테스트**: 9개 신규 (PR 본문 4건 + `chained_over_left_associative`, `orphan_over_does_not_panic`, `matrix_bare_over`, `pile_bare_over`, `cases_bare_atop` 추가).

---

## 3. 검토 항목

### 3.1 코드 품질

- ✅ **헬퍼 추출** — `try_consume_infix_over_atop` 5 호출지점 통합. DRY 정합.
- ✅ **변경 영역 본질** — parser 의 row-collecting 루프에서 OVER/ATOP 정합. 다른 결함 도입 위험 작음.
- ✅ **방어적 처리** — `children.pop().unwrap_or(EqNode::Empty)` 로 children 비어있을 시 panic 회피.
- ✅ **주석** — 헬퍼 함수 위 주석에 본질 + #505 참조.
- 경미: 5 호출지점 모두 같은 `try_consume_infix_over_atop` → `continue` 패턴 — 매크로화 가능하나 본 PR 본질 외, 해당 사항 아님.

### 3.2 EqAlign 의 left/right 처리 점검

```rust
let consumed = if let Some(ref mut right) = current_right {
    self.try_consume_infix_over_atop(right)
} else {
    self.try_consume_infix_over_atop(&mut current_left)
};
```

✅ **활성 측에서만 OVER/ATOP 정합** — `current_right` 가 `Some` 인 경우 우측, 아니면 좌측. `&` 토큰 후 우측으로 진입 → `right.push()` 와 정합.

### 3.3 tokenizer 의 \n/\r 추가

✅ **본질 정합** — HWP 수식의 행 구분은 `#`, 탭은 `&`. 실제 개행 문자는 무의미한 포맷팅.

⚠️ **잠재 위험**: 한컴 EqEdit 토크나이저와의 정합 미입증 — 컨트리뷰터 주장 (PR 본문) 외 검증 자료 없음. 27 fixture × 344 pages 일괄 회귀 panic 0 보고로 1차 안전 확인.

### 3.4 테스트 (정합)

- ✅ **9 회귀 테스트** — PR 본문 4건보다 풍부.
- ✅ **fixture 인라인** — 4 script (pi=151, 165, 196, 227) 를 inline string + HWP 권위 height (HWPUNIT) 하드코딩. **`.hwp` 파일 의존 없음** → 작업지시자 환경에서도 그대로 실행 가능.
- ✅ **수락 기준**: scale_y ∈ [1/1.30, 1.30] (PR 본문 1.20 보다 마진 +0.10).
- ✅ **분수 인식 결정 검증**: `pi165_fraction_recognized` — 분수 인식 시 height ≥20% 증가.
- ✅ **부수 점검**: `chained_over_left_associative` (좌결합 중첩), `orphan_over_does_not_panic` (panic 방어), `matrix_bare_over` (matrix 동일 클래스).

⚠️ **점검**:
- `count_fractions_and_atops` 헬퍼는 `EqNode` variant 추가 시 누락 위험 (e.g. `Subscript`, `Superscript`, `Paren`, `Cases`, `Pile`, `EqAlign` 재귀). 본 PR 본질 외.

### 3.5 검증 게이트 점검

| 게이트 | PR 본문 보고 | 검토 노트 |
|--------|------------|----------|
| cargo test --lib | 1102 통과 | 정합 |
| cargo test --test issue_505 | 4 통과 (PR 작성 시) → **9 통과** (본 검토 추가분 포함) | 정합 |
| cargo test --test issue_418 | 회귀 0 | 정합 |
| cargo test --test issue_501 | 회귀 0 | 정합 |
| svg_snapshot | **5/6** (PR 작성 시) — 사전 CRLF/LF 회귀, 본 정정 무관 | ⚠️ **현재 (2026-05-02) 환경에서 6/6 통과 재확인 필요** — PR #506 머지 후 사전 회귀 정정 가능성. PR #507 cherry-pick 후 재실행 필수. |
| clippy | 0건 | 정합 |

### 3.6 외부 영역 정합 (PR #506 / Task #509)

- ✅ **회귀 위험 작음** — 변경은 `src/renderer/equation/` 내. Task #509 (`paragraph_layout.rs`, PUA) 와 충돌 0.
- ✅ **PR #506 (HWP 3.0 파서) 와 무관** — HWP3 별도 파서.

### 3.7 PR 본문 비-목표 (별도 이슈 후보)

✅ **정합** — 본 PR 본질만 정합. 한컴 PDF baseline / 인라인 CASES baseline 정렬 / LONGDIV 미구현 / `parse_fraction_until_rbrace` 평행 경로 통합은 별도 task.

### 3.8 외부 컨트리뷰터 첫 PR 점검

- ✅ 컨트리뷰터 (`cskwork`) 의 첫 PR.
- ✅ 내부 워크플로우 정합 — 수행 계획서 / 구현 계획서 / 단계별 보고서 / 최종 보고서 / 전략 문서 5종 작성 (`mydocs/...`). 외부 컨트리뷰터로서 매우 정합한 절차 준수.
- 메모리 `feedback_pr_comment_tone` 적용 — 차분하고 사실 중심 댓글.

---

## 4. **시각 검증 게이트 — 해소됨** ✅ (2026-05-03 재검토)

### 4.1 시각 검증 fixture (해소)

PR #507 의 본질은 **시각 결함 정정** (`12r²` squashing → `(1/2) x²`). 최초 검토 (2026-05-02) 시점에 `samples/` 의 fixture 부재로 머지 차단했으나, 컨트리뷰터의 후속 commit (`4b1feeac`) 으로 fixture 가 추가되어 해소.

**현재 상황 (2026-05-03):**

| 항목 | 결과 |
|------|------|
| `samples/issue-505-equations.hwp` | ✅ **추가됨** (12,800 bytes, 1 섹션 / 4 문단 / 4 수식) |
| 저작권 정합 | ✅ 컨트리뷰터 직접 작성 (원본 미적분03.hwp 비공개 회피) |
| 재현 가능 빌더 | ✅ `examples/build_issue_505_fixture.rs` — `samples/equation-lim.hwp` 베이스 + 4 paragraph clone + Equation script 교체 |
| 시각 판정 입력 | ✅ pi=151 (대조군), pi=165 (본 이슈 핵심), pi=196, pi=227 (회귀 검증) |

### 4.2 메인테이너 재현 절차 (현재 가능)

```bash
git fetch origin pull/507/head:pr-507-review
git checkout pr-507-review
cargo build --release --bin rhwp
./target/release/rhwp export-svg samples/issue-505-equations.hwp -o output/svg/pr507/
```

| 페이지 | fixture | 비고 |
|--------|---------|------|
| 1 | pi=151 | CASES+EQALIGN, 분수 없음 (대조군) |
| 2 | pi=165 | **본 이슈 핵심** — `{1} over {2} x ^{2}` |
| 3 | pi=196 | CASES+EQALIGN, `x^3 -ax+bx` |
| 4 | pi=227 | CASES+EQALIGN, `x^3 +ax+b` |

본 검토 환경에서 검증 결과 (2026-05-03):
- 4 페이지 SVG 정상 산출 (`output/svg/pr507/issue-505-equations_001~004.svg`)
- `cargo test --test issue_505` 9/9 통과 (회귀 0)
- `cargo build --release` 정상 (27 초)

### 4.3 적용 메모리

- `feedback_v076_regression_origin` — 외부 컨트리뷰터가 자기 환경의 시각 자료를 정답지로 사용 → 작업지시자 환경 회귀 위험. **머지 전 작업지시자 직접 시각 검증을 게이트로**.
- `feedback_pdf_not_authoritative` — 컨트리뷰터의 환경별 출력은 권위 미입증. cargo test + svg_snapshot 등 결정적 검증과 병행 필요.
- `feedback_visual_regression_grows` — 페이지 총 수 byte 비교 / cargo test 통과만으로는 시각 결함 검출 불가. 작업지시자 시각 판정이 절차의 핵심 게이트.

### 4.4 한컴 버전별 정답 불일치 → 메인테이너 시각 판정 방식 전환

이슈 #345 (2026-04-30) 에서 확인된 바와 같이, **한컴 버전별로 동일 샘플의 렌더링 정답이 일치하지 않는다**:

- exam_eng.hwp 케이스: 한컴 2010 (14쪽) 부정확, 한컴 2020 (8쪽) 정답
- 단일 한컴 버전을 권위 정답지로 삼으면 다른 버전 환경에서 회귀
- 외부 컨트리뷰터가 자기 환경의 한컴/한글뷰어/PDF 출력을 정답지로 사용한 사례 (PR #360, v0.7.6 회귀의 origin) 가 이미 존재

이러한 정답 불일치 환경에서 PR/회귀 검증의 신뢰 게이트는 **작업지시자(메인테이너)의 한컴 2010 + 한컴 2020 환경 직접 시각 판정** 으로 운영한다 (메모리 `reference_authoritative_hancom`, `feedback_v076_regression_origin`, `feedback_pdf_not_authoritative`).

따라서 본 PR 의 시각 결함 정정 (`12r²` → `(1/2) x²`) 도 **컨트리뷰터의 before/after 스크린샷이 아니라 메인테이너 환경에서의 직접 시각 판정** 으로 게이트 통과 여부를 결정한다. 이를 위해 원본 `.hwp` 가 `samples/` 에 있어야 한다.

### 4.5 수정 요청 (해소)

~~**컨트리뷰터에게 요청:** `samples/미적분 기출문제_03.미분계수와 도함수1-1.hwp` 추가~~

→ **2026-05-03 해소.** 컨트리뷰터가 저작권 회피 + 직접 작성 fixture (`samples/issue-505-equations.hwp` + `examples/build_issue_505_fixture.rs`) 로 commit `4b1feeac` 에서 대응.

남은 절차: 메인테이너가 본 fixture 4 페이지를 한컴 2010/2020 으로 직접 시각 판정 → cherry-pick 머지 또는 추가 수정 요청.

---

## 5. 위험 정리

| 위험 | 가능성 | 비고 |
|------|--------|------|
| OVER/ATOP 정정으로 row-collecting 루프 변경 → 다른 fixture 회귀 | 🟨 작음 | PR #396 회귀 테스트 통과 + 27 fixture × 344 pages 일괄 panic 0 |
| tokenizer `\n`/`\r` 추가로 다른 토픽 결함 | 🟨 작음 | `#`/`&` 명시 행/탭 구분 정합 |
| svg_snapshot PR 작성 시 5/6 — 본 PR cherry-pick 시점 6/6 재확인 필요 | 🟢 매우 작음 | PR #506 머지 후 사전 회귀 정정. cherry-pick 후 재실행 게이트 |
| ~~시각 검증 fixture 부재~~ | ✅ 해소 | commit `4b1feeac` (`samples/issue-505-equations.hwp` + `examples/build_issue_505_fixture.rs`) |
| 시각 판정 게이트 (작업지시자 한컴 2010/2020) | 🟧 게이트 | fixture 4 페이지 SVG 정상 산출. 작업지시자 직접 시각 판정 대기 |

---

## 6. 결정 (2026-05-03 재검토)

**권장**: 🟧 **시각 판정 게이트 진행** — fixture 추가로 머지 차단 사유 해소. 작업지시자의 한컴 2010/2020 직접 시각 판정 후 cherry-pick 머지.

**근거:**
1. 코드 변경은 작고 본질 정합 (헬퍼 추출 + DRY).
2. 회귀 테스트 9건은 인라인 fixture 기반 → 작업지시자 환경에서 실행 가능.
3. ~~머지 차단 사유 (시각 판정 fixture 부재)~~ → commit `4b1feeac` 로 해소.
4. fixture HWP 가 baseline `samples/equation-lim.hwp` 의 정합한 메타데이터를 보존 + Equation script 만 교체로 작성 → 한컴 호환성 위험 작음.
5. 본 검토 환경에서 4 페이지 SVG 정상 산출 + 9/9 회귀 테스트 통과 검증.

**남은 게이트 (작업지시자):**
1. 본 fixture 4 페이지 SVG (`output/svg/pr507/issue-505-equations_001~004.svg`) 를 한컴 2010 + 한컴 2020 출력과 비교
2. pi=165 (page 2) 의 `{1} over {2} x ^{2}` 분수 인식 시각 (squashing 해소) 확인
3. pi=151/196/227 (page 1/3/4) 회귀 결함 없는지 확인
4. 시각 판정 통과 후:
   - cherry-pick 머지 (또는 squash merge) → `pr_507_report.md` 작성
   - 시각 판정 미통과 시 추가 수정 요청

**머지 시 추가 정합 사항:**
- 이슈 #505 milestone 미지정 → v1.0.0 추가 권장 (PR #507 milestone 은 v1.0.0)
- 이슈 #505 assignees 없음 → 메모리 `feedback_assign_issue_before_work` 적용. 메인테이너 assign 권장 (사후 처리)
- README 기여자 목록 갱신 (cskwork 첫 PR)
- mergeStateStatus = BEHIND → cherry-pick 시 자동 해소. force-merge 하지 않음
- svg_snapshot 6/6 재확인 (PR #506 사전 CRLF/LF 회귀 정정 후)

---

## 7. PR 본문 산출물 점검

PR 본문 보고 산출물:
- 수행 계획서: `mydocs/plans/task_m100_505.md`
- 구현 계획서: `mydocs/plans/task_m100_505_impl.md`
- 단계별 보고서: `mydocs/working/task_m100_505_stage{1-4}.md` (4 파일)
- 최종 보고서: `mydocs/report/task_m100_505_report.md`
- 1:1 정합화 전략: `mydocs/tech/all_in_one_parser_fidelity_strategy.md`

✅ 외부 컨트리뷰터로서 내부 워크플로우 (수행 → 구현 → 단계 → 보고) 를 정합하게 준수. 매우 정합.

---

## 8. 메모리 정합

- `feedback_check_open_prs_first` — 본 PR 처리 정합 (이슈 #505 → PR #507 연결 확인)
- `feedback_pr_comment_tone` — 차분하고 사실 중심 댓글
- `feedback_hancom_compat_specific_over_general` — 본 PR 의 정정은 case-specific (CASES+EQALIGN 중첩 한정)
- `feedback_release_sync_check` — cherry-pick 시점 git pull --ff-only origin main 점검
- `feedback_v076_regression_origin` — 외부 환경 시각 자료 정답지 사용 위험 → 본 PR 의 머지 차단 근거
- `feedback_pdf_not_authoritative` — 컨트리뷰터 환경 출력 권위 미입증
- `feedback_visual_regression_grows` — cargo test 통과만으로는 시각 결함 검출 불가
- `feedback_assign_issue_before_work` — 이슈 #505 assignees 없음 점검 필요

---

## 9. 다음 단계 (2026-05-03 갱신)

작업지시자 시각 판정 게이트 진행:

1. **시각 판정 자료 위치**: `output/svg/pr507/issue-505-equations_{001..004}.svg`
2. 작업지시자가 본 SVG 와 한컴 2010 + 한컴 2020 의 동일 fixture 출력 비교
3. 시각 판정 결과:
   - **통과** → `pr_507_review_impl.md` 작성 (cherry-pick 절차) → 작업지시자 승인 → cherry-pick 머지 → `pr_507_report.md`
   - **부분 통과** (특정 fixture 만 결함) → 컨트리뷰터에게 추가 수정 요청
   - **미통과** → 본질 재진단 (parser 정정 외 추가 영역 필요 가능성)
