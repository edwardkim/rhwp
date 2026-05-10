---
PR: #740
제목: Skia form control static replay (P7)
컨트리뷰터: @oksure (Hyunwoo Park) — 20+ 사이클 핵심 컨트리뷰터 (5/10 사이클 영역 영역 10번째 PR)
처리: 옵션 A — 4 commits cherry-pick + 자기 정정 + no-ff merge
처리일: 2026-05-10
머지 commit: db76b8dd
---

# PR #740 처리 보고서

## 1. 처리 결과

✅ **머지 완료** — 옵션 A (4 commits cherry-pick + 자기 정정 + no-ff merge `db76b8dd`)

| 항목 | 값 |
|------|-----|
| 머지 commit | `db76b8dd` (--no-ff merge) |
| Cherry-pick commits | `58b839a6`, `2889d9f4`, `a8093f27`, `85c05bcd` |
| 자기 정정 commit | `4be49daa` (impl 닫힘 brace 누락 추가) |
| Part of | #536 (멀티 렌더러 트래킹) |
| 시각 판정 | ✅ SVG 시각 판정 통과 (작업지시자) |
| 자기 검증 | native-skia 빌드 ✅ + 24/24 PASS + form-01.png 생성 |

## 2. 정정 본질 — `src/renderer/skia/renderer.rs` (+245/-2)

### 2.1 양식 개체 5종 정적 드로잉

native Skia 렌더러 영역 영역 placeholder → 실제 외형 변환:

| 타입 | 외형 |
|------|------|
| **PushButton** | 둥근 사각형(RRect) + 중앙 정렬 캡션 |
| **CheckBox** | 사각 체크박스 + 체크마크(V자, value≠0 시) + 캡션 |
| **RadioButton** | 원형 테두리 + 내부 점(value≠0 시) + 캡션 |
| **ComboBox** | 입력 필드 + 드롭다운 영역(삼각형 화살표) + 텍스트 |
| **Edit** | 입력 필드 + 텍스트 |

### 2.2 기술 사항
- `FormObjectNode` 의 `form_type`, `caption`, `text`, `value`, `fore_color`, `back_color` 활용
- CSS `#rrggbb` → Skia `Color` 변환 utility (`parse_css_color`)
- glyph 크기 bbox 높이 비례 자동 조정 (8~14px)
- 기존 `draw_placeholder()` 호출 → `draw_form_control()` 교체

### 2.3 CJK Fallback Chain (commit `85c05bcd`)

`make_form_font(size)` 메서드 영역 영역 한글 폰트 fallback:
1. `custom_typefaces` (사용자 지정)
2. `font_mgr.match_family_style` — Malgun Gothic, 맑은 고딕, NanumGothic, 나눔고딕, AppleGothic
3. `font_mgr.legacy_make_typeface` (legacy fallback)

`Font::default()` 5개소 영역 영역 `self.make_form_font()` 대체 — PushButton/CheckBox/RadioButton 캡션 + ComboBox/Edit 텍스트.

## 3. Copilot 리뷰 반영 (commit `2889d9f4`)
- RRect import 정합
- bg_color 일관 사용
- ComboBox/Edit text-only 렌더링

## 4. 자기 정정 (commit `4be49daa`)

### 4.1 결함 발견
CI 의 Build & Test job (`75178101688`) 영역 영역 native-skia tests 실패 (FAILURE) — 본 환경 영역 영역 동일 결함 재현:

```
error: this file contains an unclosed delimiter
    --> src/renderer/skia/renderer.rs:2061:3
 793 | impl SkiaLayerRenderer {
     |                        - unclosed delimiter
```

### 4.2 본질 진단
4번째 commit `85c05bcd` (CJK fallback) 영역 영역 standalone `fn draw_form_control` 영역 영역 `impl SkiaLayerRenderer { ... }` 의 method 영역 영역 변환됨. 그러나 `impl` 의 닫힘 brace `}` 가 누락 — `fn draw_form_control` 닫힘 (line 1024) 직후 `impl` 닫힘 brace 1개가 빠짐.

### 4.3 정정
line 1025 직전 영역 영역 `}` 1줄 추가.

## 5. 본 환경 검증

| 검증 | 결과 |
|------|------|
| `cherry-pick` (4 commits) | ✅ 충돌 0건 (auto-merge 정합) |
| 본 환경 자기 정정 (impl 닫힘 brace) | ✅ 1줄 추가 |
| `cargo build --release --features native-skia` | ✅ 통과 (31.44s) |
| `cargo test --release --features native-skia --lib skia` | ✅ **24/24 PASS** |
| `rhwp export-png samples/form-01.hwp` | ✅ `form-01.png` 13353 bytes |
| 작업지시자 SVG 시각 판정 | ✅ 통과 |

## 6. PR supersede 체인 — Issue #536 단계적 진전

| 단계 | PR | 컨트리뷰터 | 본질 |
|------|-----|----------|------|
| P4 | #599 | @seo-rii | PNG raster backend |
| P5 | #626 | @seo-rii | equation replay |
| P6 | #720 | @seo-rii | raw SVG fragment replay |
| **P7** | **#740** | @oksure | **form control static replay** |

`feedback_pr_supersede_chain` 권위 사례 강화 — Issue #536 단계적 진전.

## 7. 영향 범위

### 7.1 변경 영역
- native Skia PNG/VLM 경로 영역 영역 양식 개체 5종 실제 외형 (placeholder 대체)

### 7.2 무변경 영역
- WASM/CanvasKit form replay 별건 (P8+ 후보)
- form native replay (HTML form rendering 별 영역)
- 다른 PaintOp (Image, Equation, Path, Text 등)
- HWP3/HWPX 변환본 영역 영역 시각 정합

## 8. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @oksure 20+ 사이클 (5/10 사이클 영역 영역 PR #728/#729/#730/#734/#735/#737/#738/#739/#740 영역 10번째 PR) |
| `feedback_pr_supersede_chain` | **Issue #536 단계적 진전 권위 사례 강화** — P4 → P5 → P6 → **P7** |
| `feedback_image_renderer_paths_separate` | native Skia 경로 영역 영역 변경 — WASM/CanvasKit 영역 영역 무영향 |
| `feedback_visual_judgment_authority` | 작업지시자 SVG 시각 판정 ✅ 통과 |

## 9. 잔존 후속

- 본 PR 본질 정정 영역 영역 잔존 결함 부재
- Issue #536 OPEN 유지 (멀티 렌더러 트래킹) — P8+ 후보 영역 영역 가능 (WASM/CanvasKit form replay 등)

---

작성: 2026-05-10
