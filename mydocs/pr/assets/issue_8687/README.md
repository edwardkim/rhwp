# #8687 글꼴 이름과 실제 대체 출력 증적

실제 검증을 실행한 제품 source는 `e4aefbcce7bf3db783ec39f3223e79b53082fbc1`, 초기 기준 devel은
`08963f6825956984d38b0d9392f44777db7107ab`이다. 입력·독립 PDF의 전체 SHA-256과
MCP job은 [input-provenance.json](input-provenance.json)에 있다. 원본은 HWP가 아닌 HWPX다.

| 식별자 | 검토할 입력 | 독립 Print PDF | 목적 |
| --- | --- | --- | --- |
| repro | [font_list_name_fallback.hwpx](../../../../pdf/issue8687/font_list_name_fallback.hwpx) | [2024 PDF](../../../../pdf/issue8687/font_list_name_fallback-2024.pdf) | 사용하지 않는 목록 이름도 전체 관측 환경을 거부하는 오류 |
| used | [font_list_name_used_fallback.hwpx](../../../../pdf/issue8687/font_list_name_used_fallback.hwpx) | [2024 PDF](../../../../pdf/issue8687/font_list_name_used_fallback-2024.pdf) | 본문이 목록 이름을 실제 사용해도 선언된 함초롬돋움으로 대체 |
| control | [공개 원본](../../../../samples/header-footer-paper-frame.hwpx) | [기존 2024 PDF](../../../../pdf/pblsketch_review_20261030/header-footer-paper-frame-2024.pdf) | 정상 KoPub 본문·머리말·꼬리말 보존 |

두 진단 입력은 ZIP의 `Contents/header.xml`만 바꿨으며 본문·줄 정보·개체·용지는 원본과 같다.
`repro`는 언어별 3번 face에 `산돌고딕 L\,한컴돋움`과 TTF 대체 `함초롬돋움`을 선언하고,
본문에서 사용하는 5번 face는 정상 `함초롬돋움`으로 둔다. `used`는 5번도 같은 목록 이름과
대체 선언을 사용한다. 독립 한컴 2024 Print는 두 입력에서 HCRDotum을 선택했다.

## 전쪽 측정과 직접 판독

96 DPI, Print profile, 2024 조판 세대, PDFium smooth RGB, raw 2px 실루엣,
색상 경계 보정 0px, 제외 영역 없음. 입력·PDF·Native·WASM은 각각 1쪽이다.
정확한 raw 값·분모·불일치 픽셀은 [visual-results.json](visual-results.json), 전쪽 원문은
[tsv/](tsv/)에 보존한다. SVG raster는 보조 진단이며 이 표의 Native·Canvas 결과를 대신하지 않는다.

| 입력 | Native raw | fresh WASM Canvas raw | 90% 미만·누락·측정 불가 |
| --- | ---: | ---: | --- |
| repro | 91.21255% | 91.09617% | 0 |
| used | 91.21255% | 91.09617% | 0 |
| control | 91.10204% | 91.12426% | 0 |

각 경로의 `*-review-p001.png`, `*-overlay-p001.png`, `*-compare-p001.png`와 실제 PNG는
[native/](native/)·[wasm/](wasm/)에 있다. [본문 3배 확대](used-font-backends-body-3x.png)는
원래 96 DPI의 `(80,118)-(400,182)`을 nearest-neighbor 3배로 확대한 비교다. 위치·glyph·
자간·세 줄의 간격을 독립 Print와 대조했다. 해당 입력에는 수식·첨자가 없다.
머리말·꼬리말과 BOX도 전쪽 review에서 직접 확인했다.

정상 공개 원본의 BOX 높이·기준선 차이는 #8165의 기존 수용 기록과 같으며,
기준 Native와 후보 control PNG는 byte-identical이다. 근거는
[PR #8165 메인터너 기록](../pr_8165_maintainer/README.md)이다. 이 변경은 BOX 배치를 바꾸지 않는다.
일치율이나 이 기존 차이를 근거로 좌표·쪽수 기대값을 새로 고정하지 않았다.

[content-relations.json](content-relations.json)은 Native와 fresh WASM의 같은 쪽에서
HEADER C, 본문 세 줄, FOOTER C, BOX의 각 1회 보존·순서·소유 경로를 확인한 진단 증적이다.
정식 회귀는 `tests/cases/issue_8687_declared_font_fallback_names.rs`의 API 관계 검사이며,
설치 글꼴이나 위 실물 픽셀의 고정값에 의존하지 않는다.

## 실제 자원과 Studio 화면

Native의 최종 선행 진단 빌드는 두 번째 보정을 완료한 뒤 `e4aefbc`를 커밋하기 전에
수행했다. 당시 HEAD는 `86aaa4f`이며 제품 source bytes는 이후 기록한 `e4aefbc`와 같다.
최적화 WASM과 전체 사전 검증은 `e4aefbc`에서 수행했다.
[font-programs.json](font-programs.json)에 제품 파일 blob 대조와 Native binary·최적화 WASM·실제 HCRDotum
프로그램의 SHA-256을 기록했다. 요청 family 이름만으로 실제 글꼴 공급을 판정하지 않았다.
[wasm-observation.json](wasm-observation.json)의 실제 공개 WASM API는 부적합한 관측 요청 0건,
환경 갱신 오류 없음, 실제 표시 체인 `함초롬돋움`을 확인한다. 이 API 관측 대역과 실제 paint
공급 증거는 서로 다른 검사다.

[studio/](studio/)는 실제 CanvasView 화면의 페이지 사각형 캡처다.
Chrome 156.0.8078.79, DPR 1, zoom 1, screen profile, Canvas2D 및 CanvasKit software를 사용했다.
세 입력·두 backend 모두 초기화 때 받은 JS·WASM 응답을 CDP에서 직접 수집했다.
WASM은 package 해시와 일치한다. Vite의 JS 응답은 WASM URL 한 줄만 절대 경로로
바꾼다는 것을 source map 원문으로 확인했고, 그 한 줄을 역변환한 모든 bytes의 해시가
빌드 JS와 일치한다. 나머지 변환이나 다른 코드 차이를 허용한 것이 아니다.
CanvasKit 실제 backend 선택·runtime readiness 통과·자원 오류 0·미등록 글꼴 fallback 0을
[provenance](studio/provenance.json)에 보존했다. 켜진 caret·편집 가이드는 화면 UI이며
Print Sweep 점수로 평가하지 않는다.

## 최소 글꼴 환경과 편람

Rust 회귀는 `bwrap`으로 시스템·사용자·`/opt` 글꼴을 숨기고 저장소의 공개 글꼴 5개만
Fontconfig에 노출한다. 기존 테스트가 명시 공급하는 라이선스 확인 fixture는 보존한다.
프로필은 [실행 wrapper](minimal-font-profile.sh.txt)·[Fontconfig](minimal-font-fontconfig.xml)에 있다.
보존본 wrapper는 같은 Fontconfig 내용을 정식 증적 경로에서 읽도록 경로만 바꿨다.
`sh mydocs/pr/assets/issue_8687/minimal-font-profile.sh.txt <검증 명령>`으로 재현할 수 있다.
`/run/systemd/resolve`는 DNS를 위한 읽기 전용 mount이며 글꼴 노출을 추가하지 않는다.

첫 편람 검사는 격리 밖에서 실행해 strict gate의 145쪽에서 실패했다. 기준과 후보 모두 같은
99.95409%였다. 이를 통과로 간주하거나 기준 TSV·허용치를 바꾸지 않았다.
설치 KoPub 6개 face의 SHA는 고정 웹폰트와 모두 같았다. 차이가 난 `행정안전부장관`은
원본에서 `정부상징 부처명_16040911`, 쪽번호는 `Haansoft Batang`을 요청한다.
[4배 확대 진단](handbook/p145-environment-diagnosis-4x.png)의 위→아래는 설치 글꼴·
공개 글꼴 격리·독립 Print다. 최소 환경에 정부상징 파일을 다시 공급하면 해당 문구 영역은
첫 실행과 byte-identical이다. KoPub 파일 차이로 단정하지 않는다.

145쪽 단독 자동 웹폰트 공급은 raw 99.99738%, 공개 프로그램을 직접 지정한 별도 진단은
99.99869%다. 둘 다 strict gate의 2자리 표시값은 100.00%이나 실제 raw 100%는 아니다.
정식 전쪽 검사는 직접 font-path를 지정하지 않는 최소 환경의 자동 웹폰트 공급으로 실행했다.
기준·후보 모두 383/383쪽, 최저 raw **99.96465%**, strict gate `passed`, 종료 코드 0이다.
383쪽의 실제 PNG가 기준·후보에서 모두 byte-identical이다.
[전쪽 TSV·gate·실제 공급 provenance](handbook/results.json)를 보존했다.
전체 회귀의 최종 결과는 [작업 보고서](../../../report/task_m100_8687_report.md)에 연결한다.

작업지시자의 명시 지시에 따라 `ubuntu-ted`, `ubuntu-ted2`, 현재 실행 호스트 `ubuntu-ted3`의
정부상징 설치본과 현재 작업의 진단용 복사본을 제거했다. 캐시 갱신·파일 부재·Fontconfig 목록
제외를 모두 확인했다. [삭제 및 환경 진단 기록](handbook/environment-diagnosis.json)에 연결한다.
최소 환경에서는 처음부터 이 글꼴을 숨겼으므로 진행 중인 정식 전쪽 검증 조건은 바뀌지 않았다.

기존 `canvaskit-font-preparation-issue7688.test.mjs`의 실제 자원·style·glyph·원본 보존·
실패 fallback 검사 11건도 모두 통과했다. 첫 실행은 새 브라우저의 로컬 글꼴 선택창에서
대기해 CDP timeout이 났다. 재실행은 표시된 `대체 글꼴로 보기` 버튼을 자동 클릭했으며
기존 테스트 source와 assertion은 그대로다. timeout 실행을 통과로 기록하지 않았다.

전쪽 최저값인 편람 130쪽의 남은 차이는 작업지시자가 `130쪽은 해결불가능임. pass`로
명시 수용했다. raw 99.96465%·원래 TSV·strict gate 결과는 그대로 보존한다. 이 판정으로
다른 페이지나 다른 실패의 검증 조건을 바꾸지 않는다.

SVG 실제 browser raster의 별도 소비 확인은 [svg/results.json](svg/results.json)과
[review](svg/review_001.png)·[overlay](svg/overlay_001.png)에 보존했다. used raw는
91.34948%이며 정상 control은 기준·후보 모두 TSV 표시값 91.73%다. SVG의 기본 설치 검색은
HCRDotum `1c0dce5a…`를 임베딩했고 Native/WASM의 공급 프로그램은 `1d043790…`다.
실제 프로그램 신원을 구분해 기록했으며 같은 bytes라고 주장하지 않는다. 양쪽 모두
원본의 선언 대체인 함초롬돋움이고, SVG의 색상 프린지와 본문 glyph·자간·줄간격을 직접 확인했다.

직접 PDF backend도 두 입력을 실제 내보내고 PDFium 96 DPI로 독립 Print와 대조했다.
[결과와 실제 내장 글꼴](direct-pdf/results.json)의 used raw는 91.67619%, control은
91.68260%다. HCRDotum 임베드와 내용 6종의 각 1회 보존을 확인했고
[같은 배율의 본문 확대](direct-pdf/used-body-print-3x.png)에서 모양·간격을 판독했다.
이 자료 역시 Native PNG/fresh WASM 전쪽 필수 gate와 별개인 실제 소비 경로 증적이다.

최종 제품 head는 `87c073fa7094603f6915c43851da7ed66697565b`, PR base는 `45684640be5eddcf5972b063260250345b34d928`다.
무충돌 rebase와 두 patch의 동일성·핵심 파일 blob 동일성을 확인했다.
작업지시자의 명시 승인에 따라 위 원래 source의 증거를 재사용하며 새 head에서 재출력한 것으로
표시하지 않는다. [재사용 기록](rebase-reuse.json)·[전체 검증 요약](validation-results.json)을 따른다.
