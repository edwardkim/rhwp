---
kind: reference
status: active
canonical: mydocs/manual/verification/visual_verification_governance.md
last_verified: 2026-07-16
---

# 한글 PDF baseline 인프라 (`tools/hangul_pdf_baseline.py`)

- 도입: #1658 / 목적: cut↔render↔한글 3자 줄높이 fidelity 의 **한글 권위 기준** 제공.
  한글이 그린 각 텍스트 줄의 baseline Y 를 PDF 에서 추출하고, rhwp render(SVG)와 줄 pitch 대조.

## 원리
- 한글 baseline: pyhwpx `save_as(PDF)`(또는 기존 PDF) → PyMuPDF(fitz) span origin.y. pt→px(×96/72).
- 다단 표 보정: 같은 행의 여러 셀 baseline 을 tol(3px) 로 **행 클러스터링** 후 행 pitch 계산.
- rhwp baseline: `rhwp export-svg` 의 `<text y>`.
- `--compare`: 페이지별 한글/rhwp 줄수·median pitch·pitch diff.

## 사용
```
python tools/hangul_pdf_baseline.py <file.hwp> [-o out.tsv]          # 한글 baseline
python tools/hangul_pdf_baseline.py <file.hwp> --compare --exe <rhwp>  # rhwp render 대조
python tools/hangul_pdf_baseline.py <file> --pdf pdf/xxx-2022.pdf --compare  # 권위 PDF 사용
```

## 중요 — 자동 생성 PDF 의 권위성 (PageCount 가드)
`save_as(PDF)` 는 문서의 **인쇄 설정(맞춰찍기/배율)** 을 적용할 수 있어 편집기 레이아웃과 다른
**압축 PDF** 를 만든다(예: 산업통상부 별표4 = 편집기 25쪽 vs 생성 PDF 13쪽, 배율 축소로 줄 pitch 과소).
→ 도구는 **생성 PDF 쪽수 ≠ 편집기 PageCount 시 경고**한다. 권위 비교에는:
- **`pdf/` 권위 PDF(한글 2022 편집기 내보내기)** 를 `--pdf` 로 지정(샘플), 또는
- 배율 100%·맞춰찍기 OFF 로 생성된 PDF 사용.
PageCount 일치(무경고) 시에만 baseline/pitch 를 권위로 신뢰한다.

## 검증·발견 (도입 시)
- **byeolpyo1**(국토부 별표1, 거대 셀 표): 생성 PDF 4쪽 = 편집기 4쪽(무경고, 권위).
  → **한글 줄 pitch 28.80px = rhwp 28.80px — 줄높이 drift 없음.** (이 문서는 cut 로직 문제였고
  #1658 행분할 수정으로 페이지수도 4=4 일치 완료.)
- **byeolpyo4**(산업통상부 별표4): 생성 PDF 13쪽 ≠ 25쪽(경고) → 인쇄 배율 축소로 비권위. pitch
  비교 불가(권위 PDF 또는 배율 100% 생성 필요).

## 시사점 (fidelity 재평가)
- 거대 표 문서(byeolpyo1)에서 **한글·rhwp 줄높이가 일치**한다는 것은, #1658 의 잔여 over/클리핑이
  **줄높이 fidelity drift 가 아니라 cut 로직/인쇄배율 아티팩트** 쪽임을 시사한다(앞선 가설 정정).
- byeolpyo4 의 정확한 진단은 **배율 100% 권위 PDF** 가 선행되어야 한다.

## 배율 100% 권위 PDF 생성 시도 — 차단 (탐색 결과)
byeolpyo4 의 권위 PDF(편집기 25쪽) 자동 생성을 시도했으나 차단:
- `hwp.HParameterSet.HPrint`: `ZoomX/Y=100`(배율 100%)이나 `PrintMethod=4`(맞춰찍기류 추정).
- `save_as(path, format="PDF")` 는 **HPrint 설정을 반영하지 않고**, doc 저장 배율을 적용해 일관되게
  **13쪽**(≈ 50% area, pitch≈14px) 생성. `PrintMethod=0` 설정 후 `Execute("Print")` 도 save_as 에
  영향 없음. `save_as(arg=...)` 의 PDF 옵션 키는 미문서화.
- → 탐색한 pyhwpx/COM API 로 PDF export 배율 100% 강제 불가. (또는 한글 자체의 편집기↔PDF export
  레이아웃 불일치 가능성.)

## 결론·후속
- **권위 PDF 자동 생성은 본 환경 API 로 차단.** 신뢰 비교는 (a) `pdf/` 권위 PDF(샘플) `--pdf`, 또는
  (b) save_as 가 PageCount 와 일치하는 문서(무경고)에 한정.
- **확정 발견(byeolpyo1, 무경고 권위)**: 한글·rhwp 줄높이 일치(28.8=28.8) → #1658 잔여(별표4 Δ+3,
  클리핑)는 **줄높이 fidelity 가 아니라 cut 로직/한글 PDF 배율 아티팩트** 쪽(가설 정정).
- 후속 정공법: 한글 PDF export 배율 100% 강제 방법(HWP COM PDF 액션 파라미터 추가 연구) 또는
  pdf/ 권위 PDF 확보 후 per-행 baseline diff.
