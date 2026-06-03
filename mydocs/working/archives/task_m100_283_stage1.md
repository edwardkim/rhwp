# Task #283 단계 1 완료보고서 — 기준선 + Times 글리프 실측

## 수행 내역

### 1. 현재 SVG 스냅샷

`samples/equation-lim.hwp` 를 Task #280 완료 코드(폰트 스택 재정렬 적용 후) 로 export:
- `current.svg` — 원본
- `current.png` — Chrome 렌더 (DSF=4)
- `current_paren_crop.png` — 파렌 영역 크롭 (f(2+h)-f(2) 부분)

### 2. Times New Roman `(` 글리프 실측

Chrome headless 에서 `Canvas.measureText` + `SVG.getBBox` 병행 측정:

| 항목 | 값 | em 비율 |
|------|-----|---------|
| advance width | **4.89px** | 0.333 |
| bbox width | 5.0px | 0.341 |
| bbox height | 14.0px | 0.955 |
| ascent (baseline 위) | 11.0px | 0.75 |
| descent (baseline 아래) | 3.0px | 0.20 |

Times 레퍼런스 `f(2+h)` 도 별도 SVG 로 렌더 → `times_reference.png` 생성.

### 3. 현재 rhwp 파렌 (path) 수치 분석

`current.svg` L13 의 첫 `(`:
```
M41.25,2.93 Q38.17,10.27 41.25,17.60
```

- 박스 할당 폭 `paren_w` = `fs * 0.3` = **4.40px**
- 시작/끝 x = 41.25 (mid_x + 0.2w)
- 제어점 x = 38.17 (박스 좌측 끝)
- 곡선 시각 폭 (midpoint x - start x) = **1.54px** (박스 할당의 35% 만 사용)
- 나머지 65% 는 시각적 whitespace

## 핵심 발견

| 비교 | Times 글리프 | rhwp path | 차이 |
|------|--------------|-----------|------|
| 곡선 시각 폭 | 5.0px | 1.54px | **-69%** (path 가 훨씬 얇음) |
| 박스/advance 점유 | 4.89px | 5.57px (paren_w + pad) | +14% (path 가 더 넓음) |
| 높이 | 14.0px | 14.67px | +5% (거의 같음) |

**문제의 본질**: 우리 path 파렌은 **할당 박스는 큼 + 실제 곡선은 얇음** → 박스 안에 곡선이 동떨어진 "그림자" 처럼 보임 + 글자와 gap 이 벌어져 어색. Times 글리프는 advance 폭 안에 곡선이 꽉 차 있음.

## 튜닝 방향 제안

측정 데이터가 **옵션 B (높이-조건부 글리프 전환)** 를 우세하게 지지:

- 텍스트 높이 파렌 (`f(2+h)` 등) → `<text>(</text>` 사용. Times advance 4.89px 안에 5px bbox 가 자연스럽게 들어감.
- 스트레치 파렌 (분수·sum 감쌈) → 기존 path 유지. 이 경우 높이가 fs 를 크게 초과하므로 path 의 유연성 필요.

옵션 A+C (path 튜닝) 만으로는 "advance 대비 작은 곡선" 구조적 제약 극복 어려움. 다만 **단계 2 에서 3 후보 모두 프로토타입** 하여 결정.

## 산출물

`mydocs/working/task_m100_283_stage1/`:
- `current.svg` (원본 SVG 사본)
- `current.png` (전체 렌더)
- `current_paren_crop.png` (파렌 영역 크롭)
- `times_reference.png` (Times 레퍼런스 `f(2+h)`)
- `glyph_metrics.json` (원시 측정 JSON)
- `metrics.md` (정리된 측정 분석)

## 완료 조건

- [x] 현재 파렌 수치 분석 (박스 4.40 / 곡선 1.54 / 여백 69%)
- [x] Times 글리프 실측 (advance 4.89 / bbox 5.0 / height 14)
- [x] 시각 비교 기준 이미지 확보
- [x] 단계 2 튜닝 후보 3안 도출

## 다음 단계

단계 2: 3 후보 프로토타입 (A+C 보수·강화, 옵션 B 글리프 전환) → 시각 비교 → 최종안 확정.
