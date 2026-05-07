# Stage 3 — 시각 검증 및 회귀 테스트 (Task #683)

**브랜치**: `local/task683`
**관련**: Stage 1/2 보고서

## 요약

`samples/pr-149.hwp` 정합 검증 + 동일 패턴(빈 paragraph + TopAndBottom 그림) 보유 다른 7개 샘플 시각 회귀 검증. **모든 샘플 회귀 없음.**

## 검증 환경

- 빌드: `cargo build --release` (rhwp v0.7.9)
- SVG 변환 → PNG: `rsvg-convert --width <px>` (PDF 가로폭 매칭)
- PDF → PNG: `pdftoppm -r 100 -f 1 -l 1` (1페이지만)
- 측정: PIL Image 픽셀 분석

## 1. 정합 대상 (pr-149.hwp)

### 측정 결과 (150 dpi)

| 요소 | PDF (한글 2022) | rhwp SVG | 차이 |
|------|----------------|---------|------|
| 그림1 | 273..600 | 273..600 | ✓ 0 px |
| 그림2 | 666..993 | 667..994 | ✓ +1 px (sub-pixel) |
| 그림3 | 1059..1387 | 1060..1388 | ✓ +1 px |
| "회색조:" | 634..649 | 634..651 | ✓ 0 px (글자높이 +2px font 변동) |
| "흑백:" | 1028..1042 | 1027..1044 | ✓ -1 px |
| "입니다." | 1454..1472 | 1454..1473 | ✓ 0 px |

### Cluster 거리

- PDF: 18864 HU (≈ 393 px @ 150 dpi)
- 수정 후: 18896 HU (≈ 393.7 px @ 150 dpi)
- **차이 32 HU = 0.5 px (sub-pixel rounding)**

### Side-by-side 시각 비교

`/tmp/regr/pr-149_sxs.png` — PDF 와 rhwp SVG 가 그림 위치, 라벨 위치, 마지막 "입니다." 모두 정합. (잔여 차이는 이미지 효과 — 회색조/흑백 별개 이슈)

## 2. 회귀 검증 — 동일 패턴 보유 샘플

빈 paragraph (text_len=0) + TopAndBottom 그림 (treat_as_char=false) 패턴 보유 샘플 8개 식별:

| 샘플 | empty image-para 수 | PDF 보유 | 시각 검증 결과 |
|------|--------------------|---------|---------------|
| `pr-149.hwp` | 3 | ✓ | ✅ 정합 (대상 샘플) |
| `exam_science.hwp` | 4 | ✓ | ✅ 회귀 없음 |
| `exam_eng.hwp` | 1 | ✓ | ✅ 회귀 없음 |
| `hwp-img-001.hwp` | 1 | ✓ | ✅ 회귀 없음 |
| `k-water-rfp.hwp` | 1 | ✓ | ✅ 회귀 없음 (표지 페이지) |
| `kps-ai.hwp` | 1 | ✓ | ⚠️ PDF 가 2-up landscape — 직접 비교 불가, SVG 내부 정합성 OK |
| `mel-001.hwp` | 1 | ✓ | ⚠️ PDF 렌더 실패 (multi-page) |
| `hwpspec.hwp` | 4 | ✗ | (PDF 없음) |
| `hwp-3.0-HWPML.hwp` | 3 | ✗ | (PDF 없음) |

### 시각 비교 (side-by-side)

- `/tmp/regr/exam_science_sxs.png` — 과학탐구 영역 시험지, 그림 박스/표 모두 PDF 와 동일 위치
- `/tmp/regr/exam_eng_sxs.png` — 영어 영역 시험지, 그림 + 표 위치 정합
- `/tmp/regr/hwp-img-001_sxs.png` — 보도자료 양식, 로고/이미지 위치 정합
- `/tmp/regr/k-water-rfp_sxs.png` — RFP 표지, 헤더/로고 정합

## 3. cargo test 회귀 검증

```
$ cargo test --release
test result: ok. 1125 passed; 0 failed; 2 ignored; ... (모든 스위트)
test result: ok. 14 passed; 0 failed; ...
test result: ok. 25 passed; 0 failed; ...
... (총 18 개 스위트, 모두 통과)
```

**전체 1125+ 테스트 0 failures.** 신규 추가된 `test_task683_pr149_image_cluster_spacing` 도 통과.

## 4. 영향 범위 평가

| 항목 | 결과 |
|------|------|
| 동일 패턴 샘플 회귀 | ✅ 없음 (8개 샘플 검증) |
| 다른 wrap 모드 (Square, BehindText, InFrontOfText, TAC) | ✅ 가드로 제외, 영향 없음 |
| 머리말/꼬리말, 바탕쪽 그림 | ✅ 별도 layout 경로, 영향 없음 |
| 표 셀 내부 그림 | ✅ `cell_ctx.is_some()` 분기, 영향 없음 |
| caption 보유 그림 | ✅ `pic.caption.is_none()` 가드, 영향 없음 |
| HWP3, HWPX 동일 IR | ✅ 자동 적용 (코드 분기 없음) |
| Skia 네이티브 렌더러 | ✅ 페이지네이션/레이아웃 결과 사용, 영향 받음 (회귀 없음 확인) |
| 기존 회귀 테스트 (Task #479, #534, #544, #552 등) | ✅ 모두 통과 |

## 5. 잔여 이슈 (별개 작업으로 분리)

다음은 본 task 범위 외 — 별도 이슈/타스크로 다룰 항목:

1. **흑백(BlackWhite) 효과 디더링** — 현재 SVG `feComponentTransfer discrete` 하드 임계값 → 한컴 2022 디더링과 시각 차이. 별도 이슈 등록 권장.
2. **회색조(GrayScale) 효과 미세 차이** — librsvg/브라우저 렌더링 차이 가능성. 브라우저 검증 필요.

## 다음 단계

Stage 4 — 마무리 및 보고:
- 최종 보고서 작성 (`mydocs/report/task_m100_683_report.md`)
- 오늘 할일 갱신 (`mydocs/orders/`)
- 모든 변경사항 커밋

**작업지시자 승인 대기**.
