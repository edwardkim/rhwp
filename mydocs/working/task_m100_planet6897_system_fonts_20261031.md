# planet6897 통합 검토 — 시스템 글꼴 공급

> **기준 실행 파일 정정 (2026-11-01):** 이 기록에서 `base-rhwp` SHA
> `dea7033a0dd1d17a53a8c602e2f5fbe045b040f49e2880f99a341cd5f669ea11`을
> #8397 고정 커밋의 원 PR 실행 파일로 사용한 동일 환경 비교는 잠정 자료다.
> 깨끗한 `99a176b7cb34fde54454c03433baa1382b2de9fa`에서 다시 빌드한 실행 파일
> SHA `545abe7699e2aa8f527d6868e390b1dff76bade449823e05773c0443d10204e3`의
> 같은 GIF JPEG 출력은 이전 `base-rhwp`와 다르고 현재 후보와 같다. 이전 기준
> 실행 파일의 빌드 상태는 확인되지 않아, 아래에서 그 파일과 비교한 원 PR 대비
> 하락·동일 판정은 새 실행 파일로 재확인하기 전까지 확정하지 않는다. 제출된
> TSV와 직접 대조한 수치는 별도로 보존한다.
> `base-99a-fresh-build.log`, `verify-fresh-base-gif.log`,
> `fresh-original-revalidation.log`를 참고한다.

2026-11-01 06:24 KST에 GitHub API로 원 PR head를 재확인했다. #8161은
`bd786fc5fefb94126ac146411f603c9cbcf336a6`, #8346은
`923ae70036a6b6e5ce94bf9c45cd2f8326cf8551`로 각 개별 리뷰에 기록한 조회값과
같다. 두 PR head 변경을 이유로 순서대로 체리픽을 다시 시작할 상황은 아니다.
같은 시각 `git ls-remote upstream refs/heads/devel`은 고정 base
`bc9dfdd057258de59afebeb5121c4bee7398e07f`와 일치했다. PR 제출 직전에는 다시 확인한다.

## Visual Sweep 메타파일 글꼴 공급

시스템 Fontconfig에서 지원 글꼴 2,069개를 자동 열거한 WASM 비교에서 실제 요청 문자에
필요하지 않은 메타파일 글꼴까지 등록해 64 MiB 상한을 초과했다. `sample16` HWPX와
기존 한컴 2024 Print의 전쪽 비교가 종료되지 않는 재현이었다.

메타파일과 그림자 텍스트의 글꼴 후보를 공통 glyph coverage로 선택한다. 같은 family의
분할 CFF 프로그램을 실제 공급 순서로 조사하고, CFF가 없을 때는 공급되지 않는 두 번째
TTF를 후보로 인정하지 않는다. cmap에는 있지만 표시 윤곽이 비어 있는 기호도 공급
완료로 취급하지 않는다. 시스템 글꼴 열거를 줄이거나 바이트 상한을 높이지 않는다.

Node 집중 검사 15건은 모두 통과했다. 같은 시스템 글꼴에서 HWPX 64쪽 WASM 비교도
종료 코드 0이며 #8397의 해당 문서 정밀 TSV보다 하락한 쪽은 없었다. 로그와 산출물은
`output/pr-review/planet6897-ordered-20261031/metafile-font-selection-final-tests.log`와
`output/pr-review/planet6897-ordered-20261031/check-sample16-metafile-candidate2-all/`에
있다. 글꼴 파일은 저장소에 추가하지 않는다.

전체 문서의 Native·WASM 정밀 대조와 선택된 실제 글꼴의 원인 분석은 별도로 계속한다.
위 64쪽 결과만으로 전체 통합 검토 완료나 다른 문서의 무하락을 주장하지 않는다.

## 정밀 기준 하락의 원인 분리

`sample16` 18쪽의 base와 후보를 같은 시스템 글꼴, 같은 명시 글꼴 환경과 Print로
재측정하니 Native PNG SHA-256과 미대응·합집합 정수가 모두 같았다. 이전 환경이 다른
base 출력과의 차이를 코드 회귀로 단정하지 않는다. 다만 이 시스템 글꼴 조건의 결과는
#8397에 보존된 정밀 기준보다 낮다.

당시 사용한 글꼴 목록에는 `HCI Poppy`의 완성된 기호 포함 프로그램이 명시됐고, 현재
시스템에는 같은 family의 다른 프로그램도 등록돼 있다. 원래 프로그램을 우선 공급하는
진단은 다른 시스템 글꼴을 노출한 상태에서 미대응/합집합을 `15/206244`에서
`15/206246`으로 바꿔 기존 정밀 기준을 복원했다. 이는 특정 입력에 대한 진단이며
자동 선택의 공통 규칙이나 다른 페이지의 검증을 대신하지 않는다. 입력·환경·로그는
`output/pr-review/planet6897-ordered-20261031/sample16-p18-full-system-poppy-diagnostic/`에
있다. 검증 글꼴 프로그램 자체는 저장소에 추가하지 않는다.

편람 145쪽은 원본에 지정된 정부상징 글꼴이 현재 시스템에 설치돼 있고, 기존 Print는
대체 한컴바탕을 사용한다. #8389의 실제 공급자 계약을 따라 원본 설치 여부를 무시한
문서별 강제 치환은 적용하지 않았다. #8040 기호 슬롯은 원 PR의 독립 Print 계약과
함께 별도 분석 중이다. Native 12행과 WASM 8행의 기존 정밀 기준 하락을 해소하기 전에는
검토를 완료하지 않는다.

같은 진단을 HWP 2024 원본 18쪽에도 실행해 `15/206244`에서 `15/206246`으로
복구했다. 이 두 결과는 프로그램 선택에 따른 재현이며 제품의 기본 자동 선택을
수정한 결과가 아니다. 해당 시스템에는 같은 family의 서로 다른 CFF 프로그램이
존재한다. 기존 glyph 목록의 합집합 크기만으로 프로그램의 원본 신원을 추측하지 않는다.

공통 공급 환경의 `programId`는 식별 및 환경 fingerprint에 사용되지만 스타일 해소는
현재 최종 family를 반환한다. 다만 이번 sample16 환경 JSON은 Poppy programId를
포함하지 않아 이 반환 경계가 이번 결과의 직접 원인이라고 단정하지 않는다.
서로 다른 동일 family 프로그램의 선택과 backend 전달을 독립적으로 검증해야 한다.

## 최신 base 재적용과 회귀 수집 점검

`bc9dfdd057258de59afebeb5121c4bee7398e07f` 위에 체리픽을 다시 적용한 후, 기존
생성 목록을 갱신하지 않고 최소 글꼴 전체 회귀를 실행했다. 이 목록에서는 12,320건이
통과하고 45건이 건너뛰어졌지만 새 base #8410의 PDF 패턴·문자 획 검사 두 모듈이
빠져 있었다. 이 결과만으로 최신 source의 전체 검사가 완료됐다고 판단하지 않는다.

`node scripts/rust-test-suite-manifest.mjs --prepare`와 고정 base `--check`를 실행해
두 모듈을 수집 목록에 포함했다. 사용자 요청에 따라 이미 통과한 검사를 다시 반복하지 않고
같은 최소 글꼴 환경에서 누락된 6건을 따로 실행했으며 모두 통과했다. 전체 목록과 누락분의
결과는 각각 독립된 실행으로 기록한다. 생성기 계약 검사 23건도 통과했다.
첫 번째 실행과 집중 검사 사이 제품 Rust 코드는 바뀌지 않았고, 회귀 검사 주석만 변경했다.

Visual Sweep의 재개 기능은 입력 해시를 확인하므로 기존 raw gate 단위 검사의 가상 경로가
존재하지 않아 실패했다. 테스트가 입력 파일과 해시를 준비하도록 보정했으며 90% 경계와
반올림 전 판정 기대값은 유지했다. Visual Sweep Python 검사 210건과 글꼴 공급 Node 검사
28건이 통과했다. 해당 실행 로그는 검토용 `output/pr-review/planet6897-ordered-20261031/`
아래에 두며 Git에는 포함하지 않는다.

리베이스 전 후보의 TSV 하락 12행(Native)·8행(WASM)은 새 후보의 결과가 아니다. 새 base의
제품 코드로 Native와 fresh WASM을 빌드하고, 같은 시스템 글꼴과 독립 Print를 사용해 새
출력 경로에서 다시 측정해야 한다. 빌드 head와 바이너리 해시가 맞지 않으면 재측정
스크립트가 시작하지 않도록 준비했다.

## #8040 전체 그림의 자르기 오판 원인과 보정 계획

#8040 두 원본의 2쪽에서 이전 후보와 재적용 후보의 Native PNG는 각각 동일한
`(558, 908)–(947, 988)` 그림 영역에서만 6,912픽셀 다르다. 두 실행의 해당 쪽 render tree와
독립 Print 래스터 해시는 각각 일치한다. 원본 그림의 저장 crop은 `(0, 0, 66180, 13980)`,
전체 기준 `imgDim`은 `(66180, 13980)`, 디코딩 BMP는 `882×186`이다. crop은 원본 단위에서
전체 범위와 정확히 같다.

`compute_image_crop_src`에서 `66180 / (66180 / 882)`가 부동소수점으로
`881.9999999999999`가 된다. 새 base #8410의 `image_source_is_full`은 변환 결과의 정확한
동일성으로 판정하므로 전체 그림을 잘린 그림으로 오인한다. 실제 SVG도 해당 그림에
`viewBox="0 0 881.9999999999999 186"`을 생성하며, 이전 base SVG는 중첩 crop viewport가
없다. 이전 후보와 새 후보의 미대응 픽셀은 각 1개지만 합집합은 각각 202315와 202439다.
이 현상만으로 #8040의 모든 TSV 하락을 설명하지는 않는다.

보정은 공통 `compute_image_crop_src`에서 유효한 crop 기준 크기와 원본 crop이 원점부터
전체 범위까지 정확히 같을 때, 변환 전의 전체 소유 관계에 따라 디코딩 이미지 크기를 직접
반환한다. 다른 crop은 기존 계산과 #8410의 정확 판정을 유지한다. 원본의 실제 한 픽셀
자르기와 2픽셀 타일의 반쪽 선택은 전체 그림이 아니므로 그대로 잘려야 한다. 이 조건은
문서명이나 출력 점수가 아니라 저장 좌표의 동등성을 이용한다.

검증은 보정 전 실패하는 전체 그림 및 실제 잘림 반례, #8410 패턴·저장 왕복 6건,
#8040 동일 입력의 독립 Print 대조와 Native/fresh WASM 기준 TSV를 사용한다. 보정 전
출력과 로그는 `output/pr-review/planet6897-ordered-20261031/replay-bc9-new-sweep/`에 보존한다.

수정 전후의 임시 계약 진단은 전체 그림의 정확한 원점·끝을 수정 전 실패/수정 후
통과로 재현했다. 실제 1원본 단위 잘림과 #8410의 2픽셀 이미지 반쪽 선택은 양쪽에서
통과했다. 이는 공통 함수의 진단이며 정식 회귀나 Native/fresh WASM 증거를 대체하지 않는다.
수정 후 `release-test` Native 진단 SVG는 해당 2쪽의 불필요한 중첩 crop viewport를
제거했다. 같은 문서의 4쪽 Native 재측정은 1·3·4쪽이 수정 전과 같고, 2쪽 raw는
`99.999505711504%`다. #8397 기준 `99.999506311804%`보다 약 `0.000000600300%p`
낮다. 나머지 세 쪽도 기존 기준보다 낮으므로 이 보정만으로 무하락을 달성하지 못했다.
진단 로그와 전쪽 TSV는 `output/pr-review/planet6897-ordered-20261031/crop-fix-8040-native/`에
있으며, 최종 release Native·fresh WASM 재측정은 아직 필요하다.

#8040 기호 대체를 재검토하며 현재 시스템의 `HY Sinmyeongjo`와 U+3260을 Fontconfig에
질의했다. 기본 일치 결과는 Noto Sans CJK 계열부터 시작하며, 한컴 Print에 임베드된
대응 윤곽이 대다수 일치한 Gulim과 동일한 선택을 보장하지 않았다. 따라서 시스템의 첫
대체 face나 특정 문서에 Gulim을 강제하는 방식은 공통 해결 근거가 되지 않는다. 선택 결과와
실제 글리프 공급은 추가 추적이 필요하다.

공통 보정 후 최소 글꼴 환경에서 `native-skia`를 켜고 #8410 이미지 패턴·문자 획의
기존 두 모듈을 선택해 실행한 결과 6건 모두 통과했다. 테스트 모음 전체가 아니라 선택한
6건의 결과다. 로그는 `output/pr-review/planet6897-ordered-20261031/crop-fix-8410-native-skia-tests.log`다.

#8040 수정 후 2쪽의 raw 미대응 1픽셀은 이미지가 아니라 본문의 `<1` 글자 중 숫자
`1` 부근에서 발생했다. 후보 PNG는 `(580, 621)`이 회색이고 같은 독립 Print 위치는
흰색이며 2px 이웃 판정에서도 대응 내용이 없다. Render tree에는 해당 위치를 포함하는
`<1 ` 텍스트 run이 있다. 같은 문서를 같은 시스템 글꼴로 다시 출력한 진단 도구의 PNG와
Visual Sweep PNG SHA-256은 모두 `3f241958ebcb452a099b85b55d8b2ad0d37036cc1e2f21ebe9b336433c7a55ea`다.
글꼴 trace에서 이 형태의 문자 쌍은 원문 HFT `한양신명조`, Native OTF 메트릭,
실제 선택 family `HY Sinmyeongjo`로 기록된다. family만으로 파일 단위 글꼴 프로그램이나
해당 1픽셀의 근본 원인을 확정하지 않는다. 해당 PNG·trace와 로그는
`output/pr-review/planet6897-ordered-20261031/crop-fix-8040-p002-trace.*`에 있다.
독립 PDF 2쪽의 해당 CID 글꼴은 문서에 보이는 이름과 달리 FontDescriptor의
`/FontName`이 `/INPILL+Gulim`이다. `/CIDToGIDMap /Identity`와 ToUnicode로 확인한
숫자 `1` 글리프 38의 윤곽은 로컬 Gulim의 `1`과 같고, 두 글꼴의 unitsPerEm은 모두
1024다. 같은 방법으로 비교 가능한 F4 매핑 43개도 로컬 Gulim과 모두 일치했다.
이는 해당 PDF에 사용된 실제 프로그램의 증거다. RHWP가 원문 HFT와 별개로 대체할 조건이나
해당 시스템에서 실제 선택한 파일의 해시를 입증하지는 않으므로 강제 치환하지 않았다.
시스템 글꼴을 그대로 노출한 분석에서 원문 한양신명조의 표시 치환만 Gulim으로 바꾸고
원본 메트릭을 유지해 같은 네 쪽을 재출력했다. 1·2쪽은 raw 미대응이 0이 됐지만
3쪽 `99.791269977850%`, 4쪽 `99.699431809542%`로 #8397의 해당 기준보다 각각
`0.021869064446%p`, `0.018562864253%p` 낮아졌다. 따라서 family 전체를 Gulim으로
바꾸는 것은 채택하지 않는다. 서로 다른 글리프·언어 슬롯과 원본 프로그램의 대응을 더
확인해야 한다. 이 출력은 보정된 제품의 정상 결과가 아니다.

동일한 보정 후 두 번째 입력 `exam-social-insert-delete.hwp`도 Native 진단 4쪽을
독립 Print와 비교했다. raw는 각 `99.959017156908%`, `99.999505606423%`,
`99.813013588419%`, `99.717936122913%`이며, #8397 정밀 기준보다 각각
`0.000075822890%p`, `0.000000590802%p`, `0.000125453877%p`, `0.000058550882%p`
낮다. 단위 테스트 성공이나 90% 초과를 무하락의 대체 근거로 쓰지 않는다.

공통 보정 후 #7905의 Native 진단 빌드는 6쪽 raw가 각각 `99.980577882690%`,
`98.864278150371%`, `99.803773584906%`, `100%`, `100%`, `100%`다. 각 쪽 모두
#8397 정밀 기준보다 높으며 최저는 2쪽이다. 2쪽 PNG SHA-256은 보정 전과 모두
`94b04df710b3995822817311db3683dfb75861177a029fa04f9aab3446612f72`로 같아 기존
review·overlay를 재사용해 직접 확인했다. 본문에 보이는 글꼴·미세 위치 차이는 별도
검토 대상으로 남는다. 이 결과는 Native 진단 프로필이며 fresh WASM이나 최종 release
산출물의 전쪽 검증을 대신하지 않는다.

#7905 2쪽 Native 진단 trace를 같은 입력과 자동 글꼴 환경으로 다시 렌더했으며 PNG
SHA-256이 Visual Sweep 출력과 동일함을 확인했다. 사용자 화면에서 차이가 보이는 조문
제목의 원문 글꼴은 HFT `한양견고딕`, Native trace의 선택 family는 `HY Gyeongothic`,
메트릭 경로는 `nativeOtf`다. 독립 Print의 해당 제목은 Type 3 글꼴이며 PDF 텍스트
API는 family 이름을 제공하지 않는다. 좌표 대조에서 `정부출연연구기관의 범위` 첫 글자의
Print bbox 시작은 약 `(114.56, 591.52)` CSSpx이고 Native render tree run 시작은
약 `(114.7, 591.8)` CSSpx이다. 두 bbox는 산출 계층이 달라 이 수치만으로 차이의
원인을 특정하거나 고정 px 회귀로 삼지 않는다. HFT 원본 윤곽과 실제 선택 프로그램의
대응을 추가 검증해야 한다. 진단 trace는
`output/pr-review/planet6897-ordered-20261031/crop-fix-7905-p002-trace.json`이다.

PDF 텍스트 연산자 확인 결과 #7905의 `정부출연연구기관의` 제목은 Type 3 `/T2`를
사용하며, 본문의 같은 단어는 `/T1`을 사용한다. 시스템 Skia의 `HY Gyeongothic`
후보 목록에서 보통체 한글 `정`을 공급하는 후보는 `HYGyeongothic-HGGGT`다.
해당 로컬 `HGGGT.otf`의 SHA-256은
`da42e4819214f960b3561d4acf51571ad0c887f258e3dc034f5c2fae5c842a68`이다. 현재 trace는
family까지 관찰하고 파일 신원을 직접 기록하지 않으므로 실제 사용 파일이라는 결론은
후보 조회와 코드 순서에 근거한 추론이다. PDF `/T2`의 `정`은 `/CharProcs /HFT224`이며
별도 원본 곡선 비교가 필요하다. 로컬 글꼴 파일은 저장소에 포함하지 않는다.
PDF 텍스트 연산자로 해당 제목의 `/T2` 사용을 확인한 뒤, `정`의 Type 3
`/HFT224` 경로와 로컬 `HGGGT.otf`의 같은 글리프를 각각 1,000단위 SVG 경로로 그려
동일한 `rsvg-convert` 경로로 래스터화했다. 알파 128 기준 내용 합집합 368,798픽셀 중
926픽셀이 다르다. 이는 큰 정규화 격자에서의 윤곽 진단이며 실제 쪽의 raw 점수나
최종 선택 글꼴 파일의 직접 관찰을 뜻하지 않는다. 두 윤곽이 바이트 단위로 같다는
가정은 사용할 수 없다.

## 전체 그림 회귀의 시각 근거

`5d1294811b`의 제품 코드로 fresh WASM을 빌드하고 #8040의 삽입·삭제 원본을 각각
한컴 2020 Print와 전 4쪽 대조했다. 두 원본 모두 Native 최저 `99.717936122913%`,
WASM 최저 `99.708907814252%`다. WASM의 여덟 쪽은 각각 #8389의 기존 정밀 TSV보다
높다. Native는 이전에 확인한 여덟 쪽의 정밀 하락이 여전히 남아 있으므로 제품 전체의
무하락 판정은 아니다.

두 원본의 2쪽을 각각 Native와 WASM review·overlay로 확인하고, 실제 crop viewport의
소유 관계를 검사하는 글꼴 비의존 회귀를 추가했다. 값과 원본·PDF 및 직접 확인 자료는
[`integration_20261031_full_image_crop`](../pr/assets/integration_20261031_full_image_crop/README.md)에
기록했다. 기존 #8410의 실제 반쪽 타일·저장 왕복 검사 여섯 건은 공통 보정 후 통과했다.

#8040이 저장한 문서 슬롯별 `font-environment.json`을 현재 제품에 적용하되 시스템 글꼴
전체를 노출해 삽입 원본을 재측정했다. 1~4쪽은 각각 `99.954648526077%`,
`99.435036482167%`, `99.712504159688%`, `99.077296240378%`다. 이 설정을 그대로
재사용하면 현재 자동 선택의 전쪽보다 낮으며 저장된 #8389 정밀 기준도 복원하지 못한다.
이 실험은 문서별 글꼴을 제품에 강제하는 근거가 아니며,
`output/pr-review/planet6897-ordered-20261031/crop-fix-8040-recorded-env-system-native/`에
명시적 환경의 결과를 보존한다.

sample16 HWPX 18쪽은 이전의 Poppy 명시 공급 진단 PNG와 현재 자동 선택 PNG를 같은
232 문턱으로 비교했다. 서로 다른 내용 마스크는 두 픽셀이며 `(684,1004)`와
`(685,1004)`에 있다. 두 픽셀은 render tree의 `UNIX AP` run 영역에 속한다. 설치된
두 HCI Poppy OTF의 `P` 글리프는 동일하지 않으며 각각 advance 604/1000과 603/1000이다.
이 관찰은 명시 공급의 두 픽셀 복구와 일치하지만, 현재 각 픽셀의 실제 선택 프로그램을
파일 해시까지 추적한 것은 아니므로 자동 선택 변경의 근거로 단정하지 않는다.
시스템 Skia의 같은 family·보통체 후보는 `HCIPoppy-HMEPO` 다음에
`HCIPoppy-IndependentASCII-Regular-CompleteGraphics-CompleteSymbols` 순서이며 두
프로그램 모두 영문 글리프를 제공한다. 글리프 선택 코드와 후보 순서는 첫 프로그램을
선택한다는 추론을 뒷받침하지만, 해당 페이지 paint의 파일 신원을 직접 기록한 결과는
아니다. 후보 조회 로그는 `poppy-system-faces.log`다.
독립 2024 Print의 18쪽에서 해당 `UNIX AP`는 Type 3 `/T3`를 사용하고 `P`는
`/HFT102`다. PDF 곡선 명령과 두 로컬 OTF의 RecordingPen 결과를 좌표 단위로
비교한 결과, 원본 보존 프로그램은 18개 윤곽 명령과 advance 603/1000이 모두
일치한다. 다른 설치 프로그램은 20개 명령과 advance 604/1000으로 일치하지 않는다.
이는 해당 Print 글리프의 원본 프로그램 근거이며 실제 자동 paint가 선택한 파일의
직접 관찰은 여전히 별도다.
동일한 현재 제품 실행 파일·환경·18쪽에서 시스템 글꼴은 모두 유지한 채 두 프로그램을
각각 명시 공급했다. 자동 선택과 `HMEPO.otf` 명시 공급의 페이지 PNG SHA-256은 모두
`9e69c6cccfaf1e34b51f3d452d1bfe3dc6a283949afedd6c6d80fe8498207373`,
미대응/합집합은 `15/206244`로 같다. Print와 `P` 윤곽이 같은 프로그램을 명시 공급하면
PNG SHA-256 `bd44db9662c0e2f8f486a16576162fcde8d9ffc83129866b774e3dfa1f2ef82f`,
`15/206246`이 되어 기존 정밀 기준이 복원된다. PNG 동일성은 그 프로그램이 실제 선택된
파일이라는 직접 추적은 아니며, 공통 우선순위 보정을 단독으로 정당화하지 않는다.
기존 독립 근거의 HFT ASCII 95개 폭을 PDF Type 3의 정수 변환
`floor(HFT폭 × 1000 / 512)`과 대조하면 원본 보존 OTF는 95개가 일치한다.
`HMEPO.otf`는 70개만 일치하고 나머지 25개는 각 1단위 크다. 폭의 일치는 윤곽이나
프로그램 전체의 동일성을 단독으로 증명하지 않으므로, 다른 family·굵기·backend와
명시 프로그램 선택 계약을 확인하기 전에 순위 규칙으로 일반화하지 않는다.
실제 Native 선택 trace용 `native-skia` 진단 빌드는 종료 코드 143으로 중단됐으며
완료로 기록하지 않는다. 원인은 로그만으로 확인되지 않았다. 같은 시각 중단된 전쪽
WASM 비교는 고정 실행 파일과 체크포인트로 재개했다.

## #7905 3쪽 그림 가장자리

공통 보정 이후 fresh WASM 3쪽은 `99.805284771022%`로 #8389 기준
`99.823427543040%`보다 `0.018142772018%p` 낮다. Native와 WASM 모두 한컴 Print에만
있는 미대응 156픽셀이 x=677의 같은 세로선이며, 출력 상단 그림의 오른쪽 경계다.
HWPX에 포함된 GIF는 581×161픽셀이고 양쪽 끝 열이 모두 어둡다. 그림의 `orgSz`는
5810×1610, `curSz`는 44156×12236, `imgClip`은 (0, 0, 44200, 12300), `imgDim`은
없다. 현재 공통 crop 폴백은 원본 가운데 578.5528455284554×161픽셀만 선택하므로
오른쪽 끝 열을 그리지 않는다. 색상 보정이나 글꼴 선택 문제가 아니다.
독립 PDF의 해당 `/Im9`도 581×161픽셀이다. PDF 이미지 변환 폭은 441.363pt이고
클립 폭은 441.243pt로 오른쪽이 0.12pt만 잘린다. 따라서 기준 출력이 수학적으로
완전한 전체 창이라고 단정하지 않고, 실제 오른쪽 끝선이 표시된다는 사실을 계약으로 쓴다.

단순히 `imgClip`이 `curSz`를 포함한다는 이유로 전체 그림을 선택하면 안 된다.
저장소 HWPX에서 같은 조건을 충족하면서 잘림 끝점이 `orgSz`보다 작은 사례가 8개
발견됐다. 현재 크기·원본 크기·변환 행렬과 crop 좌표계의 관계를 독립 Print 및
실제 부분 잘림 반례로 확인해야 한다. 기존 #3239의 비-96dpi 전체 그림과 #7015의
부분 crop 계약도 공통 경로에서 유지해야 한다. 관련 PR은 #3241이며, #8410의 반쪽
타일 검사도 영향 대조군이다. 3쪽 [review](../../output/pr-review/planet6897-ordered-20261031/crop-fix-7905-p003-wasm-review/issue7905/review/review_003.png)와
[overlay](../../output/pr-review/planet6897-ordered-20261031/crop-fix-7905-p003-wasm-review/issue7905/overlay/overlay_003.png)는 임시 진단 결과다.

한컴 2020 MCP(11.0.0.9136)로 이 HWPX를 HWP로 저장했을 때도 원본 크기·현재 크기·
7.6배 행렬·`imgClip` 값이 그대로 보존됐다. 별도 한컴 저장 결과이지 Print PDF를
재산출한 것은 아니다. 같은 엔진이 #7015 HWP를 HWPX로 저장한 결과는 `imgDim=0`,
`imgClip=(0,20745,88560,45453)`으로 실제 위쪽 잘림을 유지한다. #7688의 실제
아래쪽 잘림을 저장한 HWPX는 `imgDim=47940×5280`, `imgClip=(0,0,47940,4366)`이다.
명시 `imgDim`의 부분 선택과 비영 시작점의 부분 선택은 보정 후에도 유지해야 한다.
MCP 변환 입력과 상태·출력은 각각 `issue7015-hwpx-mcp-*`, `issue7688-hwpx-mcp-*`,
`issue7905-hwp-mcp-*` 이름으로 `output/pr-review/planet6897-ordered-20261031/`에 둔다.

별도 #6465 원본 `samples/issue6465/press_release_footer_logos.hwpx`의 13쪽 그림
`image3`은 JPEG 94×33픽셀, `orgSz=9480×3300`, `curSz=6860×2388`,
`imgClip=(0,0,7080,2460)`이며 `imgDim`이 없다. 기존 한컴 Print PDF
`pdf/pr6481-visual/pr6481-issue6465-press-release-footer-logos-2020.pdf` 13쪽의
`/Im3`는 94×33픽셀이고, PDF 이미지 변환과 clip 기록상 세로는 전체가 표시된다.
오른쪽은 뒤의 이미지와 인접한 clip 때문에 이미지 폭보다 0.12pt 좁다. 이는 이미지
전체가 무조건 표시된다는 증거가 아니라 세로 환산 대조 사례다. 이 문서의 RHWP Native
시각 실행은 별도의 글꼴 환경 검증 오류로 종료돼 그 결과를 시각 통과로 사용하지 않았다.

`curSz` 포함만으로 전체 그림을 판정하는 반례를 한컴에서 확인했다. #7688의 한컴
저장 HWPX에서 두 번째 실제 부분 잘림 그림의 `imgDim`을 0으로 바꾸고 표시 크기와
행렬을 절반으로 변경한 진단 입력을 만들었다. 한컴 2020(11.0.0.9136)에서 HWP로
저장한 뒤 Print method 0, 1-up으로 PDF를 인쇄했다. 저장된 그림의 `curSz`는
23820×2329이고 crop은 `(0,0,47940,4366)`으로 현재 크기를 포함한다. 그럼에도
Print PDF의 두 번째 이미지 객체는 원본 639×70 중 639×61픽셀이다. 원래 기준 PDF의
두 번째 객체는 639×57이다. 따라서 `imgDim` 부재·0 시작·현재 크기 포함만으로 실제
부분 잘림을 전체로 바꿀 수 없다. 합성 입력의 변경 내용과 한컴 저장·인쇄를 구분한다.
진단 HWPX SHA-256 `b276a29c42042c510c2efc401924bad76bc901cf84cb95b05277df62df5c4734`,
한컴 저장 HWP `17e991f8ac1511fb4dc1934982b14aa7149834e218d0e8e7440b2fbb3e5447da`,
Print PDF `ee6d3f2dd596d1336fb38c5d5c82f10834631bb57051c454e24c1fd58e0160b2`다.
입력·인쇄와 MCP 상태는 `output/pr-review/planet6897-ordered-20261031/issue7688-scaled-*`에 있다.
현재 RHWP와 이 합성 대조 PDF의 1쪽 raw는 Native 99.96650%, WASM 98.46927%다.
이는 가설을 반박하는 진단이며 원본 #7688 전체의 새 최종 검증으로 사용하지 않는다.
[Native review](../../output/pr-review/planet6897-ordered-20261031/issue7688-scaled-native-control/issue7688-crop-control/review/review_001.png)·
[overlay](../../output/pr-review/planet6897-ordered-20261031/issue7688-scaled-native-control/issue7688-crop-control/overlay/overlay_001.png),
[WASM review](../../output/pr-review/planet6897-ordered-20261031/issue7688-scaled-wasm-control/issue7688-crop-control/review/review_001.png)·
[overlay](../../output/pr-review/planet6897-ordered-20261031/issue7688-scaled-wasm-control/issue7688-crop-control/overlay/overlay_001.png)를 확인했다.

## sample16 35쪽 WASM 정밀 하락

HWP 2022 원본 35쪽의 fresh WASM은 `99.998158639357%`로 #8389 기준 `100%`보다
낮다. Native는 같은 기준과 현재 출력 모두 `100%`다. WASM의 미대응은 독립 Print에만
있는 3픽셀이며, 세 픽셀 모두 본문 글자가 아닌 임베디드 메타파일 그림 내부다. 전체
쪽수나 90% 통과로 하락을 면제하지 않는다. 실제 측정 PNG를 재사용한
[review](../../output/pr-review/planet6897-ordered-20261031/sample16-p035-wasm-review-existing/review/review_035.png)와
[overlay](../../output/pr-review/planet6897-ordered-20261031/sample16-p035-wasm-review-existing/overlay/overlay_035.png)로
해당 영역을 확인했고, 기존 WASM 기준과 메타파일 표시 경로를 대조해야 한다.

### 그림 원본 경계와 자르기 좌표의 추가 대조

#7905 그림은 `imgRect`가 `5810×1610`, 저장 확대 행렬이 `7.6`,
`imgClip`이 `(0, 0, 44200, 12300)`이다. #7688의 축소 반례는 `imgRect`가
`41400×4560`이고 `imgClip`이 `(0, 0, 47940, 4366)`이다. 두 값은 저장
변환과 자르기 좌표계가 함께 관여하므로 개체 크기 또는 `imgRect`와의 단순 포함 관계만으로
독립 `imgDim`을 복원할 수 있다고 판정하지 않는다. 저장소 `samples`와 `tests/fixtures`의
HWPX 954개에서 명시 `imgDim`이 있고 원점이 0인 자르기 중 `imgClip`의 양 끝이
`orgSz` 이상이면서 실제 `imgDim`보다 작은 경우를 검색했으나 일치 사례는 없었다.
이는 해당 임의의 경계 규칙이 안전하다는 증거가 아니다. 검색 명령과 결과는 각각
`output/pr-review/planet6897-ordered-20261031/find-imgrect-crop-counterexamples.py`와
`output/pr-review/planet6897-ordered-20261031/imgrect-crop-counterexamples.tsv`다.

35쪽에서 기존 #8188 CanvasKit 부분 증거 `ck-source-diagram-full.png`와 현재
fresh WASM Canvas2D의 같은 위치를 비교했다. `(421,624)`와 `(421,656)`에서 기존
CanvasKit 그림에는 내용이 있고 현재 Canvas2D의 2px 이웃에는 내용이 없다. 다만
기존 증거는 CanvasKit 부분 렌더이고 현재 전쪽 TSV는 Canvas2D이며 두 Print 래스터의
해당 픽셀 값도 다르므로, 이 비교만으로 동일 backend의 코드 회귀 원인을 확정하지 않는다.
과거 실행의 입력·PDF·프로그램과 각 출력 경로를 따로 대조해야 한다. 픽셀과 비교 명령은
`output/pr-review/planet6897-ordered-20261031/compare-sample16-p035-old-canvaskit-points.py`와
`output/pr-review/planet6897-ordered-20261031/sample16-p035-old-canvaskit-point-comparison.log`에 있다.
기존 CanvasKit 증거의 35쪽 Print 부분과 현재 Print에서 같은 정수 좌표로 자른 부분을
비교하면 RGB가 다른 픽셀은 `96,609/285,792`, 내용 문턱 232에서 불일치는
`11,009`픽셀이다. 기존 증거는 전체 높이 1,123px, 현재 전체 PNG는 1,122px이므로
단순히 같은 정수 좌표를 비교해 출력 회귀의 크기나 위치를 확정할 수 없다. 별도 raster
정책·크기 차이 확인용이며 TSV를 대체하지 않는다. 재현 스크립트·결과는
`output/pr-review/planet6897-ordered-20261031/compare-sample16-p035-print-raster.py`와
`output/pr-review/planet6897-ordered-20261031/sample16-p035-print-raster-comparison.log`다.

전쪽 대조가 진행되며 편람 HWP의 WASM 130·141·145쪽도 기존 정밀 TSV보다
낮은 것으로 확인됐다. 현재 출력과 Print의 원본 PNG를 재사용해 review·overlay를
산출했다. 130쪽은 2px 이웃 내용 미대응이 RHWP 16픽셀·Print 7픽셀이며 모두
삽입된 화면 이미지의 일정표 영역에 있다. 141쪽은 Print 전용 1픽셀 `(496,430)`,
145쪽은 RHWP 전용 12픽셀로 관측됐다. 이 수치는 새 렌더가 아닌 진행 중인 동일
전쪽 측정의 PNG 진단이며 이전 출력의 해당 픽셀과 원인을 증명하지 않는다.
[130쪽 review](../../output/pr-review/planet6897-ordered-20261031/handbook-wasm-drop-review/review/review_130.png)·
[overlay](../../output/pr-review/planet6897-ordered-20261031/handbook-wasm-drop-review/overlay/overlay_130.png),
[141쪽 review](../../output/pr-review/planet6897-ordered-20261031/handbook-wasm-drop-review/review/review_141.png)·
[overlay](../../output/pr-review/planet6897-ordered-20261031/handbook-wasm-drop-review/overlay/overlay_141.png),
[145쪽 review](../../output/pr-review/planet6897-ordered-20261031/handbook-wasm-drop-review/review/review_145.png)·
[overlay](../../output/pr-review/planet6897-ordered-20261031/handbook-wasm-drop-review/overlay/overlay_145.png)와
`output/pr-review/planet6897-ordered-20261031/handbook-wasm-drop-pixel-diagnostic.log`를 참조한다.

130쪽의 과거 PNG를 추가 확인했다. #8394의 정식 `review_130.png`에서 기존 WASM과
Print 패널을 실제 크기 `740×1006`으로 분리하고 현재 전쪽 측정 PNG와 비교한 결과,
각각 RGB 픽셀 전체가 동일하다. 양쪽 2px raw는 미대응 23/합집합 63,928이다.
반면 사용자가 비교 대상으로 지정한 후속 #8397 정밀 TSV는 `99.964042835926%`로,
12자리 표시값을 합리적인 페이지 픽셀 수 범위에서 분수로 복원하면 23/63,965에
대응한다. 따라서 #8394와의 이미지 동일성은 #8397 대비 무하락을 입증하지 않으며,
후속 측정의 실제 PNG·합집합과 글꼴 환경 차이를 확인해야 한다. 정식 기존 패널,
현재 PNG, 정수 통계와 패널 분리 offset 검증은
`output/pr-review/planet6897-ordered-20261031/compare-handbook-p130-historical.py`와
`output/pr-review/planet6897-ordered-20261031/compare-handbook-p130-historical.log`에 있다.

#8397의 `seo-rii-local-fonts.json`에 기록된 한컴바탕·KoPub 글꼴 7개는 현재
시스템의 같은 경로에서 SHA-256이 모두 일치한다. 현재 비교에는 시스템 글꼴 전체가
노출되므로 7개 파일의 바이트 일치만으로 과거와 같은 글꼴 선택 또는 공급 순서를
증명하지 않는다. 확인 로그는
`output/pr-review/planet6897-ordered-20261031/seo-8397-font-hash-check.log`다.
후속 원본 제품 코드 `11def71519`의 WASM을 별도 검증 경로에 복원해 같은 현재 글꼴
환경에서 비교하도록 준비한다. 현재 전쪽 비교는 고정 실행 파일과 별도 `pkg`를 사용한다.

#8040 삽입 원본 4쪽의 #8397 표시값을 현재 미대응 수와 함께 분수로 분석했다.
현재 미대응 수가 이전에도 같았다고 가정할 때 이전 합집합은 각각 215,122·202,557·
180,348·197,514픽셀로 표시된 12자리 값을 재현한다. 현재 합집합은 각각
214,724·202,311·180,227·197,473픽셀이다. 이는 분모 차이만으로도 해당 점수
하락이 생길 수 있음을 보여 주는 조건부 계산이다. 이전의 정수 집계나 픽셀 위치를
확보하지 못했으므로 실제 출력의 동일성이나 원인을 입증하지 않는다. 계산과 한계는
`output/pr-review/planet6897-ordered-20261031/issue8040-score-integer-analysis.py`와
`output/pr-review/planet6897-ordered-20261031/issue8040-score-integer-analysis.log`에 있다.

이미지 자체의 해상도 메타데이터를 확인했다. #3239의 원본 TIFF 네 장은 모두
`1654×2340`, 실제 DPI 태그 `200×200`이다. #7905의 대상 GIF는 `581×161`이며
DPI 메타데이터가 없다. 그러나 실제 자르기 대조군 #7015의 `1181×945` JPEG에는
`300×300 DPI`가 들어 있는데 저장된 오른쪽 자르기 끝 `88,560`은 300 DPI의
24 HWPUNIT/px이 아니라 약 75 HWPUNIT/px과 대응한다. #7688의 JPEG에는 72 DPI가
기록돼 있다. 따라서 이미지의 DPI만 일괄 적용하는 규칙도 근거가 부족하다. 저장 형식과
한컴의 실제 삽입·저장 결과를 추가 대조해야 한다. 입력별 이미지 크기·DPI 로그는
`output/pr-review/planet6897-ordered-20261031/crop-image-resolution-audit.log`다.

원본을 변경하지 않고 #7905의 별도 진단 HWPX에서 해당 그림에만 `imgDim=44200×12300`을
추가해 같은 WASM과 Print PDF로 3쪽을 측정했다. 원본 `99.805284771022%`에서 진단
입력은 raw `100.00%`가 됐다. 두 RHWP PNG에서 바뀐 9,755개 RGB 픽셀의 외접 범위는
`(94,124)~(676,286)`으로 해당 그림 내부다. 원본에 없는 전체 크기를 실험적으로 부여한
것이므로 원본 출력이 100%라는 증거나 제품 보정이 아니다. 자르기 기준이 차이를 만드는지
확인한 결과이며 실제 부분 잘림·DPI 반례를 보존하는 규칙이 여전히 필요하다.
[진단 review](../../output/pr-review/planet6897-ordered-20261031/issue7905-imgdim-diagnostic-wasm/issue7905-imgdim-diagnostic/review/review_003.png)·
[overlay](../../output/pr-review/planet6897-ordered-20261031/issue7905-imgdim-diagnostic-wasm/issue7905-imgdim-diagnostic/overlay/overlay_003.png),
[TSV](../../output/pr-review/planet6897-ordered-20261031/issue7905-imgdim-diagnostic-wasm/issue7905-imgdim-diagnostic/silhouette.tsv),
`output/pr-review/planet6897-ordered-20261031/issue7905-imgdim-diagnostic-delta.log`를 참조한다.

#7015의 한컴 Print PDF 3쪽에는 해당 JPEG가 `1181×329`로 포함돼 있다.
원본 잘림 높이 `45,453−20,745=24,708` HWPUNIT을 75로 나누면 약 329.44픽셀이어서,
JPEG에 기록된 300 DPI를 그대로 사용하는 가정보다 이 Print와 부합한다. 다만 이 사례
하나로 다른 형식이나 저장 방식에 같은 기준을 일반화하지 않는다.

#7688에서 한컴 저장·Print로 부분 잘림을 확인한 진단 입력에만 별도로 전체 기준을
`639×75`·`70×75`로 부여한 후 같은 Print와 비교했다. 기준이 없는 진단 입력 대비
Native raw는 `99.966501656307% → 99.966492926284%`, WASM은
`98.469273743017% → 98.468874566926%`다. 두 경로의 미대응 수는 각각 9·411로
같고 합집합이 각각 7픽셀 줄었다. 한쪽 점수도 개선되지 않았고 두 입력 모두 진단용이므로
75 HWPUNIT/픽셀을 새 공통 규칙으로 채택하지 않았다. 입력 생성·측정·정수 비교는
`issue7688-reference75-diagnostic-create.py`, `issue7688-reference75-diagnostic-native.log`,
`issue7688-reference75-diagnostic-wasm.log`, `issue7688-reference75-compare.log`에 있다.

#7905 그림의 실제 부분 잘림 확인을 위해 원본 HWPX의 해당 GIF에서만 자르기 끝을
`(22000,6000)`으로 바꾼 별도 진단 입력을 만들었다. 한컴 2020 MCP로 HWP 저장 후
해당 HWP를 Print했고, 다시 HWPX로 저장해 바뀐 자르기와 `imgDim` 부재가 보존됨을
확인했다. Print PDF 3쪽의 `/x59`는 원본 581×161 그림이며, PDF 이미지 변환과 clip은
각각 가로 874.000031pt 대 441pt, 세로 245.999996pt 대 122pt다. 저장 좌표를
픽셀에 역산하면 약 75.04·75.15 HWPUNIT/픽셀이다. PDF 좌표의 인쇄 반올림을 포함하므로
이 수치만으로 정확한 단위 정책을 단정하지 않는다. #7688에서도 PDF 이미지 자체의
61행뿐 아니라 clip을 함께 적용하면 약 57.73행이 보이며, 저장 자르기 높이의
75 HWPUNIT/픽셀 환산은 약 58.21행이다. #7015 Print의 독립 JPEG 높이는 329픽셀이고
75 환산은 약 329.44픽셀이다.

현재 후보가 한컴 저장 진단 HWP를 출력하면 같은 Print와 비교한 3쪽 raw는 Native
`89.24512%`, WASM `89.19755%`이며 RHWP에는 전체 그림이 보이지만 Print에는 부분만
보인다. 이는 실제 한컴 Print로 확인한 새로운 반례이며, 수동 수정 문서에 관한 진단이지
원본 문서의 전쪽 점수나 완료된 회귀 검사가 아니다. [review](../../output/pr-review/planet6897-ordered-20261031/issue7905-gif-partial-control-wasm/gif-partial/review/review_003.png)·
[overlay](../../output/pr-review/planet6897-ordered-20261031/issue7905-gif-partial-control-wasm/gif-partial/overlay/overlay_003.png),
`crop-coordinate-print-geometry.log`, 한컴 MCP 작업의 `issue7905-gif-partial-*` 상태·
출력 및 `issue7905-gif-partial-control-{native,wasm}.log`를 보존한다. 한컴이 다시 저장한
HWPX에서 전체 크기를 75 기준으로 명시한 진단과 명시하지 않은 진단을 비교 중이다.

같은 부분 잘림 진단 HWP를 한컴이 다시 저장한 HWPX에서도 원본과 같은 Print에 대한
WASM raw는 `89.19755%`다. 이 HWPX의 해당 그림에만 `imgDim=43575×12075`
(581×161픽셀의 75 HWPUNIT/픽셀)을 추가한 진단은 `100.00%`다. 반대로 #7905의
자르기 경계가 전체 이미지보다 큰 원본 사례에서 같은 75 기준을 단순 명시하면
`96.14%`가 됐다. 따라서 부분 자르기의 기준 산출과 이미지 범위를 초과하는 선택의
처리는 별개이며, 75 값 하나만 넣는 보정으로는 두 계약을 동시에 만족하지 않는다.
결과는 `issue7905-gif-partial-roundtrip-wasm.log`,
`issue7905-gif-partial-reference75-wasm.log`, `issue7905-reference75-diagnostic-wasm.log`에 있다.

독립적인 GIF 원본을 찾기 위해 저장소 HWPX를 조사했고, 별도 Print가 있는
`tests/fixtures/issue7661_backgrounds/verification/api-issue7235.hwpx`의 첫 쪽에서
`134×91` GIF·`imgDim=0`·전체에 가까운 crop을 확인했다. 해당 원본과 Print를 현재
WASM으로 확인하고 이미지 전체 기준을 명시한 진단본도 같은 PDF와 대조 중이다.

#8397 당시 소스의 표준 최적화 WASM 빌드는 컴파일 중 종료 코드 143으로 중단됐다.
로그만으로 원인은 확인하지 못했다. 원래 후보와 같은 글꼴 환경의 출력 차이를 먼저
관측하기 위해 같은 소스·공용 target·8개 빌드 작업으로 `--no-opt` 진단 패키지를
별도 경로에 빌드한다. 이 진단 패키지는 정식 fresh optimized WASM 검증을 대체하지
않으며, 정식 비교에는 표준 최적화와 실제 패키지 해시 확인이 별도로 필요하다.

독립 GIF 대조군 `api-issue7235.hwpx`의 1쪽을 기존 Print와 비교했다. 원본은
`99.706327541819%`이고 해당 그림에만 전체 기준을 명시한 진단 입력은
`99.706384519868%`다. 미대응 픽셀은 모두 454개이고 합집합만 154,594에서
154,624로 달라졌다. RHWP PNG의 RGB 차이 4,260개는 `(82,132)~(202,218)`의
그림 영역에 한정된다. 이는 별도 GIF에서도 기준 선택이 실제 그림 표시를 바꾼다는
근거이지만 미대응 수가 같으므로 새 규칙의 일반적 정확성을 증명하지 않는다.
비교 스크립트·정수 결과는 `issue7235-gif-original-vs-full.py`와
`issue7235-gif-original-vs-full.log`다.

같은 GIF에서 자르기 사각형 전체가 이미지 밖까지 나가도록
`(-5000,-2000,50000,16000)`을 준 별도 진단 HWP를 한컴 2020 MCP로 저장·Print했다.
다시 저장한 HWPX에도 음수 시작·초과 끝점과 `imgDim` 부재가 보존된다. Print PDF
3쪽에서 그림은 전체 영역으로 표시되고 `/x59`의 변환 폭·높이와 Print clip은 각각
약 `442×122pt`·`441×122pt`다. 따라서 이 진단에서는 범위 밖 여백을 그대로 확대해
나타내지 않는다. 이어 왼쪽만 음수이고 나머지는 부분 선택인
`(-5000,0,22000,6000)` 진단을 저장·Print한 결과, PDF의 그림 transform·clip은
왼쪽 시작이 0인 같은 부분 선택 진단과 동일했다. 저장본의 속성 보존 여부와 다른
경계·다른 원본을 더 확인한 뒤 공통 범위 계산에 반영해야 한다. 입력·저장·Print·
페이지별 PDF 연산자는 `output/pr-review/planet6897-ordered-20261031/issue7905-gif-overflow-*`,
`issue7905-gif-mixed-*`에 보존한다.

구현 전 추적: 원본 `Picture.crop/imgDim`은 layout에서 `ImageNode.crop/original_size_hu`로
전달되고 `RenderNode::new`에서 실제 그림 노드가 만들어진다. 이후 `PaintOp::image`,
SVG·Native Skia·Canvas는 그 노드의 같은 crop과 기준 크기를 소비해 source rectangle을
계산한다. GIF가 명시 기준 크기를 보존한 경우는 그 값을 그대로 사용하고, 기준이 없는
GIF만 픽셀 크기와 한컴 Print에서 확인한 96 DPI 단위를 이용해 이미지의 유효 영역과
교차시킨다. 저장된 모델의 crop은 변경하지 않는다. 직접 생성된 PaintOp도 같은 공통
정규화를 소비하게 하며, 무효·영역 밖 선택을 전체 그림으로 바꾸지 않는다. 부분 잘림,
전체를 덮는 범위, 음수 시작·부분 끝, 명시 `imgDim`, 비-GIF TIFF/JPEG와 #8410의
작은 패턴 선택을 대조한다. 진단 입력의 Print는 합성 속성에 대한 실제 한컴 실행이며,
독립 원본 자체로 간주하지 않는다.

음수 시작의 부분 선택을 한컴 저장본에서 HWPX로 다시 읽어
`imgClip=(-5000,0,22000,6000)`, `imgDim` 부재가 그대로임을 확인했다.
그 Print PDF 3쪽과 시작점이 0인 부분 선택 Print PDF 3쪽의 96 DPI PNG SHA-256은
둘 다 `3c0e209cf8cd0092e6cbdc5159bd862bf44e1ef376016304335132f9bdbc8cdd`다.
이는 이 두 진단에서 음수 시작 영역이 보이는 공백으로 늘어나지 않는 독립 출력 증거다.

제품 후보에는 `image_geometry::normalize_gif_crop` 공통 결과를 그림 노드 생성과 직접
PaintOp 생성에서 소비하도록 추가했다. 명시 기준은 보존하고, GIF 헤더와 유효 픽셀
크기가 있는 경우에만 75 HWPUNIT/픽셀 범위와 자르기를 교차한다. 기존 모델 원본을
수정하지 않으며 명시 기준·TIFF·JPEG는 그대로다. 현재 후보 빌드·실제 원본 6쪽 및
부분·범위 초과 진단과 정상 대조군의 Native/fresh WASM 재검증 전이므로 완료 판정이
아니다. 기존 표준 WASM 전쪽 비교는 수정 전 고정 실행 파일을 사용해 계속 진행한다.
독립 GIF 대조군의 같은 1쪽 Native raw는 원본 `99.948561680405%`, 전체 기준을
명시한 진단본 `99.948576412847%`다. 각각 미대응 79픽셀이며 합집합은
153,582·153,626픽셀이다. 이 자료도 현재 제품 보정 후 측정은 아니다.

#8397 당시 소스의 비최적화 WASM 패키지는 컴파일·패키징 메시지 뒤 세션이 종료 코드
143으로 끝났다. 바이너리 유효성 검사와 실제 3쪽 실행은 성공했으나 정식 최적화 빌드
완료 증거로 사용하지 않는다. 같은 현재 시스템 글꼴 조건에서 #7905 3쪽은 과거 진단
`99.805340653856%`(미대응 156/합집합 80,140), 현재 `99.805284771022%`
(156/80,117)이다. 두 RHWP PNG의 RGB 차이 201픽셀은 `(67,1045)~(84,1059)`의
하단 정부 로고에만 있다. 독립 Print와 과거·현재 확대 판독에서 현재 로고만 축소 시
계단형이고 과거 출력은 Print처럼 부드럽다. 156개 미대응의 큰 그림 오른쪽 경계는
과거와 현재 모두 빠진다. 기록된 #8397 정밀 TSV와 진단의 글꼴·빌드가 같다고 추정하지 않는다.

코드 추적에서 #8410이 1픽셀 미만의 source 영역 차이도 실제 자르기로 보존한 이후,
Canvas2D의 전체 그림 경로는 축소 시 `ReductionQuality::for_full_image`를 적용하지만
실제 crop의 9인자 `drawImage` 경로는 같은 축소 품질을 적용하지 않는다. 따라서 자르기
영역 자체를 전체로 바꾸지 않고 해당 영역의 크기로 축소 품질을 결정하는 공통 처리의
필요성을 후보로 조사한다. 이는 아직 실제 보정·검증 전의 원인 가설이다. PNG 차이 및
확대는 `issue7905-old-wasm-current-delta.log`, `issue7905-old-current-print-footer-focus.png`다.

sample16 HWP 2022 35쪽을 #8397 소스 `11def71519`의 비최적화 진단 WASM과 현재
최적화 WASM으로 같은 현재 글꼴·입력·Print 조건에서 실행했다. 두 RHWP PNG SHA-256은
모두 `9e862f1d33dc8bca6360b17bd3e70613dbf322928356df603875ce3fef611902`이며,
raw 정수 집계도 모두 미대응 3/합집합 162,923, `99.998158639357%`다. 이 조건에서는
기존 소스와 현재 출력이 바이트 단위로 같다는 직접 증거다. 다만 과거 패키지 빌드
세션은 143으로 종료됐고 비최적화였으며, 과거 정식 TSV의 100%와 측정 환경 차이가
해소된 것은 아니다. 원래 사용자 기준을 충족했다고 판정하지 않는다. 로그는
`baseline-8397-unopt-sample16-wasm.log`, 현재 결과는
`crop-fix-frozen-all/preserve-system-wasm/sample16-hwp2022/`에 있다.

#8397 소스 `11def71519`의 비최적화 진단 WASM과 현재 최적화 WASM을 현재 설치 글꼴
전체로 같은 편람 HWP·Print에 적용해 43·130·141·145쪽을 비교했다. 각 쪽의 RHWP
PNG SHA-256이 이전 코드와 현재 코드에서 각각 완전히 같다. raw 미대응/합집합도 차례로
`4/100375`, `23/63928`, `1/112259`, `12/150274`로 같다. 현재 글꼴·도구 환경에서
이 네 쪽은 코드 출력 보존이 직접 확인되나, 당시 공개 TSV와 합집합 등이 다른 원인과
사용자가 요구한 공개 정밀 수치 조건은 남는다. 이전 빌드 세션은 143으로 종료됐고
패키지는 비최적화이므로 이 비교는 정식 최종 WASM 검증을 대체하지 않는다. 로그와
전쪽 측정 PNG는 `baseline-8397-unopt-handbook-wasm.log` 및
`baseline-8397-unopt-handbook-wasm/handbook-old/`에 있다.

GIF 공통 보정을 포함한 Native release-test 빌드는 종료 코드 0이다. #7905 원본 전체
6쪽의 raw는 `99.98/98.86/100/100/100/100%`이며 정밀 수치도 #8397에 기록된
#8389 대조 6쪽 모두보다 높다. 한컴 저장·Print한 부분 선택과 음수 시작 부분 선택도
각 3쪽 raw 100%다. 별도 GIF #7235, JPEG #7015, 200 DPI TIFF #3239의 선택 쪽도
90% 이상이다. #7015 3쪽과 #3239 1쪽은 수정 전후 Native PNG SHA-256이 각각 같아
비-GIF 대조군의 출력 변경이 없음을 확인했다. 실행 파일 SHA-256과 각 명령·종료 결과는
`rhwp-gif-crop-candidate.sha256`, `gif-crop-candidate-native-status.log`,
`gif-crop-control-old-vs-new-native.log` 및 각 `gif-crop-candidate-native-*.log`다.
[원본 3쪽 review](../../output/pr-review/planet6897-ordered-20261031/gif-crop-candidate-native/issue7905/issue7905/review/review_003.png)·
[overlay](../../output/pr-review/planet6897-ordered-20261031/gif-crop-candidate-native/issue7905/issue7905/overlay/overlay_003.png)를 확인했다.

Canvas2D에서는 잘린 source 자체를 유지한 채 축소 비율에 따라 전체 그림과 같은
`ReductionQuality::for_source_window`를 선택하도록 공통 축소 품질 helper를 사용했다.
이 변경은 `wasm32` 전용 모듈이며 기존 Native 빌드 결과를 변경하지 않는다. GIF 보정과
함께 소스 해시를 고정해 표준 `wasm-pack --release` 최적화 빌드를 별도 패키지에 시작했고,
아직 WASM 실행·#8410 작은 타일과 전쪽 결과는 확인하지 않았다. 빌드 로그는
`gif-crop-candidate-wasm-build.log`다.

첫 최적화 WASM 빌드는 종료 코드 1이었다. 새 Canvas 품질 guard가 `self.ctx`를 빌린 채
이후 `&mut self` 경로를 호출해 Rust `E0502`가 발생했고, guard가 소유할 Canvas 핸들
복제본을 별도로 만들어 대여 수명을 분리했다. 또 과거 작업 디렉터리의 같은 이름 PDF
의존성이 공용 target에 재사용돼 현재 소스에 존재하는 타입·오류 변형을 찾지 못했다.
공유 target을 삭제하지 않고 현재 `vendor/rhwp-pdf-import-backend/src/lib.rs`의 시각을
갱신해 Cargo가 현재 경로의 의존성을 재컴파일하도록 했다. vendor 바이트는 변경하지
않았다. 표준 최적화 WASM 재빌드가 실제 `/home/tsjang/rhwp/vendor/...`를 컴파일하는
것을 확인했으며 결과는 `gif-crop-candidate-wasm-retry.log`에서 확인한다. 최초 실패는
`gif-crop-candidate-wasm-build.log`에 보존한다.

브랜치 기준 `tests/cases/` 변경 파일 33개를 첫 줄 검사 유형·주석 갱신 시각으로
다시 점검했다. 기존 점검표는 이전 변경을 반영하지 않아 현재 상태로 갱신했다.
실제 assertion이 PDF 가져오기 IR·저장 축 관계를 검사하고 페이지 래스터 점수를
판정하지 않는 다섯 파일(`issue_8303`, `issue_8355`, `issue_8362` 두 파일,
`issue_8381`)은 유형·실제 갱신 시각·직전 파일 커밋·입력과 검사 범위를 첫머리에
추가했다. 과거 시각 수치를 새로 추정하지 않았으며 assertion은 바꾸지 않았다.
이 다섯 파일의 rustfmt 검사와 `git diff --check`가 통과했다. 주석만 바꾼 이유로
전체 회귀는 다시 실행하지 않았다. 나머지 13파일은 실제 검사와 독립 시각 근거를
확인해 같은 방식으로 보완한다. 현재 목록은
`output/pr-review/planet6897-ordered-20261031/current-regression-comment-audit.tsv`다.

#8040의 `㉠`에 대한 현재 Native 글꼴 결정 추적을 추가 확인했다. 원본의 기호 슬롯은
`한양신명조`이며, 해석된 요청 이름은 `HY Sinmyeongjo`다. 실제 glyph 공급자는 시스템의
`Haansoft Batang`으로 기록됐다. 별도로 추출한 독립 Print의 해당 원문자 글꼴은 Gulim과
대응한다. 이는 현재 대체 글꼴 경로를 조사할 근거이지만, 이 관측만으로 네 쪽의 raw 하락
전체 원인이나 Gulim으로 일괄 변경해야 한다고 단정하지 않는다. 앞서 본문 전체에 Gulim을
강제한 실험은 일부 쪽을 악화시켰으므로 그대로 적용하지 않는다. 문자별 추적은
`output/pr-review/planet6897-ordered-20261031/issue8040-native-font-trace-final.json`,
Print 글꼴 대조는 `issue8040-print-symbol-font-matches.json`에 있다.

GIF·Canvas 보정 전 고정 WASM 산출물의 전체 대조가 종료됐다. 공개 기준 TSV 2,153행 중
2,145행은 유지·개선, 8행은 하락이며 누락은 없다. #7905 3쪽과 편람·sample16 쪽이
포함된다. 현재 제품 코드에 추가된 GIF·Canvas 보정의 결과는 아니며, 이 수치로 최종
통과를 선언하지 않는다. `crop-fix-frozen-comparison-wasm-final.log`와
`crop-fix-frozen-all/comparison-replay-bc9-wasm-original.tsv`에 전 행을 보존한다.

#8040 추가 진단으로 원본 얼굴은 유지하고 `HY Sinmyeongjo`의 glyph 폴백 후보에만
Gulim을 지정해 Native 전 4쪽을 재출력했다. raw는 순서대로
`99.958965941732 / 99.999505865378 / 99.813608778139 / 99.717904695342%`다.
3쪽은 이전 후보보다 나아졌지만 #8397 기준과 비교하면 1·2·4쪽이 여전히 하락한다.
이 글꼴 후보 변경만으로 사용자 기준을 충족하지 못하므로 제품 기본값에 적용하지 않는다.
실행 로그는 `issue8040-fallback-only-native.log`, 설정은
`issue8040-fallback-only-diagnostic.json`이다.

개별 원 PR 리뷰 23건의 오래된 전쪽 대조 진행 문구를 실제 완료된 고정 후보 대조 결과
(각 2,153행 중 Native 12행·WASM 8행 하락)로 갱신했다. 이 결과는 뒤이어 적용한
GIF·Canvas 보정의 재검증 결과가 아니며, 각 리뷰의 보류 판정은 유지한다.

표준 최적화 WASM 빌드 종료를 기다리는 별도 검증 세션을 준비했다. 빌드 종료 코드 0,
빌드 전후 및 검증 직전 소스 해시 일치, WASM 바이너리 컴파일 확인을 모두 통과한 뒤에만
#7905 6쪽과 부분 GIF·음수 시작 GIF·JPEG·TIFF·다른 GIF 대조군을 실행한다. 시스템
글꼴 자동 공급을 사용하며 실행 경과는 `gif-crop-candidate-wasm-status.log`에 기록한다.
이 준비는 실행 성공이나 정밀 기준 통과를 뜻하지 않는다.

최적화 WASM 빌드는 종료 코드 0이며 빌드 전후 소스 해시가 일치했다. 패키지의
`rhwp_bg.wasm` SHA-256은 `e328e845af511449b15093e01282a3413a4351c3a6ae6b67eeea33af3a713706`다.
바이너리 컴파일 확인과 검증 직전 소스 해시 확인을 거쳐 #7905 6쪽 WASM 대조가 실제로
시작됐다. 로그는 `gif-crop-candidate-wasm-retry.log`와
`gif-crop-candidate-wasm-issue7905.log`다. 전쪽 결과와 기존 TSV 대비 정밀 판정은 아직 없다.

위 #7905 전 6쪽 WASM 대조는 종료 코드 0이며 각 raw는
`99.980697560469 / 98.866573169979 / 100 / 100 / 100 / 100%`다. 기존 #8389 대조에
기록된 #8397 후보 값과 정밀 대조한 6쪽 모두 유지·개선이고 전쪽 98% 이상이다.
정수 미대응·합집합으로 재계산한 결과는 `gif-crop-issue7905-wasm-compare.log`다.
실제 산출 PNG로 만든 3쪽 review·overlay는
`gif-crop-candidate-wasm/issue7905/issue7905/review/review_003.png`와
`gif-crop-candidate-wasm/issue7905/issue7905/overlay/overlay_003.png`다. 부분 자르기와
다른 형식 대조군 및 빈 선택의 WASM 결과는 별도로 확인해야 한다.

한컴에서 저장·Print한 부분 GIF 자르기 진단 3쪽은 Native와 새 최적화 WASM이 각각
raw 100%다. 자르기 전 전체 그림으로 확장하지 않고 원본의 일부만 보존하는지 직접
판독했다. WASM의 같은 출력으로 만든 자료는
`gif-crop-candidate-wasm/gif-partial/gif-partial/review/review_003.png`와
`gif-crop-candidate-wasm/gif-partial/gif-partial/overlay/overlay_003.png`다.

새 최적화 WASM의 나머지 선택 대조도 종료 코드 0이다. 음수 시작 GIF 3쪽은 raw
100%, JPEG #7015 3쪽은 99.96%, 200 DPI TIFF #3239 1쪽은 100%, 별도 GIF #7235
1쪽은 99.71%다. 이는 각 선택 쪽의 한컴 Print 대조이며 이 수치만으로 해당 문서의
전쪽 검증이나 기존 모든 TSV 유지가 입증되는 것은 아니다. 각 결과는
`gif-crop-candidate-wasm/<대상>/<대상>/silhouette.tsv`와 같은 이름의 실행 로그에 있다.

#7015 3쪽의 수정 전 고정 WASM과 새 최적화 WASM을 같은 시스템 글꼴 및 Print로
별도 실행했다. RGB 변경 픽셀의 외접 범위 `(221,865)-(570,949)`는 렌더 트리의
그림 틀 `(200.8,852.6,392.1,107.5)` 안이다. raw 미대응은 양쪽 15개이며 합집합은
41,705→41,812로, 정밀 raw는 `99.964033089558%`→`99.964125131541%`다. 같은 쪽의
출력 유지·개선을 확인했지만 다른 문서의 무회귀 증거로 일반화하지 않는다. 해시와 좌표는
`gif-wasm-issue7015-before-after.log`에 있다. #3239 동일 조건 대조는 진행 중이다.

#3239 1쪽도 수정 전 고정 WASM과 같은 조건으로 대조했다. 새 WASM의 PNG SHA-256은
이전 출력과 같고 RGB 변경 픽셀은 0개이며, Print 대비 정밀 raw는 양쪽 100%다.
해시와 정수 집계는 `gif-wasm-issue3239-before-after.log`에 있다.

#7905 원본 전쪽 Native·fresh WASM이 90% 이상이고 3쪽을 직접 판독한 뒤, 해당 GIF가
불필요한 중첩 자르기 창에 들어가지 않는 글꼴 독립 관계 검사를
`tests/cases/integration_gif_full_source_ownership.rs`에 추가했다. 수정 전 고정
실행 파일의 SVG에서는 조건이 실패하고 현재 후보 SVG에서는 성립함을 직접 확인했다.
정식 증거는 `mydocs/pr/assets/integration_20261031_gif_crop/`에 보존한다. 최소 글꼴
환경의 새 검사 실행은 `gif-source-regression.log`에서 진행 중이며 종료 결과를 확인해야 한다.
수정 전·후 고정 실행 파일의 SVG를 같은 관계 조건으로 검사한 별도 진단은 각각 FAIL·PASS다.
이 진단은 Rust 회귀 실행을 대신하지 않으며 `gif-source-before-after-assertion.log`에 남겼다.
새 Rust 검사는 최소 글꼴 환경에서 1건 실행·1건 통과하고 종료 코드 0이었다. 실제 노출
글꼴 5개와 nextest 결과는 `gif-source-regression.log`에 있다. 다른 304건은 선택
필터로 제외됐으며 이 실행을 전체 회귀 통과로 기록하지 않는다.
새 그림 처리 변경 후 #8410의 기존 Native Skia 관련 6건도 다시 확인하도록 현재 GIF
집중 회귀가 끝난 뒤 실행하는 대기 세션을 만들었다. 새 manifest가 배정한 suite 018·026을
사용하며, Cargo 검증은 같은 `target/pr-review`에서 동시에 실행하지 않는다. 실제 실행·종료
결과는 `gif-final-8410-native-skia-tests.log`를 확인한다.
실행 결과는 최소 글꼴 환경에서 6건 실행·6건 통과, 종료 코드 0이다. 필터로 제외된
589건은 이 실행의 통과 건수에 포함하지 않는다. GIF 관련 수정 파일과 새 회귀의 rustfmt
검사도 통과했다(`gif-format-check.log`).

#8410의 공개 패턴 HWP/HWPX 저장본을 새 Native·WASM으로 독립 원본 PDF와 전쪽
대조한다. 현재 Native 2쪽 raw는 99.87%로 과거 기록 99.98%보다 낮다. 과거 공개 패턴
기록은 MuPDF, 이번 sweep은 PDFium을 사용하므로 이 두 수치만으로 코드 회귀를 확정하지
않는다. 같은 PDFium·시스템 글꼴 조건의 이전 고정 실행 파일 대조도 순차 실행하도록 했다.
새 HWP 3쪽의 실제 반쪽 타일 4개는 SVG에서 각 타일의 틀을 읽고 그 내부를 상대 위치로
검사했으며 Native·WASM 모두 각 빨강·파랑 원본 열을 보존했다. 실행 경과는
`gif-final-8410-visual-status.log`, 색상 검사는
`gif-final-8410-pattern-hwp-color-check.log`, 이전 후보는
`gif-before-8410-visual-status.log`다. 전체 경로의 대조가 끝나기 전까지 기능 보존으로
최종 판정하지 않는다.

패턴 HWP 2쪽의 동일한 현재 Native·WASM PNG를 두 독립 PDF 렌더와 각각 대조했다.
PDFium은 양쪽 raw `99.87421%`(미대응 9/합집합 7,155), MuPDF는 양쪽
`99.98481%`(1/6,584)다. 기존 #8410 문서의 MuPDF 99.98% 표시와 현재 MuPDF의
소수 둘째 자리 수치가 일치하며, 서로 다른 기준 렌더러가 이번 2쪽 수치 차이를 만든다는
직접 근거다. 이는 같은 조건의 이전·현재 출력 비교를 대체하지 않는다. 재현 결과는
`issue8403-pattern-p002-reference-renderers.log`에 있다.

같은 시스템 글꼴과 PDFium 조건에서 GIF 보정 전 고정 Native 및 현재 Native의 공개 패턴
HWP·HWPX를 각각 전 5쪽 실행했다. 두 입력 모두 5쪽 전부 RHWP PNG SHA-256이 같다.
즉 Native에서 해당 패턴 출력의 픽셀 변화는 없으며, 같은 조건의 WASM 전쪽 비교는
진행 중이다. Native 해시 비교는 `issue8403-pattern-native-same-env-compare.log`다.
HWPX 저장본 3쪽에서도 별도로 SVG의 반쪽 타일 4개를 찾아 상대 위치의 색상을 검사했다.
Native·WASM 모두 빨강/파랑 열을 보존했다(`gif-final-8410-pattern-hwpx-color-check.log`).
WASM 직접 판독 자료는
`gif-final-8410-visual/patterns-hwpx-wasm/patterns-hwpx-wasm/review/review_003.png`와
같은 경로의 `overlay/overlay_003.png`다.
HWP 저장본의 변경 전·후 WASM PNG도 같은 시스템 글꼴·PDFium 조건에서 전 5쪽 모두
SHA-256이 같다(`issue8403-pattern-hwp-wasm-same-env-compare.log`). HWPX WASM 대조는
별도로 계속한다.
HWPX 저장본의 변경 전·후 WASM도 종료 코드 0이며 전 5쪽 PNG SHA-256이 모두 같다.
따라서 이 공개 패턴의 HWP/HWPX × Native/WASM 네 경로에서 GIF 보정 전후 출력이
동일함을 확인했다. HWPX WASM 비교는
`issue8403-pattern-hwpx-wasm-same-env-compare.log`다. 이 결과는 #8410의 다른 문자 획·
비공개 원본 전체까지 자동으로 검증한 뜻은 아니다.

## HFT 굵기 전진과 OTF 준비 경계

#8040 제목의 한글은 원본 글꼴 유형 HFT, 크기 1000 HWPUNIT, 장평 95%, 자간 −4%,
굵게다. 한컴 Print의 Type3 `T5`는 `/Widths=1000`, 크기 83, `Tz=95`, 같은 run의
글자 사이 `TJ=-14.6`을 기록한다. PDF 변환 행렬을 적용한 인접 글자 전진은 약
12.80 CSSpx다. 10pt·장평·자간만 적용하면 약 12.13px이고, 기존 HFT 합성 굵기 전진
0.05em을 함께 적용하면 약 12.77px로 독립 출력과 같은 방향이다. 이 계산은 렌더링
결과의 통과 증거가 아니며 장치 양자화와 PDF 위치를 실제 실행으로 확인해야 한다.

`NativePngExporter::prepare_document` → `prepare_native_otf_fonts` →
`char_width_decision_with_hft_tracking`의 OTF 전진 선택 뒤 합성 굵기 분기를 추적했다. 실제 OTF 폭이
있고 원본 HFT의 별도 내장 메트릭 표가 없는 경우, 기존 코드는 HFT 원본 여부를
확인하지 않아 합성 굵기 전진을 빠뜨릴 수 있다. 준비 전 추측 폭 경로는 HFT 여부를
확인해 이 전진을 더한다. 원본 슬롯의 HFT 속성과 선택된 실제 프로그램의 합성 굵기
여부를 함께 소비하도록 공통 분기를 수정했으며, 실제 굵은 face 및 비HFT는 기존 경로를
유지한다. Native 빌드와 #8040 두 원본·#7905·#8161의 HFT/일반 글꼴 대조군을
순차 검증 중이다. 로그는 `hft-bold-native-build.log`와
`hft-bold-native-sweep-status.log`다. 수정 후 Native/fresh WASM 결과와 이전 기능 보존은
아직 확인되지 않았으므로 최종 판정은 하지 않는다. 앞서 시작한 전쪽 sweep은 수정 전
고정 바이너리를 사용하며 이 변경의 결과로 해석하지 않는다.
시스템 글꼴 없이 이 경계를 재현할 수 있도록 저장소의 합성 `RHWPHftOutline.otf`를
공급하고 HFT와 비HFT의 굵기 전진 관계를 비교하는 검사 초안을 `output/`에만 준비했다.
수정 후 해당 원본의 Native·WASM 시각 선행 조건을 확인하기 전에는 정식 회귀로
등록하거나 통과했다고 기록하지 않는다.

2026-11-01 KST에 GitHub API로 원 PR head를 재확인했다. #8161은
`bd786fc5fefb94126ac146411f603c9cbcf336a6`, #8346은
`923ae70036a6b6e5ce94bf9c45cd2f8326cf8551`로 개별 리뷰에 기록한 값과 같으며,
이번 확인 시각까지 두 원 PR head 변경에 따른 재체리픽 사유는 없다.

같은 날 원격 `upstream/devel`을 `git ls-remote`로 확인한 결과
`bc9dfdd057258de59afebeb5121c4bee7398e07f`이며 로컬 추적 ref와 현재 브랜치의
merge-base가 모두 같다. 제출 직전에는 원격 head를 다시 확인한다.

#8040 원문자를 둘러싼 한글 슬롯은 `신명 중명조`, 해당 기호 슬롯은 `한양신명조`임을
원본 글꼴 결정 추적과 HWPX 내보내기의 face 목록에서 확인했다. 기호 슬롯의 해당 레코드만
독립 Print에서 확인한 Gulim으로 바꾸고 나머지 슬롯은 보존한 Native 진단은 4쪽 순서로
`99.958956372490 / 99.999505977670 / 99.813762713743 / 99.726738625495%`다.
3·4쪽은 #8397 기준보다 높지만 1·2쪽은 낮으므로 이 실험을 제품 기본값으로 반영하지
않았다. 첫 실행은 전역 치환과 문서별 치환이 같은 원본 이름을 중복해 환경 유효성 검사에서
실패했고, 두 번째는 각 슬롯별 원본 레코드 주소로 분리해 정상 종료했다. 설정과 로그는
`issue8040-symbol-slot-only-diagnostic.json`, `issue8040-symbol-slot-only-native-v2.log`다.

1쪽의 대표 `㉠`과 인접 글자를 같은 영역에서 확대해 이전 base·현재 후보·기호 슬롯
대체 진단·Print를 직접 비교했다. 해당 영역의 2px raw 미대응은 세 출력 모두 0이지만
합집합은 각각 141·141·134픽셀이었다. 따라서 이 영역에서는 100%라는 보조값만으로
글리프 모양이 같다고 판단할 수 없다. 전쪽 1·2쪽의 미대응 개수도 이전 base와 후보가
같지만 합집합이 달라 정밀 백분율은 하락한다. 확대 자료는
`issue8040-p001-slot-diagnostic-detail.png`이며, 현재 사용자 기준의 하락을 면제하는
근거로 사용하지 않는다.

#8040 1~4쪽에서 이전 고정 Native와 현재 Native의 2px 미대응 마스크를 직접 대조했다.
양쪽 미대응은 각각 88·1·337·557픽셀이며 네 쪽 모두 미대응 위치 자체도 같다.
따라서 정밀 백분율 변화는 해당 마스크의 차이가 아닌 내용 합집합 크기 변화다. 1쪽의
미대응은 `(165,376)-(418,389)` 범위의 `뜨거워진 한반도, 과일 재배 지도가 바뀐다!`
본문에 있고, Print는 이 제목을 내장 Type3 `T5`로 출력한다. 이전 base의 `㉠`은 문맥상
한글 슬롯의 `Sinmyeong Jungmyeongjo` 또는 `Sinmyeong Junggothic`을 사용했으나 현재는
#8136의 기호 슬롯 `HY Sinmyeongjo`를 사용한다. 이 관찰은 #8136을 되돌리거나 기존
정밀 TSV 하락을 면제하는 근거가 아니다. 정수 마스크 대조는
`issue8040-before-current-unmatched-mask.log`에 있다.

같은 1쪽 제목의 독립 Print에서 `뜨거워진한반도`는 내장 Type3 `T5`이고 첫 글자의
96dpi 환산 bbox 시작은 x=164.80이다. 현재 SVG의 대응 글자 원점은 x=164.0667이며
원본 글꼴은 HFT `신명 중고딕`, Native 글꼴 추적의 표시 family는
`Sinmyeong Junggothic`이다. 독립 Print의 glyph bbox와 SVG 원점은 서로 다른 값이므로
이 차이만으로 배치 오차를 확정하거나 고정 보정값을 적용하지 않는다. 시스템에는 같은
family의 언어별 OTF가 여러 개 있고 한글 `뜨`는 `TEJGTHG.otf`의 cmap에서 확인된다.
실제 선택된 개별 프로그램과 Type3 윤곽·조판 기준점의 대응을 추가로 확인해야 한다.
독립 Print의 Type3 `T5` ToUnicode에서 `뜨`는 코드 `03`, `/CharProcs`의 `/HFT37`에
대응한다. 이 glyph의 외곽과 세 윤곽 제어점 범위는 시스템
`TEJGTHG.otf`의 `hft9B61`과 각각 `(54,33)-(946,102)`, `(150,312)-(467,690)`,
`(518,323)-(845,689)`로 같다. 두 glyph의 전체 경로와 실제 Native가 선택한 파일이
같다는 증명은 아니며, glyph 외곽 모양의 유력한 대응 근거로만 사용한다.
두 윤곽을 같은 1,000×1,000 격자에 각각 래스터화하면 채우기만 비교한 RGB 차이는
0픽셀이다. Print의 Type3 명령은 이 윤곽에 `20/1000 em` 획과 채우기를 함께 적용한다.
양쪽에 같은 획을 추가한 진단에서는 RGB 차이가 7픽셀이다. 이는 글자 단위의 대응
근거이며, Native에서 실제 선택된 글꼴 파일과 장치 해상도·획 적용 결과를 확인하기 전에는
전체 제목의 원인을 확정하지 않는다. 진단 이미지와 중간 SVG는
`issue8040-glyph-{pdf,otf}[-stroke]-diagnostic.*`에 있다.
같은 제목의 `뜨거워진한반도` 일곱 글자에 대해 PDF text origin과 현재 SVG 원점을
96dpi 공통 좌표로 비교하면 PDF−RHWP x는 약 +0.6266~+0.7337 CSSpx, y는 공통
−0.2236 CSSpx다. x 차이는 세 값에 가깝게 반복된다. 이 결과는 한 문단에 대한
관측이며 PDF 좌표 양자화와 표·문단 원점 중 원인 계층을 확정하지 않는다. 특정 좌표를
더하는 보정 없이 저장 원본·공통 배치 경로를 계속 대조한다. 결과는
`issue8040-title-origins.log`다.
제목의 쉼표 전후로도 따로 비교하면 앞 한글의 PDF−RHWP x 차이는 약 +0.63~+0.73px,
뒤 한글은 약 +0.04~+0.15px로 줄어든다(`issue8040-title-origins-all.log`). PDF의
쉼표는 내장 CID 글꼴 `F4`의 폭 333/1000em, 크기 83·장평 92.941177%로 기록돼 있고,
현재 추적은 원본 영문 슬롯 `한양신명조`의 Native OTF 폭 235/1000em을 사용한다.
시스템 `ENSMJ.otf`의 쉼표 hmtx도 235/1000em이다. 원본 문단은 가운데 정렬이므로
기호 폭 차이가 줄의 양쪽 원점 차이에 관여할 가능성이 있다. 다만 한컴 출력 환경의
글꼴 공급 차이와 저장 줄의 측정·배치 경로를 추가로 확인하기 전에는 특정 글꼴 치환이나
폭 변경을 공통 계약으로 확정하지 않는다. PDF 명령은 `issue8040-title-pdf-operators.log`다.
**정정:** 위 SVG 원점은 Native PNG 렌더러가 글꼴을 준비하기 전의 별도 SVG 경로다.
실제 Native PNG 실행에서 `prepare_document` 후 기록된 트리는 첫 제목 run x=169.1,
폭 91.2이며, 미준비 SVG의 x=164.1·폭 95.3과 다르다. 한컴 PDF의 대응 원점은
x=164.8이므로 실제 Native에 앞서 기재한 약 0.7px 차이를 적용하면 안 된다. 명시 글꼴
환경의 준비 후 트리도 동일한 x=169.1·폭 91.2다. 실제 PNG와 맞는 트리는
`gif-final-frozen-all/preserve-system-native/issue8040-insert/issue8040-insert/render_tree/`,
비교는 `issue8040-title-prepared-tree-comparison.log`다. 화면별 글꼴 준비 후 측정과
중앙 정렬의 생산·소비 경로가 이 차이를 만드는지 추적한다.
PDF `F4`에 포함된 서브셋의 쉼표·`㉠`·느낌표를 시스템 Gulim TTC 첫 face와 실제
윤곽 명령으로 대조했다. 글리프 ID는 서로 다르지만 세 문자 모두 분해된 윤곽 명령이
완전히 일치했고 쉼표의 프로그램 전진도 양쪽 342/1024em이다. 이는 한컴 Print에서
해당 문자를 Gulim으로 그렸다는 직접 근거다. 현재 시스템에 실제 한양신명조 OTF가 있는
상태에서 원본 슬롯을 언제 대체해야 하는지는 별개의 계약이므로 이 결과만으로 전역
치환하지 않는다. 비교 로그는 `issue8040-pdf-gulim-glyph-exact.log`다.
GIF·Canvas 보정 후 고정한 제품의 #8389·#8397 기준 문서 전쪽 Native/WASM 대조를
진행 중이다. 새 WASM의 #8040 삽입 원본 4쪽은 `pr8389-wasm-exact.tsv`에 수록된
기준의 네 행보다 모두 높다(`gif-final-8040-insert-wasm-baseline.log`). Native의
미세 하락과 다른 문서·WASM 경로의 완료 여부는 별도로 확인한다.
해당 원본의 `한양신명조` 영문·기호 슬롯에만 Gulim을 명시한 별도 Native 진단은
1~4쪽이 `99.958833105668 / 100 / 99.791293149864 / 99.699431809542%`였다.
2쪽은 개선됐지만 나머지 쪽은 기존 기준보다 낮고 3·4쪽은 이전 후보보다도 악화했다.
따라서 Print 글꼴 일괄 치환은 채택하지 않는다. 진단 설정·결과는
`issue8040-latin-symbol-slots-print-diagnostic.json`과
`issue8040-latin-symbol-slots-native/issue8040-latin-symbol/silhouette.tsv`다.

#8040 원본 1쪽의 현재 Native 후보를 독립 Print와 같은 배율로 다시 출력하고 전체
review·overlay를 생성했다. 로그 `issue8040-candidate-review.log`는 종료 코드 0이다.
이미지는 `issue8040-candidate-review/issue8040-current/review/review_001.png`와
`issue8040-candidate-review/issue8040-current/overlay/overlay_001.png`다. 이 이미지는
정밀 수치 하락이 남은 후보의 직접 판독 자료이며 기준 복구 완료 증거가 아니다.

GIF 자르기가 이미지 전체 범위 밖일 때 정규화 결과가 빈 선택이 될 수 있다. SVG의
`render_image_node_with_sampling`과 Native의 `image_conv`는 빈 선택을 명시적으로
건너뛰지만 Canvas의 `draw_image_with_fill_mode`에는 같은 명시 조건이 보이지 않는다.
이는 코드 검토에서 발견한 검증 대상이며 실제 한컴 Print의 빈 영역 계약과 Canvas 실행
결과는 아직 확인하지 않았다. 현재 WASM 빌드에 사용한 소스 해시를 유지하고 있으며,
독립 기준과 실행 결과를 확인한 뒤 필요한 공통 처리를 판단한다.

위 경계 확인을 위해 #7905 원본의 GIF 선택을 원본 영역 밖 `(50000, 0, 55000, 6000)`으로
수정한 진단 HWPX를 한컴 2020 MCP에서 HWP로 저장한 뒤 PrintToPDFEx 인쇄 PDF를 받았다.
HWP를 재파싱한 결과 해당 선택 좌표와 비어 있는 `imgDim`이 보존됐다. 3쪽 Native와
Print는 이미지 영역이 모두 비어 있고, raw 2px 실루엣은 100%다. 검토 이미지는
`issue7905-gif-empty-native/gif-empty/review/review_003.png`와
`issue7905-gif-empty-native/gif-empty/overlay/overlay_003.png`, MCP 저장·인쇄 기록은
`issue7905-gif-empty-selection-hwp-download.json`과
`issue7905-gif-empty-selection-pdf-download.json`이다. WASM 실행 결과는 아직 없다.
같은 HWP와 Print PDF의 3쪽 WASM 검증은 최적화 빌드와 선행 여섯 사례의 대조가 종료된
뒤 시작하도록 연결했다. 실행 로그는 `issue7905-gif-empty-wasm.log`이며 빌드 실패나
소스 해시 변경 시 자동으로 중단한다.

빈 선택 WASM 대조도 종료 코드 0이며 해당 3쪽 raw는 100%다. 독립 Print와 직접 비교해
선택된 이미지 영역이 비어 있는 것을 확인했다. Canvas에 명시적 빈 선택 분기가 없다는
코드 관찰만으로 결함이라고 결론 내리지 않으며, 확인한 입력에 대해 동작 보정은 추가하지
않는다. 증거는 `issue7905-gif-empty-wasm/gif-empty/review/review_003.png`와
`issue7905-gif-empty-wasm/gif-empty/overlay/overlay_003.png`다.

## HFT 합성 굵기 전진의 공급 경로 대조

#8040의 독립 Print에서 제목 글자의 실제 전진을 확인하고, 같은 원본을 Native OTF
메트릭 준비 전후에 비교했다. 준비된 경로는 HFT 원본이라도 내장 HFT 폭표가 없으면
합성 굵기의 추가 전진을 잃는다. OTF 공급과 원본 HFT 여부를 분리하는 공통 보정은
검증 중인 후보이며, 한 문서의 관찰만으로 모든 글꼴에 대한 계약이나 개선 완료를
확정하지 않는다. 수정 전후 전쪽 Native·fresh WASM과 정상 굵기 대조군이 필요하다.

집중 회귀에는 저장소의 라이선스 확인된 `RHWPHftOutline.otf`를 사용할 계획이다.
해당 시험 글꼴이 `가` 글리프를 포함하고 실제 굵은 face가 아닌 것을 확인했다.
시스템 OTF 설치에 의존하지 않으며, 실제 시각 선행 조건을 만족한 다음 등록한다.

후보 Native 빌드는 종료 코드 0이다. #8040 두 원본, #7905, HFT 대조군 #5169,
TrueType 대조군 `2026_oss_rst.hwp`의 전체 26쪽을 측정해 모두 90% 이상을 확인했다.
기존 정밀 TSV 대조에서는 20쪽이 유지·개선되고 #8040 두 원본의 2~4쪽 여섯 행이
하락했다. #8040 삽입 원본 1쪽은 `99.959092979798%`에서 `100%`로 개선됐다.
측정 로그는 `hft-bold-native-sweep-status.log`, 정밀 대조는
`hft-bold-native-comparison.log`다. 3쪽의 전쪽 비교와 겹침은
`hft-bold-issue8040-native-review/review/review_003.png`와
`hft-bold-issue8040-native-review/overlay/overlay_003.png`다. 여섯 행을 해결하고
fresh WASM을 검증하기 전에는 후보를 완료로 판정하지 않는다.

3쪽의 남은 미대응 337픽셀은 `(563, 968)–(942, 1067)` 안에 있다. 원 출력
트리에서 해당 영역은 바깥 표 셀에 속한 중첩 표이며, 이미지의 수평선은 예를 들어
RHWP y=1024와 Print y=1025~1026에 나타난다. 4쪽은 557픽셀이 여러 영역에
분산된다. 이 관찰은 중첩 표의 저장값·문단 원점·테두리 래스터 중 원인을 아직
구분하지 않으므로 특정 좌표를 임의로 이동하지 않는다.

원본 HWP 1구역 41번 문단의 바깥 1×1 표는 글자처럼 취급하며, 첫 셀의
세 번째 문단에 5×6 중첩 표가 있다. 호스트 줄의 저장 `vpos=10242 HU`,
중첩 셀 행 높이는 각 `1397 HU`다. 후보 트리의 중첩 표 원점은
`(560.2, 987.9)` CSSpx이고 독립 Print의 첫 수평선은 y=`989.016` CSSpx다.
같은 표의 다음 수평선은 y=`1026.268`, `1044.813`, `1063.520`, `1082.065`로,
수평선 차이를 다시 배분하기 전에 호스트 원점과 공통 세로 오프셋을 추적해야 한다.
원본 덤프와 PDF 벡터 대조는 `issue8040-p003-para41-section1-dump.log`와
`issue8040-p003-nested-table-source-origin.log`다. PDF 분석 시 CID 글꼴 경고는
그리기 선분 좌표를 확인하는 이 진단의 근거로 사용하지 않았다.

중첩 표의 HWP `CTRL_HEADER` 원시 바이트를 독립적으로 읽으면 세로 오프셋은 0이고,
표 위에 `(단위:%)` 캡션이 있다. 캡션 저장 줄 높이는 `950 HU`, 캡션 간격은
`566 HU`다. 공통 `caption_height_px_in_context`와 `top_caption_flow_extra`는
이 둘의 합을 표 본문 원점에 더한다. 반면 PDF에서 캡션 텍스트와 표 첫 선의 관계는
약 1.1px 더 크다. 높이 산식과 실제 캡션 배치의 근거·반례를 확인하기 전에는 줄간격을
추가하는 등의 수치 보정을 하지 않는다. 원시 레코드 기록은
`issue8040-p003-nested-table-record.log`다.

호스트 줄 높이 `8783 HU`는 표 본체 `6985` + 캡션 줄 `950` + 캡션 간격
`566` + 위·아래 바깥여백 각 `141`의 합과 정확히 같다. 따라서 저장 줄에는 이
점유가 포함되지만 이를 모두 표 본문 원점에 더해야 한다는 결론은 아니다. 캡션
문단 모양의 앞·뒤 간격은 모두 0이다. 여백과 원점의 소비 위치 및 한컴의 기준선
관계를 더 확인한다.

한컴 2020 MCP에서 같은 원본을 HWPX로 저장하고 중첩 표의 위 바깥여백만 0·141·
600 HU로 바꿔 각각 Print했다. 캡션 기준선은 순서대로 978.400·980.320·986.400,
표 첫 선은 987.097·989.016·995.091 CSSpx다. 저장본 141 HU의 두 좌표는 기존
원본 Print와 같다. 기존 RHWP SVG의 캡션 기준선은 세 입력 모두 978.427 CSSpx여서
여백이 배치에 반영되지 않는 반례를 확인했다. 작업 과정과 각 Print 결과는
`issue8040-caption-variants.log`, `issue8040-caption-*-print-*.json`,
`issue8040-caption-margin-print-comparison.log`다.

해당 저장 줄은 본체와 위·아래 여백의 합만 비교하는 기존 TAC 소유 판정에서
캡션의 점유 높이가 빠져 조건을 만족하지 못한다. 문단 인라인 경로의 줄 소유자와
직접 배치, 셀 내 대체 경로가 공통 판정으로 본체·상하 캡션·여백을 소비하도록
후보를 수정했다. 기존에 본체와 여백만으로 닫히는 저장 줄의 판정은 유지한다.
이 변경은 실제 출력과 대조군 검증을 완료하기 전까지 확정하지 않는다.

수동으로 여백을 바꾼 HWPX의 저장 줄이 원래 값으로 남을 수 있어 한컴 2020 MCP로
각 변형을 HWP로 다시 저장했다. 저장된 호스트 줄 높이는 여백 0에서 `8642 HU`,
여백 600에서 `9242 HU`이며 원본 `8783 HU`와 함께 각각 본체·캡션·상하 여백의
합과 정확히 같다. 따라서 이 세 파일은 실제 저장 줄을 대조하는 근거이고, 수동
수정 직후 HWPX는 오래된 메타데이터를 가진 별도 진단으로만 취급한다. 저장 기록은
`issue8040-caption-top-*-save-*.json`, 줄 덤프는
`issue8040-caption-top-*-saved-dump.log`다. 저장본 Print를 추가 측정 중이다.

두 재저장 HWP를 한컴 2020 `PrintMethod=0`으로 인쇄한 결과도 앞의 HWPX Print와
캡션 기준선·표 첫 선 좌표가 각각 일치했다. 비교는
`issue8040-caption-saved-print-comparison.log`, 인쇄 정보는
`issue8040-caption-top-*-saved-print-status.json`에 있다. 재저장 HWP와 해당 Print를
현재 Native 후보의 추가 대조군으로 연결했다.

캡션·여백 후보의 기존 대조군 `nested_table_text_caption.hwp` 12쪽,
`hwp3-table-caption.hwp` 1쪽, `156678235_mid_para_vpos_rewind.hwp` 7쪽을
같은 글꼴·Print 환경에서 이전 고정 실행 파일과 비교했다. Native 20쪽 모두 정밀
raw 하락이 없으며 최저값은 각각 97.77%, 98.83%, 97.81%다. 비교 기록은
`caption-tac-extra-controls-exact-compare.log`다. 이 결과는 이후 글자 기준선 후보의
대조나 fresh WASM 검증을 대신하지 않는다.

기존 #7049·#7150 표 기준선 계약의 추가 대조에서는 `issue2083` 4쪽 최저 99.90%,
`36384689` 1쪽 94.14%, `issue2470` 2쪽 최저 76.50%가 나왔다. `issue2470` 1쪽은
기준선 후보의 수정 전후 Native PNG SHA-256이 모두
`a7c8bf4f8fe34b8dc00ab4e143cb69221dc1a39aa891e78c9a1036acb95b21eb`으로 같지만,
독립 Print에 없는 회색 영역 등의 차이는 별도로 남는다. 이는 후보가 해당 페이지를
변경하지 않았다는 증거일 뿐 90% 미만이나 기존 고정 좌표의 타당성을 면제하지 않는다.
측정은 `inline-center-prior-baseline-controls/`, 1쪽 직접 판독 자료는
`inline-center-issue2470-native-review/review/review_001.png` 및
`inline-center-issue2470-native-review/overlay/overlay_001.png`다. #7049·#7150의 원
이슈와 #2470은 닫힌 상태였으며, 원 PR과 기준 SHA에서의 미달 여부를 더 확인한다.
위 76.50%는 `36382471_masked-hwpx-2020.pdf`를 사용한 결과다. 이후 기존 #7199
리뷰에서 같은 원본의 기준을 `36382471_masked-2022.pdf`로 명시한 것을 확인했으므로,
76.50%를 해당 회귀의 정식 기준값으로 확정하지 않고 기존 기준 PDF로 다시 측정한다.

캡션·여백 후보의 Native 전쪽 측정에서 #8040 두 원본의 3쪽은 각각 raw 100%가
되어 공개 정밀 기준도 넘었다. 1쪽도 100%이며 2·4쪽의 정밀 하락은 남는다.
3쪽 새 증거는 `caption-tac-issue8040-native-review/review/review_003.png`와
`caption-tac-issue8040-native-review/overlay/overlay_003.png`다. 나머지 대조군과
fresh WASM 결과는 아직 합산하지 않았다.

4쪽의 두 분수는 수식 개체가 아니라 두 행 중첩 표와 같은 저장 줄의 텍스트다.
한컴 Print의 텍스트 기준선은 각각 `824.96`, `987.68` CSSpx이고, RHWP SVG는
`820.76`, `983.45` CSSpx다. 표 내부 분자 글자의 기준선은 각각 약 `817.1`,
`979.8`로 두 출력이 맞는다. 두 호스트 저장 줄은 높이 `2364 HU`, 기준선
`1182 HU`다. 표와 텍스트가 같은 줄인 경우의 기준선 해석을 원본과 독립 출력으로
확인해야 하며, 특정 문서에 좌표를 더하지 않는다. 원본 덤프는
`issue8040-p004-para66-dump.log`다.

호스트 글자 크기만 `900→1200 HU`로 바꿔 한컴 2020에서 다시 HWP로 저장하고
Print했다. 두 저장 줄의 높이·중심 기준선은 `2364/1182 HU`로 그대로이고 표 첫 행의
Print 기준선도 `817.12px`로 유지된다. 반면 옆 글자 기준선은 `824.96→826.40px`로
내려간다. 기존 `0.85em` 논리 글자 기준선과 글자 상자 중심 `0.5em`의 차이를
저장된 줄 중심에 반영한 값과 각각 약 `0.04px` 안에서 맞는다. 인쇄 작업은 첫 시도에서
worker가 종료됐으나 같은 저장본의 재시도는 성공했다. 기록은
`issue8040-inline-fontsize-print-comparison.log`, `issue8040-inline-large-saved-dump.log`,
`issue8040-inline-large-print-*-status.json`이다.

저장 줄의 기준선이 정확히 줄 높이의 중심이고 같은 줄에 TAC 표와 실제 텍스트가
있는 경우, 글자의 논리 상자 중심을 그 줄 중심에 맞추는 후보를 추가했다. 기존
`0.85em` 문자 기준선에서 상자 중심 `0.5em`을 뺀 값을 사용하며, 표 자체는 원래
개체 기준선을 계속 소비한다. 도형과 섞인 줄 및 재계산된 저장 메트릭은 이 후보에서
제외했다. 저장본 두 크기의 실행 대조, 같은 줄 여러 크기·다른 기준선의 반례,
Native/fresh WASM 전쪽 비교 전에는 완성으로 판정하지 않는다.

저장 메트릭 재계산 여부만으로는 편집 후 줄 구성 변경이나 저장 줄 부재를 모두
구분하지 못한다. 따라서 줄 분할이 변경되지 않았고 유효한 저장 LineSeg의 높이와
기준선이 현재 구성 줄의 값과 각각 같은 경우에만 중심 기준선 처리를 적용하도록
제한했다. 이 변경의 Native 빌드와 재실행은 `inline-center-stored-guard-build.log`부터
확인한다. 기존 후보의 결과를 이 변경의 검증으로 재사용하지 않는다.

가드가 포함된 Native 빌드는 종료 코드 0이다. #8040 두 원본의 전쪽 재출력도 각각
종료 코드 0이고, 8쪽 모두 가드 추가 전 후보와 미대응·합집합 기반 정밀 raw가 같다.
`stored-guard-issue8040-exact-compare.log`에서 3·4쪽의 개선 유지와 2쪽의 기존 TSV
대비 미세 하락을 확인했다. 별도 재저장 대조본은 아직 실행 중이다.

추가 코드 추적에서 `composer/text_metric_origin.rs`의 줄 출처 판정은 문단 dirty
상태 외에 개별 LineSeg의 합성 태그, 개체 투영, 재계산된 공백 메트릭도 구분한다.
현재 추가한 조건이 이 공통 판정보다 약한지 확인해야 한다. 빌드 중인 소스를 바꾸지
않고 결과를 확보한 다음 공통 판정을 재사용하는 방안을 검증한다.

이후 `regenerated_row`를 composer 외부에서도 내부적으로 재사용하도록 노출하고,
중심 기준선 조건이 동일한 출처 판정을 소비하도록 바꿨다. 구성 줄의 높이·기준선과
실제 저장 줄의 일치 검사는 유지한다. Native 빌드와 #8040 두 원본·확대 저장본·
#8136 재현본의 일괄 재측정은 `origin-guard-native.log`와 `origin-guard-build.log`에서
진행하며 새 결과가 나오기 전에는 앞선 후보로 대신하지 않는다.

공통 출처 판정을 반영한 Native 빌드는 종료 코드 0이다. #8040 두 원본 전 8쪽의
정밀 raw는 직전 가드 후보와 모두 같다(`origin-guard-issue8040-exact-compare.log`).
공개 기준과 비교하면 두 원본 2쪽의 미세 하락은 여전히 남아 있다. 나머지 대조본과
최소 글꼴 집중 회귀는 계속 실행 중이다.

최신 후보의 관련 6개 모듈을 최소 글꼴 환경에서 집중 실행해 선택한 20건이 모두
통과했고 나머지 1,816건은 필터로 제외됐다(`origin-guard-focused-minfont.log`, 종료
코드 0). 전체 회귀 통과로 표현하지 않는다.

GIF 보정 고정 WASM의 완성된 분할 자료로 기존 TSV 2,153행을 재대조했다. 2,146행은
유지·개선이고 7행은 하락이다. 편람 6행과 sample16 HWP 2022 35쪽 1행이며 로그는
`gif-final-comparison-wasm.log`다. 상위 전쪽 실행도 종료 코드 0으로 끝났으며 정밀
대조는 하락 7행으로 종료 코드 1이다. 이 수치는 이후의 저장 줄 출처 공통화가 반영되기
전 패키지의 결과이며, 최종 후보 검증을 대신하지 않는다.

글자 기준선 후보의 Native 빌드는 종료 코드 0이다. #8040 두 원본의 4쪽 raw는
각각 `99.717936122913% → 99.988321019626%`로 개선됐고 공개 기준
`99.717994673795%`보다 높다. 1·3쪽은 각각 100%이며 2쪽은 양쪽 원본에서
공개 기준보다 약 `0.0000006%p` 낮아 원인이 남는다. 상세 정밀 비교는
`inline-center-issue8040-exact-comparison.log`, 4쪽 그림은
`inline-center-issue8040-native-review/review/review_004.png`와
`inline-center-issue8040-native-review/overlay/overlay_004.png`다.

2쪽에 남은 미대응 1픽셀 `(580, 621)`은 `<1`의 숫자 `1` 아래쪽이다. 확대하면
RHWP 숫자는 세리프, 한컴 Print 숫자는 고딕이다. 독립 PDF의 F4 ToUnicode는 이
글자를 CID `0x26`으로 매핑하고, 임베디드 숫자 윤곽은 시스템 `Gulim`의 해당 glyph와
완전히 같으며 `HY Sinmyeongjo` OTF의 숫자 윤곽과는 다르다. 근거는
`issue8040-p002-digit-font-comparison.log`와
`issue8040-p002-single-pixel-detail.png`다. 전역 Gulim 치환은 앞선 실험에서
다른 쪽을 악화시켰으므로 채택하지 않았고, 원본 HFT 슬롯·실제 글꼴 공급 및 한컴의
대체 계약을 더 확인한다. 미대응 픽셀을 숨기거나 기존 정밀값 하락을 면제하지 않는다.

같은 Print 래스터 해시를 확인해 base와 현재 Native의 내용 합집합을 직접 분해했다.
두 출력 모두 미대응 픽셀은 1개지만 합집합은 `202557 → 202311`픽셀이다. 기준에만
있는 픽셀 380개와 후보에만 있는 픽셀 134개가 여러 글자 영역에 분포한다. RGB 차이의
상당 부분은 별도 이미지 영역에도 분포한다. 따라서 숫자 한 글자의 윤곽만으로 전체
분모 차이를 설명할 수 없다. 재현 계산은 `issue8040-page2-content-union-analysis.py`,
결과는 `issue8040-page2-content-union-analysis.log`다. 이 분석은 TSV 하락 해소를
대신하지 않는다.

고정 base의 실제 실행 파일로 2쪽 SVG를 다시 생성해 후보의 문자 노드와 대조했다.
두 출력의 2,584개 문자 순서는 같고 family가 바뀐 24개는 모두 `㉠`~`㉤`다
(`㉠`·`㉤` 각 6개, 나머지 각 4개). #8136의 기호 슬롯 전환과 같은 문자이며,
합집합 차이가 큰 좌측 하단 영역에도 있다. 독립 Print의 유일한 미대응 숫자 `1`은
이 슬롯 전환 대상이 아니므로 두 현상을 구분한다. base SVG 생성 로그는
`issue8040-p002-actual-base-svg.log`다. 기호 슬롯만 Gulim으로 지정한 기존 실험의
미대응은 여전히 1픽셀이며 정밀 raw는 `99.99950597767018%`다. 영문과 기호를 둘 다
Gulim으로 바꾼 실험은 2쪽 100%였지만 다른 페이지가 하락해 일반 해법이 아니다.

#8316의 독립 2024 Print·기여자 TSV도 확인했다. 두 쪽의 제출 당시 Native raw는
`97.29823%`, `93.98321%`, WASM raw는 `97.51195%`, `99.45628%`다. 이는 통합
후보의 측정값이 아니다. 새 Native 빌드 후 같은 입력·PDF의 두 쪽을 시스템 글꼴로
측정하도록 `stored-guard-issue8136-native.log`에 연결했으며, fresh WASM과 직접
판독도 별도로 필요하다.

저장 줄 가드를 포함한 Native 후보의 두 쪽 측정은 종료 코드 0이며 raw는 순서대로
`99.48%`, `98.95%`다. 입력·Print의 SHA와 실행 기록은
`stored-guard-issue8136-native/issue8136/silhouette_manifest.json`과
`stored-guard-issue8136-native-sweep.log`에 있다. 2쪽 Native review와 overlay는
`stored-guard-issue8136-review/issue8136/` 아래에 생성했다. 이는 Native 측정이며
같은 후보의 fresh WASM이나 1쪽 변경 기호의 확대 판독을 대신하지 않는다.

#8316 기호 출력의 원본 `궁서`는 HWPX에서 TTF로 선언돼 있다. Print의 `™`, `€`,
`㉠`은 Windows 호스트 `batang.ttc`의 `Gungsuh`와 윤곽이 같고, 현재 설치된 Linux
`Gungsuh.TTF`는 `™`가 다르다. TTC를 명시 공급하면 Native 1쪽 raw가
`99.48% → 99.90%`로 개선된다. 임시 Fontconfig에서 TTC와 기존 시스템 글꼴을 모두
노출하면 `fc-match 궁서`는 TTC를 가리키지만, 명시 공급 없는 출력은 `99.48%`다.
실제 측정 PNG와 SHA가 일치하는 2024 Native trace는 `™`·`€`에 CFF `Gungseo`,
`㉠`에 `Gungsuh`를 선택했다고 보고한다.

후보 생산과 소비를 추적했다. #8397의 `99a176b7cb`에서 도입한
`skia/font_lookup.rs::matching_system_programs`는 원본 종류 인자 없이 CFF 후보를
우선하고, `skia/text_replay.rs`의 실제 paint와 `skia/renderer.rs`의 진단이 같은
후보 함수를 호출한다. HFT의 실제 메트릭 준비는 `skia/native_pua_fonts.rs`와
`document_core/native_pua_fonts.rs::native_otf_font_requests`에서 이뤄진다. 또한
`scripts/export-wasm-png-for-sweep.mjs`와 Studio `core/host-canvas-fonts.ts`도 CFF를
우선한다. 따라서 Native paint만 바꾸면 진단·PUA 측정·WASM 공급이 어긋날 수 있다.
TTF/HFT 유형이 같은 family의 여러 원본 슬롯에 함께 쓰이는 반례와 기존 #8397 HFT
계약을 함께 보호하는 방식을 먼저 확인한다. 로컬 Windows 글꼴은 검증 경로에만 보관한다.

원본 유형 추적: HWPX `header.xml`의 `font type="TTF"`는
`parser/hwpx/header.rs`에서 `alt_type=1`로 파싱되고, 글자 모양의 언어별
`font_families_source_hft`와 실제 TextStyle의 `source_font_is_hft`로 전달된다.
다만 현재 이 값은 Native typeface 후보 함수와 Canvas 글꼴 등록의 인자가 아니다.
Native HFT 메트릭 준비는 HFT 슬롯만 수집하지만 PUA 준비는 family·굵기·기울기별로
묶여 있어 같은 family의 TTF/HFT 혼재를 그대로 추가 키 없이 구분할 수 없다.
진단 observer 역시 family·문자·굵기·기울기만 받으므로 paint 선택에 유형을 추가하면
진단도 함께 변경해야 한다. WASM의 같은 family 글꼴 등록 순서는 CSS 전체에 적용되므로
혼재한 원본별 선택을 family 전역 순서 변경으로 대체하지 않는다.

원본 유형이 없는 문서와 명시 TTF를 구분하기 위해 언어별 `alt_type=1`을
`ResolvedCharStyle`에서 `TextStyle` 및 영문 슬롯으로 전달하는 필드를 추가했다.
원본 HFT로 정정된 슬롯이 동시에 TTF로 판정되지 않도록 했다. 현재는 출처 전달만
추가한 단계이며 공급자 선택·진단·WASM 소비는 아직 바뀌지 않았다. 변경을 모두
연결한 뒤 한 번에 빌드하고 최소 글꼴 회귀와 Native/fresh WASM을 재검증한다.

Native 본문 paint와 font-decision observer가 동일한 source-aware 후보 함수를 쓰도록
연결하는 중이다. 명시 TTF는 OS가 선택한 같은 family의 실제 `glyf` TrueType을 먼저
두고, 확인할 수 없으면 기존 후보 사다리를 유지한다. CFF를 무조건 TTF로 바꾸지 않으며
HFT·유형 미상은 기존 #8397 순서를 유지한다. 이 단계에는 Native PUA 측정과 WASM
Canvas 등록의 슬롯별 유형 전달이 아직 연결되지 않았으므로 최종 수정이나 검증 완료가
아니다.

원 PUA와 리터럴 숫자폭 공백의 세션 메트릭을 family·스타일뿐 아니라 원본 TTF
선언별로 구분하는 경로를 추가 중이다. 기존에 유형을 지정하지 않은 공개 등록 API는
공유 항목으로 유지하고, 유형별 등록이 없을 때만 그 항목으로 폴백한다. Native PUA
준비도 본문과 같은 유형별 후보 사다리를 소비한다. 새 세션 등록 API는 문서 IR을
바꾸지 않고 기존 배치 갱신 경로를 사용하므로 passthrough 분류에 명시했다. WASM
호스트의 등록과 Canvas 자체 선택은 아직 연결되지 않았고 빌드·실행 전이다.

HWP 원시 `LINE_SEG`와 주변 표 제어를 조사하면 #8040에서 높이의 정확히 절반을
기준선으로 저장한 표 근접 줄은 3개다. 그중 두 줄은 4쪽의 텍스트·표 혼합 사례이고
한 줄은 표 전용이므로 후자는 문자 보정에 포함되지 않아야 한다. 조사는
`inline-center-stored-shape-inventory.log`에 기록했다. 이 근접 제어 조사는 실행
결과나 corpus 전체의 정확한 소유 판정을 대신하지 않는다.

회귀 파일 주석 점검에서 첫 줄에 주석이 없는 파일과 이 브랜치에서 변경됐으나
첫 줄에 검사 유형이 없는 파일을 합쳐 96개를 찾았다. 이 중 실제 `#[test]`가 있는
파일은 91개, 지원 파일 등은 5개이며 브랜치에서 변경된 파일은 19개다. 파일별 직전
Git 커밋·시각과 경로 참조는 `regression-comment-audit.tsv`에 기록했다. 렌더링 검사와
파서·지원 검사를 구분해 각 assertion과 독립 시각 근거를 확인해야 하며, 누락된
수치를 추정하거나 주석 추가만으로 전체 회귀를 다시 실행하지 않는다.

그중 모델·API·직렬화 계약만 검사하는 파일은 실제 assertion, 입력 경로와 직전 Git
커밋을 대조해 첫 줄 유형과 갱신 시각을 추가했다. 화면을 측정하지 않는 이유를 적었으며
다른 문서의 시각 수치를 대신 기재하지 않았다. 처리 목록은
`regression-nonvisual-comment-updates.log`이고 별도로 수식 직렬화, CFB 계약,
코퍼스 저장 손실, 계산식·raw 인벤토리 파일의 첫머리도 정리했다. 주석만 바꾼
부분에 대한 전체 회귀는 반복하지 않았다. 시각 출력·쪽 배치를 검사하는 나머지 파일은
각 원본의 실제 측정과 독립 기대값을 확인해야 한다.

추가로 묶음 속성의 원본 행렬 보존, 합성 래스터 진단 전달, 글꼴 규칙 레지스트리
투영 검사에 실제 검사 유형과 직전 파일 커밋을 적었다. 이 세 파일은 화면 일치율이
아닌 API·데이터 계약을 검사하며, 주석 변경에 따른 전체 회귀는 실행하지 않았다.

최종 시각 선행 조건이 통과하면 캡션과 호스트 줄의 포함 관계, 분수 표 두 행과
옆 텍스트의 기준선 순서·상대 위치를 검사하는 글꼴 비의존 회귀를 등록할 계획이다.
초안은 `output/pr-review/planet6897-ordered-20261031/inline-center-relational-test-draft.rs`에
있으며 아직 정식 검사나 실행 완료 증거가 아니다.

초안의 첫 진단 실행은 같은 단어가 든 앞 문단을 선택해 실패했다. 표 옆 문구의
정확한 run을 선택하도록 수정한 뒤 최소 글꼴 파일 시스템 격리에서 두 원본의 두 검사가
모두 통과했다(`inline-center-relational-test-minfont.log`). 같은 영역의 수정 전·후
SVG에서 분자와 분모 기준선 사이의 상대 위치는 `0.2335 → 0.5`이고 한컴 Print도
중앙이다(`issue8040-p004-inline-center-before-after.log`). 수정 전후 영역 진단이며,
수정 전 정식 회귀 실행이나 Native/fresh WASM 전쪽 시각 검증을 대신하지 않는다.

#7150의 기존 대조군 `issue2470`은 원 PR 기준 PDF인
`pdf/issue2470/36382471_masked-2022.pdf`로 다시 측정했다. 1쪽 raw는 변경 전후 모두
`76.50004%`이며 같은 영역의 수정 전후 Native PNG 해시도 같다. 이 수치는 현재
변경에 따른 하락이 없다는 대조 증거이지 90% 통과 증거가 아니다. 한컴 2020 PDF와
정본 2022 PDF의 해당 쪽 래스터 해시가 같다는 점도 확인했다. 로그는
`issue2470-canonical-print-before.log`와 `issue2470-canonical-print-after.log`다.

`sample16` 검사 참조의 출처를 확인했다. 다수의 `issue_8160_*` 검사는 고정 base
`bc9dfdd057258de59afebeb5121c4bee7398e07f`에 이미 존재한다. 별도로
`issue_8304_hwp3_index_mark.rs`의 빈 하이퍼텍스트 대조와
`issue_8303_hwp3_hwpx_textpos_idempotent.rs`의 원본 참조는 이번 체리픽에서
도입됐다. 이 확인은 삭제된 파생 파일의 재생성 여부와 기존 검사 유지 여부를 구분하기
위한 것이며, base 검사를 일괄 삭제하거나 이전 삭제 파일을 재생성하지 않았다.

같은 family의 명시 TTF와 HFT 요청은 글꼴 공급 키를 분리했다. 기존 공통 등록값이
남아 있어도 새 유형의 글리프 등록 성공으로 판단하지 않도록 성공 여부는 해당 유형의
등록만 확인한다. 브라우저 family 선택과 Visual Sweep 공급 계약의 혼합 TTF·CFF
집중 검사는 각각 `font-source-host-tests.log`, `font-source-type-tests.log`에 기록했다.
이는 프로그램 선택 계약의 증거이며 실제 문서의 Native/fresh WASM 시각 검증이나
전체 회귀의 완료를 뜻하지 않는다.

명시 TTF 우선 후보를 Native로 다시 빌드했다. 시스템 글꼴 전체 환경에서 #8136
1쪽 raw는 `99.480161256100% → 99.904468740049%`, 2쪽은
`98.953594176524% → 98.955495004541%`다. #8040 두 입력은 직전 후보와
전쪽 정밀 raw가 같으며, 2쪽은 #8389 기준보다 각각 약 `0.000000600300%p`,
`0.000000590802%p` 낮다. 이 결과로 무하락을 선언하지 않는다. 실행 파일 SHA와
전체 측정은 `font-source-native-sweep.log`, Print 글꼴을 별도로 추가한 비교는
`font-source-issue8136-print-font.log`에 있다. Native 결과이며 fresh WASM 검증은
별도로 수행해야 한다.

#8389 대상 Native 26쪽을 원 TSV의 표시 수치와 반올림 없이 대조하자 21쪽은
유지·개선이고 5쪽은 하락했다. 앞의 #8040 두 쪽 외에 `2026_oss_rst.hwp`
1·2·6쪽이 기준보다 각각 `0.003656623101`, `0.004723037023`,
`0.004990891623%p` 낮다. 기존 대조 스크립트가 후보를 TSV 소수 자릿수로
반올림해 이 세 쪽을 놓쳤으므로 원래 비교를 무하락 증거로 쓰지 않는다.
대조 대상 전체는 `font-source-native-comparison.tsv`, 계산과 판정은
`compare-font-source-native.py` 및 `font-source-native-comparison.log`에 있다.
#7905는 6쪽 모두 기존 수치 이상, #5169도 6쪽 모두 기존 수치 이상이다.
OSS 1·2·6쪽의 raw 미대응은 각각 43·26·6픽셀로, 독립 Print와 같은 위치의
확대 판독을 통해 원인과 base 출력과의 차이를 확인해야 한다.

OSS 6쪽의 미대응 6픽셀은 `(347,380)~(347,385)`에 모여 있으며, 독립 PDF의
`및` 글자 영역이다. PDF의 일반 맑은 고딕 내장 subset에서 `및`과 인접 `디`를
시스템 `malgun.ttf`와 분해 윤곽 및 hmtx로 비교한 결과 두 글자 모두 일치한다.
PDF bbox의 `및` 시작 x는 96 DPI 환산 약 `336.432px`, 후보 SVG의 원점은
`336.827px`이다. SVG 원점은 실제 Native paint와 다를 수 있으므로 이 값만으로
원인을 확정하지 않는다. 시각 자료는 `font-source-oss-review/oss/`와
`font-source-oss-p006-detail.png`다.

OSS 6쪽 Native paint trace는 해당 `및`·`디`를 시스템 `Malgun Gothic`으로
선택하며, 같은 trace로 재생한 PNG의 SHA-256은 실제 sweep PNG와 모두
`81f2a9c35530cb74a9da4126ca08b231e9a3edb5dc6e11e4a6c96bda25670dda`다.
FontMgr의 같은 이름·일반 두 조회는 SHA-256 `6706a05313c7fafb3fd8fda9538c9ecabac7cede30076b3ea85be7716f356702`의 동일 프로그램을 반환했고,
이는 PDF 내장 부분집합의 두 글자와 윤곽·전진폭이 일치한다. 96 DPI에서 같은
글꼴·크기·색·원점으로 `및` 하나를 그린 분리 진단은 기본 힌팅이 실제 출력과
영역 RGB가 완전히 같고 PDF에 없는 획 차이 6픽셀을 재현했다. 힌팅 없음과 원본
path 재생에서는 해당 영역의 미대응이 0이다. 진단 원본·로그는 각각
`oss-glyph-raster-diagnostic.rs`, `oss-p006-hinting-diagnostic.log`다. 이는 해당
문자의 원인 분리이며 문서 전체 보정이나 회귀 통과를 의미하지 않는다.

기존 `issue_8153_rebase_legacy_print_text_20261030.md`는 #7905 등 2018/2022
기존 인쇄 경로의 보존과 2024 기하 정책의 분리를 독립 대조로 기록한다. 현재
#7905에서 2024 정책을 통째로 적용한 진단은 2022 후보보다 6쪽 모두 하락했다.
글자 외 clip 등도 바뀐 진단이므로 힌팅만의 효과로 단정하지 않는다. 2020 기준
OSS의 원본 glyph 손실을 해결하면서 #7905의 기존 2018 계약도 보존할 공통 근거와
분리 검증이 필요하며, 인쇄 세대 전체를 임의로 바꾸지 않았다.

이전 `gif-final-frozen-all` 2,153행 비교도 기준값의 소수 자릿수로 현재값을
반올림해 하락을 놓칠 수 있었다. 두 자리 기준을 그대로 비교하고 계산의 12자리
표현 오차만 제거한 재계산에서는 당시 후보가 Native 24행, WASM 20행에서 기준
표시값보다 낮다. 두 자리로만 제공된 기준은 원래 미반올림값을 복원할 수 없으므로
그 표시값과의 비교이지 실제 원점수 하락의 확증은 아니다. 당시 후보의 결과이므로
이후 보정한 현재 후보의 최종 결과와 혼동하지 않는다. 재계산 로그는
`frozen-precise-native.log`, `frozen-precise-wasm.log`다.

현재 시스템의 `fc-list`에는 2,147개 글꼴 파일이 노출된다. 기존 검증 스크립트의
`설치 글꼴 2069개` 문구는 해시 목록에 포함된 SFNT 파일 수이며 전체 시스템
글꼴 수가 아니다. 나머지 78개는 Type 1(`.pfb` 43개, `.t1` 35개)이며
Fontconfig에서 차단하지 않았다. 전체 측정 도중 추가 목록과 해시를
`font-source-full/extra-system-type1-fonts.json`에 기록했으며 종료 후 다시 대조한다.

#8040 2쪽의 같은 실행 파일로 trace를 생성한 PNG는 sweep PNG와 SHA-256
`3f241958ebcb452a099b85b55d8b2ad0d37036cc1e2f21ebe9b336433c7a55ea`가 같다.
문제의 `<1`은 실제 시스템 `HY Sinmyeongjo`를 공급받는다. 원 PDF의 숫자 `1`은
Gulim 윤곽이므로 선택된 원본 HFT 대체 OTF와 다르다. 문제 좌표 `(580,621)`은
PDF 전용이 아니라 RHWP 전용 잉크다. 이에 대해 단순한 무힌팅 적용은 해결책이라는
근거가 없으며, 다른 페이지를 악화시킨 일괄 Gulim 치환도 재도입하지 않았다.

추가된 원본 TTF 선택 조건의 입력 근거도 확인했다. HWP5 형식 설명본의 FACE_NAME
속성 표에는 하위 비트의 의미가 명시돼 있지 않으므로 비트만으로 모든 HWP의
글꼴 유형을 사양상 확정했다고 주장하지 않는다. 실제 OSS HWP DocInfo의 맑은
고딕 레코드는 속성 `0x61`, 같은 이름의 HWPX는 `type="TTF"`이며, #8040
한양신명조 HWP 레코드는 `0x62`다. 자료와 판독 절차는
`hwp-font-type-source-evidence.log`에 있다. 여러 생산기와 대체 글꼴 조합에서
하위 비트가 같은 의미인지 추가 확인이 필요하며, 이 한 사례로 일반성을 확정하지
않는다. HWPX의 명시 유형과 HWP3 파서가 별도로 확정한 유형은 구분해 검토한다.

전쪽 Native 재측정 스크립트에 이전의 sample16 글꼴 치환 환경이 남은 것을 발견해
해당 진행을 중단했다(종료 143). 이 부분은 시스템 글꼴 전체를 자동으로 사용하는
사용자 조건의 결과로 사용하지 않는다. 치환 없이 끝난 #7905·#8040 세 입력은 같은
후보 SHA와 입력·PDF·시스템 글꼴 조건이므로 재사용하고, 나머지는
`font-source-full-all-system-native.log`로 치환 설정 없이 다시 실행한다. 기존의
`설치 글꼴 2069개` 표기는 SFNT 해시 목록 수이며 Fontconfig가 추가 Type 1 78개를
차단한다는 뜻이 아니다.

#8079 원 증적은 기여자의 Native 코드 head `bba7229a1b013ffcb6bae8416ef23b53da009400`를
지목한다. 원 점수는 두 자리로만 남아 있으므로 같은 시스템 글꼴과 독립 PDF 조건의
코드 비교를 위해 원 head를 별도 검토 worktree로 준비하고 공유 `target/pr-review`에
빌드한다. 원 증적 당시에는 문서별 글꼴 경로를 우선한 점이 현재 자동 선택 조건과
다르므로 이번 재현만으로 당시 미반올림 수치를 복원했다고 주장하지 않는다.

#8079 제출 당시 코드 `bba7229a1b013ffcb6bae8416ef23b53da009400`를 빌드해
현재 시스템 글꼴·같은 OSS 원본·독립 PDF·96 DPI로 전 6쪽을 직접 측정했다.
원 코드와 현재 후보의 raw 차이는 1·2·3쪽 0, 4쪽 `-0.000000448128%p`,
5쪽 `+0.000000033929%p`, 6쪽 `+0.000000083031%p`다. PDF 래스터 SHA도
같다. 따라서 현재 시스템 조건에서 1·2·6쪽을 원 코드 대비 하락으로 부를 수는
없다. 당시 TSV의 두 자리 표시와 동일 환경 재현의 정밀 비교는 구분한다.
`issue8079-original-vs-current-oss.log`에 수치를 보존했다.

4쪽의 두 출력은 미대응 6픽셀이 같지만 실루엣 합집합이 `63379 → 63376`으로
줄었다. 차이 3픽셀은 상단 `SBOM` 글자 영역이며 자간 분리의 글리프 fit 폭이
변경된 결과와 대응한다. 위치는 같고 `S/B/O/M`의 SVG textLength는 원 코드의
`12.6933/13.6000/12.6933/17.1733`에서 현재
`12.4267/13.3333/12.4267/16.8533`으로 달라졌다. 이 자간 분리 코드는 #8389가
도입한 공통 계약이므로 점수만 맞추려고 되돌리지 않는다. 같은 영역의 Print 대비
RGB 절대차와 24 초과 차이 픽셀도 후보 쪽이 작으며, 이것만으로 전체 시각 승인을
의미하지는 않는다. 원·후·Print 부분 비교는 `issue8079-oss-p004-sbom-compare.png`,
합집합 차이는 `issue8079-oss-p004-delta.log`다. 남은 6픽셀은 다른 행의 `및`에서
6쪽과 같은 누락된 획이며, 2018 기준 보호를 유지하는 공통 해결이 필요하다.

개별 글꼴 치환 없이 시스템 전체 글꼴 자동 선택으로 다시 산출한 sample16 HWPX
2쪽 raw는 `96.750084449949%`다. 기존 2024 대조는 `휴먼명조`를 같은 이름의
TrueType으로 치환하는 별도 환경을 명시적으로 사용했다. 현재 `fc-match 휴먼명조`는
시스템 HFT 대응 OTF `HMKSM.otf`를 반환하고, 기준 PDF는 CID TrueType과 Type 3
글꼴을 포함한다. 원본 HWPX에는 `휴먼명조`가 HFT로 선언돼 있다. 이 관측만으로
기존 코드 대비 회귀나 원 PDF의 TrueType 파일 정체를 단정하지 않는다. 현재 자동
선택의 직접 검토 자료는 `sample16-system-p002-review/sample16/review/review_002.png`와
`overlay/overlay_002.png`다. 전쪽 결과와 실제 공급·독립 출력 계약을 추가 확인한다.

유형별 등록의 재준비·중복 격리 검사 및 기존 공급 검사는 Node에서 25건 모두
통과했고 TypeScript 검사도 통과했다(`font-source-mixed-contract-tests.log`,
`font-source-mixed-typescript.log`). 저장소 fixture를 명시 공급하는 최소 글꼴 Rust
단일 계약 검사는 첫 시도가 테스트 실행 전 종료 143으로 중단돼 원인은 확인되지
않았다. 공유 산출물을 유지하고 별도 로그 `font-source-rust-minfont-retry.log`로
재실행한다. 이 내용은 전쪽 또는 전체 회귀 성공을 의미하지 않는다.

sample16 2쪽 독립 2024 Print PDF의 F4 내장 TrueType 부분집합과 설치 프로그램의
글리프를 분해 윤곽으로 직접 대조했다. 실제 2쪽 `사업개요`에 쓰인 `사·업·개·요`
네 글자는 `HMKMM.TTF`와 모두 같고 `HMKSM.otf`, `ENSMJ.otf`와는 모두 다르다.
PDF 텍스트 추출로 해당 제목이 F4의 글꼴을 사용하는 것도 확인했다. 실행 가능한
대조 코드는 `sample16-p002-print-font-outline-check.py`, 결과는 같은 이름의 `.log`다.
이는 기준 PDF의 실제 글꼴에 대한 증거이며 현재 Native 후보의 선택과 동일 실행
PNG의 trace까지 확인한 것은 아니다. 원본의 HFT 선언과 OTF 우선 계약, 명시적인
한컴 2024의 실제 TTF 출력 사이를 공통 정책으로 해결해야 한다. 추측해 문서별로
치환하거나 원본 PDF를 변경하지 않는다.

동일한 시스템 글꼴·동일 Print 조건에서 현재 변경 전 고정 실행 파일
`rhwp-gif-crop-candidate`로 sample16 HWPX 2쪽을 직접 출력하면 raw 100%, 현재
`rhwp-font-source-candidate`는 96.75%다. 준비된 Native render tree에서 제목
`사업개요` 폭은 `80.0 → 84.1px`다. SVG CLI는 Native 글꼴 준비 이전 결과이므로
두 SVG가 같다는 사실을 이 실제 출력의 증거로 쓰지 않는다. `source_font_is_hft`
하나만으로 OTF 합성 굵기 전진을 추가한 미커밋 확장을 제거하고 기존의 원 HWP3
또는 확인된 원 HFT 메트릭 조건으로 복구했다. 수정 후 새 실행 파일에서 이 쪽,
#8040·#7905와 원래 개선 대상 전체를 재검증해야 한다.

기여자 원본 코드를 같은 공유 target에 빌드한 뒤 현재 코드의 첫 재빌드는 JPEG
의존성 API를 찾지 못해 종료 101이었다. 양 코드의 `rhwp-zune-jpeg` 패키지는 이름과
버전이 같지만 현재 버전에만 `transforms`와 `set_transforms`가 있다. 파일 내용은
변경하지 않고 현재 의존성 소스의 수정 시각만 갱신해 다시 컴파일하며,
`font-source-tracking-fix-build-retry.log`에서 종료 상태를 확인한다.

굵기 전진 조건을 넓힌 미커밋 부분만 제거한 후보 SHA-256
`a86bb82d53658c0814043a5aa84c6c4340114d5a1b15b992bd5bf3f58292f383`로 일괄
대조했다. sample16 HWPX 2쪽은 100%·미대응 0이고 RHWP PNG SHA가 이전의 정상
고정 후보와 완전히 같다. #7905 여섯 쪽은 전부 #8389 정밀 기준 이상이며 #8136
두 쪽과 OSS 여섯 쪽은 직전 후보와 동일하다. #8040 두 원본 1쪽은 기준보다
`0.000075822890%p`, 2쪽은 각각 `0.000000600300%p`, `0.000000590802%p`
낮다. 두 원본 3·4쪽은 기준 이상이다. `tracking-fix-focus.log`와 입력별 로그에
실행을 보존한다. 따라서 모든 HFT에 일괄 추가하거나 일괄 제거하는 방법 둘 다
필수 무하락 조건을 만족하지 않는다.

#8040 1쪽 제목의 원본 슬롯은 HFT `신명 중고딕`, 실제 시스템 표시 family는
`Sinmyeong Junggothic`이다. 독립 2020 Print의 `뜨거워진한반도`는 Type3 `T5`이며
96 DPI 환산 인접 한글 원점 간격은 약 12.8px다. 반면 sample16 2024 Print의
제목은 실제 TrueType `HMKMM.TTF`의 윤곽을 사용한다. 표시용 OTF가 있다는
이유나 HFT 선언만으로 같은 굵기 전진을 적용할 수 없다. 동일 프로그램에 속하는
반례와 원본 선언, 선택 프로그램, 재조판/저장 정보를 더 대조해 공통 계약을 정한다.

최소 글꼴 파일 시스템 격리에서 `NativePuaFonts`의 유형별 등록 계약과 굵기 전진
관계형 검사 두 건을 실행해 모두 통과했다. 3,996건은 필터로 제외됐으며 전체
회귀 통과로 기록하지 않는다(`tracking-fix-rust-minfont-focused.log`).

같은 HFT 이름을 지정한 독립 대조 입력을 한컴 2020과 2024의 Print로 출력해
보통·굵게, 10pt·20pt의 한글 전진을 비교했다. 2020에서 굵게의 추가 전진은
각각 약 `0.051963em`, `0.049973em`이며, 2024의 대조에서도 같은 계약을
확인했다. #8040 기준 PDF와 대조 입력의 굵은 `가` Type3 글리프 프로그램은
SHA-256이 같다. 상세 측정은 `output/pr-review/planet6897-ordered-20261031/`
아래 `hft-bold-contract-mcp/analyze-advances.log`에 있다.

이 근거를 적용할 때는 원본 슬롯의 HFT 선언과 실제 글리프 공급 OTF의 family를
함께 확인한다. 다른 프로그램으로 대체된 경우에는 같은 전진을 가정하지 않는다.
저장소의 시험 글꼴 이름표를 메모리에서 변경한 독립 계약 검사로 두 조건의 조합을
확인했으며, 시스템 글꼴을 숨긴 최소 글꼴 환경에서 관련 검사 3건이 통과했다
(`verified-hft-minfont-focused-final2.log`). 실제 문서의 Native와 fresh WASM
전체 TSV 및 기존 기준 대비 무하락 여부는 새 빌드 결과로 별도 확인한다.

#8040 2쪽의 1픽셀 차이에 대해 `한양신명조`를 HFT로 선언한 별도 HWPX 대조
입력을 한컴 2020에서 Print했다. 상태 응답은 11.0.0.9136, 인쇄 방식 0과
`hancom2020_pdf_driver_one_up`을 확인한다. 결과의 영문 계열 CID 글꼴 기술자는
Gulim이다. 이는 해당 인쇄 환경에서의 대체 사례를 재현하지만 모든 HFT 프로그램이나
글꼴 설치 환경의 보편 규칙을 입증하지는 않는다. 입력·Print PDF와 실행 상태는
`output/pr-review/planet6897-ordered-20261031/hft-bold-contract-mcp/`의
`hanyang-latin-probe` 접두어 파일에 보관한다.

별도 `휴먼명조` TTF 명시 HWPX를 같은 한컴 2020 MCP에서 Print한 반례도 확인했다.
PDF 글꼴 프로그램의 `가`와 `1` 윤곽은 설치된 휴먼명조 TTF와 각각 같고 Gulim과는
다르다(`human-ttf-latin-outline-check.log`). 따라서 다른 출력에서 Gulim을 썼다는
사실만으로 모든 휴먼명조 문서나 모든 2020 출력에 Gulim을 강제하지 않는다.

같은 형식·크기·문자열에서 HFT 이름만 `신명 중고딕`으로 바꾼 반례도 한컴
2020 Print로 확인했다. 한양신명조 입력은 PDF에서 한글 `가`를 Type3 `/T2`,
숫자 `1`을 `/F1`의 Gulim으로 그린다. 신명 중고딕 입력은 두 문자 모두 Type3
`/T1`을 사용하며 각 글리프의 ToUnicode 매핑과 실제 `TJ` 연산을 대조했다.
따라서 HFT 전체의 영문을 일괄 대체할 근거가 되지 않는다. 반례 입력·PDF·상태는
같은 디렉터리의 `jung-gothic-latin-probe` 접두어 파일이다.

새 회귀 파일의 첫머리 감사에서 `issue_8304_hwp3_index_mark.rs`의 설명은
찾아보기 IR·저장 계약만 검사한다고 되어 있으나, 실제 파일에는 쪽수와 문단 간격,
쪽 배경, 하이퍼텍스트 상자 및 쪽별 내용을 확인하는 함수도 함께 있다. 따라서
파일 전체를 비시각 계약으로 분류하지 않는다. 같은 입력과 독립 PDF의 최종 후보
Native·fresh WASM 전쪽 결과를 확인한 다음 실제 assertion과 함께 주석을 정정한다.

설치된 시스템 글꼴 전체를 같은 조건으로 노출해 #8397 기준 실행 파일과 현재 후보의
sample16 2022 HWP를 대조했다. 1쪽은 둘 다 `99.194628731819%`이고 2쪽은
`98.978770639656% → 98.951435651188%`, 10쪽은
`98.434303784872% → 98.341966593200%`다. 기준 자체도 공개 TSV의 100%보다
낮으나 현재 후보의 2·10쪽에는 같은 환경에서도 하락이 있다. 2쪽 Render Tree의
텍스트와 상자는 두 출력에서 같고 PNG는 다르다. 추적 PNG SHA-256이 현재 sweep
PNG와 같음을 확인했으며, 원본 `휴먼명조`의 글꼴 레코드 유형 1을 새 코드가 TTF
선택에 사용한다. 시스템에는 같은 family의 OTF와 TTF가 모두 있다. 한컴 Print는
같은 쪽에서 HFT Type3와 Gulim을 사용하므로 원시 속성만으로 모든 글자의 실제
프로그램을 TTF라고 단정할 수 없다.

HWP5 FACE_NAME 속성의 하위 비트는 공식 표 20에서 유형으로 정의하지 않는다.
공통 스타일 해소에서 HWP5의 해당 비트만으로 TTF 우선 플래그를 확정하지 않고,
OWPML의 명시적 `type`을 가진 HWPX와 구분하도록 변경했다. 분리 OTF의 기존
선택과 HWPX TTF 계약을 함께 확인할 최소 글꼴 단위 검사 및 새 후보의 Native·WASM
출력은 이 변경 뒤 다시 검증해야 한다. 앞의 `verified-hft-full` 전쪽 측정은
변경 전 실행 파일의 근거로만 사용한다.

새 Native 후보 SHA-256 `b38728ae6740b44deb2a6b5c574110d07c5b44535db34b593284a45276b2a3c7`의
sample16 2022는 동일 시스템 환경 기준 대비 1·2쪽 raw가 정확히 같고, 10쪽은
`98.434303784872% → 98.433992103604%`로 `0.000311681267%p` 낮다. 미대응은
양쪽 모두 1,416픽셀이고 내용 합집합은 `90,439 → 90,421`픽셀이다. 두 RHWP
PNG의 차이는 10쪽 `※` 한 글자 영역 110픽셀에 한정된다. 새 후보는 #8316의
기호 슬롯을 사용하며, 확대 영역에서 독립 PDF와의 절대 밝기 차이 합계는 기준
5,667, 새 후보 4,033이었다. 이는 해당 영역이 개선된 관측이지만 요청한 정밀
무하락 조건을 대신하지 않는다.

독립 Print 10쪽의 `※`는 `/T8` Type3, 문자 코드 `03`, 원 글리프 `/HFT146`이다.
실제 윤곽의 첫 곡선은 시스템의 `SPSMJ.otf` 내부 `hft3438`과 같으며, 제공된
동일 계열의 완성 기호 OTF는 `uni203B`를 같은 윤곽으로 매핑한다. 실제 문자 선택
trace의 PNG SHA-256은 sweep PNG와 일치한다. 다만 같은 윤곽을 가진 CFF를 골랐다는
사실만으로 출력 백엔드의 힌팅 정책이나 다른 글꼴에 대한 일반 규칙을 확정하지
않는다. 해당 경계를 원본 형식과 공통 출력 경로에서 더 확인한다.

새 Native 후보에서 #8040 두 원본의 1쪽은 모두 미대응 0으로 개선됐다. 2쪽은
각각 `99.999505711504%`, `99.999505606423%`로 표시 반올림 시 100.00%지만,
이전 정밀 기준에 미치지 못해 무하락 판정은 보류한다. 같은 후보의 대조군과
기존 PR의 나머지 전체 쪽도 확인 중이며 fresh WASM은 아직 실행하지 않았다.

### 원본 윤곽 프로그램 식별 실험

한컴 Print의 `※` 윤곽과 설치된 원본 윤곽 OTF의 해당 글리프는 경로 명령
30개까지 같았다. HWP5에서 원본 글꼴 종류가 확정되지 않은 경우에도 실제 선택된
프로그램에 `OriginalHFT` 원본 식별자가 있으면 Print 윤곽 경로를 사용하도록
Native·Canvas의 공통 조건을 시험했다. 명시적 TTF와 다른 프로그램은 제외했다.
실제 글꼴의 식별자에 붙은 확장 접미사도 확인해 두 후보를 각각 빌드했다.

동일 시스템 글꼴 환경의 sample16 2022 10쪽은 두 후보 모두 기존 후보와 같은
`98.433992103604%`였다. 같은 환경의 이전 base `98.434303784872%` 대비
`0.000311681267%p` 하락도 그대로다. 따라서 윤곽 경로가 실제 사용됐는지와
선택된 face의 동일성부터 더 확인해야 하며, 이 실험을 회귀 해결로 판정하지 않는다.
1·2쪽은 base와 같았다. 실행 로그는 각각 `output/pr-review/planet6897-ordered-20261031/`
아래 `hft-origin-focus-compare.log`와 `hft-origin-v2-focus-compare.log`다.

실제 Skia 시스템 face 조회에서는 같은 `한양신명조` 요청의 기본 face가
`HY Sinmyeongjo`이며 `※` 글리프가 없었다. 같은 요청의 일곱 번째 face는
family가 `한양신명조`이고 해당 글리프가 있다. 두 face 모두 시스템에서 실제
글꼴 바이트를 읽을 수 있었다. 기존 윤곽 재생은 선택된 face의 family 문자열과
기본 face의 family 문자열을 직접 비교하므로 이 두 face를 같은 요청의 구성원으로
인식하지 못한다. 시스템별 설치 구성에 의존하지 않는 합성 face 대조와 실제
출력에서 이 분기가 원인인지 확인하기 전에는 family 조건을 변경하지 않는다.
진단 코드는 `output/pr-review/planet6897-ordered-20261031/diagnose-system-face.rs`,
실행 결과는 같은 디렉터리의 `diagnose-system-face.log`에 있다.

추가 분리 실행에서 기존 Print 2022 기호는 같은 OTF의 일반 raster 출력과
픽셀이 같았고, 실제 윤곽을 사용한 진단은 해당 확대 영역의 Print와 차이가 더
작았다. 그러나 기호 영역만 합성해 계산하면 전쪽 raw는 `98.433992103604%`에서
`98.433888182271%`로 더 낮아졌다. 한 글자의 윤곽 재생만으로 사용자가 요구한
전쪽 무하락을 충족했다고 볼 수 없으며, 합성 이미지는 최종 검증 증거가 아니다.

별도의 font-only 계약에서는 독창적인 MIT 사각형 CFF 두 개에 서로 다른 기본
family 이름과 같은 현지화 family를 주고, 두 번째 face만 해당 글리프를 갖도록
했다. 2022 Print 정책에서 원본 HFT로 지정한 두 번째 face의 기준선에 0.25px를
더해도 출력 무게중심이 움직이지 않아 기존 기본 이름 비교의 결함을 재현했다.
시스템 글꼴을 사용하지 않는 같은 사례를 회귀 검사에 추가하고, 실제 선택 face의
family 레코드가 요청 family를 선언한 경우를 허용하는 공통 선택 경계를 보정했다.
이 보정은 HWP5의 미확정 HFT 여부를 추정하지 않으며 sample16의 무하락 해결을
뜻하지 않는다. 같은 원본 family가 아닌 대체 face에는 윤곽 정책을 적용하지
않는 반례도 추가했으며, 최소 글꼴 환경의 HFT 계약 집중 회귀 14건은 모두 통과했다.
기존 동일 조건 분리 실행의 0.25px 이동은 0이었으며, 새 관계형 검사는 수정 후
이를 보존한다. 로그는 `output/pr-review/planet6897-ordered-20261031/` 아래
`diagnose-raw-alias.log`, `localized-hft-family-boundaries-minfont.log`다. 같은
시스템 글꼴 환경의 대표 원본 8건·28쪽은 직전 후보와 raw가 모두 같았다
(`localized-hft-family-vs-previous.tsv`). 기존 base와 비교한 10쪽 미세 하락은
남아 있으며 나머지 전체 페이지와 fresh WASM은 별도로 확인한다.

### 굵은 원본 프로그램과 표시 대체의 전진 분리

고정 후보 `5dca4233787ee290bcfaedf6b5eca7e53f710a1a`의 Native를 기존
TSV와 시스템 전체 글꼴 환경에서 대조하고 있다. 소수 둘째 자리로만 보존된
영월·줄 번호·중첩 표는 #8397 통합 커밋 `99a176b7cb`의 실행 파일을 같은
원본·PDF·글꼴로 다시 출력해 비교했다. 영월 4쪽과 줄 번호 9쪽은 모두 PNG가
같고, 중첩 표는 17쪽 중 9쪽이 `99.790593495145% → 99.788828217498%`로
낮다. 해당 쪽에서 변경된 표 내부 문자는 `※`다. Print의 기호 Type3 윤곽은
sample16의 독립 Print 기호와 일치하지만 현재 표시는 다른 글꼴의 기호다.
각 전쪽 정밀값은 `output/pr-review/planet6897-ordered-20261031/`의
`rounded-original-system-comparison.tsv`에 있다. 원본 기호를 부분 합성한
진단은 실제 제품 출력이나 무하락 검증으로 사용하지 않는다.

sample16 2022는 같은 원 코드·같은 환경의 전64쪽 중 10쪽에서 하락했다.
55쪽 제목의 `IDC` 뒤 한글은 원 코드보다 왼쪽으로 약 1.68 CSSpx 이동했다.
한컴 Print의 `I/D/C` 원점은 각각 184.3200/194.5591/216.7982 CSSpx이고,
원 코드는 184.2667/194.6667/216.8533 CSSpx다. 대상 PDF는 이 영문을
원본 HCI Poppy Type3로 출력한다. 현재 원본 글꼴 소유는 `HCI Poppy`지만
유형이 미확정이고 표시만 `Palatino Linotype`으로 대체됐는데, #8079의
보통 TTF 전진 선택이 이 표시 대체에도 적용돼 `IDC` 총폭이 51.84→48.48px로
줄었다. 원 이슈 #8079의 독립 근거는 굵은 맑은 고딕 TTF이며 임의의 다른
원본 프로그램까지 보통 폭을 쓴다는 증거는 아니다.

원본 글리프 소유가 남고 글꼴 유형이 미확정인 슬롯은 기존 굵은 전진을
유지하고, 명시 TTF 및 원본 프로그램 소유가 없는 슬롯은 기존 #8079 처리를
유지하도록 공통 측정 경계를 수정했다. 독립 원본의 굵은 `I/D/C` 폭
199/426/370 ÷ 512em과 명시 TTF 대조군을 사용하는 시스템 글꼴 비의존
계약 진단은 수정 전 원본 폭에서 실패하고 TTF에서 통과했다
(`hci-bold-source-independent-before.log`). 수정 후 집중 회귀, 각 페이지의
Native/fresh WASM 재출력 및 모든 기준 대조는 별도로 확인한다.

시스템 전체 글꼴로 원 코드와 대조한 sample16 2024 HWP·HWPX는 각각
64쪽 모두 raw 하락이 없지만, 각 1·10·53·60쪽 PNG는 달랐다. 1쪽 상단
GIF 그림의 317픽셀은 원 코드가 Print와 같고 후보는 채널당 최대 12 차이를
보였다. HWPX의 `imgDim`과 `imgClip`은 둘 다 9960×3000이고 원 GIF는
133×40 픽셀이다. 원·후 SVG의 이미지 배치와 JPEG 양자화 표는 같지만
실제 JPEG 바이트와 디코드된 하단 9행 일부가 다르다. 이 차이는 글꼴 전진
수정과 별개이며, 원본 전체 GIF의 Print 자원 생성 경로와 의도된 잘림의
대조군을 확인하기 전에는 이미지 보존으로 판정하지 않는다. 증거는 같은
`output/pr-review/planet6897-ordered-20261031/`의
`sample16-2024-p001-image-program-analysis.log`,
`sample16-2024-p001-image-pixel-comparison.log`에 있다.

최소 글꼴 환경에서 관련 집중 회귀 32건은 모두 통과했다
(`source-bold-boundary-focused-minfont.log`, 종료 코드 0). 동일 시스템 글꼴의
원 PR 재출력과 새 Native 후보를 정밀 대조하면 sample16 2022의 1·2·17·18·19·38·55쪽은
raw가 같고 10쪽은 이전과 같은 `0.000311681267%p` 하락이 남는다
(`source-bold-boundary-sample16-initial-comparison.log`). 이는 선택한 8쪽의
중간 결과이며 전쪽 또는 WASM 무하락의 근거가 아니다. #8040 등 정상 대조군과
기존 PR 전쪽 검증은 계속한다.

동일 Native 실행 환경에서 #8040 두 문서 8쪽, #7905 6쪽, #8136 원본과
재저장본 4쪽, OSS 대조군 6쪽을 보정 직전 후보와 대조한 결과 raw 값은 모두
변하지 않았다. `source-bold-boundary-controls-comparison.log`와
`source-bold-boundary-focus.log`에 쪽 수와 종료 결과를 기록했다. 이는 이번
굵은 글꼴 보정의 영향에 대한 비교이며 원 PR 대비 기존 미세 하락이 해소됐다는
뜻이 아니다.

굵은 원본 프로그램 보정 후 sample16 2022 전64쪽의 Native 출력과 원 PR을
동일한 시스템 글꼴·동일한 PDF 래스터로 정밀 비교했다. 기존 10쪽 하락 중
5쪽은 복구됐고 7·10·31·37·53쪽의 raw 하락이 남는다. 최저값은 10쪽의
`98.433992103604%`이며 가장 큰 하락은 7쪽의 `0.000354706240%p`다.
각 쪽의 RGB 차이 영역과 렌더 트리의 텍스트 소속을 교차 확인한 결과 남은
차이는 모두 `※` 문자의 영역에 속한다. 7쪽 2곳, 10·31·37쪽 각 1곳,
53쪽 3곳이다. 원 PR과의 정밀 비교는 `source-bold-boundary-full-sample16-comparison.tsv`,
픽셀 영역과 문자 소속은 각각 `source-bold-boundary-five-pages-pixel-analysis.log`,
`source-bold-boundary-five-pages-text-ownership.log`에 기록했다. 기호 슬롯을
사용하는 다른 PR의 계약과 독립 Print 윤곽을 함께 확인하기 전에는 임의의
글꼴 강제 치환이나 점수에 맞춘 경계 조정을 하지 않는다.

남은 8개 `※` 확대 영역에서 같은 위치의 인쇄 래스터와 밝기 차이를 비교했다.
현재 출력은 7개 영역에서 원 PR 출력보다 차이가 작지만, 7쪽 첫 번째 영역에서는
차이가 더 크다. 독립 PDF 문자 추출상 8곳 모두 한컴의 같은 Type3 글꼴 `T8`을
사용하며 주변 본문 글꼴과 구분된다. 이 관측은 현재 선택이 모든 경우에 더
정확하다는 뜻도, 사용자가 요구한 raw 무하락 조건을 통과했다는 뜻도 아니다.
영역별 수치는 `symbol-eight-regions-alignment.log`, PDF 문자·글꼴 식별은
`symbol-eight-regions-pdf-identity.log`다. 전체 PDF 위치에 대응하는 확대 영역에서
정렬 이동도 진단했으나, 픽셀 점수를 맞추기 위한 좌표 보정은 적용하지 않았다.

원 PR과 현재 렌더 트리에서 8개 기호가 속한 문단 및 텍스트 run 위치와 한컴
PDF의 문자 원점을 함께 추출했다(`symbol-eight-regions-source-position.log`).
분류 변경으로 run은 나뉘지만 해당 기호가 속한 줄의 저장 위치는 유지된다.
실제 선택된 글꼴의 래스터 차이와 소수점 이하 원점은 별개로 추적하며,
렌더 트리의 반올림 좌표만으로 최종 글리프 원점이 완전히 같다고 단정하지 않는다.
현재 후보로 2024 HWP/HWPX 각 64쪽과 중첩 표 전17쪽도 동일 기준으로
재출력 중이다. 산출 완료와 원 PR 비교 결과는 별도로 확인한다.

새 후보로 중첩 표 전17쪽을 재출력하고 같은 시스템 글꼴 환경의 원 PR과
PDF 래스터 SHA까지 확인해 정밀 대조했다. 9쪽 한 쪽만
`99.790593495145% → 99.788828217498%`로 하락하며 다른 쪽은 같다
(`source-bold-boundary-nested-comparison.tsv`). 기호 슬롯은 `한양중고딕`
HFT로 선언돼 있지만 실제 선택은 `Malgun Gothic` 대체다. 설치된 원 family의
분할 OTF `SPJGT.otf`에는 `hft3438` 윤곽이 있어도 Unicode `※` cmap이 없고,
그 윤곽은 독립 Print의 같은 문자와 다르다. 반면 독립 Print의 윤곽은 신명조
원본의 확인된 기호 윤곽과 같다(`hanyang-junggothic-symbol-outline-check.log`,
`hanyang-symbol-face-comparison.log`). 따라서 family 이름 또는 동일한 내부
글리프 이름만 근거로 중고딕 OTF의 해당 글리프를 강제할 수 없다. 원본의
실제 fallback 계약과 다른 문자·글꼴의 반례를 더 확인한다.

새 후보의 sample16 2024 HWP 64쪽을 원 PR 실행 파일과 같은 시스템 글꼴로
재출력해 비교했다. 독립 PDF의 각 쪽 래스터 SHA가 같고 raw가 하락한 쪽은
0쪽이며 후보 최저는 51쪽의 `99.904534606205%`다
(`source-bold-boundary-sample16-2024-comparison.tsv`). 이 결과는 동일 환경의
Native 대조이며, 제출 당시 TSV 수치와 fresh WASM의 별도 무하락 검증을
대신하지 않는다. 중첩 표 9쪽의 새 후보 PNG SHA는 직전 후보와 같다.

같은 새 후보의 sample16 2024 HWPX도 독립 PDF의 쪽별 래스터 SHA를
원 PR 실행의 PDF와 대조하고 64쪽 전부 같은 시스템 글꼴로 비교했다.
하락하거나 상승한 쪽은 없고 후보 최저는 51쪽 `99.904424355860%`다
(`source-bold-boundary-sample16-hwpx-comparison.tsv`). 이는 동일 환경의
Native 출력에 대한 결과이며 제출 당시 TSV 및 fresh WASM의 별도 대조를
대신하지 않는다. 세 문서 추가 Native 실행은 모두 종료 코드 0이다
(`source-bold-boundary-additional-full.log`).

기존 PR의 편람 HWPX Native 보존 측정은 완료된 193/383쪽 중 145쪽에서
제출 당시 `100.000000000000%`보다 낮은 `99.988181298875%`를 보였다.
원 PR 실행 파일과 `6be28314c5` 코드 후보를 같은 시스템 전체 글꼴·같은 PDF로
145쪽에 대해 각각 재출력하니 둘 다 `99.988181298875%`였고 PDF 래스터 SHA도
같았다(`handbook-hwpx-p145-same-system-comparison.log`). 제출 기준과 동일한
환경이라고 단정할 수 없으며, 이 결과를 제출 TSV 무하락 통과로 기록하지 않는다.
두 RHWP PNG는 동일하지 않으며 RGB 차이 15픽셀은 글자가 있는
`(182,638)-(330,651)` 영역에 분포한다(`handbook-hwpx-p145-image-diff.log`).
해당 원본은 별도 PR의 보정 범위이고 이번에는 기존 PR 보존 검증으로만 다룬다.

#8040 독립 PDF 네 쪽의 숫자와 주요 원문자를 `pypdf` 텍스트 연산 및 각
글꼴의 `FontDescriptor`로 다시 집계했다. `1`은 각 쪽에서 `/INPILL+Gulim`과
하나 이상의 Type3 글꼴로 함께 나타난다. 원문자 `㉠`~`㉤`는 추출된 사례에서
Gulim이다(`issue8040-independent-pdf-font-census-resolved.log`, 글꼴 리소스
`issue8040-pdf-resource-fonts.log`). 같은 문자라는 이유로 모든 숫자에 Gulim을
선택하거나 글꼴 이름이 깨져 보이는 `/BaseFont`만 근거로 공급 프로그램을
단정하지 않는다. PyMuPDF는 이 PDF의 일부 CID 글꼴에 대해 타입 오류를 출력하므로
그 도구로 작성한 전체 글꼴 집계는 근거에서 제외했다.

#8040 2쪽의 미대응 픽셀을 독립 PDF의 `<1 ` 문자 원점과 실제 Native 트리에
좌표로 연결했다. PDF는 원점 x=`569.28`, 기준선 y=`621.60` CSSpx,
글자 크기 `15.36` CSSpx의 `/INPILL+Gulim`이다. Native의 대응 run은 bbox
`(569.3,609.3,31.0,15.3)`이고, 바로 그 run의 숫자 영역 `(580,621)`에
2px 이웃에 대응하지 않는 잉크가 있다. 추적상 해당 숫자는 section 1의
paragraph 17, nested `[0,5,0]`, 글자 모양 36의 영문 슬롯이다. 원본은
대체 이름이 없는 `한양신명조` HFT, 선택된 Native 표시는 `HY Sinmyeongjo`다
(`issue8040-p002-lt-one-exact-join.log`). 이 좌표 대조는 같은 입력의 같은 글자가
다른 프로그램으로 출력된다는 근거다. 다른 HFT·글자 모양까지 글꼴을 일괄
변경할 근거나 해당 페이지의 무하락 복구 결과로 확대하지 않는다.

2024 HWPX 1쪽 그림의 원본 GIF(133×40, 원본 256색)를 독립적으로 읽어
현재 RGB555·JPEG 인코딩 계약을 재실행했다. 후보 SVG의 해당 JPEG entropy와
독립 재현 JPEG는 바이트 단위로 일치하고, 원 PR JPEG는 일치하지 않는다.
두 출력의 JPEG 양자화표, 4:2:0 샘플링, RGB555 확장 표식 및 인쇄 프레임
메타데이터는 같다(`sample16-gif-jpeg-diagnostic-comparison.log`,
`sample16-image-jpeg-marker-detail.log`). 4×4 디더 위상을 모두 바꾼 별도
16개 진단도 원 PR entropy와 일치하지 않았다
(`sample16-gif-jpeg-phase-comparison.log`). 이는 원 PR이 다른 중간 RGB 입력이나
전달 경로를 거쳤을 가능성을 보여 주지만 어느 단계에서 달라졌는지는 확정하지
않는다. 진단은 제품 코드를 바꾸지 않았으며 raw 무하락을 대신하지 않는다.

JPEG 차이의 원본 입력과 압축 설정을 고정 실행 파일에서 직접 관측했다.
원 PR로 사용한 `base-rhwp`의 `encode_print_jpeg`에 전달된 GIF 바이트는
HWPX `BinData/image1.gif`와 SHA-256이 같고, quality 70·RGB555 확장 사용도
확인했다(`base-gif-encoder-observe.log`). JPEG encoder에 전달된 133×40 RGB
바이트의 SHA-256은 현재 코드의 독립 재현과 둘 다
`2cd9f142253b147987d6796d5b3f60201318a275efe9e6559a2b9ea74a0a8b2c`다
(`base-gif-rgb-observe.log`). 양자화표·Huffman 표와 scan 헤더도 일치하지만
entropy는 다르다(`sample16-gif-jpeg-structure-comparison.log`). 두 실행 파일의
JPEG 하단 평균 함수 기계 코드 크기는 다르지만 그 차이만으로 빌드 오류나
원본 버전을 단정할 수 없다. #8389 직전 인코더 소스를 별도 컴파일해 같은
RGB를 인코딩한 결과는 원 PR 실행 파일이 아니라 후보와 같았다
(`pre8389-jpeg-encoder-reproduction.log`). 선택 표본 `Nearest`도 어느 쪽과도
일치하지 않았다(`sample16-gif-jpeg-nearest-comparison.log`).

원 PR 재출력에 사용한 실행 파일과 고정 커밋 소스의 대응을 확인하기 위해
깨끗한 `99a176b7cb` 작업트리와 고정 `target/pr-review`, 빌드 잡 8개로 실행
파일을 다시 빌드하고 있다(`base-99a-fresh-build.log`). 기존 동일 환경
비교는 이 실행 파일의 JPEG 출력과 다시 대조하기 전까지 원 PR의 최종 기준으로
확정하지 않는다. 기존 전쪽 실행은 고정된 이전 실행 파일의 결과로 보존한다.

### 회귀 파일 첫머리 재점검

`upstream/devel...HEAD`에서 추가·변경한 `tests/cases/*.rs` 55개를 확인했다.
모듈 주석이 없는 파일은 없었으나 13개에는 첫 줄의 관계형·고정형 구분과
주석 갱신 시각이 없었다(`current-changed-regression-header-audit.log`). 각
파일의 실제 assertion을 `untyped-regression-assertion-audit.log`로 확인해
유형을 표시하고 `git log -1`의 직전 파일 커밋 SHA·시간을 기록했다. 페이지 수와
독립 좌표·고정 출력은 고정형, 서로 다른 입력·요소의 내용·위치 비교는 관계형이며,
둘이 섞인 파일은 둘 다 표시했다. 코드와 assertion은 바꾸지 않았다.

이 정리는 시각 근거까지 완성했다는 뜻이 아니다. 13개 파일에 포함된 원본별
Native·fresh WASM 수치와 전체 쪽 최저·쪽수 일치 근거를 확인하고, 고정 좌표가
있는 페이지는 양쪽 98% 이상 및 독립 Print 확대 판독 조건을 별도로 검증한다.
자료가 없는 원본을 추정 값으로 채우거나 주석의 유형 표시만으로 기존 테스트가
정책을 통과한 것으로 판정하지 않는다. 원본 경로 목록은
`untyped-regression-fixture-inventory.log`다.

#8079 검사에서 HFT 반례의 입력은 HWP인데 주석은 HWPX Print PDF를 가리켰고,
기여자의 전쪽 TSV 설명은 같은 HWP의 Print PDF를 가리켰다. 두 PDF는 SHA-256은
다르지만 PDFium 96dpi RGB 래스터의 전6쪽이 픽셀 단위로 같다
(`issue8079-issue5169-hwp-vs-hwpx-pdf-comparison.log`). 입력과 같은 HWP의
Print PDF로 주석을 정정했으며 assertion이나 검사 동작은 바꾸지 않았다. 이
동등성 확인은 최종 후보의 Native·fresh WASM 전쪽 검증을 대신하지 않는다.

회귀 주석 근거 점검 중 #8282의 HWPX 4개와 해당 한컴 2020 Print PDF를
현재 `6be28314c5` 코드의 고정 Native 실행 파일로 전쪽 재출력했다. 네 파일은
각각 1쪽이고 모두 raw `100.00%`다(`regression-8282-native-current.log`,
`regression-8282-native-current/`의 문서별 `silhouette.tsv`). 기여자 제출 TSV의
Native·WASM도 각각 1쪽 `100.00%`임을 확인했다
(`issue8282-original-tsv-evidence-inventory.log`). 제출 WASM은 현재 코드의 fresh
WASM이 아니므로 최종 양쪽 98% 및 확대 판독 조건을 통과한 것으로 아직 기록하지 않는다.

재빌드된 `99a176b7cb` 기준과 현재 코드의 같은 시스템 글꼴·같은 PDF 대조를
시작했다. #8040 두 원본의 전8쪽은 각각 1·3·4쪽이 개선됐으나 2쪽은 각각
`0.000000600300%p`, `0.000000590802%p` 하락했다
(`fresh-base-issue8040-comparison.tsv`). 중첩 표 전17쪽에서는 9쪽만
`0.001765277647%p` 하락했다(`fresh-base-nested-comparison.tsv`). 이 수치들은
이전 실행 파일의 잠정 대조와 독립적으로 재빌드한 기준에서 재현됐다.
편람 HWPX 145쪽은 두 출력의 raw가 `99.988181298875%`로 같고 PDF 래스터도
동일하지만 PNG는 다르다(`fresh-base-handbook-p145-comparison.log`). 세 비교는
시스템 전체 글꼴 환경의 재빌드 기준 대비 결과이며 제출 당시 TSV와 현재 후보의
직접 무하락 조건을 대체하지 않는다.

재빌드 기준 `545abe7699e2aa8f527d6868e390b1dff76bade449823e05773c0443d10204e3`과
현재 코드 후보를 같은 시스템 글꼴·같은 한컴 PDF로 비교한 결과, sample16 2022
전64쪽 중 7·10·31·37·53쪽의 미세 하락이 재현됐다
(`fresh-base-sample16-2022-comparison.tsv`). 이전 실행 파일을 사용해 확인한
기호 영역과 같은 범위다. 2024 HWP·HWPX 기준 재측정은 계속 진행한다.

`6be28314c5` 이후 WASM 관련 소스에 변경이 없음을 확인하고, 현재 head의
fresh WASM을 고정 `target/pr-review`, 빌드 잡 8개로 시작했다
(`source-bold-boundary-fresh-wasm-provenance.log`,
`source-bold-boundary-fresh-wasm-build.log`). 종료 코드·패키지 해시·Native와
같은 원본의 전쪽 WASM 결과를 확인하기 전에는 이를 통과로 기록하지 않는다.
#8079·#7092 회귀에 쓰인 나머지 입력의 현재 Native 전쪽도
`regression-8079-7092-native-current.log`에서 묶어 검증 중이다.

재빌드된 #8397 실행 파일과 현재 코드의 sample16 2024 HWP 전64쪽을
동일 시스템 글꼴·동일 PDF로 대조하면 raw 하락은 없고, PNG가 다른 쪽은
10·53·60쪽이다(`fresh-base-sample16-2024-comparison.tsv`). 이전 실행 파일과
달랐던 1쪽 GIF JPEG는 재빌드 기준과 후보에서 같아졌다. 나머지 세 쪽의 RGB
차이는 각각 5·8·1픽셀이며 모두 `※` 뒤 한글 run에 있다. 기존 기준은 `※`와
한글이 같은 run이고 후보는 기호 슬롯과 한글 슬롯으로 나뉜다. 해당 영역의
PDF 대비 RGB 절대 차이는 후보가 각각 소폭 크다
(`fresh-base-sample16-2024-pixel-differences.log`). 점수가 같다는 이유로
픽셀이 같거나 미세 차이가 해소됐다고 판정하지 않으며, run 분할 후 실제 그리기
원점을 더 정밀하게 확인한다.

재빌드한 같은 기준의 sample16 HWPX 전 64쪽은 raw 하락이 없었고 PNG가 다른
쪽은 세 쪽이다(`fresh-base-sample16-hwpx-comparison.tsv`). 출력 좌표 추적에서
글줄 분할 전후 같은 위치를 두 `f32` 값의 합으로 계산해 서로 다른 좌표로
변환하는 경우를 확인했다. 글꼴이 필요 없는 독립 계산 재현은 기존 계산에서 실패하고
합산 후 변환 시 통과했다(`native-run-origin-old-fails.log`,
`native-run-origin-rounding-contract.log`). 현재 Native Skia의 글자·배경·선·표식
원점을 같은 순서로 변환하는 보정을 적용했으며, 최소 글꼴 집중 검사와 실제 출력
대조가 끝나기 전이므로 시각 회귀 해결로 판정하지 않는다.

검증 중 `upstream/devel`에 #8417이 병합됐다(`da39c53661`). Windows의 CFB
스트림 경로 구분자를 정규화하는 테스트 보정이며 현재 브랜치의 기존 변경과는
파일이 겹치지 않는다. 실행 중인 컴파일을 마친 뒤 새 base로 리베이스하고 해당
기능과 현재 작업을 함께 재검증한다.

기존 고정 실행 파일 `rhwp-localized-hft-family-candidate`의 시스템 전체 글꼴
Native 측정은 17개 대상 전쪽을 완료했다(`current-preserve-native.log`,
`current-preserve-native-status.json`). 제출된 기존 TSV 2,153행과의 비교에서
73행 하락, 기준 정밀도 부족 30행을 확인했다
(`current-preserve-native-complete-comparison.log`,
`current-preserve-native-partial-comparison.tsv`). 이는 새 좌표 보정 전 실행
파일의 결과이며 현 작업 트리의 측정으로 취급하지 않는다. 정확한 현재 비교에는
새 실행 파일과 같은 기준 PDF 및 시스템 글꼴 환경이 필요하다.

#8389 원 PR 본문을 다시 조회해 sample16 2022의 HWP와 독립 Print PDF SHA-256이
현재 파일의 `fec3671c9a83cb8388cbdaab38754cb17a8da54d0314ca7e3f56860e871506fe`,
`c4e05635726869d737f1640db4e4f29c868df9e1c31e350b0b06e115fd210b33`과 각각
일치함을 확인했다. 기준 파일 변경을 현재 차이의 원인으로 간주하지 않는다.
원 PR 본문 조회본은 `output/pr-review/planet6897-ordered-20261031/pr8389-remote-body.md`에
보관했다.

fresh WASM 후보 `cee8e58be7191a14febfb73e8b79b2ad59946ac1c7f13332d46ae92157aef55f`
의 #8136 두 쪽은 표시값 각각 99.86%, 98.37%다. 기여자 제출 WASM의 2쪽
99.45628%보다 낮지만, 당시 글꼴 공급 상태가 같다는 근거가 아직 없으므로 이
수치만으로 코드 회귀의 원인을 확정하지 않는다. 같은 글꼴 환경의 기준 출력과
확대 판독을 추가 확인한다. 현재 두 쪽 TSV는
`output/pr-review/planet6897-ordered-20261031/regression-fresh-wasm-batch/issue8136/issue8136/silhouette.tsv`,
2쪽 review와 overlay는 `issue8136-fresh-wasm-review/issue8136/`에 있다.

Native 원점 정밀도 변경 후보 `42f64c3df533a32fb6c66ee3a8ac7c7539ce86df4b9e71d5a5d929faaa0b6008`를
같은 시스템 글꼴과 독립 PDF로 20쪽 대조했다. 재빌드 원 기준 대비 8쪽이 하락했고,
그중 sample16 2022의 53쪽은 보정 직전 후보 `99.513290301888%`에서
`99.512492382693%`로 추가 하락했다. 다른 주요 하락도 그대로 남았다.
`native-run-position-focus-comparison.tsv`와 원본·PDF hash가 같은 전후
`silhouette-metrics/p000053.json`을 대조했다. 계산상 run 분할에 따른 반올림
차이는 재현되지만 이 보정은 요구된 전쪽 무하락을 충족하지 못해 코드와 해당
합성 단위 검사를 되돌렸다. 표시 개선·정밀 무하락을 동시에 만족하는 공통 원인을
계속 추적한다.

#8136의 기여자 제출 점수와 후보의 차이를 확인하기 위해 #8397 통합 기준
`99a176b7cb34fde54454c03433baa1382b2de9fa`에서 빌드한 WASM과 현재 WASM을
같은 시스템 전체 글꼴·같은 독립 PDF로 재측정했다. PDF의 페이지 PNG 해시가 같은
것을 확인하고 정수 미대응·합집합으로 계산한 raw는 1쪽
`97.686961933409% → 99.859185441941%`, 2쪽
`98.362147406733% → 98.365864729914%`다. 같은 조건의 기준 대비 두 쪽은
모두 개선이며, 이 결과가 기여자 측정 당시 글꼴 환경까지 같다는 뜻은 아니다.
정밀 표와 원본 측정 로그는 `output/pr-review/planet6897-ordered-20261031/`의
`issue8136-base-current-system-wasm-exact.tsv`,
`issue8136-original-base-system-wasm.log`다.

사용자는 2026-11-01에 페이지별 무하락 판정을 Visual Sweep 표시값으로
변경했다. 원본 실루엣 수치를 소수 셋째 자리에서 반올림해 TSV와 같이 소수 둘째
자리로 비교하며, 같은 표시값은 통과하고 낮은 표시값은 하락이다. 페이지별로
판정하고 다른 페이지의 상승으로 하락을 상쇄하지 않는다. 이전 정밀값은 원인
추적용으로 보존한다. 이 기준에서 #8040 두 2쪽은 각각 기존·현재
`100.00% → 100.00%`로 통과다. 전체 Native·WASM은 현재 실행 중이며 중간
대조표는 `current-full-system-native-display-comparison.tsv`,
`current-full-system-wasm-display-comparison.tsv`다.

sample16 기존 100.00%의 원본 글꼴 공급 설정을 보존 호스트에서 확인했다.
`raw100-proof.json`에 기록된 설정 파일의 SHA-256
`9d42f0162206f4b829c603092c8db445ff5979d1d79f4d9a6b0155918459b43f`과
가져온 파일의 해시가 일치하며, 해당 설정은 `휴먼명조`를 `Gulim`으로 명시
대체한다(`original-hft-programs-2020-pua-reference.json`). 당시 명시 공급한
217개 글꼴 프로그램의 해시는 현재 시스템 글꼴 목록에서 모두 확인했다
(`issue8160-original-font-system-hash-inventory.tsv`). 현재 실제 글꼴 추적은
같은 문서의 `휴먼명조` 요청에서 시스템 `Human Myeongjo`를 선택한다
(`sample16-2022-p10-face-source-trace.json`). 설정 차이가 확인됐지만 이것만으로
사용자의 표시값 무하락 조건을 면제하지 않으며, 두 출력의 실제 차이를 계속 확인한다.

#8282 기존 기능의 네 HWPX 입력과 독립 한컴 2020 Print는 Native·fresh WASM
각각 1쪽 전부 raw 100.00%다. 8개 overlay에서 어울림 글상자·그림, 글자처럼
배치된 그림, 앞뒤 줄과 후속 문단을 직접 확인했다. 원본·PDF 해시, 전체 TSV와
대표 review·overlay를 `mydocs/pr/assets/issue_8282/integration-planet6897/`에
기록했고 두 경로의 산출 PNG 해시도 각 review 생성본과 같음을 확인했다. 해당
회귀 파일에는 측정 근거 주석만 추가했으며 assertion이나 제품 코드는 변경하지 않았다.

사용자는 시스템 글꼴 전체를 제공하면서 기존 기준의 대체 설정도 해당 검증에
적용하도록 지정했다. 기존 설정 `original-hft-programs-2020-pua.json`의
SHA-256을 정본과 확인하고 시스템 글꼴 자동 탐색을 유지해 재측정했다. sample16
2022 HWP는 Native 전 64쪽이 `100.00%`로 기존 표시값과 같으며 90% 미만은 없다
(`sample16-2022-original-font-env-native/sample16-hwp2022/silhouette.tsv`,
종료 코드 0). WASM은 같은 조건으로 진행 중이다. 2024 HWP·HWPX에는 정본
`original-hft-programs-2024-pua.json`(SHA-256
`53a7cff101203c61080a59769730f6f7050e5af22870b8a9d233bff7a95ec64c`)을
적용해 두 경로를 재측정 중이다. 이 설정은 검증에만 사용하며 제품의 자동 글꼴
선택을 변경하지 않는다. 기존 두 PR의 동일 문서·쪽 중복은 표시값이 더 높은
기준을 선택해 한 번 비교한다(`authorized-font-env-*-display-comparison.tsv`).

리베이스로 포함한 #8417의 CFB 경로 정규화 두 곳을 현재 트리에서 확인했다.
최소 글꼴 Linux 환경의 `issue_8225_group_child_ctrl_data` 집중 회귀는 재실행에서
3건 통과·실패 0건, 종료 코드 0이다(`rebase-8417-focused-minfont-retry.log`,
`rebase-8417-focused-minfont-retry.exit`). 원 PR의 Windows 검증 기록과 현재
Linux 재실행을 구분한다.

#8079 굵은 글자 폭 검사의 OSS HWP·HWPX와 HFT 대조군 issue5169 HWP는 각 6쪽
전쪽에서 Native 최저 99.92% 이상, fresh WASM 최저 99.99% 이상이었다. 해당
검사의 1쪽은 Native·WASM 모두 98% 이상이며, 동일 쪽의 Print·review·overlay에서
숫자와 제목의 글줄을 판독했다. 전쪽 TSV, SHA와 대표 이미지는
`mydocs/pr/assets/issue8079_bold_regular_advance/integration-planet6897/`에
보존했다. 회귀 파일은 측정 근거 주석만 보강했다.

#8136 합성 슬롯 입력의 두 쪽은 같은 한컴 2024 Print에 대해 Native
`99.90%·98.96%`, fresh WASM `99.86%·98.37%`다. 두 출력의 각 쪽을 직접
대조했으며 전체 TSV와 review·overlay는
`mydocs/pr/assets/issue_8136/integration-planet6897/`에 보존했다. 회귀 파일의
측정 주석만 갱신했고 검사 로직은 변경하지 않았다.

곡선 따옴표 원본의 기존 한컴 2022 PDF는 버전 메타데이터는 있지만 Print 경로를
입증하는 기록을 확인하지 못했다. 입력의 저장 제품은 `hancom-office-2010`으로
확인했고, 독립 한컴 2020 MCP Print를 새로 요청해 결과와 전쪽 재측정을 기다린다.

곡선 따옴표 원본의 저장 제품은 `hancom-office-2010`이며, 독립 기준을 한컴 2020
MCP로 다시 인쇄했다. job `43cf4494-584e-49b2-9b5a-05dd90e9d1ae`, 한컴
`11.0.0.9136`, 인쇄 방식 0과 `hancom2020_pdf_driver_one_up`, 7쪽, PDF SHA-256
`a90c88f4d42d06b91e49edb0da5172ee64d8bc3e13bcbfb406e0bfbd5b21fbd6`을 확인했다.
새 Print의 전쪽 최저는 Native 96.96%, fresh WASM 97.73%다. 회귀가 확인하는
3·4쪽은 Native 98.21%·98.35%, WASM 98.26%·98.29%이며 기존 review·overlay에
쓰인 RHWP와 PDF PNG가 새 기준과 각각 같음을 해시로 확인했다. 기준 PDF·전쪽 TSV·
인쇄 출처와 같은 쪽 이미지는 `pdf/issue7092/` 및
`mydocs/pr/assets/issue_7092/integration-planet6897/`에 보존한다. 회귀는 주석만
갱신하고 검사 로직은 변경하지 않았다.

같은 #8136 원본을 사용하는 #8315 기호 전진폭 검사에도 기존 두 쪽 Native·fresh
WASM 측정과 Print 증거를 연결했다. 검사 함수나 제품 코드는 변경하지 않았다.


### 2026-11-01 JPEG 채움 회귀와 빌드 캐시

리베이스 후 최소 글꼴 전체 회귀의 #8153 JPEG 채움 HWP·HWPX 검사 2건이
실패해 원 해결 PR #8389의 독립 Print 자원을 확인했다. 인코딩 직전 RGB는 이전·현재
실행 파일이 같고 SHA-256은 `eae737a4ae28a14fc121d0c797b22856990671dc6e476fdf5e4cd9b5a2f864d2`,
품질은 38이다. 실패한 실행 파일의 압축 결과는 966×322 독립 Print의 마지막 세 줄에서
42픽셀이 다르다. 현재 소스를 별도로 빌드한 인코더는 같은 RGB의 한컴 Print 및
독립 libjpeg 결과와 일치한다. 합성 입력 17×2, 17×3, 16×16, 19×17, 33×18,
33×31, 33×34, 64×322도 libjpeg와 일치했다. 이전 패딩 방식으로 바꾸면 17×2와
33×18 등에서 도리어 차이가 생기므로 구현을 변경하지 않는다. 원인은 이전 인코더
산출물이 Cargo 캐시에 남아 링크된 것으로 추적하며, 인코더 소스의 변경 시각을 갱신해
공용 target에서 실제 재컴파일하고 전체 회귀를 다시 검증한다. 바이트·합성 대조 자료는
`output/pr-review/planet6897-ordered-20261031/jpeg-*`에 있다.

재컴파일 후 #8153 HWP·HWPX 집중 검사 2건은 모두 통과했다. 로그는
`output/pr-review/planet6897-ordered-20261031/jpeg-clean-cache-focused.log`다.

### 2026-11-01 #8079 대조 문서 46쪽 후속 원점

독립 한컴 2024 Print 46쪽과 동일한 HWPX의 Native overlay에서 첫 표와 본문을
대조했다. 글이 있는 호스트 문단의 저장 줄 13920 HU, 개체 오프셋 2334 HU,
위·아래 바깥 여백 각 140 HU와 표 높이 28437 HU가 닫는 끝은 후속 문단의
저장 첫 줄 44971 HU와 같다. 실제 조판 커서도 그 위치에 도달했지만 후속 문단의
앞 간격 500 HU를 다시 더한다. 이후 본문과 다음 표가 함께 내려간다.

측정의 `typeset/paragraph/flow.rs`와 paint의 `layout.rs`가 같은 저장 줄·개체
종료 관계를 조회해 앞 간격의 소유를 정한다. 호스트 문장·표 항목은 기존대로
유지하며, 편집되거나 저장 끝이 닫히지 않는 경우 및 여러 개체의 소유가 모호한 경우는
기존 흐름을 유지한다. 보정 전후 26·28·46쪽과 저장 경계가 다른 대조군을 직접
비교하고 전쪽 Native/fresh WASM 및 회귀 검사는 이후에 수행한다.

첫 후보는 후속 글은 제자리로 옮겼지만 46쪽 raw가 76.446%로 남았다. 실행 추적에서
측정은 저장 표 끝에서 앞 간격을 돌려준 뒤 표 배제 처리로 다시 표 끝까지 전진했고,
최종 글자처럼 표의 측정 커서는 저장 689.213px보다 앞 간격만큼 컸다. 따라서 배제
처리 이후에도 동일한 저장 원점 계약을 다시 소비하도록 연결한다. 중간 후보의 출력은
`output/pr-review/planet6897-ordered-20261031/k113-visible-host-review/`에 보존한다.

배제 처리 뒤 같은 경계를 소비한 Native 후보의 46쪽 raw는 75.14032%에서
99.74005%로 개선됐다. 7·8·26·28쪽은 각각 99.70643%·98.16573%·99.84160%·
95.06099%이며, 46쪽에서는 호스트 글·두 표·후속 문단의 표시와 순서를 확인했다.
[46쪽 review](../../output/pr-review/planet6897-ordered-20261031/k113-shared-origin-review/k113-shared-origin/review/review_046.png),
[overlay](../../output/pr-review/planet6897-ordered-20261031/k113-shared-origin-review/k113-shared-origin/overlay/overlay_046.png).
이 결과는 선택 쪽 Native이며 전쪽·fresh WASM과 회귀 검증은 이후에 수행한다.

### 2026-11-03 #4895 저장 왕복 검사 전제 보정

현재 제품 코드 `d56930dda7`의 최소 글꼴 전체 회귀에서 12,377건 중
12,375건이 통과하고 `issue_4895_soft_hyphen_literal` 2건만 실패했다.
검사 원형은 [통합 PR #4936](https://github.com/edwardkim/rhwp/pull/4936)의
`82f28ae864`에서 들어왔고, [PR #8026](https://github.com/edwardkim/rhwp/pull/8026)의
`7346a928ee`는 제어 표기와 리터럴 표기를 분리했다. 실패한 검사는 저장 원본
U+00AD의 존재를 화면 표시용 `extract_page_text_native`로 세고 있었다.
현재 `export-text`는 표시 문자열을 사용하므로 두 `hcar-001` 입력에서 0개로
나오지만, 파서 덤프에서 각 입력의 표 셀 문단에 원본 U+00AD가 4개씩 있다.

기존 저장 왕복 검사의 자료 선택만 원본 IR의 중첩 표 문단까지 순회하도록
보정했다. 최상위 문단만 세는 첫 후보는 같은 2건이 다시 실패했고, 중첩 표까지
포함한 최종 후보는 동일한 최소 글꼴 환경에서 2건 모두 통과했다.
전체 실패 로그는 `output/pr-review/planet6897-ordered-20261031/logs/book-validation/final-full-minfont-nextest.log`,
최종 집중 재실행 로그는 같은 위치의 `final-soft-hyphen-minfont-nextest-v2.log`다.
제품 코드와 저장 기대값은 바꾸지 않았다.
