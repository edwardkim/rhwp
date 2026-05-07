# PR #626 1차 검토 보고서

## 1. PR 메타 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | #626 |
| 제목 | render: replay equations in native Skia PNG output |
| 컨트리뷰터 | @seo-rii (Seohyun Lee, me@seorii.page) — PR #599 직후 follow-up 컨트리뷰터 |
| base / head | `devel` ← `seo-rii:render-p5` |
| state / mergeable | OPEN / MERGEABLE / **BEHIND** (PR base 79 commits 뒤) |
| 변경 | 5 files, +854 / -5 |
| commits | 3 (`208bbc7` 본질 + `dd6333e` docs + `2dafca3` atop fix) |
| labels / milestone | 없음 / 미지정 |
| 연결 이슈 | **closes 키워드 부재** (Follow-up to #599) |
| 작성일 / 갱신 | 2026-05-06 00:53 / 01:42 |

### CI 상태 (모두 통과)
- Build & Test ✅
- Analyze (rust / python / javascript-typescript) ✅
- Canvas visual diff ✅
- CodeQL ✅
- WASM Build SKIPPED

### 댓글 영역
- 댓글 없음

---

## 2. 본 PR 의 본질 영역

### 본질
PR #599 (Issue #513 Supplementary PUA-A SVG 출력 + export-png CLI + VLM 게이트웨이) 의 follow-up 영역 — native Skia PNG 렌더링 경로의 수식 replay 영역. P4 까지 `PaintOp::Equation` 영역이 native Skia 경로에서 placeholder 박스로만 그려졌던 영역 → 실제 equation layout tree (`EquationNode.layout_box`) 영역의 Skia canvas 영역 직접 replay.

### 영향 범위
- `render_page_png_native` 영역
- `export-png` CLI 영역
- VLM/Vision 영역의 사용 흐름 — PR #599 도입 영역

### 변경 영역
| 파일 | +/- | 변경 |
|------|-----|------|
| `src/renderer/skia/equation_conv.rs` | +717 | **신규** — equation layout node → Skia draw call 변환 영역 |
| `src/renderer/skia/renderer.rs` | +130/-3 | `PaintOp::Equation` placeholder → render_equation 호출 영역 |
| `src/renderer/skia/mod.rs` | +1 | `equation_conv` 모듈 등록 |
| `README.md` | +3/-1 | P5 단계 명시 (native Skia equation replay) |
| `README_EN.md` | +3/-1 | 동일 영역 |

### 지원 영역 (PR 본문 명시)
- fraction (분수)
- atop (`{}` over `{}`)
- sqrt (제곱근)
- superscript / subscript
- matrix
- limit
- bracket / paren
- decoration (밑줄 / 위 줄 등)
- font style

### 미지원 영역 (P5 범위 외, 후속)
- raw-svg replay
- form replay
- native CanvasKit replay
- VLM preset 확장 (#613)
- PNG DPI metadata (#614)

---

## 3. 본 환경 정합 상태 점검

### 본 환경 devel 의 직전 영역
PR #599 + 후속 영역 (직전 commits):
```
2648bb4 Merge local/devel: CI fix — pr599_png_gateway example 의 native-skia feature 게이트
8ccb3ed fix(ci): pr599_png_gateway example 의 native-skia feature 게이트 추가
876d820 PR #599 후속 정정: Skia 한글 fallback + char-fallback + --font-path + VLM 옵션 + export-png CLI
a89e602 fix: cap native skia raster pixel count
1c1e0b7 docs: clarify native skia render path
0f2d2c0 feat: improve native skia image replay
```

### 본 환경 영역 (현재 영역 직접 확인)
- `src/renderer/skia/` 영역: `image_conv.rs` / `mod.rs` / `renderer.rs` 존재
- **`equation_conv.rs` 부재** — 본 PR 신규 추가 영역
- `renderer.rs:724` 영역: `PaintOp::Equation { bbox, .. } => draw_placeholder(*bbox, "equation"),` — placeholder 영역 잔존 영역, 본 PR 정정 영역의 본질 영역
- `Cargo.toml:45` 영역: `native-skia = ["dep:skia-safe"]` — opt-in feature 영역 정합

### 본 PR 영역의 정체성 영역
- **opt-in feature 영역** — 기본 빌드 영역 영향 부재
- PR #599 의 follow-up 영역 — 동일 컨트리뷰터의 P4 → P5 진행 영역
- `feedback_small_batch_release_strategy` 영역 정합 — 작은 단위 영역 회전 영역

---

## 4. 본 환경 cherry-pick simulation 결과

본 환경 임시 clone (`/tmp/pr626_test`) 에서 진행:

### cherry-pick
- 3 commits 모두 cherry-pick 통과 — **충돌 0** (auto-merge 없이 통과)
- 본 환경 devel 보다 79 commits 뒤 영역에서도 src 영역 충돌 부재

### 결정적 검증 결과 (모두 통과)

| 항목 | 결과 |
|------|------|
| `cargo build --release` (default) | ✅ |
| `cargo test --lib --release` (default) | ✅ **1155 passed** (회귀 0) |
| `cargo test --features native-skia skia --lib --release` | ✅ **22 passed** (PR 본문 명시 검증 영역 정합) |
| `cargo clippy --lib --release -- -D warnings` (default) | ✅ 0 |
| `cargo clippy --features native-skia --lib --release -- -D warnings` | ✅ 0 |
| `cargo check --target wasm32-unknown-unknown --lib` | ✅ 통과 |

### 신규 native-skia 테스트 영역 (PR 본문 명시)
22 tests 영역에서 신규 테스트 영역 확인:
- `renderer::skia::renderer::tests::renders_equation_layout_as_colored_ink` ✓
- `renderer::skia::renderer::tests::renders_atop_equation_layout_as_colored_ink` ✓ (2dafca3 영역의 fix 정합)

→ 수식이 placeholder 박스 영역 부재 + 실제 colored ink 영역 렌더 영역 검증.

### 본 PR 영역의 정합성
- PR #599 영역의 후속 영역 정합 (동일 컨트리뷰터, opt-in feature 영역 확장 영역)
- 본 환경의 직전 영역 (`876d820` PR #599 후속 정정 + `a89e602` cap raster pixel) 영역과 충돌 부재
- WASM 영역의 영향 부재 (native-skia feature 영역은 WASM 영역 외)

---

## 5. 옵션 분류

### 옵션 A — 전체 cherry-pick (3 commits 단계별 보존)
**진행 영역**:
```bash
git checkout local/devel
git cherry-pick 208bbc7 dd6333e 2dafca3
```

**장점**:
- src + docs + atop fix 모두 보존
- 3 commits 단계별 보존 → bisect 영역 정합
- 컨트리뷰터 author (seorii, me@seorii.page) 3 commits 모두 보존
- README 양쪽 (한국어 + 영어) 영역의 P5 단계 명시 영역 정합 — `feedback_external_docs_self_censor` 영역 정합 (외부 공개 문서 영역)

**잠재 위험**:
- 단계별 분리 영역의 미세 영역 — 단일 PR 의 본질 영역이 1 단위 (수식 replay) 영역에 가까운 영역

### 옵션 A-2 — squash 머지 (1 단일 commit)
**진행 영역**:
```bash
git checkout local/devel
git merge --squash local/pr626
git commit --author="seorii <me@seorii.page>" -m "render: replay equations in native Skia PNG output (P5)"
```

**장점**:
- 단일 commit 영역 정리 — devel 영역의 가독성 영역
- atop fix 영역 (마지막 commit) 의 영역 자연 흡수 영역

**옵션 A 와의 차이**:
| 항목 | 옵션 A | 옵션 A-2 |
|------|--------|---------|
| commit 수 | 3 (단계별 보존) | 1 (squash) |
| author 보존 | 3 commits 모두 seorii | 1 commit seorii |
| bisect 영역 | docs / 본질 / atop fix 분리 가능 | 단일 commit |
| 본 사이클 패턴 | PR #589/#599 (seo-rii 직전) 9 commits 보존 영역 | PR #642/#601 squash 영역 |

### 옵션 B — 부분 cherry-pick (본질 + atop fix, docs 분리)
**진행 영역**:
```bash
git checkout local/devel
git cherry-pick 208bbc7 2dafca3  # 본질 + atop fix
# docs (dd6333e) 는 본 환경 README 영역 검토 후 별도 영역
```

**잠재 위험**:
- 의미 영역 부재 — docs 영역도 PR 의 본질 영역의 일부 (P5 단계 명시 영역)

### 권장 영역 — 옵션 A (3 commits 단계별 보존)

**사유**:
1. **본 환경 결정적 검증 모두 통과** — cargo test 1155 (default) / 22 (native-skia 영역) / clippy 0 / WASM 영역 정합
2. **PR #599 의 후속 영역 정합** — 동일 컨트리뷰터의 P4 → P5 단계 진행 영역. PR #599 영역의 9 commits 단계별 보존 영역 패턴 정합
3. **opt-in feature 영역** — 기본 빌드 영역 영향 부재, 회귀 위험 영역 좁음
4. **신규 native-skia 테스트 영역 정합** — 수식 replay 영역의 회귀 차단 가드 영역 영구 보존
5. **README 양쪽 (한국어 + 영어) 영역 동기화** — 본 PR 영역의 docs commit 영역 정합

### 옵션 영역 요약 표

| 옵션 | 진행 가능 | 결정적 검증 | author 보존 | 권장 |
|------|----------|------------|-------------|------|
| **A** (전체 3 commits) | ✅ 충돌 0 | ✅ 1155/22/0 | 3 commits | ⭐ |
| **A-2** (squash) | ✅ 충돌 0 | ✅ 동일 | 1 commit | ⭐ |
| **B** (부분, docs 분리) | ✅ | ✅ | 2 commits | ❌ |

---

## 6. 잠정 결정

### 권장 결정
- **옵션 A 진행** — 3 commits 단계별 cherry-pick (PR #599 영역의 단계별 보존 영역 패턴 정합)
- 본 환경 결정적 검증 진행 (default + native-skia feature 영역) + WASM 영역 점검
- 시각 판정 영역 — `export-png` CLI 영역의 수식 PNG 출력 영역 작업지시자 시각 판정 영역

### 시각 판정 영역 자료 영역 후보
- `samples/exam_math.hwp` 영역 — 수식 영역 다수 영역
- `samples/exam_science.hwp` 영역 — 수식 영역
- `samples/eq-01.hwp` 영역 — 단순 수식 영역
- `samples/issue-505-equations.hwp` 영역 — 수식 영역 권위 영역

### 검증 영역 (옵션 A 진행 시 본 환경 직접 점검)
1. cherry-pick (3 commits) — simulation 영역 통과 영역 정합
2. `cargo test --lib --release` 1155 passed
3. `cargo test --features native-skia skia --lib --release` 22 passed (수식 replay 영역 정합)
4. `cargo clippy` (default + native-skia) 0
5. `cargo check --target wasm32-unknown-unknown --lib` 통과
6. **`cargo build --release --features native-skia --bin rhwp`** — export-png CLI 영역의 수식 replay 영역 시각 판정 자료 영역 영역 영역
7. `rhwp export-png samples/exam_math.hwp` 영역 — 수식 영역 PNG 영역 출력
8. **시각 판정 ★** — 수식 영역의 placeholder 박스 부재 + 실제 colored ink 영역 렌더 영역

---

## 7. 메모리 룰 관점

본 PR 검토에 적용되는 메모리 룰:
- `feedback_small_batch_release_strategy` — opt-in feature 영역 (native-skia) 의 작은 단위 영역 회전 영역 정합. P4 → P5 단계 영역 정합 — 본 PR 의 follow-up 영역 패턴 영역
- `feedback_assign_issue_before_work` — closes 키워드 부재 영역 (Follow-up to #599 영역으로 명시 영역, 별도 이슈 영역 부재 정합)
- `feedback_external_docs_self_censor` — README 양쪽 (한/영) 영역 동기화 영역 — 외부 공개 영역의 자기검열 영역 정합
- `feedback_pr_comment_tone` — PR 본문의 차분 + 사실 중심 영역 정합 (구체 회사명 비교 / 최상급 주장 부재)
- `project_dtp_identity` — DTP 엔진 영역의 native PNG 렌더링 영역의 본질 영역 강화 (수식 영역의 정합 영역 진전)

---

## 8. 다음 단계 (CLAUDE.md PR 처리 4단계)

1. (완료) PR 정보 + Issue 연결 + base/head + mergeable + CI 상태 확인
2. (현재) `pr_626_review.md` 작성 → 승인 요청
3. (필요 시) `pr_626_review_impl.md` 작성 → 승인 요청
4. 검증 (빌드/테스트/clippy + 시각 판정) + 판단 → `pr_626_report.md` 작성

### 작업지시자 결정 요청
1. **옵션 결정** — 옵션 A (3 commits 단계별, 권장) / A-2 (squash) / B (docs 분리)
2. **시각 판정 영역** — `export-png` CLI 영역의 수식 PNG 출력 영역 작업지시자 시각 판정 영역 진행 가/부
   - 시각 판정 자료 영역 후보: `samples/exam_math.hwp` / `eq-01.hwp` / `issue-505-equations.hwp` 영역
3. **native-skia 빌드 영역** — 본 환경 영역에서 `cargo build --release --features native-skia` 영역 진행 가/부 (기본 영역 영향 부재 영역, 시각 판정 자료 영역 생성 영역)

결정 후 본 환경 cherry-pick + 결정적 검증 + 시각 판정 + `pr_626_report.md` 작성.
