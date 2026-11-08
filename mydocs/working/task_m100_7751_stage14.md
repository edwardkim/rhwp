# #7494·#7751 최신 devel 리베이스와 제출 검증 — 14단계

새 base는 `b7a538803`(#7953)이며 이전 base는 `fe5725dc3`이다.
리베이스 전 head `b8ce9d753`와 전체 작업을 backup branch로 보존했다.
30커밋은 충돌 없이 적용됐고 리베이스 직후 head는 `843164b5a`다.

## 변경 영향과 검증 순서

새 base는 PDF 가져오기 기본 feature와 parser/PDF·의존성·WASM 노출을 바꿨다.
기존 HWP 조판·paint·그림 처리의 소스 diff는 없다. 이전 출력은 실제 이전 SHA의
증거로 보존하며 새 head의 실행으로 바꾸어 기록하지 않는다.

1. 공용 `/home/tsjang/rhwp/target/pr-review`에서 jobs8로 Native와 fresh WASM을
   순차 빌드한다. WASM은 기존 host 대체 경로와 codegen-units16을 사용한다.
2. 같은 HWP·기존 독립383쪽 Print·글꼴을 사용해 새 제품 전쪽 Native/fresh WASM을
   다시 산출한다. raw90%·누락0·쪽수·고정 devel 대비 하락 기준을 유지한다.
   대표 PNG와 실제 Studio DOM·SVG·Direct PDF 소비도 새 제품에서 확인한다.
3. 정식 focused·전체 nextest·Native Skia3종·모든 Rust lint와 suite/unit 정책을
   같은 target에서 순차 검증한다. 중단된 이전 nextest를 전체 통과로 세지 않는다.
4. upstream HEAD를 다시 확인하고 승인된 통합 PR을 제출한다. code candidate의
   CI 성공 뒤 같은 PR에 archive review·구현 기록·오늘할일을 trailing commit한다.
   trailing CI·정상 merge·post_merge.md의 issue/comment/정리까지 완료한다.

TSV·명령·실행 결과·빌드 지문은 ignored
`output/pr-review/pr7494-20261016/stage14-*`에 보존한다.
GitHub 본문에는 전쪽 TSV와 정확한 head SHA의 실제 PNG를 표시한다.
아직 실행하지 않은 단계는 완료로 판정하지 않는다.

## 새 제품의 전쪽·소비 경로 검증

실제 source `465fd10fded3bec623d3b884461b3aa6bba9e227`에서 Native 빌드는
469.302초, fresh WASM 빌드는436.040초에 각각 exit0이었다. Native library도
별도 --lib 빌드 후 고정해 이전 산출물을 새 library로 오인하지 않았다.

같은 독립 Print의 PNG383쪽은 원본 PDF SHA와 쪽별 PNG SHA를 대조해 재사용했다.
RHWP 출력은 Native Skia와 fresh WASM Canvas 각각383쪽 모두 새로 산출했다.
최저 raw는93.07066%/92.98362%,90% 미만·누락·고정 devel 대비0.01%p 이상
하락은 양쪽 모두0쪽이다. 두 경로의 새 PNG·render tree는 각13단계 생산본과
전383쪽 모두 바이트가 같았다. 실제 생산 SHA는 바꾸어 기록하지 않는다.

전383쪽의 bbox를 제외한 전체 구조·소유·내용·순서도 Native/WASM이 같았다.
Native usize::MAX 범위의 내부 sentinel만32비트 값으로 대응했고 실제 문단 ID는
바꾸지 않았다. 원본 각주1·미주0을 유지하고 두 경로의 FnMarker/FootnoteArea가
11쪽에 각각1개씩 있다. 독립 Print의 각주 판독은 별도 증거로 유지한다.

실제 한컴 저장 색조 HWP5개는 Native/fresh WASM/Direct PDF 각각 전쪽 raw100%다.
실제 Studio DOM·Canvas·HFT·WMF crop·FontFace 브라우저8건도8PASS였다.
Canvas550패치와 DOM550패치를 각각 screenshot/픽셀로 독립 Print RGB와 비교했다.
SVG 실제 브라우저3/130/375쪽 raw는99.99690/98.61065/99.46269%였다.
Direct PDF3쪽도 새 PDF를 래스터화해 raw100%였다.63쪽 연결선과130쪽 SVG payload는
예상한 기존 손실 방지 오류로 exit1이며 단순한 변환 실패를 성공으로 바꾸어 세지 않는다.

새 대표 review·overlay29개와 출처는
[안정 증거 경로](../pr/assets/pr7494_issue7751/README.md)에 보존했다.
이 중 DOM 이전/Print/보정 비교는13단계의 실제 screenshot이며,14단계 실제 브라우저에서
같은550개 DOM 색 값을 재확인했다. 이전 screenshot을 새 캡처로 표시하지 않는다.
375쪽 서식의 상위 원점·내부 칸·제목과 후속 본문도 직접 대조해 기존 보정을 유지했다.

Studio2,023PASS/0FAIL/0SKIP, TypeScript exit0, Visual Sweep Python156PASS다.
정식 focused는 새 source1·Native 색조2·종이/프레임2·편람 완료 경계5·placeholder2·
Direct PDF14의26건 모두 PASS였다. Native Skia 전체 lib·기본 전체 nextest·lint와
정책 검사는 아직 이 focused 결과로 대신하지 않는다.


## 전체 필수 로컬 검증과 직접 판독의 추가 보류

기본 전체 nextest는10,594PASS/0FAIL/45기존skip으로 종료됐다. 빌드12분15초,
실제 테스트716.078초이며 전체 명령1453.922초다. SVG snapshot4건과 마지막
텍스트 중첩 partition15도 통과했다. Native Skia lib는root3,861+workspace182의
4,043PASS/0FAIL/12기존ignore였다. fmt·Native/WASM Clippy·workspace build·
workspace 전체 target Clippy·고정 base manifest/unit-tier까지 모두 exit0이었다.
이 결과는 실제 source465fd10fd의 결과이며 후속 코드 변경의 통과로 바꾸어 쓰지 않는다.

문서의 이전100% 체크표 한 곳을 승인된 raw98% 이상·직접 판독·±2px 조건으로
맞췄고, 로컬 검증 가이드의 이전SVG raster 설명도 현재 기본 Skia/Canvas 직접 PNG와
별도 SVG 진단으로 구분했다. source/test/workflow는 이 문서 정정으로 바꾸지 않았다.

289쪽의 완료 경계 근거를 보강하려고 새 Native/WASM review를 추가 생성했다.
직접 확대 판독과 독립 PDF의B/f* path 좌표를 대조한 결과, 그룹에 속한 화살표만
약26.3px 왼쪽에 있음을 발견했다. 예를 들어 Native x232.5 대 PDF258.76px,
두 번째 그룹 x230.2 대 PDF256.5227px다. 묶이지 않은 선의 위치는 약0.4px 차이다.
기준 devel도 가로 원점은 같고 세로 차이는 이 브랜치의 표 흐름 보정으로 개선됐다.
따라서 이는 이번 색조 수정의 신규 회귀는 아니지만 현재 보정 문서의 미해결 배치다.
raw99.6%·CI 대신 이 차이를 먼저 고치며 PR은 아직 생성하지 않는다.
소스·측정/배치 소비·반례를 추적하는15단계로 이어간다.
