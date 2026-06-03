# Task #280 단계 4 완료보고서 — 시각 비교 검증

## 수행 내역

### 1. `samples/equation-lim.hwp` before/after/pdf 비교

| 파일 | 설명 |
|------|------|
| `task_m100_280_stage1/before_crop.png` | 변경 전 — Cambria Math 매칭, 굵은 획 |
| `task_m100_280_stage4/after_crop.png` | **변경 후 — Times New Roman 매칭, 얇은 세리프** |
| `task_m100_280_stage1/pdf_crop.png` | 한컴 PDF (HyhwpEQ) — 기준 |

#### 시각 관찰

**before_crop.png** (변경 전):
- `lim` 글자 획이 굵음 (Cambria Math 의 heavy stroke)
- `f`, `h` 이탤릭도 두꺼워 보임
- 전체적으로 "볼드 인상"

**after_crop.png** (변경 후):
- `lim` 글자가 얇은 클래식 세리프
- `f`, `h` 이탤릭이 Times New Roman 스타일로 얇음
- `(`, `)` SVG path 곡선은 그대로 (Phase 2 대상)
- PDF 와 훨씬 유사해짐

**pdf_crop.png** (기준):
- HyhwpEQ 전용 폰트, 가장 얇고 매끈함
- 독점 폰트라 완전 일치는 불가 — after 가 "충분히 근접" 목표 달성

### 2. 확인 체크리스트

- [x] `lim` 및 본문 글자가 가는 세리프로 바뀜 (Windows 기본 Times New Roman 매칭)
- [x] 전체 너비, 첨자 위치, 분수선 위치 **변화 없음** (레이아웃 계산은 불변)
- [x] 특수 기호(→, √, ∫, ∑, ∩, ∪) 가 브라우저 폴백으로 정상 표시됨
- [x] 다른 수식 샘플 깨짐 없음 (exam_math.hwp 페이지 5장 육안 확인)

### 3. `samples/exam_math.hwp` 회귀 검증

20페이지짜리 2025학년도 수능 수학 영역 샘플에서 5개 페이지 육안 검증:

| 페이지 | 수식 종류 (확인 내용) | 결과 |
|--------|----------------------|------|
| p001 | `f(x)=x³-8x+7`, `lim_{h→0} f(2+h)-f(2)/h`, 분수 `a₄/a₂+a₂/a₁=30`, piecewise | ✅ 전부 얇은 세리프, 레이아웃 변화 없음 |
| p005 | `f(1)=f(2)=0, f'(0)=-7`, 분수 `37/4~45/4`, `sin A : sin C = 8:5`, `18+15√3` | ✅ sin/sqrt 정상, 프라임(') 정상 |
| p009 | `(x³+2)₅`, `P(A\|B)`, `P(A∩B)`, `P(A∪B)`, 분수 `1/2, 3/5, 7/10, 4/5` | ✅ 집합 기호(∩, ∪) 브라우저 폴백 정상 |
| p013 | `lim_{x→0} 3x²/sin²x`, `∫₀¹⁰ (x+2)/(x+1) dx`, `10+ln N` | ✅ **`lim`·`sin`·`ln`·`∫` 모두 동일 크기**(함수명 1.2x 확대 없음 확인) |
| p017 | 벡터 `a⃗=(k,3), b⃗=(1,2)`, `a⃗+3b⃗=(6,9)` | ✅ 벡터 화살표(Decoration) 정상 |

`exam_math/` 서브디렉토리에 전체 20개 페이지 SVG 원본 보관 (디버깅용, 커밋 제외).

### 4. 핵심 확인: 함수명 크기 규칙

조사 단계에서 "함수명 1.2x 배율 적용은 HWP 일반 규칙 아님" 으로 판단했던 것을 p013 에서 직접 확인:
- `lim`, `sin`, `ln`, `∫` 모두 본문과 동일 크기로 렌더링됨
- 레이아웃 변경 없이 폰트만 얇아짐 → 의도한 결과

## 산출물

### 커밋 대상

- `mydocs/working/task_m100_280_stage4.md` (이 문서)
- `mydocs/working/task_m100_280_stage4/after.svg`
- `mydocs/working/task_m100_280_stage4/after.png`
- `mydocs/working/task_m100_280_stage4/after_crop.png`
- `mydocs/working/task_m100_280_stage4/exam_math_p{001,005,009,013,017}.png` (회귀 검증 증거)

### 커밋 제외

- `mydocs/working/task_m100_280_stage4/exam_math/` — 20개 원본 SVG (파일 과다, 필요 시 재생성 가능)
- `rhwp-studio/render-stage4.mjs`, `render-stage4-regression.mjs` — 탐색용 스크립트 (삭제 예정)

## 완료 조건

- [x] 변경 후 SVG/PNG 생성
- [x] before/after/pdf 3종 시각 비교 — "볼드 인상" 해소 확인
- [x] `exam_math.hwp` 회귀 (5개 페이지, 다양한 수식 종류)
- [x] 함수명(`lim`, `sin`, `ln`, `∫`) 크기가 본문과 동일한지 확인

## 다음 단계

단계 5: 최종 보고서 작성 + `mydocs/orders/20260424.md` 갱신 + Phase 2 후속 이슈 등록.
