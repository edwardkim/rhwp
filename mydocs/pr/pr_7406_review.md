---
kind: snapshot
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-26
---

# PR #7406 리뷰 — KoPub 라틴 전진폭

## 최종 판정

**머지 보류.** 메인터너 보정으로 PrEP 본문/표 겹침을 15건에서 1건으로 줄였고 67~70쪽의 국소 Visual Sweep은 통과했다. 추가 OLE 차트 보정 뒤 남성 차트가 있는 26쪽은 94.19%, 분리한 HWP/HWPX 샘플은 각각 90.67%/90.58%로 통과했다. 여성 차트가 있는 27쪽은 저장된 줄간격 복원 후 66.83%에서 91.85%로 개선됐다. 앞선 head의 전체 140쪽 Native 검증은 양쪽 모두 140쪽을 출력했지만 9쪽이 실루엣 90% 미만이어서 `re_review_required`였다. 후속 보정으로 40쪽은 60.17%에서 99.998%, 51·57쪽은 100%, 61쪽은 63.30%에서 99.95%, 82쪽은 60.42%에서 99.88%가 됐다. 다른 미달 페이지와 새 head의 전체 회귀·fresh WASM·전체 스윕은 남아 있다. 현재 원 PR을 승인하거나 통합하지 않는다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| 원 PR·작성자·base | [#7406](https://github.com/edwardkim/rhwp/pull/7406), `planet6897`, `devel` |
| 원 head / `-x` cherry-pick | `f49ebdefb9254d153e20a8cfd1424be398290eb4` / `c611aa5aff6e72ac02df722e07d4bda0d2b8b6e1` |
| 검토 base / branch | `c80a8370ab294259557c850c2e54495ecd0e79c0` / `review/planet6897-7406-20260925` |
| 관련 이슈 | [#7390](https://github.com/edwardkim/rhwp/issues/7390); 이 검토로 닫지 않음 |
| 원격 참고값 | 2026-09-25 조회: OPEN, Draft 아님, `MERGEABLE/CLEAN`, 원 head의 CI 성공. 머지 전 재조회 필요 |
| 메인터너 보정 | KoPub 줄 구성·각주·표 분할과 겹침 상한 15→1 보정 진행. OLE 차트의 OOXML 값축·백분율 레이블·범주·미리보기 색상표 보정. 검증 및 단계별 커밋 진행 중 |

## 변경과 조판 원칙 검토

원 PR은 KoPub Dotum/Batang의 Basic Latin hmtx 폭을 `text_measurement.rs`에서 사용하고, 집중 검사와 p94 Native 이미지를 추가했다. 변경 폭은 줄 나눔과 표 안 텍스트 및 선행 본문 위치에 전파된다. `tests/fixtures/text_overlap_baseline.tsv`의 PrEP 상한도 14→15로 올렸다.

최초 검토에서 section 2 para 27의 비-TAC TopAndBottom RowBreak 표가 선행 본문·캡션 위로 올라오는 **미충족**을 확인했다. 메인터너 보정에서는 행 컷·예약 높이·페인트 점유를 함께 수정했고, 67~70쪽 국소 게이트와 정식 회귀가 통과했다. 전체 문서의 남은 차이는 계속 검토한다. 호출 경로·전후 좌표와 명령은 Git 제외 `output/pr-review/planet6897-7406-20260925/result.md`에 기록한다.

OLE 차트는 HWPX의 `Chart/chart*.xml`과 HWP의 중첩 `OOXMLChartContents`가 편집 가능한 값·레이블을 담고, 중첩 EMF 미리보기가 문서 전용 색을 담는다. #7406은 OLE 안의 OOXML `chartSpace`이므로 [일반 OOXML 차트 경로](../tech/chart_ole_v1_boundary.md)로 그린다. `colorIndex=-1`인 누적 막대에서 색만 미리보기로 복원하고 축·막대·레이블은 OOXML 데이터로 렌더한다. `samples/issue7406`의 값을 바꿔도 낡은 미리보기 막대를 재사용하지 않는 집중 검사와 한컴 PDF 비교를 수행했다. HWPX 구조 근거는 `mydocs/tech/hwp_ole_spec.md` 및 사용자가 제공한 `hwpx_complete_guide.pdf`의 4.2절이다.

## 검증 입력과 결과

| 항목 | 결과 |
| --- | --- |
| 원 HWPX | `samples/issue2006/1790387_prep_final_report.hwpx`, SHA-256 `c68baed24096386f9041930d24d39409b61ac99463bf04dfd242440dfdeb739f`, `hancom-office-2024` 저장 메타데이터, 140쪽 |
| 신규 한컴 PDF | [1790387_prep_final_report-2024.pdf](../../pdf/issue2006/1790387_prep_final_report-2024.pdf), 원본을 HWP MCP `--engine 2024`로 변환, SHA-256 `04b95a6e41420fb45934ce2ee5abd8cf6dac4ce12fd47977dacbe7fca28018a8`, 140쪽, Hancom PDF 1.3.0.550 |
| KoPub 집중 검사 | 검토 source head `regression_suite_016` 2/2 PASS |
| 겹침 baseline | base 14건, 원 PR 단독 15건, 메인터너 보정 후보 1건. 상한은 15→1로 축소했고 검사 PASS. 최종 head 재검증 필요 |
| Native Visual Sweep | 26/27/67/68/69/70/94/108쪽 영향 검증은 각각 **94.19/91.85/99.99/96.50/99.94/100.00/99.57/99.84%**. 앞선 head의 전체 140쪽은 rhwp/PDF 모두 140쪽, `re_review_required`: 1쪽 88.11%, 40쪽 60.17%, 51쪽 66.99%, 57쪽 88.08%, 61쪽 63.30%, 82쪽 75.40%, 90쪽 61.83%, 93쪽 75.61%, 105쪽 77.50%. 2·4·10·138쪽은 양쪽 모두 빈 페이지라 지표 없음·내용 픽셀 100% 일치. 후속 39/40/41쪽 **99.09/99.998/99.99%**, 50/51/52쪽 **99.95/100/100%**, 56/57/58쪽 **99.61/100/99.93%**, 60/61/62쪽 **100/99.95/99.55%**, 81/82/83쪽 **99.44/99.88/97.65%**; 다섯 국소 gate 통과. 최종 head 전체 스윕 필요 |
| OLE 분리 입력 | HWP `a29d415a…`, HWPX `a64ee3e5…`, [한컴 PDF](../../pdf/issue7406/7406_OLE__CHART.pdf) `bd3a5a0e…`. 한 페이지 Native HWP **90.67%**, HWPX **90.58%**, 두 gate 통과 |
| 전체 release-test·lint·fresh WASM | 앞선 코드 후보의 nextest 10,241 PASS/50 skip 및 lint 통과. 추가 OLE 수정 후 최종 head 기준 재실행 필요; fresh WASM 미검증 |

기존 `...-hwp2020-20260814.pdf`의 절차적 생성 출처는 [#7399 검토](archives/pr_7399_review.md)에서 미검증이었다. 이번에는 원본에 대응하는 새 한컴 2024 PDF를 생성해 판정에 썼다. PDF의 KoPubDotum subset과 Mac 글꼴 공급을 확인했고 94·108쪽은 99% 이상이다. 68~70쪽의 표·본문 배치 차이를 글꼴 예외로 분류하지 않는다.

## 시각 증적과 남은 차이

물리 69쪽의 표 보정과 94쪽 KoPub 대조군은 기존 이미지에 남겼다. OLE 분리 샘플의 백분율 축·막대 길이·색·레이블을 직접 확인했다. 원본 27쪽은 도형 줄 뒤에 저장된 720HU 줄간격을 복원해 다음 본문·그림 위치를 기준 PDF에 맞췄다. 39→40쪽 1×1 셀 분할은 원본 위·아래 안 여백 각 850HU가 작은 셀 높이 282HU에 의해 축소된 문제를 고쳤다. 측정/컷 높이 차이 20.8px을 해소하고 저장 경계의 마지막 줄을 39쪽에 보존했다. 빈 문단의 시작 좌표가 표 조각 끝과 일치하면 문단 앞 간격도 한 번만 소비한다. 50→51쪽은 표 앞 한 줄 캡션의 저장 LineSeg가 지정한 표 시작 49275HU(본문 상대 657.0px)보다 분할 스캔의 흐름 시작이 26.7px 늦었던 문제다. 같은 저장 좌표를 첫 조각 예산에 적용해 한컴 PDF의 `치과의사`·`약사` 행을 50쪽에 남기고, 51쪽은 `간호사` 행부터 재개한다. 정식 회귀는 수정 전 FAIL/후 PASS, PrEP 관련 7건 PASS이며 새 이미지에서 표 외곽·뒤 문단을 직접 대조했다. 앞선 head의 전체 140쪽 review·overlay는 Git 제외 `output/pr-review/planet6897-7406-20260925/visual/full-native-final/prep7406/`, 국소 재캡처는 `visual/p40-padding-impact/prep7406/`와 `visual/p51-caption-anchor/prep7406/`에 있다. 대표 이미지:

57쪽은 가운데 정렬 TAC 표의 host 문단 왼쪽 여백 1000HU를 대체 배치 경로가 정렬 폭에 반영하지 않아 표 괘선이 한컴 PDF x=206px보다 6px 왼쪽이었다. 문단의 좌우 여백을 적용한 가용 줄 폭에서 표를 가운데 정렬했다. 정식 회귀는 수정 전 x=200.19px FAIL/후 PASS, PrEP 관련 8건 PASS. 56~58쪽 국소 sweep은 통과했고 표의 좌우 괘선과 57쪽 뒤 내용을 직접 확인했다. 재캡처는 Git 제외 `output/pr-review/planet6897-7406-20260925/visual/p57-centered-margin/prep7406/`에 있다.

61쪽의 분포도는 OLE 차트가 아니라 원본 `BinData/image16.tiff`(676×539) 그림이다. Para 기준 그림의 paint 상자만 host 왼쪽 여백 1000HU를 빠뜨려 PDF보다 13.3px 왼쪽에 놓였고, 그 여백은 typeset 쪽 공통 배치 상자에는 이미 포함돼 있었다. 두 경로의 상자를 맞췄다. 그림 설명의 끝과 다음 저장 LineSeg 시작이 같은 43579HU인데 문단 앞 간격 500HU를 다시 더하던 것도 그 경계에서만 제거했다. 그림 원점·뒤 두 문단의 정식 회귀는 수정 전 FAIL/후 PASS, PrEP 관련 9건 PASS. 60~62쪽 국소 sweep은 **100/99.95/99.55%**로 통과했고 그래프 본체·캡션·뒤 본문을 직접 판독했다. 재캡처는 Git 제외 `output/pr-review/planet6897-7406-20260925/visual/p61-caption-flow/prep7406/`에 있다.

81→82쪽의 1×1 저장 RowBreak 표는 한컴 PDF에서 ‘대상자 7’ 라벨·응답이 81쪽 끝, `vpos=0`으로 재개하는 ‘대상자 8’이 82쪽 처음이다. 기존 컷은 빈 종료 줄의 600HU 후행 간격까지 81쪽 예산으로 더해 약 4px 초과하자 대상자 7까지 되돌렸다. HWPX 저장 줄의 양수→0 되감김과 빈 종료 줄이 함께 확인될 때 그 후행 간격만 컷·페인트 공통 높이에서 제외했다. 정식 회귀는 수정 전 FAIL/후 PASS, PrEP 관련 10건 PASS(140쪽 수 고정 포함). 최종 바이너리의 81~83쪽 sweep은 **99.44/99.88/97.65%**로 통과했고 표 외곽·대상자 7/8·뒤 본문을 직접 대조했다. 재캡처는 Git 제외 `output/pr-review/planet6897-7406-20260925/visual/p82-tail-final/prep7406/`에 있다.

사용자 지적에 따라 물리 78쪽(인쇄 쪽 66)을 현재 head에서 다시 대조했다. 77/78/79쪽의 2px 관용 내용 실루엣은 **99.91/98.73/95.65%**로 국소 gate를 통과한다. 78쪽 엄격 내용 픽셀 일치율은 22.57%라 review가 붉지만, ‘사회적 장벽’ 시작 y=94.5/95.1px, ‘4. 소결’ y=437.9/438.8px, 마지막 본문 y=766.5/767.1px(rhwp/PDF)로 배치 차이는 1px 이내다. 줄바꿈과 누락도 직접 대조했다. PDF는 KoPubDotum Light/Bold subset, rhwp SVG와 Mac 폰트 매핑은 KoPub돋움체 Light/Bold이다. 두 바이너리가 동일한지는 확인하지 않았으며, 이 페이지를 `font_mismatch_exception`으로 판정하지 않는다. 재캡처는 Git 제외 `output/pr-review/planet6897-7406-20260925/visual/p78-user-check/prep7406/`에 있다.

![#7406 OLE 차트 분리 샘플 Native review](assets/pr7406_20260925/ole_chart_review_001.png)

![#7406 OLE 차트 분리 샘플 Native overlay](assets/pr7406_20260925/ole_chart_overlay_001.png)

![#7406 원본 27쪽 Native review](assets/pr7406_20260925/native_review_027.png)

![#7406 원본 27쪽 Native overlay](assets/pr7406_20260925/native_overlay_027.png)

![#7406 원본 40쪽 Native review](assets/pr7406_20260925/native_review_040.png)

![#7406 원본 40쪽 Native overlay](assets/pr7406_20260925/native_overlay_040.png)

![#7406 원본 50쪽 Native review](assets/pr7406_20260925/native_review_050.png)

![#7406 원본 50쪽 Native overlay](assets/pr7406_20260925/native_overlay_050.png)

![#7406 원본 51쪽 Native review](assets/pr7406_20260925/native_review_051.png)

![#7406 원본 51쪽 Native overlay](assets/pr7406_20260925/native_overlay_051.png)

![#7406 원본 57쪽 Native review](assets/pr7406_20260925/native_review_057.png)

![#7406 원본 57쪽 Native overlay](assets/pr7406_20260925/native_overlay_057.png)

![#7406 원본 61쪽 Native review](assets/pr7406_20260925/native_review_061.png)

![#7406 원본 61쪽 Native overlay](assets/pr7406_20260925/native_overlay_061.png)

![#7406 원본 78쪽 Native review](assets/pr7406_20260925/native_review_078.png)

![#7406 원본 78쪽 Native overlay](assets/pr7406_20260925/native_overlay_078.png)

![#7406 원본 81쪽 Native review](assets/pr7406_20260925/native_review_081.png)

![#7406 원본 81쪽 Native overlay](assets/pr7406_20260925/native_overlay_081.png)

![#7406 원본 82쪽 Native review](assets/pr7406_20260925/native_review_082.png)

![#7406 원본 82쪽 Native overlay](assets/pr7406_20260925/native_overlay_082.png)

![#7406 69쪽 Native review](assets/pr7406_20260925/native_review_069.png)

![#7406 69쪽 Native overlay](assets/pr7406_20260925/native_overlay_069.png)

![#7406 94쪽 Native review](assets/pr7406_20260925/native_review_094.png)

![#7406 94쪽 Native overlay](assets/pr7406_20260925/native_overlay_094.png)

## 보류 해제와 contributor 안내

90·93·105쪽 등 다른 90% 미만 페이지를 순서대로 보정하고 영향 페이지를 다시 캡처한다. 사용자가 1쪽의 글꼴 차이는 무시하도록 지정했지만, 나머지 배치 차이와 자동 게이트 미달은 여전히 보류 사유다. 새 head의 전체 Native/fresh WASM Visual Sweep·nextest·lint를 마친 뒤 판정한다. 앞선 head에서 기준 PDF 140쪽과 rhwp 전체 출력 140쪽의 개수를 확인했으며, 선택 페이지만의 통과나 페이지 수 일치만으로 전체 시각 검증을 대신하지 않는다. 머지 시에는 실제 merge SHA와 CI URL, 같은 이미지의 merge SHA 고정 raw URL로 기여자에게 결과를 안내한다.
