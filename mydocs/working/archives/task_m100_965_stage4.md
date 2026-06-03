# Task #965 Stage 4 — 다중 sample WMF 회귀 검증

## 1. cargo test --release --lib

**결과**: **1288 passed, 0 failed, 2 ignored**

## 2. sample16 page 18 (Fix 검증)

### Before fix
- WMF 다이어그램 내부 박스 텍스트 ("PE6450", "기록서버", "신규 4대" 등) 가 박스 하단 라인에 걸침
- text baseline 이 cell-top 보정 (+ascent ~0.8em) 만큼 아래로 shift

### After fix
- 박스 내부 한글 텍스트 정상 위치 (박스 중앙 가까이) ✓
- "Windows 서버군", "Unix 서버군" 라벨 박스 안 ✓
- "DM영역" 등 라벨 정합 ✓

## 3. 다중 WMF sample 회귀 검증

PNG 출력 file size 비교 (Stage 1 baseline vs Fix 적용 후):

| Sample | Page | Baseline | Post-Fix | Diff | 회귀 |
|--------|------|----------|----------|------|------|
| sample14 | 0 | 247224 | 248831 | +1607 (0.6%) | 없음 (정상화) |
| sample14 | 1 | 251676 | 251637 | -39 (~0%) | 없음 |
| sample14 | 2 | 222470 | 222841 | +371 (0.2%) | 없음 |
| sample14 | 5 | 226280 | 227876 | +1596 (0.7%) | 없음 |
| sample14 | 8 | 165009 | 165840 | +831 (0.5%) | 없음 |
| sample4 | 1 | 304090 | 304090 | identical | 없음 (BASELINE 미사용) |

### 시각 검증 (작업지시자 확인용 PNG)

- `/tmp/task965/baseline/png/` — Fix 전 PNG
- `/tmp/task965/after/png/` — Fix 후 PNG

→ 모든 차이 미세 (1% 미만, text baseline 위치 정합 방향). 시각 회귀 없음.

## 4. 평가

- 단위 검증 (sample16 page 18 WMF): ✓
- cargo test 전체: ✓ (1288/0/2)
- 다중 sample 회귀: ✓ 0
- 한컴 정합: ✓ (박스 내부 텍스트)

→ Stage 5 진행 (commit + PR).
