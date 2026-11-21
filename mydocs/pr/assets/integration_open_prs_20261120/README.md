# 원 PR 네 건 통합 검증

고정 `upstream/devel`은 `0a437a7eefb19bfe81fc12866887e4548ea9554d`, 제품 검증 소스는
`25f9587840d09b08af2b37be178ab93379a7b9dc`다. 원본 커밋 15개에 도형 캡션과 셀 너비
구조체의 호출을 맞추는 메인터너 보정 1개를 적용했다.

[PR #9201](../../archives/pr_9201_review.md) · [PR #9202](../../archives/pr_9202_review.md) ·
[PR #9206](../../archives/pr_9206_review.md) · [PR #9207](../../archives/pr_9207_review.md)

## 최종 제품 소스 재검증

제품 소스 `3901aa782d4445efb171ec4ffb026f2a459c2daf`에서 macOS 최소 글꼴 환경의 전체
Nextest 13,839건이 통과했다. 실패 0건, 제외 45건이며 테스트 바이너리는 55개다.
Native·WASM Clippy, 워크스페이스 빌드·전체 타깃 Clippy, 포맷·suite 검사 및
Visual Sweep Python 268건도 통과했다. Native Skia 라이브러리 검사는 워크스페이스 전체
4,034건 통과, 실패 0건, 무시 12건이다. Native Skia 그림 PNG 2건과 직접 PDF 출력 36건도
통과했다. 검증 전후 일반 macOS 글꼴 목록의 해시가 일치했다.
[실행 ID와 로그 해시](final-current-test-receipt.json).

최적화한 WASM을 포함한 Studio 배포본을 새 브라우저에서 열어 초기화 요청의 JS·WASM 응답
해시를 빌드 산출물과 대조했다. 실제 편집 경로에서 캡션, 문단 모양 복사, 한 줄 셀의
HWP·HWPX 저장 계약도 확인했다. [배포 응답과 편집 결과](final-product-consumers.json).
Studio 실제 브라우저 캡처 24건에서 Canvas2D 8건과 CanvasKit 16건을 확인했다.
대상 네 쪽의 화면·인쇄 렌더러가 요청과 일치했고 글꼴 폴백은 0건이었다.
[검증 세션의 글꼴 공급·캡처와 판정](actual-studio-renderers/README.md).
같은 소스로 빌드한 테스트용 Node WASM에서 Studio·편집기 SDK·공용 PDF 검사 2,822건이
통과했다. 실패·제외는 0건이다. [Node WASM과 실행 결과](final-current-test-receipt.json).
정본 보호 입력도 Native·fresh WASM 각각 23조건·2,023쪽을 대조했다. 전체 하락·누락은 0쪽이며
시행령 19쪽과 #7905 시행규칙 1쪽의 공식 표시값이 두 출력 경로에서 올랐다.
[전쪽 검증과 생산자 신원](final-protected/validation.json) · [상승 쪽 review·overlay](final-protected/gain-reviews/README.md).
상승한 네 값을 정본 하한에 반영했다. 새 하한으로 필수 편람 HWPX Native 383쪽 결과를 다시
판정해 하락 0쪽을 확인했고, Python 검사 268건도 통과했다.
[하한 갱신 영수증](../visual-protected-baseline/updates/integration-open-prs-20261120.json).
공식 한컴 2020·2024 Print 비교 16조건·46쪽의 최종 재실행에서 제품 PNG, 기준 PDF PNG, 전쪽
TSV가 각각 이미 보존한 증적과 바이트 단위로 일치했다.
[입력·PDF·TSV와 브라우저 신원](final-official-claim-refresh.json).
기여자 공통 검사는 네 원본의 Native·fresh WASM 총 8조건에서 새 정본 하한을 충족했다.
raw 최저는 99.99756%다. [공통 검사 결과](final-current-test-receipt.json).

## 초기 통합 소스 검증

| 항목 | 결과 |
| --- | --- |
| Ubuntu 최소 글꼴 전체 Nextest | 제품 검증 소스에서 13,844건 통과, 45건 제외, exit 0 |
| Rust 검증 | fmt·Native/WASM·workspace Clippy 및 빌드 통과, Native Skia 라이브러리 4,035건 통과·12건 무시, 직접 PDF 36건 및 그림 2건 통과 |
| 실제 Studio | 테스트 2,822건 통과, 제품 빌드·반복 저장·재열기 확인 |
| 원 PR 시각 비교 | Native·fresh WASM 26조건 540쪽, 90% 미만 및 고정 기준 대비 2%p 초과 하락 0쪽. 두 출력 경로의 #9202 9·10쪽만 제품 PNG가 달라졌다. |
| 편람·공개 보호 | 편람 HWPX Native 383쪽과 기여자 공통 보호 8조건 통과 |
| 메인터너 정본 보호 | 보정 전 46조건 4,046쪽을 대조해 미달 2쪽을 확인했다. 보정 후 해당 두 문서는 Native·fresh WASM 160쪽 모두 보호값을 충족했다. |

정본 보호는 Chrome Canvas 기반 fresh WASM과 Native Skia의 Print 출력을 각각 독립 한컴 Print와
96 DPI로 대조한 색상 경계 보정 전 2px raw 실루엣의 공식 두 자리 값을 사용했다. Mac에서
44조건 2,520쪽을 산출했다. 외부 그림을 포함한 sample10의 2조건 1,526쪽은 동일 소스의 Ubuntu
전쪽 증적과 정본을 다시 대조했다. [정본 TSV](../visual-protected-baseline/final.tsv) ·
[전쪽 비교](protected-comparison.tsv) · [생산자 해시와 재검증](protected-verification.json).

## 중첩 표 Native의 글꼴 공급 보정

중첩 표 5쪽에서 차이 6픽셀은 `※` 두 곳의 왼쪽 획에 모였다. 사용자 글꼴 디렉터리를 먼저
등록하면 Native의 256개 사용자 공급 한도에 도달하여 뒤에 지정한 독립 기호 OTF가 누락됐다.
실제 글꼴 선택 추적에서 시스템 대체 글꼴 Malgun Gothic이 선택됐고, OTF 파일 유무에 따라 출력이
바뀌지 않았다. 동일 SHA-256의 기호 OTF를 먼저 지정하니 세 `※`가 모두 그 프로그램을 사용했다.
원문 전진값 1300 HWPUNIT는 유지됐고, 5쪽은 99.99%에서 100.00%로 올라 Native 17쪽이 정본과
일치했다. 같은 공급 순서로 고정 기준을 재출력한 결과도 17쪽 TSV와 제품 PNG가 모두 같다.
정본 글꼴 환경과 프로그램 해시 집합은 동일하다.
[보정 전 review](nested-before-p005-review.png) · [overlay](nested-before-p005-overlay.png) ·
[보정 후 review](nested-after-p005-review.png) · [overlay](nested-after-p005-overlay.png) ·
[Native 전쪽 TSV](nested-native.tsv).

## 보호 미달 두 쪽의 메인터너 보정

| 문서·쪽 | WASM 보호값 | 보정 전 raw | 보정 후 raw | 보정 후 대조 |
| --- | ---: | ---: | ---: | --- |
| 가상융합산업 진흥법 시행령 19쪽 | 99.74% | 99.73446% | 100.00000% | [review](maintainer-repair/decree-p019-review.png) · [overlay](maintainer-repair/decree-p019-overlay.png) |
| 종합부동산세법 시행규칙 #7905 1쪽 | 99.98% | 99.96967% | 99.99540% | [review](maintainer-repair/issue7905-p001-review.png) · [overlay](maintainer-repair/issue7905-p001-overlay.png) |

Google 서명과 배포 SHA-256을 확인한 Chrome 157.0.8103.79에서 두 쪽을 각각 다시
산출했다. 같은 환경에서 고정 기준과 후보의 PNG 해시가 각 쪽마다 일치했고, Chrome .80에서
얻은 PNG 해시와도 같았다. 시행령 차이는 그래프 이미지 범례 주변에, 시행규칙 차이는 제목 아래
시행일·개정 문구와 일부 본문 글자에 남는다. 두 쪽에서 통합으로 인한 출력 변화는 측정되지 않았다.

한컴 2022 Print의 RGB555 색상 전달과 #7905의 머리말·꼬리말 제어문자 높이 소유를 독립 출력으로
확인했다. 소스 보정 후 Native·fresh WASM에서 시행령 각 74쪽과 시행규칙 각 6쪽을 같은 기준
PDF 및 보호값에 대조했다. fresh WASM은 Mac에서 locked wrapper의 `--no-opt`로 빌드했다.
[원인과 독립 대조](maintainer-cause.md) · [전쪽 결과와 생산물 해시](maintainer-repair/verification.json).

Visual Sweep은 기준 PDF의 제품·인쇄 드라이버를 읽어 RGB555를 자동 선택한다.
수동 플래그 없이 다시 실행한 Native·WASM 160쪽의 PNG·TSV·render tree는 모두 앞서 보관한
출력과 같은 SHA-256이었다.

보정한 제품 소스에서 집중 회귀 10건과 Visual Sweep Python 검사 268건이 통과했다.
필수 편람 HWPX Native 검사도 383쪽 전부 통과했다. 공식 표시 최저값은 99.97%이며
페이지별 보호값 하락은 0쪽이다.

신규 PNG JSON 필드와 문서 저장에 영향을 주지 않는 세션 설정을 기존 계약에 등록했다.
등록 후 계약 검사 7건을 통과했다. 전체 Nextest 재실행은 13,837건 통과, 실패 0건,
제외 45건이다. 별도로 실행한 지식지도 Python 검사 3건은 고정 devel에서도 동일하게 실패했다.
Native·WASM Clippy, 워크스페이스 빌드 및 전체 타깃 Clippy도 통과했다.
Native Skia 라이브러리는 3,852건 통과, 제외 12건이다.

## 최적화한 fresh WASM 재검증

시행령과 #7905를 Native 및 Mac locked wrapper에서 `wasm-opt`를 완료한 fresh WASM으로
각각 산출했다. 두 입력·두 백엔드 전 160쪽의 제품 PNG, 기준 PDF PNG, 렌더 트리 및
실루엣 TSV가 앞선 같은 조건의 실행과 바이트 단위로 일치했다.
[실행과 파일 해시](maintainer-repair/optimized/verification.json).

| 대표 쪽 | Native | 최적화한 fresh WASM |
| --- | --- | --- |
| 시행령 19쪽 그래프 | [review](maintainer-repair/optimized/decree-native-p019-review.png) · [overlay](maintainer-repair/optimized/decree-native-p019-overlay.png) | [review](maintainer-repair/optimized/decree-wasm-p019-review.png) · [overlay](maintainer-repair/optimized/decree-wasm-p019-overlay.png) |
| #7905 1쪽 시행일 | [review](maintainer-repair/optimized/issue7905-native-p001-review.png) · [overlay](maintainer-repair/optimized/issue7905-native-p001-overlay.png) | [review](maintainer-repair/optimized/issue7905-wasm-p001-review.png) · [overlay](maintainer-repair/optimized/issue7905-wasm-p001-overlay.png) |

Native의 시행령 19쪽에서 그래프의 선과 범례를 확인했다. 표의 글자 획과 경계에는 미세한 차이가
있으며 2px raw는 100.00000%다. #7905의 시행일·개정 문구는 기준선이 맞고 QR과 본문
가장자리에 미세한 차이가 있으며 Native raw는 99.99246%다. 이 결과는 기존 보호 PDF에 대한
재현이다. 한컴 2020·2024 재인쇄본과의 비교는 별도 결과로 기록한다.

## 공식 비교 PDF 재인쇄

기존 `Creator: Hwp 2022` 7종은 한컴 2024, #7905는 한컴 2020에서 같은 원본을
Hancom PDF로 인쇄했다. 각 출력의 MCP 엔진·한컴 빌드·Print 경로·입력과 PDF SHA-256은
[재인쇄 기록](official-print-references.json)에 있다. #9040은 10쪽 첫 행의
`Pension Policies and Politics in East Asia`와 후속 행을 새 독립 PDF에서도 확인했다.
기존 보호 PDF 대비 검증과 새 인쇄본 대비 후보 출력 검증은 각각의 기준 해시로 기록한다.

| 대조 대상 | 결과 | 전쪽 증적 |
| --- | --- | --- |
| 정본 보호 입력과 기존 인쇄본 | Native·fresh WASM 46조건, 4,046쪽. 하락·누락 0쪽, 상승 4쪽. | [조건별 TSV와 검증 영수증](final-protected/validation.json) |
| #7905 종합부동산세법 시행규칙, 한컴 2020 Print | Native·fresh WASM 각 6쪽. 공식 표시 최저값은 두 경로 모두 2쪽 98.82%. 1쪽은 Native 99.99568%, WASM 99.99268%. | [한컴 2020 영수증과 이미지 해시](maintainer-repair/hancom-2020-issue7905/evidence.json) · [Native TSV](maintainer-repair/hancom-2020-issue7905/native/silhouette.tsv) · [WASM TSV](maintainer-repair/hancom-2020-issue7905/wasm/silhouette.tsv) |
| 가상융합산업 진흥법 시행령, 한컴 2024 Print | Native·fresh WASM 각 74쪽. 최저 4쪽 Native 90.59%, WASM 90.51%. | [전쪽 TSV와 4·7·16쪽 review·overlay](maintainer-repair/hancom-2024-decree/README.md) |

#7905의 1쪽에서 시행일·개정 문구와 바로 아래 본문을 직접 대조했다. 두 경로에서 같은 쪽의
문구 순서와 조판 소속을 확인했다.
[Native review](maintainer-repair/hancom-2020-issue7905/native/review-001.png) ·
[Native overlay](maintainer-repair/hancom-2020-issue7905/native/overlay-001.png) ·
[WASM review](maintainer-repair/hancom-2020-issue7905/wasm/review-001.png) ·
[WASM overlay](maintainer-repair/hancom-2020-issue7905/wasm/overlay-001.png).
