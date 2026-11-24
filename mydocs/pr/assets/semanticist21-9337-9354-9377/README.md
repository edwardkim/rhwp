# 표 편집·영역 누름틀·본문 저장 줄 검증

2026년 11월 24일, semanticist21 님의 세 기여를 `devel` 커밋 `2c0c1d205a54b8d5bd262241bac1391cffe0d801`에서 검토를 시작했다. 작업 중 병합된 [#9385](https://github.com/edwardkim/rhwp/pull/9385)를 반영한 기준은 `4cd460efd53377f73c896295f4b1c07b624d5398`이다. 원 PR의 head는 #9337 `b69c815caa964dbbba1d55a9aae6654c875a5a05`, #9354 `fc4e2cf07a1439760d2cfb67c8230cb3140e2a78`, #9377 `b3301e1905edd7fe504fe400c3560e27c421ad33`이다. 세 커밋은 모두 보완 후보의 조상이다. 세 기여를 합친 보완 전 커밋은 `b8eba3318b884e88c1baa47303aff3c694876f79`, 첫 보완 소스는 `c9395989d6000d0c9c19e70e079aad9446ebf28a`였다. 기존 회귀에서 드러난 두 경계를 고친 제품 소스는 `a1bc2e5fe686b767d86f3cbbab939f69b816ad07`이다.

## 독립 한컴 인쇄와 직접 판독

| 문서 | 입력 수 · 렌더러별 쪽수 | Native 최저 | fresh WASM 최저 | 변경 영역 |
| --- | ---: | ---: | ---: | --- |
| #9337 병합 셀과 계약서 | 7개 · 10쪽 | 99.38% | 99.35% | 병합 셀 조작 4종과 실제 계약서에서 편집한 2쪽은 모두 100.00%. 계약서 표의 윗변과 뒤의 인쇄 문구가 원래 자리를 지킨다. |
| #9354 머리말·꼬리말 | 10개 · 10쪽 | 100.00% | 100.00% | 넓은 표 앞의 두 글자는 앞줄 양끝에 놓인다. 좁은 표는 글자와 같은 줄에 놓인다. |
| #9377 원본과 두 HWPX 편집본 | 3개 · 72쪽 | 99.09% | 99.14% | 24쪽 모두에서 줄바꿈과 문단의 쪽 소속이 유지된다. 한컴이 인쇄한 세 문서의 쪽별 96 DPI 픽셀은 전부 일치한다. |

공개 계약서는 Windows 한컴 2020, 나머지는 Windows 한컴 2024의 PDF 인쇄 드라이버로 인쇄했다. 각 입력과 PDF의 SHA-256, 인쇄 버전·작업 번호, 렌더 호환 조건을 기록했다. 각 쪽의 대표 판정값은 96 DPI, 2px 범위의 색상 경계 보정 전 내용 실루엣 일치율이다. 표와 본문의 글꼴은 문서가 지정한 실제 서체를 확인해 공급했다. [입력·인쇄본 20쌍](sources-and-print.tsv) · [표와 영역 글줄 전쪽](focus-pages.tsv) · [본문 문서 전쪽](edited-body-lines.tsv) · [한컴 인쇄끼리의 픽셀 대조](line-print-pixels-96dpi.tsv) · [계약서 글꼴](contract-font-programs.tsv) · [본문 문서 글꼴](line-font-programs.tsv)

- 공개 계약서 2쪽의 높이 58,083 HU인 세 칸을 2,000 HU로 줄였다. 두 형식 모두 표 윗변은 77.5 px에서 유지되고 바뀐 행 높이는 774.4 px에서 26.7 px로 줄었다. 셀 속성과 크기 조절로 키운 병합 셀의 경우에도 뒤 문단까지 같은 간격으로 움직였다. [두 편집 경로·두 저장 형식](corrected-table.tsv) · [계약서 관측값](contract-geometry.jsonl) · [2쪽 확대](contract-edited-hwpx-comparison.png)
- 머리말·꼬리말에서 ‘앞뒤’의 마지막 표 뒤에서 친 글자는 문자열 위치 2에 들어간다. 같은 자리에 있는 두 누름틀 중 고른 하나만 입력을 소유한다. 각주·미주도 HWP와 HWPX에서 선택한 필드 범위를 보존했다. 한컴이 다시 저장한 HWP·HWPX와 누름틀·표 제거 대조군에서도 글줄과 표의 소속을 확인했다. [필드별 재열기](corrected-fields.tsv) · [머리말 확대](trailing-header-comparison.png) · [꼬리말 확대](trailing-footer-comparison.png)
- HWPX 원본의 26개 문단을 각각 편집하고 HWP·HWPX로 저장·재열기했다. 52건 모두 원래 24쪽이며 본문 텍스트와 문단 소속이 유지됐다. [쪽이 바뀌던 보완 전 결과](reopen-pages.tsv) · [보완 뒤 52건](corrected-reopen.tsv) · [문단 112 편집본 확대](line-112-comparison.png)

## 전체 회귀에서 확인한 두 인접 문서

첫 보완 소스의 전체 회귀에서 14,080건이 통과하고 두 건이 실패했다. 원 검사의 입력·수정 이력과 한컴 Print를 대조해 표 편집과 글줄 구성을 각각 보정했다.

| 원본과 조작 | 첫 보완본 | 최종 제품과 직접 판독 |
| --- | --- | --- |
| `셀보호2.hwp`의 이웃 셀에 반대 부호의 크기 변화 적용 | 표 선언 높이가 17,932 HU에서 18,232 HU로 늘었다. HWP와 HWPX 저장본은 Native·WASM 모두 75.92%였다. | 실제 적용된 높이 변화량을 합쳐 외곽 높이 17,932 HU를 보존한다. 두 형식 모두 Native·WASM 100.00%이며 마지막 행 괘선과 뒤 셀의 시작점이 한컴과 일치한다. |
| `print-object-space-overflow.hwpx` 본문에서 오른쪽에 놓인 ‘(단위 : 천원)’과 뒤 인라인 표 | ‘천원)’이 앞줄로 올라가 1쪽이 Native 9.14%, WASM 9.25%였다. | 문단의 어절·글자 설정을 줄 끝 판정에 반영해 원본 9쪽 모두 Native·WASM 100.00%다. 같은 옵션만 바꾼 본문과 머리말 대조군에서도 한컴의 줄바꿈을 재현했다. |

8개 추가 입력은 두 경로에서 각각 32쪽이며 전쪽 100.00%였다. 최신 `devel`에 병합된 #9385의 변환 클라이언트 0.10.0으로 대기열을 조회한 뒤 Windows 한컴 2024에서 설정 대조군 4개와 표 편집 저장본 2개를 인쇄했다. 나머지 2개 원본은 저장소가 보존한 독립 Print를 사용했다. [추가 입력·기준 PDF](additional-print-sources.tsv) · [추가 64행](additional-pages.tsv) · [표 편집 확대](compensation-edited-hwp-comparison.png) · [본문 줄 확대](body-overflow-comparison.png)

## 브라우저에서 확인한 편집과 연속 저장

Chrome 156에서 실제 셀 속성 창의 높이를 52.9 mm로 바꿨다. 병합 셀 높이는 8,000 HU에서 14,995 HU로, 표 높이는 106.7 px에서 199.9 px로 바뀌었고 표 뒤 문단과의 간격은 12.4 px였다. 실행 취소·다시 실행 모두 편집 전후의 위치와 높이를 복원했다. 기존 마우스 드래그 E2E 10개 검사에서는 행 경계가 40.2 px 움직였고 이웃 행과 표 전체의 높이 관계를 보존했다. [속성 창과 공개 편집 API의 측정값](browser-editing.json)

실제 Studio 파일 열기부터 머리말·꼬리말의 한글 조합 입력, 보조 평면 문자, 실행 취소와 다시 실행까지 32개 검사가 통과했다. 같은 위치의 누름틀을 찾는 F11 대조 검사도 46개가 통과했다. HWP와 HWPX 사이에서 편집·저장·다시 열기·재저장·다시 열기를 44가지 조합으로 실행해 필드, 병합 셀, 계약서, 문단별 쪽 소속을 확인했다. [두 차례 저장한 44개 결과](repeat-saves.tsv)

공통 공개 문서 네 개를 양쪽 렌더러로 검사했다. 라틴 글자, 좁은 회전 도형, 잘린 그림, 그룹 도형에서 여덟 건 모두 통과했다. [쪽별 TSV와 review·overlay 그림 목록](public-controls/results.tsv)

## 보호 기준과 이슈 범위

첫 제출 검증은 devel `4cd460efd5`와 제품 소스 `a1bc2e5fe6`에서 Native와 최적화 WASM을 각각 독립적으로 빌드해 진행했다. 기준과 후보 WASM의 SHA-256은 각각 `fd537eb1848805e7aee42e5c1370998626b8faedf92af59fbe6560512da226ff`, `769d9a0e730cd98f51937ea6107642378613ecae65f1a213a1427c89a7cab3be`다. Chrome 156, 96 DPI, 문서별 한컴 인쇄 PDF와 등록된 글꼴 공급 순서를 맞췄다. 같은 병합 셀을 높이 15,000 HU로 바꾸는 브라우저 대조군에서 기준은 표 뒤 간격이 12.4 px에서 0.6 px로 줄고, 후보는 12.4 px를 유지했다. Cargo가 기존 바이너리를 재사용한 첫 기준 빌드는 제외했고, 이 차이가 재현된 기준 빌드로 전쪽을 다시 측정했다.

23개 보호 문서에서 Native와 WASM 각 2,023쪽의 쪽별 점수와 PNG SHA-256이 기준과 후보에서 모두 같다. Native 2,023쪽, WASM 2,009쪽이 공식 하한을 충족했다. 화학물질 표시 기준 WASM 26쪽은 양쪽 모두 정본 99.99%에서 100.00%로 측정됐다. 정본 갱신에는 전쪽 하락 0건이 필요하므로 이번 결과는 측정값으로 기록했다. [TSV 원문별 경로·SHA-256·크기](page-tsv/index.tsv) · [4,046행 전쪽 대조](protected-pages.tsv) · [공식 하한에 미달한 14행](protected-below.tsv) · [동일 픽셀의 인쇄 차이 확대](../semanticist21-20261124-rework/final/protected-details.png)

작업지시자는 직전 devel에서도 같은 픽셀로 나타난 기존 WASM 14쪽에만 수용 예외를 승인했다. 이 페이지들의 공식 하한은 100.00%다. 시행령 8쪽, 중첩 표 1쪽, HWP 3 sample10 2쪽, 형식 사양서 2쪽, 영월 서식 1쪽에서 기존 99.87~99.99%가 재현됐다. [직전 병합에서 기록한 14쪽](../semanticist21-9288-9314-20261124/corrected/protected-below.tsv)

세 원 PR의 자동 종료 연결 이슈는 0개다. 본문에 참고한 [#6714](https://github.com/edwardkim/rhwp/issues/6714)·[#6882](https://github.com/edwardkim/rhwp/issues/6882)·[#5847](https://github.com/edwardkim/rhwp/issues/5847)·[#8310](https://github.com/edwardkim/rhwp/issues/8310)은 검토 시점에 이미 닫혀 있었다. 같은 기여자의 [#9298](https://github.com/edwardkim/rhwp/pull/9298#issuecomment-7089121727)은 앞서 게시한 JPEG 방향 수정과 별도로 남은 원본 8개 쪽의 조판 보완 요청을 기다리는 별도 PR이다.

## 최신 devel의 렌더링과 원 기여별 재검증

[#9388](https://github.com/edwardkim/rhwp/pull/9388)이 추가된 `e961a7877186de626643a6466cbecb137874c530`을 로컬에서 병합한 최종 제품은 `ff9194594e1d7396c3730b92f387e2a4d3c62b3a`다. 기준, 원 기여 3개, 후보의 Native와 최적화 WASM을 각각 별도 빌드로 고정했다. 한 차례 공용 Cargo 경로에서 원 PR과 후보가 같은 Native 실행 파일을 반환해 그 후보 측정을 폐기했다. 소스별 Cargo 빌드 식별 경로, 실행 파일 해시와 편집 결과로 다시 확인했다. Native 글꼴 공급 목록에 변환 중인 빈 임시 파일이 잡힌 측정도 폐기했다. 빌드별 변환 캐시를 분리해 원본·기준·후보의 20개 조합을 재측정했고, 실제 글꼴 프로그램의 순서와 SHA-256을 대조했다. 다시 측정한 516쪽에서 출력 PNG와 실루엣 값은 같았다. 표 높이 두 조작과 머리말·꼬리말 배치는 같은 기준으로 원 기여 및 최신 devel에서 반례를 재현했다. [원 기여와 후보의 전쪽 대조](original-candidate.tsv)

| 원 PR | 원 head 대비 대조한 쪽·경로 | 상승 | 동일 | 하락 |
| --- | ---: | ---: | ---: | ---: |
| [#9337](https://github.com/edwardkim/rhwp/pull/9337) | 20 | 0 | 20 | 0 |
| [#9354](https://github.com/edwardkim/rhwp/pull/9354) | 20 | 12 | 8 | 0 |
| [#9377](https://github.com/edwardkim/rhwp/pull/9377) | 144 | 0 | 144 | 0 |

식량안보와 제주 원본·한컴 인쇄본은 #9388의 SHA-256과 대조했다. 같은 입력에서 최신 devel과 후보의 출력 PNG SHA-256이 Native와 WASM에서 908행 모두 같았다. 세 원 PR의 주 대상은 경로별 92쪽 전부에서 Native 99.09%·WASM 99.14% 이상이었다. [#9388에서 반영한 두 문서의 전쪽 대조](9388-preservation.tsv) · [대상·대조군의 Native와 WASM TSV](page-tsv/index.tsv)

| #9388 공개 문서 | 각 렌더러의 쪽수 | Native 전쪽 최저 | WASM 전쪽 최저 | 새 기준 대비 PNG 차이 |
| --- | ---: | ---: | ---: | ---: |
| 식량안보 보고서 | 141 | 98.26% | 98.20% | 0 |
| 제주 관광교육 계획 | 313 | 98.18% | 98.15% | 0 |

최종 제품의 전체 Rust 회귀 14,091건·집중 18건·Native 공식 3종 4,072건·Studio 2,835건과 필수 lint·배포 빌드가 통과했다. 23개 보호 문서의 양쪽 렌더러 4,046행을 새 기준과 같은 조건에서 다시 측정했다. 쪽별 PNG가 모두 일치했다. 승인된 기존 WASM 14쪽 중 실제로 하한보다 낮은 쪽은 14쪽이다. [로컬 검사와 실제 산출물 해시](local-checks.json) · 중첩 표 Native는 21개 글꼴을 정본 순서대로 공급했다. 원 해시가 일치한 15개, 공통 허용 바탕 TTC 1개와 이번 세 PR의 Native에 한해 승인한 로컬 파일 5개를 사용했다. 기준과 후보는 17쪽 모두 100.00%였고 PNG가 같았다. 다섯 파일 각각의 정본·실제 SHA-256과 공급 순서 전체의 SHA-256은 로컬 검사 기록에 있다. 중첩 표 WASM은 앞서 승인된 조건에서 17쪽의 PNG가 직전 검토와 같았다. 21개를 모두 공급한 보조 검사에서도 Linux의 기존 차이는 같은 두 픽셀이었다. [전쪽 보호값](protected-pages.tsv) · [공식 하한에 미달한 쪽](protected-below.tsv)

[#9390](https://github.com/edwardkim/rhwp/pull/9390)·[#9391](https://github.com/edwardkim/rhwp/pull/9391)의 최신 기준 `dd1dc8509769df46593ca91e23c06167d0436607`은 검토 지침과 운영 기록 8개를 갱신했다. 실제 병합 트리에서 제품·검증 입력이 그대로임을 확인했다. 변경된 인쇄 지침을 적용해 목표·대조군 28개와 #9388의 두 문서에서 기준 PDF 해시와 출력 경로를 대조했다. 한컴 2024 MCP Print 23개, 한컴 2020 MCP Print 3개와 기존 데스크톱 인쇄본 4개였다. [인쇄 지침](https://github.com/edwardkim/rhwp/blob/dd1dc8509769df46593ca91e23c06167d0436607/mydocs/manual/verification/visual_verification_governance.md)
