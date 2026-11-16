# Dependabot 16개 PR 통합 검토 — 원인과 검증 범위

현재: 최소 글꼴 Rust 필수15단계·전체13,617건, Native/fresh WASM 각각18종전쪽1,771쪽,
제품3종 build·Studio/Chrome 실제 소비·WASM 일반/암호화 반복 저장을 완료했습니다.
정본 보호는 양쪽 모두 하락·누락0이며 base/head 전쪽 원본 PNG도 같습니다.
추가 공개8조건·명시적편람383쪽·실제 renderer30캡처·Firefox 편집저장을 완료했습니다.
실제 소비 중 발견한 기존 CanvasKit 빈 일반 선 패턴과 검증 브라우저 종료를 보정했고,
최종 Studio 전체2805PASS·0FAIL·2SKIP 및 강제 종료 없는 실제 backend 검증을 완료했습니다.
4개 수식 backend 비교는 report-only pixel parity 차이이며 runtime hard gate0과 구분합니다.
PR 생성·CI·병합은 아직 하지 않았습니다.
아래 시간별 실행 기록은 당시 상태이며, 완료 범위는 각 최종 JSON 증거를 따릅니다.

2026-11-16, `review/integration-20261116-dependabot`에서 최신 고정 devel
`6c1d134d42213ce0fb5f66b5ad2bf343b7b64fe0` 위로 원 작성자와 원 SHA를 보존해
16개 PR을 누적 체리픽했습니다. [원 PR·SHA·파일 목록](source-prs.json)을 따릅니다.
원 PR 각각에 `jangster77` reviewer를 지정한 뒤 API에서 확인했습니다.
현재 계정의 열린 self PR은 없었고 저장소 작업 트리는 시작 시 clean이었습니다.

## 원 PR 주장과 공통 소비

- #9110·#9115·#9117: Studio·Chrome·Firefox의 Vite8.3.8과 lockfile 갱신입니다.
  package 설치·TypeScript·기존 단위 검사·제품 및 확장 build·실제 browser 경로를 확인합니다.
- #9111·#9113: Puppeteer25.17.0의 기존 자동화 API와 Chrome/확장 검증 경로를 확인합니다.
- #9112: Anthropic SDK1.8.0의 기존 transport·메시지 변환·스트림 테스트를 확인합니다.
  실제 유료 API 호출이나 비밀 키를 테스트 기록에 넣지 않습니다.
- #9114·#9118·#9120·#9122·#9124: ureq·flate2·zip·crc32fast·aes의 원본 입출력·
  네트워크 mock·압축/해제·암호화 fixture와 전체 회귀를 확인합니다.
- #9116·#9121: read-fonts0.47·skera0.10의 글꼴 읽기·부분집합 생성과 Native/fresh WASM
  출력 소비를 확인합니다. #9125의 Skia0.157은 PNG/PDF의 API·그리기 변경 가능성이 있으므로
  Native Skia 범위, 원본 출력 및 정본 보호 문서·편람을 확인합니다.
- #9119·#9123: full SHA로 고정한 rust-cache·install-action만 갱신합니다.
  trigger·권한·cache key/save-if·nextest tool 선택을 바꾸지 않았습니다.
  O3 action 의존성 변경으로 YAML/actionlint·관련 workflow 정책 검사·실제 CI를 확인합니다.

## 텍스트 충돌과 lockfile 정합화

[충돌 기록](conflict-resolutions.json)의 Cargo.toml 인접 줄 두 건은 원 PR들의 목표를 모두
유지했습니다: read-fonts0.47·skera0.10·skia-safe0.157.0입니다. 기능을 선택적으로 버리지 않았습니다.
첫 `--locked` Native 빌드는 lockfile 갱신 필요로 컴파일 전에 exit101을 반환했습니다.
명시적인 의존성 갱신 작업 범위에서 `cargo update --offline -p skera --precise 0.10.0`을 실행했습니다.
도달하지 않는 read-fonts0.46 항목을 제거했습니다. 다만 이 재해결이 bindgen의 syn 선택도
3.0.3에서2.0.119로 바꾸어 prettyplease0.3.0의 syn3 File과 불일치하는 실제 컴파일 실패를
만들었습니다. bindgen의 원래 syn3 간선을 복원해 두 라이브러리의 AST 타입을 맞춥니다.
다른 패키지 버전은 추가로 올리지 않았습니다. 이 정합화 중 오류도 같은 검토 범위에서 수정합니다. 이 실패를 제품 회귀나 빌드 성공으로 기록하지 않습니다.
이후 검증은 다시 `--locked`를 사용합니다.

## 실행 순서와 남은 상태

현재는 16개 체리픽과 lockfile 정합화 단계입니다. Native 빌드부터 수행하고 API 비호환이 있으면
원 업데이트와 공용 호출 경로를 대조해 필요한 메인터너 호환 보정만 추가합니다.
전체 Rust 회귀는 최소 글꼴 Mac sandbox, Visual Sweep은 원본 글꼴을 공급한 일반 Mac에서
수행합니다. 순차 Cargo·공용 target/pr-review·jobs8·threads8·총 capture8 이하를 유지합니다.

검증이 끝나면 개별 원 PR review와 주장별 증거를 작성합니다. 통합 PR의 CI action 변경 때문에
옵션2를 적용하여 Full CI·정상 병합 뒤 별도 기록 PR에서 통합 PR review·오늘할일을 처리합니다.
통합 병합과 devel 반영을 확인하기 전에는 원 PR을 close하지 않습니다.
원 PR별 개별 판정·원 SHA·실제 검증을 댓글로 남긴 뒤 supersede close하고 원 bot branch는 보존합니다.
사용자가 요청한 비공개 글꼴 ZIP과 다른 작업의 자료를 보존합니다.

## 1차 실제 검증 완료

- 새 Skia0.157 Native CLI `--locked --profile release-test --features native-skia` 빌드를
 5분8초에 완료했습니다. 제품 API 호환 보정은 현재까지 필요하지 않았습니다.
- Studio·Chrome·Firefox npm ci, Studio TypeScript와 전체 단위2,798PASS·0FAIL·2SKIP을
 완료했습니다. [결과](npm-validation.json)와 [실제 해석 버전](resolved-versions.json)을 보존합니다.
- actionlint 전체 호출은 기존 advisory 두 곳의 SC2016 info를 보고했습니다. 같은 base와 후보의
 두 스크립트가 action SHA 외에는 byte-identical이며, 단일 인용 echo가 Markdown backtick을
 그대로 출력하는 의도임을 확인했습니다. 제품 결함으로 수정하거나 허용치를 바꾸지 않았습니다.
 Shellcheck를 제외한 actionlint 구문 검사와 Node CI 계약·Python workflow250건은 통과했습니다.
 [명령·결과](workflow-validation.json), [기존 진단 분석](actionlint-existing-shellcheck.json)을 구분합니다.
- 새 sandbox는 CoreText1890face 중 OS528face만 노출하며 비공개183개 프로그램 읽기를 모두
 거부했습니다. 기존 라이선스 확인 vendor bundled 글꼴은 명시적으로 허용했습니다.
 최초 프로필이 이 두 공개 DejaVu 파일까지 막아 발생한 컴파일 실패는 환경 구성 오류로 보존하고,
 [수정된 격리 조건](minimal-font-qualification.json)에서 필수 게이트를 다시 실행 중입니다.

## 글꼴 공급 판단과 검증 진행 기록

과거 nested 생산 기록의 특정 `gulim.ttc` 원본 hash는 확보하지 못했으므로 동일 파일이라고
기록하지 않습니다. 사용자가 추가 질문 없이 판단하도록 지정하여, 이미 확보한 보호 글꼴 묶음의
파일 hash·공급 순서를 고정하고 현행 정본 baseline과 같은 입력·PDF의 전쪽 출력을 대조했습니다.
[판단 근거](font-supply-decision.json)의 base nested 17쪽은 최저 99.86%이며 공식 보호 표시값보다
하락한 쪽은 없습니다. 이는 해당 공급 조건의 실제 재현 결과이며, 후보와 fresh WASM의 완료를
대신하지 않습니다. 원본 동일성 추정·점수 완화·폰트 예외 승인은 적용하지 않았습니다.

2026-11-16 13시대 KST, 최소 글꼴 sandbox의 최적화 release 빌드는 실제 통과했고 release lib
검사가 실행 중입니다. 일반 Mac Native 캡처는 RAM·swap 경합을 줄이기 위해 기존 프로세스를
보존한 채 대기시키고 Rust 게이트 종료 후 자동 재개하도록 했습니다. 중복 Cargo는 실행하지
않았습니다. 대기 시간이 포함된 캡처 wall time을 성능 측정값으로 사용하지 않습니다.
개별 원 PR 16개의 검토 초안은 임시 output에서 준비하며 최종 증거·CI 확인 전 승인으로 기록하지
않습니다. fresh WASM·전쪽 보호 비교·제품 소비 경로도 선행 게이트 통과 후 순차 실행하도록
준비했으며 아직 완료로 집계하지 않습니다.

## 공개 fixture 차단 보정과 체리픽 내용 확인

전체 nextest의 첫 컴파일 시 `ttfs/redistributable/happiness-sans/HappinessSansVF.ttf`를
격리 프로필이 차단했습니다. 이 파일은 기존 저장소가 공식 원본·라이선스와 함께 추적하는
variable font fixture이며 테스트의 `include_bytes!` 입력입니다. 해당 디렉터리만 허용하고
OS528face·비공개183개 파일 읽기 차단을 다시 확인했습니다. 제품·회귀 기대값은 변경하지
않았습니다. 이 실행은 테스트 본문 전 컴파일 실패로 기록하고, 성공한 release build/lib는
같은 제품 입력·원래 프로필 snapshot으로 보존한 채 전체 nextest부터 다시 실행합니다.

[체리픽 payload 감사](cherry-pick-provenance-audit.json)에서 원 PR 16개 중 15개는 추가·삭제
내용이 그대로입니다. 12개는 patch-id도 동일하며 나머지는 인접 갱신의 문맥 차이입니다.
#9121은 #9116이 이미 추가한 동일 read-fonts0.47 package stanza를 중복 추가하지 않은 차이만
있습니다. skera·skrifa·write-fonts의 갱신과 최종 read-fonts0.47 노드는 보존했습니다.

## CI action의 원본 및 정책 경계 확인

[원본 action과 YAML 감사](action-source-audit.json)에서 rust-cache의 정확한 이전/새 SHA를
비교해 macOS purge 호출 수정과 dist 재생성을 확인했습니다. install-action2.90.3은 다른 tool의
manifest와 release 문서 변경이며 nextest manifest·설치 runtime을 바꾸지 않습니다.
저장소 workflow8개는 승인된 두 action의 SHA와 version 주석만 치환한 결과가 실제 후보와
완전히 같습니다. trigger·권한·cache key·save-if·설치 tool을 바꾸지 않았습니다.
로컬 Mac에서 action의 sudo purge를 실행하지 않았으며 실제 GitHub CI는 로컬 게이트 뒤 확인합니다.

## 최소 글꼴 전체 회귀 완료

수정한 격리 프로필에서 [전체 nextest](full-nextest-result.json)는 실제 13,617건 실행·
13,617PASS·0FAIL·45SKIP, 테스트 본문 474.565초로 완료했습니다. runner 시작 source와
현재 head 사이의 제품·테스트·빌드·공개 fixture 입력 변경이 없음을 확인했습니다.
Native Skia 별도 범위·lint·fresh WASM·전쪽 보호 및 실제 소비 검증은 진행 중이므로
이 결과만으로 모든 로컬 게이트 완료나 PR 준비 완료로 기록하지 않습니다.

## Native Skia 범위와 HTTP 실제 소비 확인

[Native Skia 필수 3종](native-skia-validation.json)의 lib·누락 그림 placeholder·직접 PDF
검사를 모두 실제 통과했습니다. Native root Clippy도 통과했고 나머지 lint/build는 순차 진행 중입니다.

#9114 ureq3.5.2는 [실제 Native 웹폰트 공급](native-webfont-network-validation.json)으로도
확인했습니다. 저장소의 기존 짧은 입력을 사용하고 별도 빈 캐시에 공개 등록 URL의 WOFF/WOFF2
자원2개를 제품 HTTP client로 내려받았습니다. 재사용 실행과 네트워크를 프로세스별로 차단한
재사용 실행까지 성공했으며, 원본/변환 프로그램 receipt·PNG hash가 같고 캐시 파일도 바뀌지
않았습니다. [실제 표시 PNG](ureq-native-font-cold.png)에서 한글·라틴·숫자·문장부호를 판독했습니다.
이는 HTTP·decode·캐시·Native 소비 확인이며 독립 PDF 일치율이나 Visual Sweep 증거를 대신하지
않습니다. 개인 폰트·시스템 네트워크 설정은 바꾸지 않았고 다운로드 글꼴 바이너리는 Git에 넣지 않습니다.

## Rust 게이트 전체 완료와 격리 증거 보존

[순차 Rust 게이트15개](rust-gates-validation.json)는 fmt·release build/lib·전체 nextest·
Native Skia3종·Native/WASM Clippy·workspace build/all-target Clippy·Native all-target Clippy·
doc test와 suite 정책까지 모두 exit0로 완료했습니다. 실제 `cargo test --doc` 명령을 사용했습니다.
제품·테스트 입력은 runner 시작 source와 동일합니다. 개인 CoreText 목록도 종료 후 다시 비교해
1,890face가 그대로임을 확인했습니다. 실제 사용한 프로필 원문과 hash를
[프로필 snapshot](minimal-font-profile-snapshots.json)에, 격리 환경의 OS528face 목록을
[CoreText TSV](minimal-coretext-os-faces.tsv)에 보존합니다. 일반 기여자에게 이 Mac 경로나
비공개 폰트를 요구하는 정책 변경은 아닙니다.

## Native 정본 전쪽 비교 완료

[Native 보호 검증](protected-native-validation.json)은 18개 입력의 base·후보 각각1,771쪽을
실제로 산출해 통과했습니다. 두 단계 모두 정본 공식 표시값 하락0쪽이며, **같은 쪽1,771개의
base·후보 원본 PNG hash가 전부 같습니다.** 독립 PDF와 모든 픽셀이 같다는 뜻은 아닙니다.
상승한 쪽도0이므로 Native 보호 기준을 임의로 변경하지 않습니다.
[빌드 신원과 base 재사용 근거](native-and-base-build-producers.json)를 별도로 보존합니다.
일부 캡처 wall time에는 메모리 조정을 위한 보존 대기가 있으므로 성능 비교로 쓰지 않습니다.
이후 같은 제품 source의 fresh WASM과 해당 전쪽·실제 제품 소비 검증을 계속합니다.


## WASM 재생 경로 원인과 실제 제품 소비 완료

[브라우저 경로 대조](wasm-raster-mode-diagnosis.json)에서 같은 Chrome·WASM·글꼴·PDF의
GPU 재생은 법령1쪽99.96967%, 소프트웨어 재생은99.98059%였습니다. Mac·Linux PDFium의
기준 PNG는 디코딩한 픽셀까지 동일하므로 PDF 변환이나 의존성 변경으로 단정하지 않습니다.
현재 GPU raw PNG는 과거 커밋의 동일 쪽과도 픽셀까지 같습니다. 기존 지침의 소프트웨어 Print
비교 경로를 명시하여 base/head 각각18종 전쪽을 별도 output에 다시 산출합니다. 보호값·DPI·
마스크·원본·PDF·글꼴·2px·90%를 바꾸지 않으며 이전 GPU 결과도 보존합니다. 집중1쪽 통과만으로
전쪽 보호 완료를 선언하지 않습니다.

[제품 소비 검증](production-consumers-validation.json)은 Studio·Chrome·Firefox production build,
Studio AI 패널·설정·공급자 mock3종과 Chrome 확장의 smoke·다운로드·수명주기3종을 실제
통과했습니다. 루트 pkg·Studio public/dist·Chrome/Firefox dist와 HTTP 응답의 JS/WASM hash를
새 package와 대조했습니다. 유료 API 호출이나 실제 비밀 키는 사용하지 않았습니다.
Firefox 편집·저장 E2E와 정본 보호·공통 게이트·출력 백엔드 검증은 별도로 남아 있습니다.
[Native 대표22쪽 판독](native-direct-reading.json)은 같은 쪽의 review·overlay 쌍으로 확인했습니다.


## 최신 devel의 영향 경계와 재사용 판단

추가된 #9126의 두 제품 경로와 새 회귀·실제 저장 증거를 확인했습니다.
[읽기 경로 감사](upstream-protected-read-path-audit.json)에서 보호18종의 중첩·머리말·꼬리말·
master page를 포함한 실제 모델을 순회했고, 새 미기입 안내문 개체 줄 분기에 해당하는
글상자 문단은 모두0개였습니다. serializer 변경은 Print 캡처가 호출하지 않는 저장 경로입니다.
정본 TSV·manifest 변경은 없고 현재 후보와 새 devel의 merge-tree도 exit0입니다.
단순 devel 전진만으로 검증된 head를 반복 리베이스하지 않는 현행 지침에 따라 기존 생산 SHA와
증거를 유지합니다. 원래 source를 새 source의 출력으로 바꾸어 적지 않으며, PR 직전 최신
head·병합 시뮬레이션을 다시 확인하고 실제 merge CI에서 새 필드 기능 검사도 확인합니다.


## 실제 WASM 입출력·암호화의 반복 저장

[반복 저장 실행](repeated-wasm-io-validation.json)은 같은 fresh WASM 공개 API에서 기존
본문에 marker를 넣고 HWP/HWPX 각각 일반·암호화4경로로 저장→재열기→재저장→재열기를
완료했습니다. 매 단계 본문 전체가 같았고 marker 삭제 후 저장·재열기도 원래 본문과 정확히
같았습니다. 암호화 산출물은 암호 없음·틀린 암호의 열기를 거절했습니다. AES·zip·DEFLATE의
실제 WASM 소비를 확인한 것으로, 시각 일치율·전체 형식의 모든 내용 보존을 대신하지 않습니다.
원본 입력은 수정하지 않았고 생성 문서·임시 난수 암호는 Git에 넣지 않습니다.


## fresh WASM 전쪽 완료와 실제 PNG 바이트 재확인

[fresh WASM 보호 검증](protected-wasm-validation.json)은18종의 base·후보 각각1,771쪽을
완료했고 공식 표시값 하락·누락·쪽수 불일치0건입니다. 상승도0건이라 Native와 함께
정본 보호 하한을 바꾸지 않습니다. [실제 파일 재확인](whole-png-identity-audit.json)에서
Native/fresh WASM base·후보의 원본 PNG 총7,084개를 파일 바이트로 다시 해시했고,
캡처 manifest와 모두 일치하며 각 backend의 같은 쪽 base/head도 전1,771개가 동일했습니다.
PDF와 모든 RGB가 같다는 뜻이나 기존 미세 잉크 차이의 새 해결 주장은 아닙니다.

## 실제 CanvasKit 인쇄 소비 보정

공개 image-crop의 Print에서 기존 base WASM도 빈 일반 선 패턴 `[]`를 MakeDash에 전달하여
같은 예외가 발생했습니다. 원 경로 c7be19d7d07과 배열 지원 통합 #8120(80968497c2c)을
추적했습니다. Canvas/SVG의 빈 패턴은 실선인데 CanvasKit 소비가 이를 구분하지 않았습니다.
일반 빈 패턴만 실선으로 그리며 명시적 물리 패턴의 빈 배열 거부는 유지합니다.
실제 CanvasKit 선·사각형 raster 관계 검사는 보정 전 2FAIL/1PASS에서 보정 후 12PASS로
확인했습니다. [원인·실행 근거](canvaskit-empty-stroke-cause.json)를 따릅니다.
홀수 패턴 일반화나 readiness 예외는 추가하지 않았습니다. 최종 브라우저·제품 build는 재검증 중입니다.

## 브라우저 검증 종료 경로

새 Puppeteer의 stdout/stderr 로그 수집기가 종료된 Mac 검증 Chrome의 crashpad/updater가
상속한 파이프 두 개를 기다리고 있었습니다. 실제 자식 종료·fd peer·보고서 완료를 확인했습니다.
공용 E2E helper는 browser.close가 성공하고 해당 로컬 자식 종료가 확인된 경우에만 그 자식의
stdout/stderr를 닫습니다. 사용자 원격 브라우저와 종료 실패 상태는 그대로 보존합니다.
[수정 전후 관계 검사와 원인](browser-lifecycle-cause.json)을 기록하고 강제 exit 없이
정상 종료하는 실제 출력 검사를 다시 수행합니다.
Firefox160의 확장 문맥 검사는 공식 Remote Agent 계약에 필요한
`MOZ_REMOTE_ALLOW_SYSTEM_ACCESS=1`을 이번 격리 테스트 프로세스에만 공급합니다.
[Mozilla 공식 계약](https://firefox-source-docs.mozilla.org/remote/Security.html)을 따르며
사용자 Firefox 프로필과 시스템 설정은 변경하지 않습니다.

## Firefox 실제 다운로드와 BiDi 검증

Firefox160에서 확장은 실제 viewer를 자동으로 열었지만 Puppeteer의 browser.pages 캐시에
그 탭이 누락돼 검사만 시간 초과했습니다. 확장 API의 실제 탭과 BiDi getTree를 대조했습니다.
기존 E2E는 같은 실제 viewer 문맥을 getTree로 찾아 script.callFunction에서 기존 DOM 명령을
실행하도록 보완했습니다. 새 viewer를 만들어 대체하지 않았으며 제품 코드 변경은 없습니다.
최종 원 E2E 명령이 정상 exit0으로 완료되어 다운로드3건·viewer1개·편집 보존을 확인했습니다.
[원인과 실행 영수증](firefox-bidi-validation.json)을 따릅니다.
