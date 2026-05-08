# PR #626 처리 보고서

## 1. 처리 결과

| 항목 | 값 |
|------|-----|
| PR | #626 — render: replay equations in native Skia PNG output |
| 컨트리뷰터 | @seo-rii (Seohyun Lee, me@seorii.page) — PR #599 직후 follow-up 컨트리뷰터 |
| 연결 이슈 | 부재 (Follow-up to #599, closes 키워드 부재) |
| 처리 옵션 | 옵션 A — 3 commits 단계별 cherry-pick |
| devel commits | `067a18b` (본질) + `dac1caa` (docs) + `2f6918a` (atop fix) |
| 처리 일자 | 2026-05-07 |

## 2. cherry-pick 결과

3 commits 단계별 보존 (author seorii, committer edward):

| Stage | hash | 변경 |
|-------|------|------|
| 본질 | `067a18b` | `src/renderer/skia/equation_conv.rs` 신규 (+717 LOC) + `renderer.rs` `PaintOp::Equation` placeholder → render_equation 호출 |
| docs | `dac1caa` | README 양쪽 (한국어 + 영어) P5 단계 명시 |
| atop fix | `2f6918a` | atop 수식 replay 정정 (`renders_atop_equation_layout_as_colored_ink` 테스트 정합) |

## 3. 본 환경 결정적 검증

| 항목 | 결과 |
|------|------|
| `cargo build --release` (default) | ✅ |
| `cargo test --lib --release` (default) | ✅ **1155 passed** (회귀 0) |
| `cargo build --release --features native-skia --bin rhwp` | ✅ (export-png CLI 영역) |
| `cargo test --features native-skia skia --lib --release` | ✅ **22 passed** (수식 replay 신규 테스트 포함) |
| `cargo clippy --lib --release -- -D warnings` (default) | ✅ 0 |
| `cargo clippy --features native-skia --lib --release -- -D warnings` | ✅ 0 |
| Docker WASM 빌드 | ✅ **4,578,641 bytes** (PR #622 baseline 동일 — native-skia 영역은 WASM 외) |

### 신규 native-skia 테스트 영역
- `renders_equation_layout_as_colored_ink` ✓
- `renders_atop_equation_layout_as_colored_ink` ✓

## 4. 본질 정정 영역

### 수식 layout tree → Skia draw call 변환 영역 (`equation_conv.rs` 신규 +717 LOC)

지원 영역 (PR 본문 명시):
- fraction (분수)
- atop (`{}` over `{}`)
- sqrt (제곱근)
- superscript / subscript
- matrix
- limit
- bracket / paren
- decoration (밑줄 / 위 줄 등)
- font style

### `PaintOp::Equation` placeholder → render_equation 호출 (`renderer.rs` +130 LOC)
```rust
PaintOp::Equation { bbox, equation } => {
    canvas.save();
    let scale_x = if equation.layout_box.width > 0.0 && bbox.width > 0.0 {
        bbox.width / equation.layout_box.width
    } else {
        1.0
    };
    if (scale_x - 1.0).abs() > 0.01 {
        canvas.translate((bbox.x as f32, bbox.y as f32));
        canvas.scale((scale_x as f32, 1.0));
        render_equation(canvas, &self.font_mgr, &equation.layout_box, 0.0, 0.0, equation.color, equation.font_size);
    } else {
        render_equation(canvas, &self.font_mgr, &equation.layout_box, bbox.x, bbox.y, equation.color, equation.font_size);
    }
    canvas.restore();
}
```

본질 — bbox 폭 차이 영역의 x축 scale 영역 정합 + 색상 + font size 반영.

### 미지원 영역 (P5 범위 외, 후속)
- raw-svg replay
- form replay
- native CanvasKit replay
- VLM preset 확장 (#613)
- PNG DPI metadata (#614)

## 5. 메인테이너 PNG 시각 판정 ★ 통과

권위 영역 — `samples/exam_math.hwp` (20 페이지) → `output/png/pr626_after/exam_math/` (1.5 MB):
- 수식 영역 (분수 / atop / sqrt / superscript / matrix / limit / bracket 등) placeholder 박스 부재 + 실제 colored ink 영역 렌더 영역 정합

작업지시자 평가: "png 시각판정 통과입니다."

## 6. devel 머지 + push

### 진행
1. `git cherry-pick 208bbc7 dd6333e 2dafca3` (3 commits)
2. 충돌 0 (`equation_conv.rs` 신규 영역 + `renderer.rs` auto-merge 통과)
3. devel ← local/devel ff merge
4. push: `8a8c4f1..2f6918a`

### 분기 처리
- 본 cherry-pick 시점 origin/devel 분기 0 — `feedback_release_sync_check` 정합

## 7. PR close

- PR #626: 한글 댓글 등록 + close (`gh pr close 626`)
- 연결 이슈 부재 (Follow-up to #599, closes 키워드 부재) — Issue close 영역 부재

## 8. 본 PR 의 정체성 영역

### opt-in feature 영역
- `native-skia` feature 영역 — 기본 빌드 영역 영향 부재
- WASM 영역의 영향 부재 (native-skia 는 WASM 영역 외)
- 본 환경 default cargo test 1155 passed (회귀 0) 정합

### PR #599 의 follow-up 영역
- 동일 컨트리뷰터의 P4 → P5 단계 진행 영역
- PR #599 (P4: PNG 게이트웨이 + Skia 한글 fallback + char-fallback + --font-path + VLM 옵션 + export-png CLI) → 본 PR (P5: 수식 replay)
- `feedback_small_batch_release_strategy` 영역 정합 — 작은 단위 영역 회전 영역

### DTP 엔진 영역의 native PNG 렌더링 영역 본질 영역 강화
- `project_dtp_identity` 영역 정합 — 수식 영역의 정합 영역 진전
- VLM/Vision 영역의 사용 흐름 영역 (PR #599 도입) 영역의 PNG 출력 품질 영역 보강

## 9. 메모리 룰 적용

- `feedback_small_batch_release_strategy` — opt-in feature 영역의 작은 단위 회전 영역 정합. P4 → P5 단계 영역 정합
- `feedback_external_docs_self_censor` — README 양쪽 (한/영) 영역 동기화 영역 정합 — 외부 공개 영역의 자기검열 영역 정합
- `feedback_pr_comment_tone` — PR 본문의 차분 + 사실 중심 영역 정합
- `project_dtp_identity` — DTP 엔진 영역의 native PNG 렌더링 영역 본질 영역 강화
- `reference_authoritative_hancom` — 작업지시자 PNG 시각 판정 영역 정합 (한컴 편집기 + `pdf/exam_math-2022.pdf` PR #670 영구 보존 영역 비교 영역)

## 10. 본 사이클 (5/7) PR 처리 누적 — **11건**

| # | PR | Task / Issue | 결과 |
|---|-----|--------------|------|
| 1 | PR #620 | Task #618 | 시각 판정 ★ + close |
| 2 | PR #642 | Task #598 | 시각 판정 ★ + close |
| 3 | PR #601 | Task #594 | 옵션 A-2 + close + Issue #652 신규 |
| 4 | PR #659 | Task #653 | 시각 판정 ★ + close |
| 5 | PR #602 | Issue #449 | close + Issue #449 reopen |
| 6 | PR #668 | Task #660 | 첫 PR + 시각 판정 ★ + close |
| 7 | PR #609 | Task #604 | 11 commits 단계별 + 시각 판정 ★ + close |
| 8 | PR #670 | (이슈 미연결) 한글 2022 PDF 199 | 메모리 룰 갱신 + close |
| 9 | PR #621 | Task #617 | 옵션 B + 시각 판정 ★ + close |
| 10 | PR #622 | Task #619 | 옵션 A + web editor 시각 판정 ★ + close |
| 11 | **PR #626** | (Follow-up to #599) 수식 replay | **옵션 A + PNG 시각 판정 ★ + close** |

본 PR 의 **opt-in feature 영역의 작은 단위 회전 + PR #599 의 follow-up 단계별 진행 + 신규 native-skia 테스트 영역 영구 보존 + 메인테이너 PNG 시각 판정 ★ 통과 + DTP 엔진 영역의 본질 영역 강화 패턴 모두 정합**.
