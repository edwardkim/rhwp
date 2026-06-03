# Stage 1 완료 보고서 — Task #1224: 후보 폰트 실측 및 대체폰트 선정

- **이슈**: #1224 (M100)
- **브랜치**: `feature/issue-1224-font-fidelity`
- **단계**: Stage 1 / 4 (코드 무변경, 측정·선정)
- **작성일**: 2026-06-01

## 수행 내용

1. 측정 도구 구축: venv 에 fonttools 4.63 + Pillow(freetype) + numpy 설치, rsvg-convert/pdftoppm 활용.
2. 후보 OFL/무료 폰트 11종 확보 및 4가지 방법으로 실측(디자인 bbox / 렌더 높이 / 페이지 매칭 / 획 밀도).
3. **페이지 매칭 실측으로 근본 원인 재규명** + 후보 획 밀도 비교로 대체폰트 선정.

## 핵심 결과 (반전)

이슈 진단(em-fill 0.87 vs 0.67, "크고 두껍게")을 동일 스케일 고해상도로 재검증한 결과:

| | 잉크 높이 | 밀도(획 두께) |
|--|--|--|
| rhwp (Noto Sans CJK KR) | 17px | 0.378 |
| PDF (한컴 돋움) | 17px | 0.265 |

- **잉크 높이 동일(17=17)** → 크기/em-fill 차이는 사실상 없음. 이슈의 0.67 은 em=15px 저해상도 오차.
- **밀도 +43%** → 진짜 원인은 **Noto Sans CJK KR Regular 의 과도한 획 두께**. "두껍게" 진단은 정확.

→ 해결 방향: small-face 폰트 탐색이 아니라 **더 가벼운 weight 폰트로 폴백**.

## 선정 폰트

**Noto Sans KR Light (wght 300)**

- 획 밀도 예측 0.260 ≈ 목표 0.265 (편차 2%, 후보 중 최량)
- 현 폴백(Noto Sans CJK KR)과 동일 디자인 계열 → 형태 연속성·한컴돋움 중립 고딕 정합
- SIL OFL 1.1, 한글 11,172 완비, 가변폰트 wght=300 인스턴스화
- 차점: Gowun Dodum(0.273, 형태 괴리), Baekmuk Gulim(0.290, 품질 열위)

## 산출물

- `mydocs/tech/font_fidelity_measurement_1224.md` (측정 상세·전체 후보 표)
- `output/poc/task1224/` 시각 판정 자료:
  - `page_rhwp_noto.png` / `page_pdf_hancom.png` — 동일 스케일 6쪽 본문 줄 대조
  - `cand_compare.png` — 상위 후보 weight 시각 비교

## 검증

- 코드 무변경 단계 → 빌드/테스트 회귀 대상 없음.
- 측정 재현 가능: `/tmp/fonts1224/density.py`, `render_ink.py`, `emfill.py`.

## 다음 단계 제안

Stage 2 (폴백 체인) 진행 시, 선정 폰트를 **non-bold 돋움/고딕 본문**에 한해 매핑하고
임베딩(Stage 3)이 CI/Linux 결정적 충실도를 담당하는 설계로 진행.

## 승인 요청 사항

**선정 폰트(Noto Sans KR Light)와 "원인=획 두께" 재규명을 확정**해 주시면 Stage 2 착수.
