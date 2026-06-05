# PR #1101 처리 보고 — 글자겹침(hp:compose) 동그라미 글자 누락 + 한컴2024 시각 정합

## 1. 결정

**MERGE 수용** — 본질 해결 + 자동 검증 + 시각 검증 모두 통과.

| 항목 | 값 |
|------|-----|
| 번호 | #1101 |
| 작성자 | HaimLee-4869 (Lee eunjung) — 기존 컨트리뷰터 (8번째 PR) |
| 연결 이슈 | 없음 (closes 미지정) |
| 별도 발견 | #1126 (CanvasKit charOverlap 미지원 — 본 PR 영역 외) |
| 처리일 | 2026-05-25 |
| Merge commit | `3ad7f75f5ded1d2ea6b0990b42d106c5b4226e1b` |
| Merge 방식 | `gh pr merge 1101 --merge --admin` (BEHIND 영역 — 선행 사례 #251/#273 정합) |

## 2. 검증 결과

### 자동 검증 (통과)

| 항목 | 결과 |
|------|------|
| cherry-pick `687fba81` (검증 전용) | ✅ 충돌 없음 |
| `cargo build --release` | ✅ 통과 (2m 32s) |
| `cargo fmt --all -- --check` | ✅ 위반 0 |
| `cargo clippy --lib --release -- -D warnings` | ✅ warnings 0 (4m 52s) |
| `cargo test --release --tests` | ✅ svg_snapshot 8 passed / tab_cross_run 1 passed (전체 통과) |
| WASM 빌드 (Docker) | ✅ 성공 (4m 26s) |
| CI | ⚠️ 미실행 (fork 첫 푸시 패턴, 본 PR 결함 아님) |

> `cargo clippy --lib --tests` 영역의 60건 `unused_must_use` 위반은 devel 기존 결함 (본 PR 무관, 별도 영역).

### 시각 검증 — k-water-rfp.hwpx 13페이지 (통과)

작업지시자 제공 fixture 영역. 13페이지 안 3개의 `<hp:compose>` 영역:
- `circleType="SHAPE_REVERSAL_RECTANGLE"` (border_type=4, 반전 사각형)
- `charSz="-2"` (음수 영역 — PR 가설 영역)
- 속성 폼 `composeText="3"`/`"2"`/`"1"` (PR 정정 (1) 영역)

**PR 적용본 SVG (`output/poc/pr1101/round1/k-water-rfp_013.svg`)**:
```
<rect x="192.93" y="426.69" width="22.67" height="22.67" fill="#000000" stroke="#000000" stroke-width="0.8"/>
<text x="204.27" y="438.02" fill="#FFFFFF" font-family="..." font-size="18.13" ...>3</text>
... (2 / 1 영역 동일 구조)
```

- ✅ 3개의 반전 사각형 정상 출력 (검은 채움 + 흰 글자)
- ✅ 글자 누락 없음 — PR 정정 (1) `composeText` 속성 폼 파싱 작동
- ✅ `font-size = 18.13` — 원래 크기 22.66 × 0.80 (charSz=-2 → 0.80) 가설 정합

## 3. 변경 영역 요약

3 파일, +58/-28:

| 파일 | 영역 |
|------|------|
| `src/parser/hwpx/section.rs` (+3) | `parse_compose` 의 `composeText` 속성 폼 파싱 추가 |
| `src/renderer/svg.rs` (+30/-14) | `draw_char_overlap` + `draw_char_overlap_combined` — 테두리 색=글자색, 음수 charSz 영역 0.10 step 축소, 정원→타원 (rx=ry×0.85) |
| `src/renderer/web_canvas.rs` (+25/-14) | 동일 정정 영역 (4 함수 일관) |

메모리 룰 `feedback_image_renderer_paths_separate` 정합 — svg.rs + web_canvas.rs 4 함수 일관 정정.

## 4. 별도 발견 — #1126 등록

PR #1101 검토 중 발견. 본 PR 영역 외:

**증상**: `samples/hwpx/k-water-rfp.hwpx` 13페이지의 SHAPE_REVERSAL_RECTANGLE 글자겹침이 rhwp-studio (CanvasKit 렌더러) 에서 출력 안 됨. SVG/HWP 출력은 정상.

**원인**: `rhwp-studio/src/view/canvaskit-renderer.ts:289` 영역에서 `charOverlap` op 가 `unsupportedOps` 분기로 무시.

**이슈**: https://github.com/edwardkim/rhwp/issues/1126

## 5. 후속 권장 영역

| 항목 | 우선순위 | 영역 |
|------|---------|------|
| CanvasKit charOverlap 구현 | 본 PATCH 후속 | #1126 |
| 회귀 가드 svg_snapshot 추가 (k-water-rfp 13페이지 영역) | M100 후속 | 별도 영역 |
| 한컴 2020/2022 PDF 영역 시각 재검증 (정답지 등급) | M100 후속 | 권위 자료 영역 |
| 타원 0.85 비율 폰트 metric 영역 정밀화 | M200 영역 | 후속 |

## 6. 메모리 룰 정합

- ✅ `feedback_image_renderer_paths_separate` — svg.rs + web_canvas.rs 4 함수 sweep
- ✅ `feedback_visual_judgment_authority` — k-water-rfp 13페이지 시각 검증 게이트
- ✅ `feedback_check_open_prs_first` — 동일 컨트리뷰터 사이클 (#1088 회귀 거절) 영역 점검
- ⚠️ `feedback_pdf_not_authoritative` — 한컴2024 PDF 정답지 등급 외 (한컴 2020/2022 영역 재검증 후속 권장)
- ✅ `feedback_v076_regression_origin` — 시각 검증 게이트로 자동 검증 신뢰 회피
