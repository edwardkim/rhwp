# Stage 4 완료 보고서 — Task #1224: 시각 검증·레이아웃 불변·문서화

- **이슈**: #1224 (M100)
- **브랜치**: `feature/issue-1224-font-fidelity`
- **단계**: Stage 4 / 4
- **작성일**: 2026-06-02

## 1. 레이아웃 불변 증빙 (회귀 없음)

base(`stream/devel`) vs current 바이너리로 `dump-pages -p 5` 출력을 **바이트 단위 비교**:

| 문서 | 결과 |
|------|------|
| 3-09월_교육_통합_2023 | 완전 동일 (147 lines) |
| 3-10월_교육_통합_2022 | 완전 동일 (54 lines) |
| 3-11월_실전_통합_2022 | 완전 동일 (135 lines) |

→ 렌더 폰트(weight)만 교체, **메트릭 DB·문단 배치·페이지네이션 무변경** 확정.

## 2. 다문서 시각/밀도 검증 (base=Noto vs cur=ExtraLight vs PDF)

| 문서 | base(Noto) | cur(ExtraLight) | PDF |
|------|-----------|-----------------|-----|
| 3-09월 (문26 본문 줄) | 0.378 | **0.277** | 0.265 |
| 3-10월 (좌측 본문 밴드) | 0.046 | 0.035 | (정렬차) |
| 3-11월 (좌측 본문 밴드) | 0.030 | **0.025** | 0.026 |

- 전 문서에서 **cur < base** 일관(ExtraLight 경량화 확인).
- 정렬 양호한 3-09·3-11 은 PDF 와 정합. 시각 자료: `output/poc/task1224/`
  (`final_before_after_pdf.png`, `weight_compare.png`, `multidoc_compare.png`).

## 3. 폰트 자산 정합성 수정 (Stage 3 보완)

검증 중 번들 ExtraLight 의 **typographic family 가 "Noto Sans KR"** 이라, fontconfig 가
일반 `"Noto Sans KR"` 요청까지 ExtraLight 로 가로채는 전역 부작용 발견(`fc-match` 검증).
→ family/typo family 를 **독립 `"Noto Sans KR ExtraLight"`(weight 400)** 로 재명명.
이후 `fc-match "Noto Sans KR"` 은 시스템 폰트로, ExtraLight 요청만 번들 폰트로 분리됨.

## 4. 문서화

- `tech/font_fallback_strategy.md` §11 추가: 메트릭↔글리프 시각 정합, 원인(획 두께) 측정,
  ExtraLight 채택, 적용 경로/한계(임베딩 cmap·family 명명), 유지보수 체크리스트.

## 5. 검증 종합

| 항목 | 결과 |
|------|------|
| 레이아웃 불변 | dump-pages 3문서 바이트 동일 ✓ |
| 시각 충실도 | 3-09 본문 0.378→0.277 ≈ PDF 0.265, 다문서 cur<base ✓ |
| `cargo test --lib` | 1494 passed, 0 failed |
| fontconfig 분리 | `"Noto Sans KR"` 섀도잉 해소 ✓ |

## 6. 산출물

- 최종 보고서: `report/task_m100_1224_report.md`
