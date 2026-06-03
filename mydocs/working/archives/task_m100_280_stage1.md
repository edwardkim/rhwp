# Task #280 단계 1 완료보고서 — 기준선 확보

## 목적

변경 전 상태의 SVG 출력과 한컴 PDF 출력을 스냅샷으로 확보하여, 단계 4 시각 비교의 근거로 삼는다.

## 수행 내역

### 1. 샘플 파일

재현용 샘플을 `samples/` 에 추가:

- `samples/equation-lim.hwp` — 제보 수식 (`lim _{h→0} {f(2+h)-f(2)} over h`)
- `samples/equation-lim.pdf` — 한컴 Office 에서 생성한 정답 (HyhwpEQ 폰트 서브셋 임베딩)

### 2. 변경 전 SVG 출력

```
./target/release/rhwp export-svg samples/equation-lim.hwp -o output/equation-lim/
```

결과: `output/equation-lim/equation-lim.svg` (3142 bytes)

폰트 스택 (현재):
```
'Latin Modern Math', 'STIX Two Math', 'Cambria Math', 'Pretendard', serif
```

### 3. 스냅샷 이미지

Chrome headless (deviceScaleFactor=2) 로 SVG 및 PDF 렌더링:

| 파일 | 내용 | 비고 |
|------|------|------|
| `before.svg` | 변경 전 SVG 원본 사본 | |
| `before.png` | SVG 를 Chrome 렌더링한 전체 뷰 | |
| `before_crop.png` | 수식 영역만 크롭 (260x80, CSS) | 핵심 비교 이미지 |
| `pdf.png` | Chrome PDF viewer 의 전체 뷰 | |
| `pdf_crop.png` | PDF 수식 영역만 크롭 (220x60, CSS) | 핵심 비교 이미지 |

## 비교 관찰

**before_crop.png** (현재 rhwp SVG):
- `lim` 글자 획이 굵음 (Cambria Math 매칭)
- `(` `)` 가 크고 얇은 SVG path 곡선
- 전체적으로 "굵고 약간 어색한" 인상

**pdf_crop.png** (한컴 PDF / HyhwpEQ):
- `lim` 및 변수 글자가 가는 클래식 세리프
- `(` `)` 가 폰트 글리프 (적절한 굵기·크기)
- 전체적으로 매끈하고 얇은 수학 타이포그래피

차이가 분명히 시각적으로 확인되며, 단계 3 에서 폰트 스택 재정렬 후 단계 4 에서 동일 크롭으로 after 이미지를 생성해 직접 비교 가능.

## 산출물

### 커밋 대상

- `samples/equation-lim.hwp`
- `samples/equation-lim.pdf`
- `mydocs/plans/task_m100_280.md` (수행계획서)
- `mydocs/plans/task_m100_280_impl.md` (구현계획서)
- `mydocs/working/task_m100_280_stage1.md` (이 문서)
- `mydocs/working/task_m100_280_stage1/` (스냅샷 디렉토리)
  - `before.svg`, `before.png`, `before_crop.png`
  - `pdf.png`, `pdf_crop.png`

### 커밋 대상 아님 (작업 중 생성)

- `output/equation-lim/equation-lim.svg` (재생성 가능)
- `rhwp-studio/render-stage1.mjs` (탐색용 스크립트, 삭제 예정)

## 완료 조건

- [x] 샘플 파일 2종 확보
- [x] 변경 전 SVG/PNG 스냅샷 생성
- [x] PDF 참조 이미지 생성
- [x] 시각 비교 기준 확립

## 다음 단계

단계 2: `canvas_render.rs` 영향도 조사 → Phase 1 범위 결정.
