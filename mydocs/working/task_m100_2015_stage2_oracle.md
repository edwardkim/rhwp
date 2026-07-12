# #2015 시각 오라클 구축 (Windows/래스터라이저 부재 환경)

- 이슈: #2015 / 브랜치: `fix/2015-saved-bounds-rowbreak-overflow`
- 목적: 이 환경에 `rsvg-convert`/`pdftoppm` 부재 → visual sweep 불가 문제 해소.
  Stage 3 엔진 수정의 **픽셀 검증(before/after)** 을 가능케 한다.

## 구성 (자립형, 시스템 바이너리 불필요)

`scripts/visual_oracle_native.py`

| 축 | 수단 | 비고 |
|---|---|---|
| rhwp 래스터 | `rhwp export-png`(native-skia) | 네이티브 렌더(프로덕션 경로). `--features native-skia` 빌드 필요 |
| PDF 래스터 | PyMuPDF(fitz) | poppler 불필요, 이미 설치됨 |
| 비교/오버레이 | PIL + numpy | task1274_visual_sweep.py 시맨틱 정합 |

- 96 DPI 기준: A4 → 794×1122~1123px (rhwp/PDF 정합, ±1px 패딩).
- 산출: `rhwp_png/ pdf_png/ overlay/ review/ metrics.json`.
- overlay 색: 빨강=rhwp-only 잉크, 파랑=PDF-only, 주황=양쪽 위치차, 회색=일치.
- proxy = 잉크영역 존재 시 `ink_match`, 없으면 `pixel_match`.

## 실행

```bash
python scripts/visual_oracle_native.py \
  --hwp samples/task1749/saved_bounds_cumulative_page_break.hwpx \
  --pdf samples/task1749/saved_bounds_cumulative_page_break-2024.pdf \
  --pages 4,5 --dpi 96 --out output/poc/i2015/oracle_baseline
```

## 기준선 (base=origin/devel, 수정 전)

| page | pixel_match | ink_match | proxy | ink_union | diff |
|---|---|---|---|---|---|
| p4 | 90.69% | **13.87%** | 13.87% | 96343px | 82985px |
| p5 | 90.38% | **11.62%** | 11.62% | 97020px | 85746px |

- PR #1887 보고값(9.64%/5.10%)과 계열 동일. 절대값 차이는 렌더 경로 차이
  (native-skia PNG vs rsvg SVG→PNG, PyMuPDF vs pdftoppm, ink cutoff)이며,
  **before/after 내부 비교용 지표로 일관**되면 충분하다.
- overlay(`review_004.png`) 시각 확인: 상단 정합→하단 발산의 누적 세로 드리프트가
  PR 커밋 review 이미지와 동일하게 재현됨(부동 표 앵커 이중계상 결과).

## 용도

Stage 3 에서 `vert_offset` 이중계상 수정 후 동일 명령 재실행 → `ink_match` 상승·overlay
드리프트 감소를 픽셀로 확인. 최종 한컴 편집기 시각 판정의 대체가 아니라 1차 자동 게이트.
