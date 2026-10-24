# #7445 보정35 — 문서 대체 필드의 이름 선택과 온새미로 제목 복구

## 독립 근거와 원인

원 해결 #4739/#4800(`72952055b3`), #4961(`950de31563`), 명시 환경 #6389/#7179(`263b61a64a`), 주소별 공급 선택 `8c61edaaf5`·`36440d7dc0`, 이 브랜치 보정32·34의 의도를 대조했다. 기존 코드는 주 글꼴과 `substFont`에 같은 조회 이름을 적용했다. 온새미로 HWP의 대체 필드는 `한컴바탕`, HWPX는 영문 `HCR Batang`이고, 독립 원본 PDF의 제목·하단 문구는 두 형식 모두 Haansoft Batang이다. 원본 요청 `a로케트`가 이 프로그램과 같다는 뜻은 아니다.

같은 문장을 가진 생성 HWPX를 한컴2024에서7조건,2020에서3조건 실제 Print했다. 생성 진단 입력을 정상 저장 원본이나 배치 golden으로 쓰지 않았다. Windows에 HCR이 실제 설치돼 있어 단순 프로그램 결측도 아니었다. typeInfo 없음/Gothic 대조도 영문 대체 필드의 결과가 같았다.

|주 글꼴/문서 대체|독립2024|독립2020|
|---|---|---|
|a로케트 / 영문 HCR Batang|Haansoft Batang|Haansoft Batang|
|a로케트 / 함초롬바탕|HCRBatang|HCRBatang|
|a로케트 / Arial|영문ArialMT·한글Haansoft Batang|미실행|
|a로케트 / 한컴바탕|Haansoft Batang|미실행|
|주 글꼴 HCR Batang / Arial|HCRBatang|HCRBatang|
|주 글꼴 함초롬바탕 / Arial|HCRBatang|미실행|

2024는13.0.0.3901의 `frame_print_to_pdf_ex_one_up`,2020은11.0.0.9136의 `hancom2020_pdf_driver_one_up`이다. 모두 Print method0/preprocess none이다. [입력/PDF SHA·실제 embedded font 이름의10조건 aggregate](../../../../../../assets/font-rules/evidence/issue-7445-document-substitute-lookup.json)에 연결한다. 글꼴·표본·PDF 바이트를 이 aggregate에 포함하지 않았으며 프로그램 윤곽 동일성을 주장하지 않는다.

## 공통 수정과 소비 경로

`lookup_font_name_decision → document_requests → 실제 Native 조회/브라우저 공급 snapshot → FontEnvironment → 측정·paint`가 같은 유효 대체 후보를 소비한다. 영문 HCR을 전역 별칭으로 바꾸지 않고 **부모와 대체 모두 TTF·비내장인 문서 대체 필드**에서만 유효 후보를 조회한다. 원본 IR·직렬화·원본 존재 우선순위·명시 사용자 환경·내장 프로그램은 보존한다. HFT/미상 선언, 한글 HCR 대체와 주 글꼴 HCR은 유지한다.

canonical v2에 issue-scoped change16을 append하고 projection을 재생성했다. 규칙은 `rule.rust-style-resolution.8f1a484b6e8dff748728`, boundary는 `rust-style-resolution.document-substitute`다. layout-name1규칙만 추가됐고 나머지 네 projection의 의미는 유지한다. raw 대체 이름과 유효 후보를 trace에서 구분하고 후보 조회를 실제 공급/paint 증명으로 표시하지 않는다. 실제 trace의 lifecycle 감사도 통과했다.

## 원본 전쪽 실제 출력과 고정 base

동일 원본·독립 Print,96DPI/PDFium153 smooth_rgb_v1/관용2px/raw90%·각 형식의 compat2022/2024를 유지했다. 원본·PDF·글꼴·폭 표·90%/0.1%p 기준은 변경하지 않았다. 두 원본은 각각46쪽이며 누락·추가 쪽0이다.

|원본|Native 전46쪽 최저|fresh WASM 전46쪽 최저|각90% 미만|
|---|---:|---:|---:|
|HWP|91.43430%|94.45835%|0|
|HWPX|91.43430%|94.45835%|0|

[HWP Native](onsamiro-hwp-native.tsv) · [HWP fresh](onsamiro-hwp-fresh.tsv) · [HWPX Native](onsamiro-hwpx-native.tsv) · [HWPX fresh](onsamiro-hwpx-fresh.tsv).

HWPX46쪽은 앞 단계 Native/fresh90.48362%에서 양쪽100%로 개선됐다. Native의 #8064 하한 대비 하락13쪽과 고정809 base 대비 하락11쪽이 모두 해소됐다. HWP도 두 기준 대비0.1%p 이상 하락0이다. [Native 전92행 대조](native-paired-pages.tsv).

정확한80968497의 frozen fresh package도 같은 원본·글꼴·PDF로 각각 전46쪽 새로 산출했다. base 최저는 HWP90.54007%,HWPX91.26532%이며 후보는 두 형식 모두 base 대비0.1%p 이상 하락0이다. [base HWP](base809-onsamiro-hwp-fresh.tsv) · [base HWPX](base809-onsamiro-hwpx-fresh.tsv) · [fresh 전92행 대조](fresh-paired-pages.tsv). base에는 새 자동 선택 ABI가 없어 기존 setter에 null을 전달해 실제 base의 기본 None 계약을 실행했다. 새로운 ABI가 base에도 있다고 가장하거나 guard를 수정하지 않았다. 공급 글꼴의 준비/hash 검사는 유지했다. #8064 전체 WASM 래칫을 실행했다는 뜻은 아니다.

입력/PDF SHA256:

- HWP `e8592e74c9a8425c4ee2c5824d012ebe45e9f6dd36880b784ba594b4fd0a31ce`; `pdf/[2027] 온새미로 1 본교재-hwp-2020.pdf`는 `450c793f33a134198936504b38d3b21c01a343a11c09c13e5a7a00e39e2d58c7`.
- HWPX `85071b419b3467466ec391ecde187ad25723ce564178c2ed37c53b09e731138c`; `pdf/[2027] 온새미로 1 본교재-hwpx-2024.pdf`는 `051aa8c4b94ca7cddc21e18d7fdc45b370e492b65c711c801d2c95479117cbab`.

## 회귀와 직접 판독

같은 최종 `issue_7445_document_substitute_lookup.rs` SHA `eaf0298b1bd32e0b3c5f0bca0f6e38c536008394d56e1f1f655c9f16b7ff68e7`에서 수정 전3FAIL→후3PASS를 확인했다.

1. 7언어 슬롯의 대체 필드/primary·영문/한글·TTF/HFT/미상·원본 직렬화를 구분한다.
2. 실제 가용성 API의 원본 우선·내장·명시 사용자 환경과 raw trace 선언을 보존한다. snapshot의 programId는 모형이며 실제 폰트 바이트 증명이 아니다.
3. 두 원본 전쪽이 Native/fresh90% 이상이고46쪽 양쪽100%·review/overlay 직접 판독을 확인한 뒤 제목 source-cell 소유·순서·누락·중복·겹침과 독립 Print의 시작44.16pt/끝213.78pt를 ±2CSSpx로 보호했다. 이전 끝266.853667px는 독립285.04px와 달라 실제 FAIL이며 후보284.826667px는 PASS다. 원본 저장 데이터는 그대로다.

기존 KoPub/World 공급 API48조건과 다른 굵기·세대 반례는 유지했다. 모형 공급 목록에는 독립 Print에서 확인한 유효 대체 Haansoft Batang을 추가했고 HCR도 유지했다. 기존 본문41쪽의 네 경로·소유/순서/포함·±2px 검사는 그대로 통과했다. 별도 기존 명시 환경5·trace9와 함께 기존16PASS다. 새3개와 일부 중복되는 정식 자동 suite의 실제5실행5PASS(0.619초,컴파일 포함298.051초,exit0)를 확인했다.784개는 필터 미선택이며 전체 회귀 skip으로 합산하지 않는다. prepare의1685sources/7413attrs/48targets와 fmt·manifest/base809·unit tiers/base809 및 registry/projection42PASS를 확인했다. 전체 nextest/Clippy·전체 필수 gate 완료가 아니다.

46쪽 제목 폭·문구·하단과1쪽 표지를 Native/fresh review·standalone overlay로 직접 읽었다. 작은 raster/color 경계는 남고 raw100을 RGB 동일성으로 표현하지 않는다. 수정 전후 모두 나온 page0 동일 문단 table overlap18.9px 경고는 숨기지 않았다. 실제1쪽은 양쪽 raw100%와 독립 Print 판독에서 글자 누락·겹침이 보이지 않았다.

|비교|review|overlay|
|---|---|---|
|이전 f0e8 HWPX46|[review](before-f0e8-native-hwpx-p046-review.png)|[overlay](before-f0e8-native-hwpx-p046-overlay.png)|
|후보 HWPX46 Native|[review](after-native-hwpx-p046-review.png)|[overlay](after-native-hwpx-p046-overlay.png)|
|후보 HWPX46 fresh|[review](after-fresh-hwpx-p046-review.png)|[overlay](after-fresh-hwpx-p046-overlay.png)|
|HWP46 Native 정상 대조|[review](after-native-hwp-p046-review.png)|[overlay](after-native-hwp-p046-overlay.png)|
|HWP46 fresh 정상 대조|[review](after-fresh-hwp-p046-review.png)|[overlay](after-fresh-hwp-p046-overlay.png)|
|표지 Native 경고 대조|[review](cover-native-hwpx-p001-review.png)|[overlay](cover-native-hwpx-p001-overlay.png)|
|표지 fresh 경고 대조|[review](cover-fresh-hwpx-p001-review.png)|[overlay](cover-fresh-hwpx-p001-overlay.png)|

## 실제 producer와 남은 범위

- Native CLI `4cb0d0dede787cfbb59d290952ba31381416111c39a16e3e2fece5ac89485979`, SDK `9f049070d3a1ca63b2d39f2abaa9aef18622d5038484efe20b03eb4614ff2a6d`; locked release-test/native-skia jobs8,exit0,124.807초.
- fresh JS `d7d8cf9b5f7ce8a0f00b49a39aa1fee49e331cce6774e7ba57c3e9d078cd7030`, WASM `2751ae6749025a541a712ea236e66244d759298e70949d3702b4da9a3270b0f6`; locked web/pkg/no-opt,exit0,335.040초. Mac 로컬 대체 빌드이며 Docker 최적화 통과가 아니다.
- root pkg/Studio public/frozen hash 일치. 실제 Studio7736의 새 탭 초기화 JS/WASM 응답 각각HTTP200/hash 일치·초기화 true·오류0이다. 문서 paint·CanvasKit·Actual36 검증을 대신하지 않는다.
- source는 f0e8fc51a2 위 이 단계의 runtime3파일이다. style resolver SHA `259128949ab7df6601518a53e7433b46cdea568455c874a5d72675156c3795e9`, generated layout-name `d1144a6c73d54dee52e9d61d1bdb24cc8ddb58587577bc6cddeeced170ec94bd`, font trace `7e168aaad36c4fa63d0e36b86b4c1dd8fbb0a9653f1d553c6aaf3503c3530ea9`를 커밋 전 실제 파일과 대조했다.
- 공개 request 비교179조건 중35조건이 달라지고 실제 Native environment 선택은31조건이 달랐다. 온새미로 HWPX 외 교육30조건을 Native/fresh 전쪽 실행 중이다. environment가 같았던 나머지4조건도 secondary CSS/glyph 후보가 달라졌으므로 실제 출력 대조를 추가했다. 선택 JSON만으로 영향 없음을 주장하지 않는다.
- 교육2022 18쪽은98.34424→98.24070%(-0.10354pp)의 기존 미달이 남는다. frozenV6와 현재 PNG hash가 같음은 이번 필드 조회 수정이 만든 하락이 아님을 보일 뿐, 수용/이관 근거가 아니다. 현재 브랜치에서 계속 개선한다. WMF·다른 기존 문서·최신 전체 Native 감사·전체 nextest/필수 gate/Actual36도 남아 있다. PR 준비 완료가 아니다.
- 초기 잘못된 registry 초기화 명령·시스템 Python의 PDFium 미준비·추적 reason 규약 오류 실행은 최종 증거에서 제외했다. 본인 산출물만 보존/정정한 뒤 canonical reducer로 변경을 append하고 최종V18의 Native/fresh 전쪽·actual trace audit·정식 suite를 다시 실행했다. 로그·중간 JSON·진단 source는 ignored `output/pr-review/7445-completed-20261009/rebase-80968497-20261021/document-substitute-lookup-v18/`에만 있다. 지침·원격·원본·다른 작업장은 변경하지 않았다. 공유 target은 승인된 후보 빌드 캐시로 순차 사용했고 임의 삭제하지 않았다.
