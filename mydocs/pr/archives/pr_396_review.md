# PR #396 검토 — 수식 렌더링 개선 (TAC 높이 + 한글 이탤릭) (#174, #175)

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#396](https://github.com/edwardkim/rhwp/pull/396) |
| 작성자 | [@oksure](https://github.com/oksure) (Hyunwoo Park) |
| base / head | `devel` ← `oksure:contrib/equation-rendering-improvements-v2` |
| state | OPEN |
| mergeable | MERGEABLE |
| mergeStateStatus | BEHIND (자동 머지 검증 통과) |
| 변경 통계 | +90 / -11, 5 files |
| CI | 모두 SUCCESS |
| 이슈 | [#174](https://github.com/edwardkim/rhwp/issues/174), [#175](https://github.com/edwardkim/rhwp/issues/175) |
| 정황 | PR #388 의 devel 기반 재제출 (PR #387 → #395 와 같은 패턴) |

## 작성자 정황

@oksure — 신뢰 컨트리뷰터 (PR #334 / #335 / #395 머지 이력)

## 변경 내용

### 1. TAC 수식 높이에 HWP 권위값 사용 (#174)

`src/renderer/layout/paragraph_layout.rs` 의 인라인 수식 (TAC) 처리 2 곳:
- `eq.common.height` (HWP 저장값) 우선 사용
- layout_box.height 와 다르면 baseline 도 비례 조정

### 2. 수식 SVG 의 Y축 스케일링 (#174)

`src/renderer/svg.rs` 의 Equation 분기:
- 기존: `<g transform="scale(scale_x, 1)">` — X 만 스케일
- 변경: `<g transform="scale(scale_x, scale_y)">` — Y 도 스케일

### 3. CJK / 한글 이탤릭 제거 (#175)

`src/renderer/equation/{layout, svg_render, canvas_render}.rs`:
- CJK 텍스트 (\u{3000}-\u{9FFF}, \u{F900}-\u{FAFF}, \u{AC00}-\u{D7AF}) 감지
- italic 스타일링 + 너비 보정 비활성화

### 4. 단위 테스트 2 개

`equation/layout.rs::tests`:
- `test_cases_korean_no_overlap` — CASES 한글 행 겹침 방지
- `test_korean_text_width_not_italic` — 한글 너비가 라틴보다 큼

## 검증

### CI

모든 CI SUCCESS (Build & Test, CodeQL javascript / python / rust).

### dry-run merge

devel 위에 자동 머지 성공 (Task #418 + PR #395 영역과 인접하지만 분리 가능).

## Canvas 경로 시각 비교 발견 결함

작업지시자 시각 판정 중 web 에디터 (Canvas 경로) 가 SVG 와 다르게 표시되는 다중 결함 발견 — PR #396 의 직접 정정 대상은 아니지만 SVG 와 일관성을 위해 본 PR 후속 정정 진행:

### 결함 1: Canvas 분수선 y 좌표

- **위치**: `src/renderer/equation/canvas_render.rs::Fraction`
- **증상**: 분모와 분수선이 겹침
- **원인**: `line_y = y + lb.baseline` (SVG 는 `- fs * AXIS_HEIGHT`)
- **정정**: SVG 와 동일하게 `- fs * super::layout::AXIS_HEIGHT` 추가

### 결함 2: Canvas 수식 X/Y 스케일링 누락

- **위치**: `src/renderer/web_canvas.rs::Equation`
- **증상**: bbox 와 layout_box 크기 차이 시 수식이 정확한 영역에 안 그려짐
- **원인**: PR #396 의 SVG Y축 스케일 정정이 Canvas 경로에 미적용
- **정정**: `ctx.translate(bbox.x, bbox.y)` + `ctx.scale(scale_x, scale_y)` 추가

### 결함 3: Canvas Limit (lim) 폰트 크기

- **위치**: `src/renderer/equation/canvas_render.rs::Limit`
- **증상**: `lim` 글자가 다른 글자보다 1.5~2 배 크게 표시
- **원인**: `fi = font_size_from_box(lb, fs)` 가 lb.height 반환 — Limit 의 lb 는 "lim + 첨자" wrapper 라 base_fs 보다 큼
- **정정**: SVG 와 동일하게 `fi = fs`
- **다른 텍스트 분기는 정상** — LayoutBox 가 본인 텍스트만 포함해서 lb.height ≈ fs

## 평가

### 강점

1. **이슈 명확 해결** — #174 큰 수식 겹침, #175 CASES 한글 정상화
2. **신뢰 컨트리뷰터** (이력 PR #334/#335/#395)
3. **CI 통과** + 단위 테스트 2 개 추가
4. **devel 기반 재제출** — base 정정 안내 따름 (PR #388 → #396)
5. **SVG 경로 정정 정확** — slope/intercept 합성 형태로 수치 검증 가능

### 약점 / 점검 필요

1. **Canvas 경로 누락 정정 3 건** — 메인테이너 후속 commit 으로 정정
   - 분수선 y, 수식 스케일, Limit 폰트 크기
   - PR 작성자가 SVG 경로 위주로 작업 — Canvas 일관성은 후속

## 처리 결과 — 옵션 B (cherry-pick + 후속 정정)

### Stage 1: cherry-pick

`local/pr396` 브랜치 (`local/devel` 분기) 에서 PR head 까지 2 commit cherry-pick:

| commit | 작성자 | 내용 |
|--------|--------|------|
| `2e62ef0` (← `5cf2b79`) | @oksure | 수식 렌더링 개선 (TAC 높이 + 한글 이탤릭) |
| `9d4669e` (← `dbaa74b`) | @oksure | Copilot 리뷰 반영 (테스트 명시적 실패 + Canvas CJK 일관성) |

### Stage 2: 메인테이너 후속 정정

| commit | 내용 |
|--------|------|
| `7048d3e` | Canvas 경로 분수선 y 좌표 정정 (`- fs * AXIS_HEIGHT`) |
| `dff4b07` | web_canvas.rs Equation 스케일 적용 |
| `d47b7c7` | Canvas Limit `fi = fs` 정정 |

### Stage 3: 검증

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | ✅ **1031 passed** (1029 → +2 작성자 신규) |
| `cargo test --test svg_snapshot` | ✅ 6/6 |
| `cargo test --test issue_418` | ✅ 1/1 (Task #418 보존) |
| `cargo clippy --lib -- -D warnings` | ✅ warning 0 |
| `cargo check --target wasm32-unknown-unknown --lib` | ✅ |
| WASM 빌드 (Docker) | ✅ 4,106,811 bytes |
| 작업지시자 시각 판정 | ✅ 통과 |

## 다음 단계

1. `local/pr396` → `local/devel` → `devel` 머지 + push
2. PR #396 close + 작성자 댓글 (처리 결과 + Canvas 결함 정정 안내)

## 참고

- PR: [#396](https://github.com/edwardkim/rhwp/pull/396)
- 이전 PR: [#388](https://github.com/edwardkim/rhwp/pull/388) (CLOSED — base=main)
- 동일 작성자 PR: [#395](https://github.com/edwardkim/rhwp/pull/395) (그림 밝기/대비, 머지 완료)
- 이슈: [#174](https://github.com/edwardkim/rhwp/issues/174), [#175](https://github.com/edwardkim/rhwp/issues/175)
