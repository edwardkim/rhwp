# Task #283 단계 4 완료보고서 — 시각 비교 + 회귀 확인

## 수행 내역

### 1. `equation-lim.hwp` before/after/PDF 3면 비교

동일 crop 좌표 `(130, 145, 220, 55)` 로 재렌더:

| 판 | 출처 | 파렌 |
|----|------|------|
| BEFORE | 단계 1 스냅샷 (path) | 얇은 moon, 글자와 gap 벌어짐 |
| AFTER | 단계 3 구현 (glyph) | **본 파렌 형상, 글자와 자연 밀착** |
| PDF | `samples/equation-lim.pdf` 레퍼런스 | 자연스러운 Times 글리프 |

`compare.png` — 3면 합성 이미지.

**핵심 관찰**: AFTER ≈ PDF. BEFORE 와의 차이 뚜렷:
- 파렌 두께 복원 (얇은 moon → 본 bowl)
- 글자(`f`)와의 gap 제거 (약 1.3px 축소)
- 폰트 일관성 확보 (path 는 stroke=0.59px 로 얇은 선, glyph 는 Times 본연의 두께 변화)

### 2. `exam_math.hwp` 회귀 (스트레치 경로 보전)

20페이지 export → Chrome 렌더 → 4페이지 시각 확인.

| 페이지 | 관찰 | 결과 |
|--------|------|------|
| 1 (p001) | 표지, 문제 목록 | 이상 없음 |
| 5 (p005) | `f(1)=f(2)=0`, `f'(0)=-7`, `y=f(x)` 다수 글리프 파렌 | **글리프 렌더 정상** |
| 9 (p009) | 확률 수식 — `P(A\|B)` (글리프) / `P(A∩B)`·`P(A∪B)` (스트레치 path) | **분기 올바름** |
| 13 (p013) | 극한, 분수, 적분 — `lim x→0 3x²/sin²x`, `∫₀¹⁰ (x+2)/(x+1) dx` | **정상** |

**스트레치 파렌 (분수·합집합·교집합 감쌈) 은 기존 path 경로 그대로 작동** 확인. 임계치 `body.height ≤ fs * 1.2` 가 의도대로 분기.

### 3. 실제 SVG 검증

`samples/equation-lim.hwp` export:
- 단계 1: `<path>` 4개 (파렌 4곳 모두 path)
- 단계 4 (현재): `<path>` 0개, `<text>(</text>` 2개 + `<text>)</text>` 2개

## 산출물

`mydocs/working/task_m100_283_stage4/`:
- `before.svg` / `before.png` / `before_crop.png` — 단계 1 SVG 재렌더 (동일 crop)
- `after.svg` / `after.png` / `after_crop.png` — 단계 3 코드 출력
- `pdf.png` / `pdf_crop.png` — PDF 레퍼런스 (task #280 stage1 재사용)
- `compare.png` — 3면 비교 합성
- `exam_math_00{1,5,9,13}.png` — 회귀 확인 페이지

## 완료 조건

- [x] before/after/PDF 3면 시각 비교
- [x] AFTER 가 PDF 레퍼런스에 근접함 확인
- [x] exam_math.hwp 회귀 — 글리프/스트레치 분기 올바름
- [x] 스트레치 파렌 (분수·합집합) 기존 경로 보전 확인

## 다음 단계

단계 5: 최종 결과 보고서 + `mydocs/orders/20260424.md` 갱신 + 후속 이슈 판단 (기타 괄호 `{`·`[` 확장 여부).
