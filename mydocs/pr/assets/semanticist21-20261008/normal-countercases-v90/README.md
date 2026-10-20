---
kind: evidence
status: active
canonical: mydocs/manual/pr_review/visual_fixture_evidence.md
last_verified: 2026-10-10
---

# 기존 정상 대조군 전쪽 측정과 경계 판독

Production source는 `9b53bd699126205b7a93fff326167f575252f1ea`에 커밋한 v86 retry2다. 실제 Native/fresh WASM 빌드는 부모3874 위 당시 미커밋 변경에서 수행했고 현재 production source와 동일함을 확인했다. Frozen Native binary SHA `a41dbbebdf51f7c2fa928c6f2a7d5bd80a80a91bb9c904b3320fbd1afed8f2e2`, WASM SHA `ab5e01324ad024dd34979f03a08d2da6844d26ff31c49069c17de0fe849779b6`를 사용했다. v91에서는 실제 선언 face를 우선 공급해 Native와 WASM 전14쪽을 새로 출력했다. 두 경로의 PNG14개가 모두 바이트 동일하다. 이 폴더의 대표 review/overlay는 그 새 Native 출력으로 갱신했다. 패널 합성은 종료된 PNG를 소비하며 원본 raster를 변경하지 않는다.

## 기존 원본과 독립 Print

| 대상 | 기존 원본 | 독립 Print | 전체 쪽수 / 두 경로 최저 |
| --- | --- | --- | --- |
| #6442 출입증 | [HWP](../../../../../samples/issue6442/access_pass_form.hwp) | [Print](../../../../../pdf/pr6485-visual/pr6485-issue6442-access-pass-form-2020.pdf) | 3 / 96.92594% |
| #6030 평가 양식 | [HWP](../../../../../samples/issue6030/2386771_agritech_review_form.hwp) | [Print](../../../../../pdf/semanticist21-20261008/issue6030-2386771_agritech_review_form-e1f413ce7fea-2020.pdf) | 2 / 99.15282% |
| #3308 개요 번호/법령 양식 | [HWPX](../../../../../samples/task3307/issue3307_outline_number.hwpx) | [Print](../../../../../pdf/task3307/issue3307_outline_number-2020.pdf) | 9 / 97.34742% |

Native/WASM 각 원본의 전쪽 TSV는 90% 미달0·누락0·Print 쪽수 일치다. 96dpi/print profile/2px 이웃 관용 내용 실루엣 조건을 사용했다. TSV의 `not_evaluated`는 직접 판독이나 PR 승인을 대신하지 않는다.

## 실제 선언 face 보정 뒤 현재 Native/WASM review

이 폴더의 대표 PNG는 v91 Native의 새 출력으로 갱신했다. Native 전14쪽은 terminal exit0/310.051초/source불변·누락0·쪽수일치이며, 각문서 최저값은 #6442 96.92594%, #6030 99.15282%, #3308 97.34742%다. 같은 조건의 WASM도 terminal exit0/402.187초/source불변·전14쪽 누락0·쪽수일치이며 Native와 점수/PNG가 모두 같다. 초기18개 전체 검토의 승인과는 구분한다. 원래 v90 대체face의 99.19302/98.69917/97.36291%는 역사 진단값이다.

| 문서 / 쪽 | 현재 Native/WASM 보조값 | PNG |
| --- | --- | --- |
| issue6442 / 1 | 96.92594% | [review](issue6442-review_001.png), [overlay](issue6442-overlay_001.png) |
| issue6442 / 3 | 99.14498% | [review](issue6442-review_003.png), [overlay](issue6442-overlay_003.png) |
| issue6030 / 1 | 99.15282% | [review](issue6030-review_001.png), [overlay](issue6030-overlay_001.png) |
| issue6030 / 2 | 99.71610% | [review](issue6030-review_002.png), [overlay](issue6030-overlay_002.png) |
| issue3308 / 7 | 97.34742% | [review](issue3308-review_007.png), [overlay](issue3308-overlay_007.png) |
| issue3308 / 9 | 99.83119% | [review](issue3308-review_009.png), [overlay](issue3308-overlay_009.png) |

v91의 현재 Native 대표6개 review를 모두 직접 읽었다. #6442 출입증 앞/뒤의 셀 포함·앞뒤 배열·마지막 외곽, #6030 점수/평가 항목 행과 마지막 서명란, #3308 법령 양식의 문장 순서·안쪽 행 소속·마지막 두 열 표와 닫힘을 대조했다. 내용 누락·새 쪽수 차이 또는 큰 배치 이동은 관측하지 않았다.

완전한 피델리티 해결은 주장하지 않는다. #6442 앞면의 rhwp에 보이는 옅은 내부 격자, Print와의 글자 굵기·괘선 색상 차이가 남는다. #6030 제목/본문 글자 굵기와 작은 괘선/정렬 차이, #3308 7쪽의 작은 글자/줄 간격 차이도 남는다. 이 차이의 원인·정확한 upstream base 동일성은 이 자료만으로 판정하지 않았으며 새 회귀를 면제하거나 별도 정상 fixture를 추가하는 근거로 사용하지 않는다. 배치 대조 범위의 관측과 잔여 paint/font 차이를 구분한다.

## 이전 대체 face 진단과 재현 기록

v88/v90 출력의 embedded TTF name table에서 #6442의 선언 `HY견고딕`·`한컴돋움`, #6030의 `한양견고딕`이 실제 `Noto Sans KR ExtraLight` / Regular / weight400임을 확인했다. 당시 최저99.19302/98.69917/97.36291%를 올바른 face 공급 완료로 판정하지 않는다. 이전 진단 출력은 ignored output에 남기며 현재 대표 PNG는 v91 결과다. 실제 한컴 `Hwp/HYGTRE.TTF`와 `All/HDOTUM.TTF`를 fc-scan/SHA로 확인해 재실행했다.

현재 명령은 ignored `native-actual-original-faces-normal-countercases-full-v91-retry1-command.json` 및 `wasm-actual-original-faces-normal-countercases-full-v91-retry1-command.json`, 원본/Print/font SHA와 쪽수는 각 run_manifest에 있다. 전14쪽 Native/WASM PNG 동일성은 `genuine-faces-normal-native-wasm-audit-v91.json`, 대표 합성은 `normal-countercases-genuine-faces-direct-review-v91/`에 있다. 기존 정상 입력·baseline·회귀 assertion은 변경하지 않았다. TSV·로그·중간 JSON·font 바이너리는 Git에 커밋하지 않는다.

## v91 실제 font 바이트와 재실행

기존9편집의 함초롬바탕도 실제Haansoft Batang으로 대체돼있음을 확인해, actualCanvas검증과같은 HCRBatang regular SHA8664669bd2d6a4fe8b7538706d2bb2a1004c074aa02dca59051e515e731294f5, 실제HY견고딕/한컴돋움 TTF3개를byte동일하게 `verified-declared-faces-v91/`에 고정해 우선 공급했다. 한컴2020 Hwp/All/Install과기존명시경로를 함께 사용한다. Input/Print/source/baseline은 유지한다.

올바른full-font SVG109–159MB에서 첫Chrome실행은실제30초navigationtimeout으로중단돼점수측정전exit1이다. 코드결함/미달회귀로분류하지않고 실패로그를보존한다. 지원된 `RHWP_VISUAL_RASTER_TIMEOUT_MS=180000`만늘려Native재실행을완료했다. 실제command는ignored `native-actual-original-faces-normal-countercases-full-v91-retry1-command.json`, 새raster/TSV는같은run에있다. 이전실행을덮어쓰거나공차/기준값/입력을바꾸지않았다. 대응 WASM도 같은 제한으로 정상 종료했다. metric 공차는 변경하지 않았다.

#6442 옅은내부격자는별도로같은actualHY/HDOTUM조건의upstream21ed선택3쪽NativePNG와당시currentPNG가SHA2638e4bad47b22a7c2192b8c053764f1f2fbeb302b96d011fecd3723dd3b9ebb로byte동일했다. 이선택쪽의기존paint차이이며baseWASM/전쪽일치나Paint해결로보고하지않는다. glyph굵기/괘선색/작은간격차이는남고완전피델리티해결을주장하지않는다.
