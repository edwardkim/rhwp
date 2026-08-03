---
kind: verification
status: completed
canonical: mydocs/manual/verification/visual_sweep_guide.md
last_verified: 2026-08-02
---

# Task #3738 Stage 13 — HWP p30/p66/p68–p70 visual sweep

## 입력과 방법

- 입력: `samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp`
- 정답지: `pdf/pr3740/hwp/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구)-2020.pdf`
- renderer: `target/review-planet6897-20260802/release-test/rhwp`
- 방법: `scripts/visual_sweep.py --pages 30-32,66,68-70 --dpi 144`

요청한 7쪽은 모두 rasterized 되었고 missing page는 없다. SVG와 render tree의 문서 전체 산출은 각각
223쪽이다. 기준 PDF는 215쪽이므로, 이 선택 sweep은 해당 경계의 회귀 근거이지 전체 pagination 완료
근거가 아니다.

## 판정

| 쪽 | 확인 결과 |
| --- | --- |
| p30 | body tail의 마지막 유지 줄과 각주 29 `Dattani, Nikesh`가 같은 p30 하단에 있다. |
| p31–p32 | p31은 각주 29 없이 tail·`5. 독일`로 재개하고, p32의 그림 35는 보존된다. |
| p66 | native raster에서 표 본문과 각주 76·77의 ink 겹침은 보이지 않는다. 사용자 UI 재현과의 차이는 다음 Stage에서 별도 검증한다. |
| p68 | 그림 49와 `그림 49. OPTN 생존 장기기증 원칙` caption이 footnote top보다 10.3px 위에서 함께 끝난다. |
| p69 | 그림 49 caption 없이 `나. 생존 장기기증 승인 절차`가 시작한다. |
| p70 | 그림 49 caption 고아가 없다. |

overlay의 평균 pixel match는 91.57926%, 최저는 90.45733%이며, 평균 ink match는 22.50404%다. 폰트
raster 차이 때문에 이 수치를 문서 전체 정합률로 해석하지 않는다. p68의 `question_marker_flow_drift`
1건은 PDF red marker 4개와 rhwp marker 0개의 heuristic 차이로 발생했으며, frame overflow와
line-order overlap은 없고 review PNG에서 그림·caption·각주 물리 겹침도 없다.

## 증적

수정 전 p68·p69의 compare/overlay/review PNG 6장과 수정 후 p30, p66, p68–p70의 PNG 15장은
[`mydocs/pr/assets/pr_3740_issue3738_stage13/`](../pr/assets/pr_3740_issue3738_stage13/)에 보관한다.
원본 HWP/HWPX/PDF의 canonical 보관 위치는 [`pdf/pr3740/README.md`](../../pdf/pr3740/README.md)다.
