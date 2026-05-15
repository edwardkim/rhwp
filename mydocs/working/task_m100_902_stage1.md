# Task #902 Stage 1 보고서 — 다중 sample WMF binary + PDF 추출

**Stage**: 1 / 5
**상태**: 정밀 측정 부분 가능 (PDF 의 WMF text 추출 한계) → Stage 2 결정 필요

## 1. 가용 자료 (HWP3 sample + PDF)

| Sample | HWP3 | HWP5 | HWPX | PDF | Pic 수 | 페이지 |
|--------|------|------|------|-----|-------|-------|
| sample14 | ✓ | ✓ | ✓ | ✓ | 13 | ? |
| sample16 | ✓ | ✓ | ✓ | ✓ | 7 | 62 |
| sample17 | ✓ | ✓ | ✓ | ✓ | 2 | 12 |
| sample18 | ✓ | ✓ | ✓ | ✓ | 6 | 69 |
| sample19 | ✓ | ✓ | ✓ | ✓ | 0 | - |

sample19 는 picture 없음. sample14 picture 13건 (대부분 raster?), sample16 picture 7건 (대형 4건 WMF diagram).

## 2. sample16 paragraph 394 WMF 정밀 측정

### 2.1 WMF binary

```
rec[0] SETMAPMODE mode=8 (MM_ANISOTROPIC)
rec[1] SETWINDOWEXT (y=72, x=56)    ← 단 1회 호출, 매우 작은 값
rec[2] SETWINDOWORG (y=4161, x=6333)
SETVIEWPORTEXT/ORG: 호출 안 됨
```

### 2.2 rhwp 현재 출력 (HWP3 → SVG)

- 외부 SVG: `<image x="76.69" y="298.99" width="608.48" height="411.89">` (px)
- WMF embedded SVG: `viewBox="0 0 6333 4212"` (Task #860 자동 확장)
- font-size: 42, 67, 75, 83, 100, 117 (viewBox 좌표)
- font-size 117 effective rendering = 117 / 4212 × 411.89 px = **11.44 px**
  - @ 96 DPI = 3.02mm ≈ **8.6 pt** 텍스트

### 2.3 PDF (한컴 viewer 2022 변환) 측정

- pdf/hwp3-sample16-hwp5-2022.pdf page 18 의 WMF picture 영역
- Page: 892×1261 px @ 108 DPI
- WMF picture 영역: top ~400~860, height ~460 px = ~108 mm (HWP3 dump 의 109mm 일치)
- **WMF picture 내부 텍스트 는 PDF 에서 raster vector graphics 로 렌더 — pdftohtml/pdftotext 추출 불가**
- 시각 추정: 한컴 viewer 의 WMF 텍스트는 **~10pt (3.3mm)** 추정

### 2.4 차이 추정

| 측정 | rhwp | 한컴 추정 | 차이 |
|------|------|----------|------|
| 텍스트 effective 크기 | ~8.6 pt | ~10 pt | ~16% |

## 3. WMF binary pattern 다중 sample 비교 (부분)

sample14 의 WMF picture (Task #860 fixture):
- viewBox 다양: 4231×1301, 4176×1512, 1997×816 등 — 모두 자동 확장
- 대부분 raster image (PNG embedded) — 텍스트 비교 제한적

sample17/18 WMF pictures: 일부 있으나 sample16 paragraph 394 와 유사한 텍스트 다이어그램 후보 추가 검사 필요.

## 4. PDF 측정 한계

PDF 의 WMF picture 가 raster vector graphics 로 렌더되어:
- `pdftotext`: 텍스트 추출 안됨 (WMF 영역)
- `pdftohtml -xml`: 텍스트 좌표/크기 추출 안됨 (WMF 영역)
- 가능 방법:
  - PDF 페이지를 고해상도 PNG 렌더 + 픽셀 측정 (시각 비교)
  - pdfimages 로 WMF 이미지 추출 + 분석
  - 작업지시자의 시각 정합 판정 (가장 권위 — 한컴 viewer 시각 비교)

## 5. ROOT CAUSE 재확정

WMF binary 의 `SetMapMode(MM_ANISOTROPIC) + SetWindowExt(56, 72) + 미사용 ViewportExt` 패턴에서:
- WMF spec 의 정의: MM_ANISOTROPIC 는 SetViewportExt 와 ratio 형성. ViewportExt 없으면 mapping 불완전.
- 한컴 사적 동작: 단순 ratio 또는 element bbox 기반 ratio 추정. **정확 동작 reverse engineering 필요**.

## 6. Stage 2 방향 후보

### α (이슈 본문): WMF SetMapMode + WindowExt + ViewportExt 의 정밀 ratio 처리
- WMF spec 기준 mapping + 한컴 사적 ratio 후보 (1, 6333/56=113.1, 4212/72=58.5 등) 적용
- Hancom 변환기 reverse engineering 또는 추정

### β (이슈 본문): viewBox 결정 알고리즘 변경 (Task #860 자동 확장 재검토)
- 자동 확장 비활성 + SetWindowExt 그대로 사용
- Task #860 sample14 fixture 회귀 위험

### γ (이슈 본문): font-size scale factor 추가 (한컴 사적 ratio)
- 단순 font-size × R 적용
- R 정확값 도출 필요

### ε (Stage 2 신규 후보): 작업지시자 시각 판정 기반 ratio 도출
- 작업지시자가 한컴 viewer 와 rhwp-studio 의 sample16 page 18 시각 비교
- "rhwp 가 X% 작음" 판정 → 그 비례로 R 적용
- 한컴 viewer 동작 reverse engineering 우회

## 7. Stage 2 결정 요청

- α: 정밀 reverse engineering (시간 매우 들음)
- β: viewBox 알고리즘 변경 (회귀 위험)
- γ: 추정 ratio 단순 적용 (정확값 부재)
- **ε**: 시각 판정 기반 ratio 권장 (timebox 내 최선)

## 8. 산출물

- 본 보고서: `mydocs/working/task_m100_902_stage1.md`
- /tmp/task902_svg/ — sample16 page 18 SVG + WMF SVG 디코드
- /tmp/task902_pdf/ — sample16 page 18 PDF 추출
- /tmp/task902_s14/ — sample14 SVG 전 페이지
