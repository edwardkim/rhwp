# #7744~#7748 독립 한컴 Print 대조

제품 소스 `72ae15825b9a8bdcf21099099a7d19ce94e30aab`의 Native Skia와 최적화 fresh WASM을
각각 같은 원본의 독립 한컴 PDF에 대조했다. 어느 쪽도 제외하지 않고 57쪽씩 비교해 98% 미달·누락
0쪽을 확인했다. 판정은 96 DPI, PDFium, 색상 경계 보정 전 2px raw 실루엣을 사용했다.
[정밀 최저값과 산출물 SHA](measurements.json) · [원본과 독립 PDF의 출처 및 SHA](print-references.json)

| 문서 | 독립 한컴 Print | 전체 쪽 | Native 최저 | fresh WASM 최저 | 전쪽 TSV | 대표 review |
| --- | --- | ---: | ---: | ---: | --- | --- |
| #7744 진안군 공동주택 신청서 | [2020](../../../../pdf/issue7744-7748/20544835_jinan_apt_form-hwp-2020.pdf) | 2 | 99.99661% | 99.99663% | [Native](issue7744/native/silhouette.tsv) · [WASM](issue7744/wasm/silhouette.tsv) | [Native 1쪽](issue7744/native/review_001.png) · [WASM 1쪽](issue7744/wasm/review_001.png) |
| #7745 재활용 보도자료 | [2020](../../../../pdf/issue7382-regression-review/156726122-recycling-press-release-2020.pdf) | 8 | 99.46065% | 99.46644% | [Native](issue7745/native/silhouette.tsv) · [WASM](issue7745/wasm/silhouette.tsv) | [Native 7쪽](issue7745/native/review_007.png) · [WASM 7쪽](issue7745/wasm/review_007.png) |
| #7746 한글 97 안내문 | [2024](../../../../pdf/HWP5-nopassword-123456-hwpx-2024.pdf) | 24 | 98.29106% | 98.22386% | [Native](issue7746/native/silhouette.tsv) · [WASM](issue7746/wasm/silhouette.tsv) | [Native 12쪽](issue7746/native/review_012.png) · [WASM 12쪽](issue7746/wasm/review_012.png) |
| #7747 중첩 표 | [2024](../../../../pdf/issue7744-7748/issue2007_nested_cell_pagination_42065-hwp-2024.pdf) | 17 | 99.71613% | 99.68965% | [Native](issue7747/native/silhouette.tsv) · [WASM](issue7747/wasm/silhouette.tsv) | [Native 12쪽](issue7747/native/review_012.png) · [WASM 12쪽](issue7747/wasm/review_012.png) |
| #7748 전직시험면제 표 | [2024](../../../../pdf/issue7744-7748/21761835_jeonjik_exemption_table-hwp-2024.pdf) | 6 | 100.00000% | 100.00000% | [Native](issue7748/native/silhouette.tsv) · [WASM](issue7748/wasm/silhouette.tsv) | [Native 6쪽](issue7748/native/review_006.png) · [WASM 6쪽](issue7748/wasm/review_006.png) |

#7744의 1쪽은 진안군수 귀하와 구비서류가 각각의 행에 있고, 2쪽은 지원 제외 안내가 본문에
이어진다. #7745에서는 배박·감귤박 제목과 그림이 7쪽 3단 중첩 표 안에 짝을 이룬다. 한컴
2024의 두 인쇄본에는 4쪽의 숫자와 괄호를 인쇄하는 HCI 하위 CID 폰트가 누락됐다. 원본의
저장 제품에 맞는 한컴 2020 보존본에서 문자가 모두 보이며, 독립 2020 교차 인쇄에서도 확인했다.

#7746은 12쪽 마지막 문단에서 `URL`이 첫 줄 끝에 들어가며 다음 줄은 `페이지를`로 이어진다.
13쪽 첫 문장은 `해 보고 수정할 수 있습니다.`로 시작하고 원고지 항목은 15쪽에 있다.
보존한 한컴 2024 인쇄와 비교한 12쪽은 Native 99.64370%, fresh WASM 99.64522%다. 새로 인쇄한
한컴 2024 PDF 기준으로도 두 경로의 24쪽이 모두 98% 이상이다. 그 비교에서 보정으로 바뀐
12·13쪽의 점수가 각각 올랐다. 공백 압축과 폭만 바꾼 여섯 독립 Print에서 `URL`의 줄 소속이
후보와 일치했다. [독립 폭 경계와 후보](issue7746/url-fit-boundaries.json)
Native CLI의 direct PDF를 96 DPI로 열어 12쪽의 `URL`과 13쪽 시작 문장을 확인했다.
SVG 기반 PDF의 텍스트 추출에서도 `URL`이 12쪽 해당 문단의 첫 줄에 들어갔다.
최저값이 나온 1쪽은 삽입 그림 옆 설명 줄의 시작점과 둘째 줄 첫머리의 `Enter` 표지 표시가
한컴 인쇄와 다르다.

#7747에서는 12쪽의 중앙선거관리위원회 문단과 해당 문단을 포함하는 중첩 구조를 확인했다.
#7748은 5쪽 표에서 이월된 설계와 교통·지질및지반이 6쪽 머리행 다음에 들어가며, 디자인 행
및 표 뒤 설명 3번이 이어진다. 원본이 지정한 나눔명조와 Print에 실제 들어간 한컴바탕의
차이는 기존 명시적 세션 설정으로 재현했다. 두 렌더링 경로에서 원본과 인쇄 기준의
한컴바탕 TrueType 선택을 맞춰 6쪽 전체 raw 100%가 나왔다.
[Print 글꼴 환경](issue7748/print-font-environment.json)

## 정본 보호 문서 대조

Ubuntu에서 정본 보호 46조건 4,046쪽을 모두 측정했다. 77쪽의 표시값이 정본보다 낮았다.
77쪽은 모두 같은 원본·PDF·글꼴 공급·출력 경로로 `upstream/devel`의
`411a9f944cd6ff202e1f8c5b835ecccbc837486f`를 다시 렌더링했고, 후보 PNG와 SHA-256이
전부 일치했다. 이후 `upstream/devel`의 `2f4f60fbffa1cbed1d7baa8b6d36f40f0940dee0`까지
업무 기록, 조사 문서, 독립 Print 기록 대조 도구가 추가됐다. 두 기준 커밋의 제품 코드, Rust 회귀
검사, 렌더링 입력과 정본 보호 TSV는 동일하다. 최신 Print 기록 대조 도구의 검사 6건도 통과했다.
[4,046쪽의 정본·후보 표시값](protected-comparison.tsv) ·
[미달 77쪽의 기준 소스·후보 PNG 해시](protected-comparison.json) ·
[#9237의 macOS 보호 검증 기록](../../archives/pr_9237_review.md)

최소 글꼴 Rust 회귀, Native Skia, lint, workspace 빌드와 필수 편람·공통 보호 검사의 결과를
실행 영수증에 보존했다. [로컬 실행 영수증](local-validation.json)
