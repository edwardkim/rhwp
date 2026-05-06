# PR #599 처리 보고서 — 핀셋 cherry-pick (9 commits) + 메인테이너 5개 후속 정정 + AI 파이프라인 + VLM 연동 도입

**PR**: [#599 render: add native Skia PNG raster backend](https://github.com/edwardkim/rhwp/pull/599)
**작성자**: @seo-rii
**관련**: refs #536 (멀티 렌더러 지원 트래킹 이슈, OPEN 유지 — P5+ 후속 단계 포함)
**선행 PR**: PR #498 (P3 — Canvas visual diff, CLOSED + cherry-pick 완료)
**처리 결정**: ✅ **commit 단위 cherry-pick (9 commits) + 메인테이너 후속 정정 + devel merge + push + PR close**
**처리일**: 2026-05-06

## 1. 처리 결과 요약

| 항목 | 결과 |
|------|------|
| 결정 | ✅ commit 단위 cherry-pick (9 commits) + 메인테이너 후속 정정 (5개 영역) + devel merge + push + PR close |
| 시각 판정 | ★ **통과** (SVG/PNG 한글 + 공백 정상 표시) |
| Devel merge commit | `f7d5563` |
| **PR mergeable** | UI MERGEABLE / 그러나 PR base 73 commits 뒤 (단순 머지 절대 금지, commit 단위 cherry-pick) |
| Cherry-pick 충돌 | 0 건 (Skia 본질 영역 본 사이클 처리분과 0 중첩) |
| Author 보존 | ✅ seo-rii (`me@seorii.page`) 보존 |
| Issue #536 (refs) | OPEN 유지 (멀티 렌더러 트래킹, P5+ 후속) |
| 광범위 페이지네이션 sweep | 164 fixture / 1,614 페이지 / 회귀 0 |

## 2. PR 의 본질 (P4 단계, refs #536)

`PageLayerTree` → native Skia replay → PNG 첫 raster backend:
- `LayerRasterRenderer` / `RasterRenderOptions` / `RasterRenderOutput` 추가
- `SkiaLayerRenderer` 신규 (`src/renderer/skia/`)
- `DocumentCore::render_page_png_native(page)` API
- `native-skia` feature gate (기본 빌드 영향 0)
- raster size guard (invalid dimension / scale / dpi reject + max pixel count)
- CI workflow 자동화

## 3. PR base skew + commit 단위 cherry-pick

### 3.1 Base skew

- PR base: `eaac8bd` (5/4 PR #563 후속)
- 본 devel HEAD: `7bce24d` (PR #593 후속, 73 commits ahead)
- **단순 머지 시 본 사이클 cherry-pick 모두 revert** (PR #571 패턴)

### 3.2 본질 commit 단위 cherry-pick 가능성

PR #599 의 9 commits 변경 영역 분석 — **본 사이클 처리분과 0 중첩**:

| 영역 | 본 PR | 본 사이클 처리분 | 중첩 |
|------|------|-----------------|---|
| `src/renderer/skia/` (신규) | ✅ | ❌ | 0 |
| `src/renderer/layer_renderer.rs` (신규) | ✅ | ❌ | 0 |
| `src/renderer/skia/image_conv.rs` (신규) | ✅ | ❌ | 0 |
| `Cargo.toml` (feature 추가) | ✅ | ❌ | 0 |
| `.github/workflows/ci.yml` | ✅ | ❌ | 0 |
| `README.md` / `README_EN.md` | ✅ | ❌ | 0 |
| `src/document_core/queries/rendering.rs` (API 추가) | ✅ | ❌ | 0 |

→ commit 단위 cherry-pick 시 충돌 0 → cherry-pick 가능 (PR #571 close 결정과 다른 케이스).

### 3.3 cherry-pick 결과

```
1c1e0b7 docs: clarify native skia render path
a89e602 fix: cap native skia raster pixel count
... (9 commits 모두 author seo-rii 보존)
```

## 4. 메인테이너 5개 후속 정정 (`876d820`)

PR #599 본질만으로는 한컴 fixture 가 정상 표시되지 않음:
- 한글 폰트 매칭 실패 → 사각형(豆腐) 표시
- 공백류 (NBSP/U+2007/U+200B) 두부 표시
- AI 파이프라인 / VLM 연동 옵션 미존재
- CLI 명령 미존재 (PR 본문 비목표 영역이지만 본 환경 사용 사례 정합)

### 4.1 Skia 한글 폰트 fallback chain (`renderer.rs`)

```rust
families.extend([
    "Noto Sans KR", "Noto Serif KR", "Noto Sans CJK KR", "Noto Serif CJK KR",
    "Nanum Gothic", "Nanum Myeongjo",
    "Malgun Gothic", "맑은 고딕",
    "Batang", "바탕",
    "Apple SD Gothic Neo", "AppleMyungjo",
    "DejaVu Sans", "Arial", "sans-serif",
]);
```

→ SVG 의 CSS font chain 과 동일 패턴.

### 4.2 `--font-path` 동적 폰트 로딩 (`with_font_paths` API)

`SkiaLayerRenderer::with_font_paths(&[PathBuf])` 추가:
- `custom_typefaces: HashMap<String, Typeface>` 캐시
- ttfs 디렉토리의 한컴 전용 폰트 (HY견명조 등) 동적 로드
- SVG 의 `--font-path` 와 동일 패턴

### 4.3 char 단위 fallback 렌더링 (공백 두부 정정)

```rust
for ch in text.chars() {
    let codepoint = ch as i32;
    let primary_has = primary_typeface.as_ref()
        .map(|tf| tf.unichar_to_glyph(codepoint) != 0)
        .unwrap_or(false);
    let chosen_font = if primary_has {
        &primary_font
    } else {
        // chain 에서 글리프 보유한 typeface 찾기, 모두 미보유 시 advance 만 진행
    };
    canvas.draw_str(&s, (cursor_x, y as f32), chosen_font, &paint);
    cursor_x += chosen_font.measure_str(&s, Some(&paint)).0;
}
```

→ NBSP / U+2007 / U+200B 등 whitespace-only 두부 방지.

### 4.4 VLM 옵션 (AI 파이프라인 + Vision-Language Model 연동)

`PngExportOptions` + `VlmTarget` 신규:

| 옵션 | 동작 |
|---|---|
| `--vlm-target claude` | Claude Vision 정합 (1568 longest edge / 1.15 MP) |
| `--scale <배율>` | 직접 배율 |
| `--max-dimension <픽셀>` | 한 변 한도, 자동 scale 계산 |

자동 scale 계산:
- max_dimension + max_pixels 결합 (둘 다 한도 안)
- 0.5% 안전 마진 (ceil + 부동소수점 오차)

### 4.5 `export-png` CLI + 매뉴얼

`src/main.rs::export_png` 추가 (native-skia feature gate). 매뉴얼:
- `mydocs/manual/export_png_command.md` (한글)
- `mydocs/eng/manual/export_png_command.md` (영문)

## 5. 결정적 검증 결과

| 게이트 | 결과 |
|--------|------|
| `cargo build --release` | ✅ Finished |
| `cargo build --release --features native-skia` | ✅ Finished |
| `cargo test --lib --release` | ✅ **1134 passed** / 0 failed / 2 ignored (회귀 0) |
| `cargo test --features native-skia skia` | ✅ **20 passed** |
| `cargo test --test svg_snapshot` | ✅ 6/6 passed |
| `cargo clippy --release --lib --features native-skia` | ✅ 0건 |
| Docker WASM 빌드 | ✅ **4,581,465 bytes** (PR #593 baseline +0 — feature gate 정합 입증) |

## 6. 광범위 페이지네이션 sweep

| 통계 | 결과 |
|------|------|
| 총 fixture | **164** (158 hwp + 6 hwpx) |
| 총 페이지 | **1,614** |
| Export 실패 | 0 |

→ Skia feature 가 default 빌드에 미포함이라 기본 SVG 경로 영향 0.

## 7. VLM 옵션 게이트웨이 검증

| 옵션 | 출력 dimension | pixel count |
|---|---|---|
| (기본) | 1123 × 1588 | 1.78 MP |
| `--scale 2.0` | 2246 × 3175 | 7.13 MP |
| `--scale 0.5` | 562 × 794 | 0.45 MP |
| `--max-dimension 1024` | 725 × 1024 | 0.74 MP |
| **`--vlm-target claude`** | **898 × 1269** | **1.14 MP ≤ 1.15 MP** ✓ |

## 8. 시각 판정 (★ 게이트)

### 8.1 작업지시자 시각 검증 결과

- **PNG 한글**: ★ 통과 (한글 fallback chain 적용 후 정상 표시)
- **PNG 공백**: ★ 통과 (char-fallback 적용 후 두부 표시 0)
- **VLM 옵션**: 5개 조합 모두 정상 동작
- **CLI**: `export-png --vlm-target claude` 등 정합

## 9. 후속 이슈 등록

본 사이클 비목표 영역 → 후속 task 분리:

- **[#613](https://github.com/edwardkim/rhwp/issues/613)** — VLM 프리셋 확장 (GPT-4V / Gemini / Qwen-VL / LLaVA)
- **[#614](https://github.com/edwardkim/rhwp/issues/614)** — DPI 메타데이터 옵션 (`--dpi` PNG pHYs chunk)

## 10. PR / Issue close 처리

### 10.1 PR #599 close
- 영문 댓글 등록 (cherry-pick 결과 + 5개 메인테이너 정정 영역 상세 + 후속 이슈 안내 + 컨트리뷰터 학습 기회 제공)
- close 처리

### 10.2 Issue #536 (refs)
- OPEN 유지 (멀티 렌더러 지원 트래킹, P5+ 후속 단계 포함)
- PR 본문이 `refs #536` 만 명시, `closes` 아님 → 정합

## 11. 메모리 정합

- ✅ `project_dtp_identity` — DTP 엔진 + 다층 레이어 / WebGPU / 마스터 페이지 인프라 토대 (M200+ 후보 B WebGPU 합리화 근거)
- ✅ `feedback_image_renderer_paths_separate` — SVG (`svg.rs`) / Canvas (`web_canvas.rs`) / **Skia (`skia/renderer.rs`)** 별도 image 함수, 시각 결함 정정 시 모든 경로 점검
- ✅ `feedback_visual_regression_grows` — 시각 판정 게이트 정합 운영 (★ 통과)
- ✅ `reference_font_path` — `/home/edward/mygithub/ttfs` 한컴 전용 폰트 위치 활용
- ✅ `feedback_per_task_pr_branch` — refs #536 P4 단계 단일 본질 PR 정합
- ✅ `feedback_pr_comment_tone` — close 댓글 차분/사실 중심 + 컨트리뷰터 학습 기회 제공
- ✅ `feedback_check_open_prs_first` — PR OPEN 상태 확인 후 진행
- ✅ `feedback_small_batch_release_strategy` — 본 사이클 (5/1 ~ 5/6) 누적 21번째 PR
- ✅ **PR #571 패턴과 다름** — base skew 동일 73 commits 이지만 본질 영역 0 중첩으로 cherry-pick 가능

## 12. 본 PR 의 우수성 — 본 사이클 가장 광범위 신규 영역 흡수

본 PR 의 처리 본질에서 가장 우수한 점:

1. **DTP 엔진 정체성 정합** — `project_dtp_identity` 메모리 룰 + multi-renderer support tracking
2. **commit 단위 cherry-pick 으로 base skew 우회** — PR #571 close 패턴과 다른 처리 (본질 영역 격리)
3. **AI 파이프라인 / VLM 연동 도입** — Claude Vision 프리셋 (`--vlm-target claude`) 으로 외부 사용 사례 확장
4. **`export-png` CLI 명령 추가** — 본 PR 비목표 영역이었으나 본 환경 사용 사례 (이슈 #608 `Publish releases` 정합 + Task #612 CLI binary release pipeline 의 추가 산출물)
5. **5개 영역 메인테이너 후속 정정** — 컨트리뷰터 영역 + 본 환경 정합 영역 분리 처리
6. **매뉴얼 한글 + 영문 동기화** — `mydocs/manual/` + `mydocs/eng/manual/`

## 13. 본 사이클 사후 처리

- [x] PR #599 close (cherry-pick 머지 + 메인테이너 후속 정정 + push)
- [x] 후속 이슈 #613 / #614 등록
- [x] 처리 보고서 (`mydocs/pr/archives/pr_599_report.md`, 본 문서)
- [ ] 검토 보고서 archives 이동 (`mydocs/pr/pr_599_review.md` → `mydocs/pr/archives/pr_599_review.md`)
- [ ] 5/5 또는 5/6 orders 갱신
