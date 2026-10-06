# Task M100 #1317 Stage 2 완료보고서 — SVG 적분기호 path 렌더

## 목표

SVG export에서 적분기호 ∫를 폰트 `<text>`가 아닌 **stroke path**로 렌더하여, 폰트 미임베딩 환경에서도 글리프 visual bbox를 결정적으로 만들고 상·하한과 정합한다.

## 변경 내용 (`src/renderer/equation/svg_render.rs`)

### 1. `integral_path(x, y, fs, color)` 헬퍼 신설

- Stage 1의 `integral_geom`을 사용해 글리프 박스 좌상단 기준 S-곡선 path 생성.
- 하단 갈고리 끝(좌하) → 상단 갈고리 끝(우상) 단일 cubic Bézier, `stroke-linecap="round"`로 갈고리 표현.
- 폰트 비의존 → SVG 뷰어/환경 무관 동일 bbox 보장.

### 2. `MathSymbol(∫)` 분기 (live 경로)

- `is_integral_symbol`이면 `integral_path()` 출력, 아니면 기존 `<text>` 유지.

### 3. `BigOp` 적분 분기 (정합용)

- 적분일 때 `<text>` → `integral_path()`로 치환 (∑/∏는 `<text>` 유지).

## 검증 (정답: `pdf/3-10월_교육_통합_2022.pdf` 9페이지)

`rhwp export-svg samples/3-10월_교육_통합_2022.hwp -p 8` → rsvg PNG 변환 비교:

| 적분 | 결과 |
|------|------|
| `∫_0^2 (2x³+3x²)dx` (좌단 문2) | 상한 "2" 상단 우측, 하한 "0" 하단 밀착 — PDF 정합 ✓ |
| `∫_0^4 {g(x)-f(x)}dx` (우단 문) | 정합 ✓ |
| `2∫_0^2 (-2x²+6x)dx` (우단 문) | 상·하한 줄기 밀착, PDF 정합 ✓ |

**수정 전(보류 사유)**: rsvg 등 폰트 대체 환경에서 상·하한이 위아래로 벌어지고 글리프에서 떨어져 보임.
**수정 후**: path glyph로 결정적 bbox 확보, 상·하한이 PDF와 동일하게 줄기에 밀착.

PDF↔NEW SVG 나란히 비교: `output/poc/pr1314/cmp_int.png`.

## 검증 결과

- `cargo build --release` 성공.
- `cargo test` 전체 통과 (snapshot 회귀 없음).

## 비고

Canvas/Skia 경로는 Stage 3에서 동일 path로 정합 예정.
