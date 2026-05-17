# Task #960 Stage 4 — 다중 sample 회귀 검증

## 1. cargo test --release --lib

**결과**: **1288 passed, 0 failed, 2 ignored**

## 2. 시험지 page 2 (Fix A 검증)

### Before fix
```
TAC_LINE pi=117 line_idx=1 ... run_tacs=[]               ← cases 누락
SVG cases formula: (402.5, 329.6) ← line 0 영역 (잘못)
```

### After fix
```
TAC_LINE pi=117 line_idx=1 ... run_tacs=[(7, 177.85, 3)] ✓
SVG cases formula: (510.5, 352.0) ✓ line 1 정상 위치
```

문14 layout 한컴 PDF 정합:
- 다항함수 f(x)에 대하여 함수 g(x)를 다음과 같이
- 정의한다.    g(x) = { x  (x<-1 or x>1) / f(x) (-1≤x≤1) }   ← cases 정상 위치
- 함수 h(x)=lim ... 에 대하여
- 보기에서 옳은 것만을...

## 3. 다중 sample LAYOUT_OVERFLOW 회귀 검증

전체 페이지 render + LAYOUT_OVERFLOW count 비교:

| Sample | Pre-Fix (upstream) | Post-Fix | 차이 |
|--------|-------------------|----------|------|
| exam_kor | 28 | 28 | 0 |
| exam_math | 0 | 0 | 0 |
| exam_eng | 13 | 13 | 0 |
| hwp3-sample14 | 0 | 0 | 0 |
| **Total** | **41** | **41** | **0** |

→ Fix A 가 추가 회귀 도입 안 함. 모든 LAYOUT_OVERFLOW 가 pre-existing.

## 4. 추가 발견 — Pre-existing 결함 (Fix A 무관)

본 Stage 검증 중 작업지시자 시각 확인으로 <보기> textbox 내부 content scramble 발견:
- 문14 의 <보기> 박스 (pi=118 InFrontOfText TAC 사각형 + 내부 글상자)
- ㄱ.ㄴ.ㄷ. prefix 누락 + inline 수식 위치 충돌

검증:
- Fix A 적용 전/후 보기 textbox 의 ㄱㄴㄷ chars 위치 + 수식 위치 **완전 동일**
- → Fix A 와 **무관 pre-existing bug**

→ 별도 issue [#962](https://github.com/edwardkim/rhwp/issues/962) 등록.

## 5. 평가

- 단위 검증 (page 2 cases formula): ✓
- cargo test 전체: ✓
- 다중 sample 시각 회귀: ✓ 0 (LAYOUT_OVERFLOW 차이 0)
- 보기 textbox: pre-existing (별도 issue)

→ Stage 5 진행 (commit + PR).
