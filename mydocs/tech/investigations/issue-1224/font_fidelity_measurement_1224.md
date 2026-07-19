---
kind: investigation
status: historical
canonical: mydocs/tech/investigations/issue-1224/README.md
last_verified: 2026-07-16
---

# 폰트 충실도 실측 보고서 — Task #1224 Stage 1

작성일: 2026-06-01
대상 이슈: #1224 (한컴 돋움 미설치 시 폴백 글리프가 PDF보다 크고 두껍게 렌더)

> **명명 참고**: 구현계획서에서 `font_emfill_comparison_1224.md` 로 예고했으나, 실측 결과
> 근본 원인이 em-fill(크기)이 아니라 **획 두께(weight/density)** 로 판명되어 정확한 이름으로 작성한다.
>
> **후속 보완**: 본 Stage 1에서는 `Noto Sans KR Light (wght 300)`를 1차 후보로 선정했으나,
> Stage 3의 실제 rsvg 페이지밀도 검증에서 `Noto Sans KR ExtraLight (wght 200)`가
> PDF 기준에 더 근접함을 확인해 최종 구현 후보로 재확정했다.

## 1. 측정 목적

이슈는 "글리프가 PDF보다 **크고 두껍게**" 보이며 em-fill 0.87(rhwp/Noto) vs 0.67(PDF/한컴돋움)
으로 진단했다. 이 수치(em=15px 기준 13px vs 10px)를 **고해상도·매칭 스케일에서 재검증**하고,
한컴 돋움에 시각적으로 근접한 오픈소스 대체폰트를 선정한다.

## 2. 측정 방법

| 측정 | 도구 | 내용 |
|------|------|------|
| ① 디자인 bbox | fonttools BoundsPen | 폰트 glyf/CFF 의 글리프 잉크 박스 높이 ÷ unitsPerEm |
| ② 렌더 잉크 높이 | Pillow+freetype, 고정 em | 대표 한글 음절 래스터 후 잉크 행 높이 ÷ em |
| ③ **페이지 매칭 실측** | rsvg-convert + pdftoppm | rhwp SVG 와 PDF 를 **동일 페이지 픽셀폭(1240px)** 으로 렌더 후 동일 본문 줄 비교 |
| ④ 획 밀도 | Pillow+freetype | 잉크 픽셀수 ÷ 잉크 bbox 면적 (= 획 두께 비례) |

대상: `samples/3-09월_교육_통합_2023.hwp` 6쪽 / `pdf/3-09월_교육_통합_2023.pdf` 6쪽
(파일명 "2023" 은 한컴 버전이 아니라 **시험 연도**. 한글 2022 편집기 정답지 = 본 PDF).

## 3. 핵심 발견 — 원인은 크기가 아니라 **획 두께**

③ 페이지를 동일 스케일(둘 다 1240px 폭)로 렌더해 같은 본문 줄("…일 때, … 의 값은?")을 측정:

| | 잉크 높이 | 잉크 픽셀수 | 밀도(픽셀/면적) |
|--|--|--|--|
| rhwp (Noto Sans CJK KR) | **17px** | 289 | **0.378** |
| PDF (한컴 돋움) | **17px** | 198 | **0.265** |

- **잉크 높이는 동일(17px = 17px)** → em-fill 차이는 사실상 없다. 이슈의 0.87 vs 0.67 은
  em=15px 저해상도 측정 오차(1px = 0.067 편차)로, 실제 글리프 크기 차이가 아니다.
- **밀도는 0.378 vs 0.265 (rhwp +43%)** → Noto Sans CJK KR **Regular 의 획이 한컴 돋움보다
  훨씬 두껍다.** 이슈의 "두껍게" 진단은 정확하며, "크게" 는 두꺼움이 유발한 착시다.

→ **해결 방향 재정의: 더 가벼운 weight 의 오픈소스 폰트로 폴백.** (small-face 폰트 탐색 불요)

## 4. 후보 폰트 획 밀도 비교 (④, em=120px)

Noto Sans CJK KR(현 폴백) 밀도를 페이지 실측 0.378 에 고정하고, 각 후보의 상대비로
페이지 밀도를 예측. 목표 = PDF 한컴돋움 0.265.

| 폰트 | h/em | 밀도 | vs Noto | 예측 페이지밀도 | 판정 |
|------|------|------|---------|----------------|------|
| **Noto Sans KR Light (wght 300)** | 0.90 | 0.188 | 0.69 | **0.260** | ★ 거의 일치 |
| Gowun Dodum (Regular) | 0.94 | 0.198 | 0.72 | 0.273 | ★ 근접 |
| Baekmuk Gulim | 0.88 | 0.210 | 0.77 | 0.290 | 근접 |
| Pretendard Light | 0.87 | 0.227 | 0.83 | 0.312 | 보통 |
| Nanum Gothic | 0.93 | 0.239 | 0.87 | 0.330 | 약함 |
| UnDotum | 0.91 | 0.244 | 0.89 | 0.336 | 약함 |
| Noto Sans KR DemiLight(350) | 0.92 | 0.250 | 0.91 | 0.345 | 약함 |
| IBM Plex Sans KR | 0.92 | 0.251 | 0.92 | 0.347 | 약함 |
| Noto Sans CJK KR(현재) | 0.93 | 0.274 | 1.00 | 0.378 | ← 문제 수준 |
| Pretendard Regular | 0.88 | 0.276 | 1.00 | 0.380 | ← 문제 수준 |
| Spoqa Han Sans Neo | 0.91 | 0.274 | 1.00 | 0.378 | ← 문제 수준 |

### 참고 — 디자인 bbox(①)·렌더 높이(②)

| 폰트 | ① bbox h/em(평균) | ② 렌더 line h/em |
|------|----|----|
| Pretendard | 0.831 | 0.870 |
| Baekmuk Gulim | 0.832 | 0.875 |
| Noto Sans CJK KR | 0.866 | 0.915 |

→ **모든 오픈소스가 0.83~0.89 로 군집**, 한컴돋움의 0.67(이슈 수치)에 닿는 폰트는 없다.
이는 0.67 이 크기 지표가 아님을 역으로 입증한다(§3 결론과 정합).

## 5. 선정 — Noto Sans KR Light (wght 300)

> 후속 Stage 3에서 실제 페이지 렌더 밀도를 재측정한 결과 최종 구현은
> `Noto Sans KR ExtraLight (wght 200)`로 변경되었다. 본 절은 Stage 1 당시의 1차 후보
> 선정 근거를 보존하기 위한 기록이다.

| 기준 | 평가 |
|------|------|
| 획 밀도 정합 | **0.260 ≈ 목표 0.265** (편차 2%) — 후보 중 최량 |
| 형태 연속성 | 현 폴백(Noto Sans CJK KR)과 **동일 Noto 디자인 계열**, 중립 고딕 → 한컴돋움과 형태 일치 |
| 커버리지 | 한글 11,172 + 라틴 완비 |
| 라이선스 | SIL OFL 1.1 (재배포·서브셋·번들 허용) |
| 자산 | 가변폰트에서 wght=300 인스턴스화 가능 (정적 Light) |

**차점 Gowun Dodum**(0.273)은 weight 는 근접하나 부드러운 손글씨풍이라 한컴돋움의 중립
고딕과 형태 괴리. **Baekmuk Gulim**(0.290)은 구형 외곽선 품질·자간 열위. 따라서 형태+weight
종합 최량은 **Noto Sans KR Light**.

## 6. 다음 단계(Stage 2·3) 전달 사항

- 폴백/임베딩 모두 **돋움·고딕·맑은고딕 계열 "일반(non-bold)" 본문**을 Noto Sans KR Light 로
  매핑. Bold 런(`is_visually_bold`)은 기존 Bold 유지(별도 weight).
- 시스템 미설치 환경(CI/Linux)에서 결정적 충실도는 **임베딩(서브셋)** 이 담당. 폴백 체인 우선순위
  변경은 해당 폰트가 설치된 환경에서 효과.
- 레이아웃·메트릭 DB(advance) 무변경. 본 건은 렌더 글리프 weight 만 교정.

## 부록 — 측정 환경

- fonttools 4.63.0, Pillow(freetype), rsvg-convert, pdftoppm (Poppler), venv `/tmp/fonttools-venv`
- 후보 출처: Noto Sans KR(google/fonts OFL, 가변), Gowun Dodum(google/fonts OFL),
  Pretendard(orioncactus v1.3.9 OFL), Spoqa Han Sans Neo(v3.3.0), IBM Plex Sans KR(OFL),
  Nanum Gothic(OFL), Baekmuk(fonts-baekmuk), UnDotum(fonts-unfonts-core)
