---
PR: #729
제목: Task #143 — LaTeX 명령어 호환 확장 (2차)
컨트리뷰터: @oksure (Hyunwoo Park) — 20+ 사이클 핵심 컨트리뷰터 (rhwp-studio + WASM API + 디버깅 툴킷 영역)
base / head: devel / contrib/latex-command-compat
mergeStateStatus: BEHIND
mergeable: MERGEABLE — `git merge-tree` 충돌 0건
CI: ALL SUCCESS (Build & Test / Canvas visual diff / CodeQL / Analyze rust+js+py / WASM SKIPPED)
변경 규모: +735 / -11, 6 files (Rust 소스 only)
검토일: 2026-05-10
---

# PR #729 검토

## 1. 메타

| 항목 | 값 |
|------|-----|
| PR 번호 | #729 |
| 제목 | Task #143 — LaTeX 명령어 호환 확장 (2차) |
| 컨트리뷰터 | @oksure — **20+ 사이클** 핵심 컨트리뷰터 (PR #728 직전 처리 영역 동일 컨트리뷰터, 5/10 사이클 영역 영역 2번째 PR) |
| base / head | devel / contrib/latex-command-compat |
| mergeStateStatus | BEHIND, mergeable: MERGEABLE — 충돌 0건 |
| CI | ALL SUCCESS (Canvas visual diff 포함) |
| 변경 규모 | +735 / -11, 6 files |
| 커밋 수 | 7 (feat 3 + fix 2 + Task 2) |
| closes | #143 |

## 2. PR supersede 체인 — 패턴 (a) 변형

### 2.1 직전 PR
- **PR #563** (Task #143 1차, @cskwork) — `fix: LaTeX 분수 수식 미리보기 파싱 보정` — closed (merged 부재 영역 영역 일부 영역 영역 별도 머지)
- 본 환경 devel 영역 영역 `parse_latex_fraction` 1건 영역 만 존재 영역 — PR #563 영역 의 부분 머지 영역 영역 영역

### 2.2 본 PR (Task #143 2차)
- 별도 컨트리뷰터 영역 영역 supersede — PR #563 영역 의 후속 영역 영역 본격 LaTeX 호환 영역 대폭 확장
- Issue #143 영역 영역 closes 명시

## 3. 결함 본질 (Issue #143)

### 3.1 목표
LaTeX 수식 문법 (\frac, \sqrt, \int 등) 영역 영역 입력 지원 영역 — rhwp 영역 영역 hwpeq 파서 영역 영역 LaTeX 호환 영역 추가.

### 3.2 채택 접근 — 메인테이너 권고 정합
- **이슈 #143 의 원래 설계**: 듀얼 토크나이저 (별도 파일)
- **본 PR**: 메인테이너 권고 영역 영역 **기존 파서 확장** 방식 (`feedback_process_must_follow` 정합 — 신규 인프라 도입 부재)
- LaTeX `\command` 구문 영역 영역 기존 hwpeq 영역 영역 같은 파서 영역 영역 공존 영역 → 별도 모드 전환 부재

## 4. PR 의 정정 — 6 영역

### 4.1 `src/renderer/equation/symbols.rs` (+87/-5)

**FontStyleKind 확장 (5개 추가)**:
```rust
pub enum FontStyleKind {
    Roman, Italic, Bold,        // 기존
    Blackboard,    // \mathbb (ℝ, ℤ, ℕ)
    Calligraphy,   // \mathcal (ℒ, ℋ)
    Fraktur,       // \mathfrak
    SansSerif,     // \mathsf
    Monospace,     // \mathtt
}
```

기호 별칭 ~80개 추가:
- 관계/논리: `\leq, \geq, \neq, \equiv, \forall, \exists, \nabla`
- 화살표: `\rightarrow, \implies, \iff, \mapsto, \to, \gets, \longrightarrow`
- 연산자: `\cdot, \times, \div, \cup, \cap, \oplus, \otimes`
- 큰 연산자: `\sum, \prod, \int, \iint, \oint, \bigcup`
- 기호: `\infty, \partial, \emptyset, \dots, \perp, \aleph`
- 함수: `\sup, \inf, \lim, \limsup, \liminf, \Pr`
- 장식: `\overbrace, \underbrace, \overleftarrow`
- 괄호: `\langle/\rangle, \lceil/\rceil, \lfloor/\rfloor, \lvert/\rvert, \lVert/\rVert`

### 4.2 `src/renderer/equation/parser.rs` (+599/-3)

**구조 명령어**:
- `\frac{a}{b}, \dfrac, \tfrac` → hwpeq Fraction
- `\text{...}, \operatorname{...}` → FontStyle(Roman)
- `\binom{n}{k}` → Paren(Fraction)
- `\overset{over}{base}, \underset{under}{base}, \stackrel{over}{base}`
- `\phantom, \vphantom, \hphantom` → 공간 placeholder

**환경 파싱** (`\begin{env}...\end{env}`):
- `pmatrix, bmatrix, vmatrix, Bmatrix, Vmatrix` (matrix style)
- `cases, aligned, split, gather, gathered, array, smallmatrix`

신규 함수: `parse_latex_environment` / `parse_latex_env_matrix` / `parse_latex_env_cases` / `parse_latex_env_eqalign`.

### 4.3 `src/renderer/equation/tokenizer.rs` (+40/-0)

```rust
// LaTeX \\ (줄바꿈) — 두 백슬래시 → hwpeq # 행 구분자
if ch == '\\' && self.peek(1) == Some('\\') { ... }

// LaTeX spacing: \, \: \; \! → THINSPACE/MEDSPACE/THICKSPACE/NEGSPACE
// LaTeX escaped braces: \{ \} \| \#
```

### 4.4 3 렌더러 동기 정정 (+9/-3) — `feedback_image_renderer_paths_separate` 정합

`canvas_render.rs` / `svg_render.rs` / `skia/equation_conv.rs` 영역 영역 동일 패턴 영역 영역 exhaustive match 업데이트:
```rust
let (new_italic, new_bold) = match style {
    FontStyleKind::Roman | FontStyleKind::SansSerif | FontStyleKind::Monospace => (false, false),
    FontStyleKind::Italic => (true, bold),
    FontStyleKind::Bold => (italic, true),
    FontStyleKind::Blackboard => (false, true),
    FontStyleKind::Calligraphy | FontStyleKind::Fraktur => (false, false),
};
```

→ 3 렌더러 영역 영역 동기 정정 정합. fix commit `69b0b84d` 영역 영역 Skia renderer 영역 영역 누락 정정 (CI 빌드 실패 정정).

## 5. 회귀 가드 (PR 영역 영역 신규 41건)

`parser.rs` 테스트 함수 영역 영역 43 → **84** 영역 영역 신규 41건. 본 PR 영역 본문 영역 영역 "LaTeX 호환 테스트 37개 (latex_compat_tests 모듈)" 영역 영역.

신규 테스트 영역 영역 cluster:
- `test_latex_dfrac_tfrac` / `test_latex_mathrm/mathbf/mathbb/mathcal/mathfrak_mathsf_mathtt` (font style)
- `test_latex_text/operatorname` (text command)
- `test_latex_overline/underline/widehat/widetilde/overrightarrow/not` (decoration lowercase)
- `test_latex_quadratic_formula/binom` (구조 명령어)
- `test_hwpeq_not_regressed` (**hwpeq 영역 회귀 가드**)
- `test_latex_begin_pmatrix/bmatrix/cases/aligned/array/smallmatrix/split` (환경)
- `test_latex_backslash_backslash_tokenizes_as_newline` (\\\\)
- `test_latex_operatorname/spacing_quad/thin_space` (간격)
- `test_latex_rightarrow/implies/infty/nabla/leq_geq` (기호 별칭)
- `test_latex_phantom/overset/underset/stackrel` (구조)
- `test_latex_escaped_braces/langle_rangle` (괄호)

## 6. Copilot 리뷰 반영 (commit `0264fe18`)

- **Bold italic 보존** — 영역 영역 정정
- **\text 공백 제한 문서화** — 영역 영역 영역

## 7. 본 환경 점검

- merge-base: `60aeaa8d` (5/9 영역 매우 가까움)
- merge-tree 충돌: **0건** ✓
- 변경 영역 영역 격리: `src/renderer/equation/` 영역 (5 files) + `src/renderer/skia/equation_conv.rs` (1 file) — 다른 layout/render 경로 영역 영역 무관
- HWP 변환본 영역 영역 무관 (hwpeq 회귀 가드 `test_hwpeq_not_regressed` 신규)

## 8. 영향 범위

### 8.1 변경 영역
- 수식 렌더링 영역 영역 LaTeX 호환 명령어 (~80 기호 별칭 + 환경 파싱 + 구조 명령어)
- FontStyleKind 5개 추가 → 3 렌더러 동기 정합

### 8.2 무변경 영역
- 기존 hwpeq 문법 (회귀 가드 `test_hwpeq_not_regressed` 영역 영역 입증)
- 다른 layout/render 경로 (paragraph_layout / paint / wrap 등)
- HWP3/HWPX 변환본 영역 영역 시각 정합 (수식 컨트롤 영역 영역 만 영향)

### 8.3 위험 영역
- LaTeX `\command` 영역 영역 기존 hwpeq 영역 영역 토큰 충돌 영역 영역 — 3 렌더러 영역 동기 정정 영역 + 신규 회귀 가드 영역 영역 입증 가드
- Blackboard / Calligraphy / Fraktur / SansSerif / Monospace 영역 영역 시각 영역 — 폰트 영역 영역 직접 영역 변환 부재 영역, italic/bold 영역 영역 매핑 영역 영역 (fallback 영역 영역 정합)

## 9. UI 명시적 모드 토글 — 별건 (PR 본문 명시)

이슈 #143 영역 영역 의 "UI 명시적 모드 토글" 영역 영역 rhwp-studio (프론트엔드) 범위 영역 → 별도 이슈 영역 영역 분리 필요.

## 10. 충돌 / mergeable

- `git merge-tree --write-tree`: **CONFLICT 0건** ✓
- BEHIND — devel 영역 영역 5/10 사이클 영역 영역 진전, 본 PR 영역 영역 equation 모듈 격리 영역 영역 충돌 부재

## 11. 처리 옵션

### 옵션 A — 7 commits cherry-pick + no-ff merge (추천)

PR 영역 영역 7 commits 영역 영역 모두 본질 영역 (feat 3 + fix 2 + Task 2). 모두 cherry-pick — 영역 영역 영역 별 영역 매우 큰 영역 (parser.rs +599 LOC) 영역 영역 squash 영역 영역 의 손실 영역 영역.

```bash
git checkout -b local/task143 e2d12b78
git cherry-pick 04187512 d672f954 45fdb335 0264fe18 69b0b84d d96ddcc4 d53b7acf
git checkout local/devel
git merge --no-ff local/task143
```

→ **옵션 A 추천**.

## 12. 검증 게이트 (구현 단계 DoD)

### 자기 검증
- [ ] `cargo build --release` 통과
- [ ] `cargo test --release` — 전체 lib + 통합 ALL GREEN (신규 41건 PASS 보장)
- [ ] `cargo test --release src::renderer::equation` — equation 모듈 영역 영역 PASS
- [ ] `cargo test --release test_hwpeq_not_regressed` — **hwpeq 회귀 가드** 영역 영역 PASS
- [ ] `cargo clippy --release --all-targets -- -D warnings` (PR 영역 영역 영역 변경 파일 영역 영역 신규 lint 부재 보장)
- [ ] 광범위 sweep — 7 fixture / 170 페이지 회귀 0 (수식 컨트롤 영역 영역 만 영향 보장)

### 시각 판정 게이트 — **rhwp-studio 영역 영역 LaTeX 입력 영역 영역 인터랙션 검증 권장**

본 PR 영역 영역 의 본질 영역 영역 **수식 시각 출력**:
- WASM 빌드 후 dev server 영역 영역 LaTeX 명령어 입력 + 시각 정합 점검
- Canvas visual diff CI 영역 영역 SUCCESS 영역 영역 일차 영역 의 가드
- 신규 41 테스트 영역 영역 결정적 검증 (AST 정합) — 시각 본질 영역 영역 작업지시자 검증 권장
- E2E 자동 테스트 신규 영역 영역 부재

## 13. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @oksure 20+ 사이클 핵심 컨트리뷰터 (5/10 사이클 영역 영역 2번째 PR) |
| `feedback_image_renderer_paths_separate` | **권위 사례 강화** — 3 렌더러 (canvas/svg/skia) 영역 동기 정정 + Skia renderer 영역 영역 누락 정정 영역 fix commit `69b0b84d` 영역 영역 자기 진단 |
| `feedback_process_must_follow` | 메인테이너 권고 영역 영역 채택 (듀얼 토크나이저 → 기존 파서 확장) — 신규 인프라 도입 부재 영역 영역 위험 좁힘 + 후속 분리 (UI 모드 토글) 명시 |
| `feedback_visual_judgment_authority` | 수식 시각 출력 영역 영역 본질 — 작업지시자 인터랙션 검증 권장 |
| `feedback_pr_supersede_chain` | 패턴 변형 — PR #563 (1차, @cskwork closed) → PR #729 (2차, @oksure 본격 확장) 영역 영역 별도 컨트리뷰터 영역 의 supersede |

## 14. 처리 순서 (승인 후)

1. `local/devel` 영역 영역 7 commits cherry-pick (옵션 A)
2. 자기 검증 (cargo build/test/clippy + 광범위 sweep + 신규 41 PASS 점검 + hwpeq 회귀 가드 PASS)
3. WASM 빌드 + 작업지시자 인터랙션 검증 (rhwp-studio 영역 LaTeX 입력 + 시각 정합)
4. 인터랙션 검증 통과 → no-ff merge + push + archives 이동 + 5/10 orders 갱신
5. PR #729 close (closes #143 자동 정합)

---

작성: 2026-05-10
