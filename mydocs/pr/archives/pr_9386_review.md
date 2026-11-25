# PR #9386 검토 — 중첩 표의 쪽 경계와 행 프레임

## 최종 판정

**표 조각과 경계 글의 쪽 소속 개선을 재현했다.** 원 PR의 제출 TSV보다 낮은 3개 출력 항목은 동일 조건에서 원 PR head와 최종 후보를 다시 비교하기 위해 통합을 보류한다.

[원 PR #9386](https://github.com/edwardkim/rhwp/pull/9386) · [이슈 #9000](https://github.com/edwardkim/rhwp/issues/9000)

KAF 회의자료에서 표 구간을 남긴 44쪽 파생본이다. 코드는 칸마다 저장 프레임을 열었던 유닛을 찾아 같은 순번에서 조각을 닫는다. 이어지는 조각은 쪽 프레임 아래끝까지 점유하고, 마지막 소진 조각은 남은 높이에서 닫는다. 검토 경계는 33~44쪽 표와 경계 글 소속, 비어 있는 선행 문단의 앞 간격, 이어받는 중첩 표의 위 바깥 여백이다.

원 PR의 전쪽 최저는 Native 94.32%·WASM 93.13%로 1쪽 표지에서 발생했고, 주장하는 33~44쪽은 양쪽 98% 이상이었다. 첨부 PDF는 PDF 1.4, A4 가로 44쪽이며 `Creator`에는 `Hwp 2018 11.0.0.1623`, `Producer`에는 `Hancom PDF 1.3.0.404`가 기록됐다. 저장소의 실행 파일 실측에서 이 빌드는 한컴 2020이며, 기존 MCP 2020 Print PDF에서도 같은 Creator·Producer를 확인했다. 원 PR이 기록한 동일 파생본의 Hancom PDF Print와 SHA-256이 첨부본에 맞아 기존 PDF를 기준으로 재사용했다. [제품 버전 실측](../../manual/verification/hangul_version_oracle.md#51-설치된-버전-확인) · [MCP 대조 기록](pr_9317_review.md#입력과-독립-인쇄-기준)

갱신된 MCP에서 같은 원본을 한컴 2020으로 다시 출력했다. client 0.10.0에서 `start --engine 2020 --target pdf --timeout-seconds 1800`을 호출하고, `status`가 성공한 뒤 같은 job을 `download`로 받았다. `status.result`와 `download.server`의 `pdf_output_mode`는 `hancom2020_pdf_driver_one_up`, `pdf_print_method`는 `0`이다. MCP는 PDF 글꼴 이름의 공백 문법을 보정하고 전체 44쪽을 확인했다. 새 한컴 빌드 `11.0.0.9172`의 인쇄본을 원 PR 인쇄본과 96 DPI로 비교하면 표시값이 100.00% 미만인 쪽이 22쪽이다. 원 PR과 통합 후보의 비교는 기존에 제출된 같은 인쇄본을 기준으로 유지한다. [두 인쇄본 대조 및 MCP 영수증](../assets/planet6897_20261125/mcp-print-reissue.json) · [MCP 기준 PDF 인쇄 계약](../../manual/mcp_hwp2024Convert_usage.md#기준-pdf-인쇄-계약)

## 문구의 쪽 소속 재현

`Standard: KAF address standard`, `landmark`, `surname, given`은 devel 34·38·43쪽에서 후보 33·37·42쪽으로 옮겨갔다. 첨부 PDF에서도 각각 33·37·42쪽에 1회씩 있다. PDF·devel·후보 모두 44쪽이다. 34쪽의 표 아래 괘선, 칸 경계, 마지막 행의 위치를 확대해 확인했다. 34쪽의 일부 겹친 글줄은 PDF에서도 겹친다.

- [쪽 소속 증적](../assets/planet6897_20261125/kaf-table-text-ownership.json)
- 34쪽 [review](../assets/planet6897_20261125/kaf-table/native/review_034.png) · [overlay](../assets/planet6897_20261125/kaf-table/native/overlay_034.png)

기존 비교는 `0fbee2c04e`와 통합 후보 `53b15ed46f`의 결과다. #9298 반영 뒤 첨부 PDF에 다시 대조한 Native 44쪽에서 점수가 하락한 쪽은 없었다. 바뀐 31쪽은 표 칸 인라인 그림의 x 위치가 253.1px에서 265.1px로 옮겨가며 일치율이 올랐다. 나머지 43쪽은 PNG와 렌더 트리가 이전 후보와 각각 바이트 단위로 같다. [Native 재측정 기록](../assets/planet6897_20261125/native-rebase-verification.json)

원본과 PDF가 사용하는 휴먼명조 TrueType을 공급한 뒤 Native와 fresh WASM 전체 44쪽을 다시 비교했다. 26쪽 Native는 92.76%에서 99.93%로 회복했다. 전쪽 최저는 24쪽에서 Native 96.88%, WASM 96.93%다. 33~44쪽은 두 출력 모두 98% 이상이다. 원 PR 제출값 대비 남은 차이는 Native 35쪽 99.19→99.18%, WASM 21쪽 99.34→99.23%와 36쪽 98.42→98.38%다. 같은 글꼴을 공급한 리베이스 전후 후보의 Native 35쪽과 WASM 21쪽 PNG는 바이트 단위로 같고, WASM 36쪽은 렌더 트리와 점수가 같았다. [글꼴 공급 및 페이지별 결과](../assets/planet6897_20261125/kaf-font-recheck.json)

[원 PR·통합 SHA 및 빌드 출처](../assets/planet6897_20261125/review-provenance.json) · [입력·기준 PDF 출처](../assets/planet6897_20261125/print-provenance.json)
