# PR #6659, #6661, #6664 통합 시각 스윕

## 검증 기준

- 검토 브랜치: `review/jeong-sik-nondraft-20260903`
- 기준: `upstream/devel` `eb2ea3addfc84e1fb472311d8c3132fc245f674b`
- 메인터너 보정: `c89a7bf56d00acd465e18e4c50864434b64b83d4`
- PR별 검토 기록: `2d54b110ff109e9b1b2ff05e3c25062054765bfa`
- 실행 바이너리: `target/review-jeong-sik-nondraft-20260903/debug/rhwp`
- 도구: `scripts/visual_sweep.py`
- rasterizer: `webfont`
- 해상도: 96 DPI
- 비교 구성: 왼쪽 rhwp, 가운데 기준 PDF, 오른쪽 overlay

## 결과

| PR | 입력과 검토 페이지 | 기준 PDF | 자동 비교 | 사람 검토 | 안정 증적 |
|---|---|---|---|---|---|
| #6659 | `samples/hwpctl_ParameterSetID_Item_v1.2.hwp` p3 | `pdf/hwpctl_ParameterSetID_Item_v1.2-2022.pdf` p3 | pixel 95.22274%, ink/proxy 80.50311%, 자동 flag 없음 | 코드 블록과 푸터의 페이지 내 위치 및 문단 흐름이 유지된다. | [review_6659_hwpctl_p3.png](../assets/pr_6659_6664_jeong_sik_integration_20260903/review_6659_hwpctl_p3.png) |
| #6659 | `samples/exam_math.hwp` p3 | `pdf/exam_math-2022.pdf` p3 | pixel 97.99917%, ink/proxy 6.36794%, 자동 flag 없음 | 양단 구획, 세로 구분선, 문제 블록과 페이지 번호가 페이지 안에 유지된다. 글꼴 폭 차이로 세부 위치 차이는 남는다. | [review_6659_exam_math_p3.png](../assets/pr_6659_6664_jeong_sik_integration_20260903/review_6659_exam_math_p3.png) |
| #6661 | `samples/issue2004_cell_image_stack.hwpx` p4-p8 | `pdf/issue2004_cell_image_stack-2022.pdf` p4-p8 | pixel 78.82516-83.19475%, ink/proxy 19.16494-30.48460%, 5쪽 모두 자동 flag 없음 | p4의 배너와 인물 그림 적층 순서 및 본문 회피가 유지되고, p5-p8 후속 본문도 페이지 경계 안에서 이어진다. | [p4](../assets/pr_6659_6664_jeong_sik_integration_20260903/review_6661_issue2004_p4.png), [p6](../assets/pr_6659_6664_jeong_sik_integration_20260903/review_6661_issue2004_p6.png), [p4-p8 contact sheet](../assets/pr_6659_6664_jeong_sik_integration_20260903/review_6661_issue2004_p4_p8_contact_sheet.png) |
| #6664 | `samples/hwpx_sample2.hwp` p9 | `pdf/hwpx_sample2-2024.pdf` p9 | pixel 79.09768%, ink/proxy 12.49530%, 자동 flag 없음 | 상단 표, 안내 문단, 신청방법 상자와 하단 단계 도식의 순서가 유지되고 페이지 밖 이탈이 없다. | [review_6664_hwpx_sample2_p9.png](../assets/pr_6659_6664_jeong_sik_integration_20260903/review_6664_hwpx_sample2_p9.png) |

## 판정

- #6659, #6661, #6664의 변경 대상 구조는 현재 보정 head에서 기준 PDF와 같은 페이지 및 같은 읽기 순서에 남는다.
- #6661의 인라인 재분류 그림은 자체 가로 오프셋을 유지하며, 메인터너 보정은 그 오프셋을 synthetic stack에만 한정한다.
- 자동 비교의 낮은 ink/proxy 값은 글꼴 폭, 줄바꿈 및 raster 차이를 함께 반영하므로 단독 합격 기준으로 사용하지 않았다.
- #6663은 working 문서만 변경하므로 별도 시각 fixture 검증 대상이 아니다.
- 이 기록은 선택 페이지의 구조적 회귀 검증이며 모든 페이지의 픽셀 동일성을 주장하지 않는다.
- 관련 이슈 #6653, #6655, #6656의 종료 여부는 실제 병합과 post-merge 검증 뒤 결정한다.
