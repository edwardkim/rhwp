# Task M100 #1317 구현 계획서 — SVG 적분기호 path 렌더

> 수행계획서: `task_m100_1317.md` / 채택 방향: 2안 (SVG path 렌더) + SSOT bbox 공유

## 설계 개요

적분기호 ∫를 폰트 `<text>`가 아닌 **stroke path(Bézier S-curve)**로 그린다. 글리프 visual bbox를 **명시 상수로 정의**하고, 상·하한 attach point와 path를 **동일 bbox 기준**으로 산출한다. SVG/Canvas/Skia 3경로가 같은 path·attach point를 사용하여 폰트 독립적으로 완전 정합한다.

### 기준 패턴

- 기존 `Sqrt`가 `<path ... stroke>`로 √를 그림(`svg_render.rs:155-163`) → 적분기호도 동일 방식.
- `INTEGRAL_SCALE(2.5)`는 글리프 크기 제어 상수로 **유지**(path 그리기 영역 높이 = 글리프 박스 높이).

### 적분 글리프 path 형상

수직 S자: 상단 갈고리(우상향 곡선) → 수직 줄기(완만한 S) → 하단 갈고리(좌하향 곡선). 한컴 2022 ∫ 형태에 맞춰 cubic Bézier 2~3개로 구성, `stroke-width ≈ fs × (0.05~0.07)`.

## 단계별 구현 (4단계)

### Stage 1 — 적분 글리프 metric 모델 정의 + layout attach point 산출 (layout.rs)

- 적분 글리프 visual bbox metric을 명명 상수로 도입:
  - 글리프 박스 내 상·하 visual 여백, 줄기 좌표, 상·하 갈고리 가로 extent.
- `layout_subsup()` 적분 분기(`layout.rs:546-585`)의 매직 offset(`fs*0.21`/`fs*0.55`/`fs*0.13`/`-fs*0.42`)을 **glyph metric 기반 계산**으로 치환:
  - 상한: 상단 갈고리 끝점 부근
  - 하한: 하단 갈고리 끝점 부근
- 렌더 변경 없음(아직 `<text>`). layout 수치는 기존과 동등하거나 개선된 기준점으로 정리.
- **검증**: `cargo test` 전체 통과, `rhwp dump -s … -p …`로 적분 문단 LINE_SEG·box 좌표 정상.
- 커밋 + `task_m100_1317_stage1.md`.

### Stage 2 — SVG path 렌더 구현 (svg_render.rs)

- 적분 path `d` 생성 헬퍼 도입: `integral_path_d(box_x, box_y, box_w, box_h, fs) -> String` (Stage 1 metric 사용).
- ∫ `<text>` 출력 2곳을 path로 치환:
  - `MathSymbol(∫)` 분기(`svg_render.rs:97-111`)
  - `BigOp` 적분 분기(`svg_render.rs:242-249`)
- 상·하한은 기존 `render_box`로 layout 좌표대로 출력(Stage 1에서 정렬됨).
- **검증**: `rhwp export-svg samples/3-10월_교육_통합_2022.hwp -p 8` → 9페이지 적분 3개 상·하한 정합 (vs `pdf/` 9페이지). 11페이지(`-p 10`) 적분 정합.
- 커밋 + `task_m100_1317_stage2.md`.

### Stage 3 — Canvas/Skia 경로 정합 (canvas_render.rs, skia/equation_conv.rs)

- 동일 적분 path를 Canvas(`bezierCurveTo`)·Skia(`Path`)로 그려 3경로 글리프·attach point 완전 일치.
- 변경 전/후 비교로 기존 Canvas 양호 상태 회귀 없음 확인. ∑/∏(BIG_OP_SCALE)는 영향 없음 보장.
- **검증**: Skia PNG 렌더 9·11페이지 vs `pdf/`. ∑/∏ 포함 다른 수식 회귀 없음.
- 커밋 + `task_m100_1317_stage3.md`.

### Stage 4 — 종합 검증 + 시각 판정

- `cargo build --release` + `cargo test` 전체 통과.
- SVG/PNG 산출물 `output/poc/pr1314/`에 정리 → 작업지시자 시각 판정.
- IR/회귀: 적분 외 수식(∑/∏/분수/√) 무변화 확인.
- `task_m100_1317_report.md` 작성.

## 영향 파일 요약

| 파일 | Stage | 변경 |
|------|-------|------|
| `src/renderer/equation/layout.rs` | 1 | glyph metric 상수 + attach point 산출 |
| `src/renderer/equation/svg_render.rs` | 2 | ∫ `<text>` → path (2곳) + 헬퍼 |
| `src/renderer/equation/canvas_render.rs` | 3 | ∫ path 정합 |
| `src/renderer/skia/equation_conv.rs` | 3 | ∫ path 정합 |

## 리스크 & 대응

| 리스크 | 대응 |
|--------|------|
| Canvas 기존 양호 상태 회귀 | Stage 3 전/후 PNG 비교, path 미세조정 |
| ∑/∏ 등 BigOp 영향 | 적분 분기(`is_integral_symbol`)에만 path 적용, ∑/∏는 `<text>` 유지 |
| path 형상이 정답과 미세 불일치 | `pdf/` 9·11페이지 기준 Bézier 제어점 튜닝 (포맷 변경은 별도 커밋 분리) |
| 인라인 수식 가로 스케일(scale_x) 영향 | path 좌표도 `<g transform scale>` 안에서 그려지므로 자동 동반 스케일 — Stage 2에서 확인 |

## 비적용 (out of scope)

- HWP3 전용 분기 추가 금지 (공통 렌더러만 수정).
- `cargo fmt --all` 전체 실행 금지 (수정 파일만 정리).
- 적분 외 큰 연산자(∑/∏) 렌더 방식 변경.
