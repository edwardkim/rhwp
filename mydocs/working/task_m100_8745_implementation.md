# #8745 암호 문서 보안 경계 구현 기록

## 상태

- 이슈: [#8745](https://github.com/edwardkim/rhwp/issues/8745), 담당 edwardkim.
- 기준: `e4ffabea38062e716f6c333cc950fa31b42b4dd7`.
- 계획 커밋: `3e4e0c67b1`. 이슈 등록·구현·필수 로컬 검증을 승인받았다.
- 구현 커밋 `f922a759aa`와 제품 검증 뒤 전체 Rust 분석에서 테스트 호출 8개 파일을 추가 보정했다. 최종 제품 검사와 전체 분석을 완료했다. PR 범위 logging 107→0, 전체 범위의 RHWP logging 48→7→0이며 별도 vendor Python 36건은 유지된다. [최종 보고서](../report/task_m100_8745_report.md).

## 선행 검증과 구현 선택

1. 기존 공개 `OpenOptions`를 쓰는 정식 suite 회귀가 수정 전 `Debug exposed a credential`로
   실패했다. `regression_suite_019`, 1 failed, 306 filtered out. 실행 로그는
   `output/codeql-password-investigation/debug-red.log`다. 소스 fragment probe와 달리 실제
   라이브러리를 연결한 결과다. 렌더링 동작 검사로 보고하지 않는다.
2. CodeQL 2.27.4 Rust 전용 공식 bundle의 GitHub release digest와 내려받은 파일의 SHA-256이
   같음을 확인했다. 로컬 `output/` 안에서만 사용한다. 저장소 workflow·query는 변경하지 않았다.
3. [첫 command/query 실험](../tech/investigations/issue-8745/command-query-probe-result.json)은
   문서 반환값을 로그로 사용하는 기존 경로와 실제 암호 출력만 경고했다.
4. [추가 양성·음성 대조](../tech/investigations/issue-8745/command-query-controls-result.json)는
   원자적 문서 교체 command도 경고하지 않고, 실제 암호 출력 및 고정 운영 credential 호출을
   계속 검출했다. 이 소규모 결과를 전체 제품의 분석 결과로 승격하지 않는다.

작은 실험을 근거로 별도 loader 계층을 늘리는 대신 기존 `DocumentCore`에 `load_bytes` command를
추가했다. 기존 공통 파서와 Core 생성 로직을 그대로 사용하고 완전히 생성된 뒤 `self`를 교체한다.
오류 시 이전 문서를 보존하며, 성공 이후 기존 조회 API로 문서를 사용한다. CLI·서비스·MCP·Native
host와 WASM 암호 constructor의 내부 호출을 이 경계에 연결한다. 외부 constructor의 시그니처와
JS `openWithPassword`는 유지한다. 호환 wrapper와 전체 경로의 분석 결과는 아래 최종 검증에 기록했다.

WASM wrapper를 만드는 데 `createEmpty()`를 사용하면 불필요한 A4 문단 조판을 한 번 수행한다.
이에 `From<DocumentCore>`로 이미 준비한 Core를 감싸도록 했으며 파싱·조판은 기존 열기 경로의
한 번만 수행한다. 성능을 측정해 개선됐다고 주장하지 않는다.

## CLI 저장 정보의 기존 근거와 보정

기존 HWP 저장은 원 기여 `86537efdaebff15d938c136f2fe2b6a8329a5c89` → 메인테이너
`750c5a919661d7e7e384097adda129c3cf30889a` → [#6694](https://github.com/edwardkim/rhwp/pull/6694)
merge `b6b9384ed9e679a8d143c6dcbaacef502614eb9e`에서 파일 크기를 보고하도록 바뀌었다.
원 patch를 확인했으며 당시 전체 회귀 8,986개 통과는 PR의 기록이다. 이번 재실행 결과가 아니다.

HWPX의 #190은 아직 암호화 반환 buffer의 길이를 보고했다. 두 저장 명령에 공통 write helper를
적용해 같은 열린 파일 handle의 metadata에서 저장 크기를 읽는다. 경로를 다시 조회하는 경쟁과
metadata 실패 시 0바이트 성공으로 처리하던 문제도 피한다. 암호화 buffer는 실제 파일 저장과
재열기에만 사용한다. JSON 필드·종료 코드 보존은 실제 CLI 왕복 회귀로 확인한다.

## 검증 진행 조건

- CodeQL baseline은 기준 SHA의 tracked Rust 소스·workspace 경로를 추출한 snapshot에서 실행한다.
  PR config를 적용한 분석과 전체 Rust 경로 분석을 구분한다. GitHub 분석을 재실행하지 않았다.
- 메모리 여유를 확보하려고 후보 extraction 첫 시도를 중단했다. 그 미완성 DB는 검증으로 사용하지
  않으며 baseline·집중 빌드가 끝난 뒤 순차 재생성한다. 사용자 서버 프로세스는 보존했다.
- 최소 글꼴 환경은 bwrap으로 홈·Windows·상용 글꼴 경로를 숨긴다. 실제 노출 파일은 저장소의
  공개 글꼴 5개이며 `output/codeql-password-investigation/minimal-font/visible-fonts.txt`에 기록했다.
- 전체 문서 metadata 검사의 기존 누락 16건은 범위 밖 4개 문서다. 새 조사 문서 두 개의 실제
  `validate_file()` 검사는 성공했다. 상대 경로로 함수를 호출한 최초 진단 명령의 오류는 절대 경로로
  바로잡았으며 검사기를 변경하지 않았다.

## 검증 완료 확인

- [x] 집중 회귀 6건: Debug·실패 보존·재시도·constructor 호환·CLI 저장 정보·PDF R2–R6 인증·HWP5 지연 이미지. 기존 분류 가드 5건을 포함해 최소 글꼴 환경에서 11건이 통과했다.
- [x] HWP3·HWP5·HWPX·PDF 기존 암호 회귀와 서비스·Native host 계약.
- [x] 같은 로컬 CodeQL의 PR 범위 전후 SARIF에서 logging 107→0과 실제 비밀 양성 대조 검출을 확인했다.
- [x] 테스트를 포함한 전체 Rust CodeQL 평가 완료와 실제 결과 대조. RHWP logging 0건, vendor Python 36건 유지. 전후 대상 집합·추출 warning·설정 동일.
- [x] 공개 벡터·작업 버퍼 16건 및 형식 관련 2건의 [개별 근거](../tech/investigations/issue-8745/crypto-result-review.md)를 정리했다. 분류 변경·알고리즘 변경은 하지 않았다.
- [x] 전체 12,831건·Native 4,077건·Rust lint 묶음·최적화 fresh WASM·실제 브라우저 열기와 저장을 통과했다.
- [x] 최종 소스의 결과·한계·호환 API 사용 가이드와 보고서 작성. 구현·분석 증적을 로컬 커밋으로 보존한다.

PR 생성·원격 push·경고 재분류·merge는 이번 승인 범위에 포함하지 않는다.

집중 검증 중 첫 재빌드는 라이브러리가 컴파일되는 동안 wrapper 호출부를 변경해 구형 라이브러리와 새 호출부가 섞여 실패했다. 이후 소스를 고정하고 전체 재빌드한 4건, PDF와 archive 런타임 binary 경로를 보완한 최종 5건은 모두 통과했다. 이 최초 실패를 제품 회귀로 분류하지 않는다.

## 지연 이미지 인증 정보의 수명

`Hwp5BinResolver`는 이미지의 지연 복호화를 위해 `Option<Vec<u8>>` 암호를 소유한다
(`src/parser/mod.rs`의 구조체·생성·`try_decode`). 따라서 문서 전체에 암호가 전혀 없거나
열기 직후 모든 암호가 폐기된다고 설명하면 안 된다. 기존 custom `Debug`는 stream 개수와
암호화 여부만 출력한다. HWPX는 복호화한 평문 ZIP을 기존 resolver가 소유하고, HWP3는
복호화된 본문을 파싱한다. 이 작업은 HWP5 지연 로딩·메모리 수명을 변경하지 않는다.

이 확인에 따라 API 주석을 정확히 하고 실제 HWP5 resolver의 Debug 비노출을 추가 검증한다.
첫 전체 회귀 종료 뒤 주석·회귀를 반영했다. HWP5 지연 이미지 resolver에 합성 암호를 공급하고
문자열·바이트 배열의 Debug 비노출과 실제 이미지 복호화 결과의 보존을 함께 검사한다.

## 기준 CodeQL 재사용

GitHub commit/tree API로 PR #8739의 실제 분석 merge와 현재 base의 루트 entry 79개를
대조했다. 다른 것은 `mydocs/` 하나뿐이며 제품 소스·Cargo·의존성·workflow·CodeQL 설정을
포함한 78개는 동일한 Git object다. [대조 원장](../tech/investigations/issue-8745/baseline-tree-equivalence.json).
따라서 CodeQL 2.27.4의 기존 GitHub SARIF를 수정 전 기준으로 재사용한다. 같은 제품 코드를
반복하던 로컬 baseline 분석은 중단했고 완료 결과로 표시하지 않는다. 후보 분석과 전체 Rust
범위의 신규 분석은 계속한다. GitHub workflow는 취소·재실행하지 않았다.

[출력 인자 원장](../tech/investigations/issue-8745/sink-invocations.json)은 기존 107개 logging 결과의
실제 macro region을 같은 source tree에서 추출했다. 106개는 좌표·개수·상태·index 등의 projection이고
#190은 출력 경로와 바이트 수다. include 위치로 보고된 세 건은 `stored_cell_reflow.inc.rs`,
`shape_item.inc.rs`, `stored_ink_fit.inc.rs` 안의 실제 macro를 추가로 확인했다. `table_meta`는
`Option<(usize, usize)>`, `ParagraphFloatPlacement`는 기하와 흐름 enum이며 `_item_desc`는
항목 종류의 상수 문자열과 index다. 암호 원문·Hwp5BinResolver 내부 암호·본문을 직접 출력하는
인자는 이 107개에서 확인되지 않았다. 이는 저장소의 모든 로그나 모든 가능한 flow에 대한
포괄적인 안전 판정이 아니며 GitHub 분류를 자동 변경하는 근거로 사용하지 않는다.

## 기존 열기 경로와 출력 보존

[실행 코드](../tech/investigations/issue-8745/output-parity.rs.txt)·[결과](../tech/investigations/issue-8745/output-parity-result.json):
현재 후보 라이브러리에 그대로 유지한 constructor와 새 command를 같은 최소 글꼴 환경에서
실행했다. HWP3 24쪽·HWP5 64쪽·HWPX 24쪽·PDF 1쪽, 합계 113쪽의 SVG가 전부 같고 각
문서의 HWP·HWPX 저장 바이트도 같다. 이는 호출 경계의 출력 보존 검사이며 독립 한컴 Print
피델리티 점수 또는 fresh WASM 픽셀 비교로 표시하지 않는다. 암호화 저장은 별도 집중 회귀의
실제 재열기로 검증했다.

## 전체 회귀가 검출한 등록 누락

첫 전체 회귀는 12,830건 중 12,829건 통과·1건 실패였다.
`issue_2724_passthrough_invalidation_guard::classification_drift_is_blocked`가
새 `load_bytes`의 미분류를 검출했다. 이 가드는 #2724 → 통합 #2797의
`f2c02b66691fd3092e714bce98de93def74fce5e`에서 도입됐다. [원 보고서](../report/archives/task_m100_2724_report.md)의
독립 Python/Rust 스캔과 5개 음성 대조, 현재 `WholeDocument` 분류 계약을 먼저 확인했다.
`load_bytes`는 기존 IR의 일부를 고치는 함수가 아니라 새 Core와 raw provenance까지 함께 교체한다.
따라서 기존 가드의 `WholeDocument` 분류에 실제 교체·실패 보존 근거를 등록하며, 가드의
탐지 범위·밀도·상한은 변경하지 않는다. 등록 누락을 놓친 첫 전체 결과는 실패로 유지한다.

[CLI 저장 크기 추가 분석 실험](../tech/investigations/issue-8745/command-query-export-probe-result.json)은
기존 반환 buffer 길이의 로그를 검출하고, 같은 파일 handle의 metadata로 얻은 크기는 검출하지
않았다. 실제 암호 로그·고정 운영 credential 양성 대조도 유지됐다. 최초 명령의 query 파일명 오류는
설치된 pack의 실제 파일명으로 바로잡았으며, 최종 결과만 분석 성공으로 기록한다.

## PR 범위의 후보 CodeQL 결과

CodeQL 2.27.4와 동일 query pack, 저장소의 `rust-pr.yml`로 후보를 분석했다. 실행은 성공했고
logging 결과는 0건이다. 하드코딩 16건·형식 관련 2건은 그대로 보고된다. query·path·severity
설정은 수정하지 않았다. GitHub의 기존 분석 107건과 후보 0건을 대조하되, 로컬 추출 진단도
별도로 확인한다. 후보는 1,604개 파일을 오류 없이 추출했고 181개에 추출 문제를 보고했다.
`proc-macro not yet built` 등 기존 매크로 진단을 무시한 채 완전한 분석이라고 주장하지 않는다.
동일 로컬 추출기의 기존 DB에서 logging 규칙을 끝까지 실행해 전후 차이를 추가로 확인한다.
전체 tracked 파일 snapshot에서는 경로 제한 없는 Rust 분석을 별도로 수행한다.

[추출 범위 대조](../tech/investigations/issue-8745/analysis-coverage.json)에서 수정 전·후 정상 파일
1,604개·문제 파일 181개가 같고, 831개 추출 warning의 경로·내용·중복 수도 모두 같았다.
[후보 SARIF 정규화 결과](../tech/investigations/issue-8745/candidate-pr-analysis.json)의 나머지
18건은 기존 GitHub 결과의 규칙·파일·행과 모두 일치한다.

같은 로컬 DB의 logging 규칙 실행도 종료 코드 0으로 완료했다.
[수정 전 로컬 결과](../tech/investigations/issue-8745/base-logging-analysis.json)는 107건이며
GitHub 원 결과와 규칙·파일·행이 전부 같다. 따라서 로컬 전후도 107→0이다. 이전 baseline
전체 분석을 중단한 사실과, 이후 logging 한 규칙을 끝까지 완료한 결과를 구분한다.
전체 Rust 분석 snapshot은 Git의 모든 tracked 파일에 현재 변경을 반영하고 정식 생성기로
28개 테스트 하네스를 생성했다. 테스트 파일이 분석에서 빠진 상태를 전체 검증으로 쓰지 않는다.

문서 내부 링크 5개 파일 검사는 통과했다. 장기 문서 metadata 검사도 변경된 manual·tech
3개 파일에서 통과했다. 최초 직접 호출에서 범위 밖 `plans/`까지 장기 문서 validator에 넣어
`kind: plan` 오류가 났으나, 검사기의 `REQUIRED_PATHS`와 기존 계획 문서의 형식을 확인해
호출 대상을 바로잡았다. 계획서의 형식이나 검사기 허용값은 변경하지 않았다.
전체 분석의 생성 하네스와 실제 Cargo 검증 하네스는 suite별 분산 위치는 다르지만 포함한
2,092개 회귀 모듈 집합이 같다. 별도 20개 integration target도 그대로 포함한다.

## 재부팅 뒤 재개

2026-11-07 21:08 KST에 작업 트리와 보존된 실행 기록을 대조했다. 고정한 제품·회귀 소스의
SHA-256은 모두 같았다. 전체 회귀는 재부팅 전에 종료 코드 0, 12,831건 통과·기존 45건 skip으로
완료되어 재실행하지 않았다. Native 검사는 컴파일 중, 전체 CodeQL은 분석 중에 중단됐다.
Native부터 나머지 순차 검증을 재개하고, CodeQL은 이미 생성 완료한 전체 DB를 재사용했다.
이전 로그·중단 기록과 재개 실행의 시작·종료를 구분해 보존한다.

Native 라이브러리와 workspace unit 4,040건은 통과했다. 이어진 placeholder 명령은
최소 글꼴 sandbox에서 NVM의 Node 실행 파일을 찾지 못해 테스트 시작 전에 종료됐다.
Node 실행 파일 하나를 읽기 전용 bind하고 허용된 글꼴 5종만 보이는 것을 다시 확인했다.
제품 코드·글꼴 범위는 변경하지 않았으며 해당 검사부터 재개한다. crypto의 2개 unit은
Native 실행에 포함되어 통과했으므로 같은 unit만 반복하는 별도 명령은 생략한다.

Native placeholder의 다음 시도는 빌드 성공 뒤 0건 실행으로 종료 코드 4를 반환했다.
최종 회귀 파일 길이에 따라 계산한 suite와 앞서 생성한 하네스의 배치가 달랐다.
정식 `--prepare`로 파생 하네스를 다시 생성했으며 전후 2,112개 테스트 source 집합은 같다
(회귀 모듈 2,092개·별도 target 20개). 완료한 전체 회귀의 실행 범위에는 누락이 없었다.
0건 결과를 통과로 쓰지 않고 Native focused 검사를 올바른 하네스로 재실행한다.

## Rust 필수 검증 완료

2026-11-07 21:42 KST에 전체 회귀 12,831건·기존 45건 skip, Native 공식 4,077건,
fmt·root Clippy·WASM32 Clippy·workspace build·workspace all-target Clippy·suite manifest
검사를 모두 완료했다. [명령·종료 코드·소스와 로그 해시](../tech/investigations/issue-8745/rust-validation.json).
재부팅과 검증 하네스 두 문제의 실패 기록은 별도로 남기며 성공으로 덮지 않는다.
제품·테스트 소스 해시는 고정한 값과 같다. 이 상태를 로컬 구현 커밋으로 보존하고,
전체 CodeQL 결과와 새 최적화 WASM의 실제 브라우저 검증을 계속한다.

## 최적화 WASM·실제 브라우저 검증 완료

구현 커밋 `f922a759aa`로 `wasm-pack`과 `wasm-opt`를 포함한 빌드를 완료했다.
WASM은 28,328,828바이트, SHA-256은
`c01a7cc9afadcd9fec29e65cbd3dcbbd5520e3e6230a49f0f88c714fda7a737c`다.
전용 Vite 7785·새 headless Chromium에서 HWP3/HWP5/HWPX 암호 열기, PDF Worker 인증,
HWP/HWPX 평문·암호화 저장 재열기의 기존 E2E 3개 스크립트가 모두 종료 코드 0으로 통과했다.
취소·오입력·재시도·기존 문서 유지·저장 실패·보호 의도·브라우저 storage 비노출을 함께 확인했다.
[명령·산출물·브라우저 실제 수신 해시](../tech/investigations/issue-8745/wasm-browser-validation.json).

초기화의 실제 WASM 응답은 모두 빌드 해시와 일치했다. PDF Worker도 별도 패키지의 같은
불변 URL을 요청했고 실제 인증·게시를 통과했다. 초기화 CDP 응답 증거를 Worker 응답마다
별도로 해시한 증거로 확대하지 않는다. 전용 서버와 브라우저는 종료했고 기존 Studio 산출물은 유지했다.
전체 Rust CodeQL 평가의 완료와 결과 대조는 계속 진행 중이다.

## 전체 분석의 로컬 자원 보정

12,288 MiB 제한의 전체 분석은 타입 추론이 계속 진행됐지만 70분 이상 종료되지 않았다.
평가 로그의 반복적인 VITAL array eviction과 프로세스 I/O(논리 읽기 약 800 GB, 쓰기 약 33 GB)를
확인했다. 제품 빌드·브라우저가 모두 끝난 시점에 가용 메모리 28 GiB를 확인하고 로컬 evaluator를
SIGINT로 종료했다(130). 이 실행은 분석 완료나 제품 실패로 분류하지 않는다.
같은 DB·소스·query config와 완료된 쿼리 캐시를 유지한 채 RAM 24,576 MiB·8 threads로
재개했다. GitHub workflow·탐지 설정·대상 파일은 변경하지 않았다. 메모리 상향의 실제 효과와
분석 완료 여부는 새 실행 결과로 판단한다.


## 전체 분석이 발견한 테스트 경로 보정

첫 전체 분석은 22:37 KST에 종료 코드 0으로 끝났으나 logging 84건이 남았다.
[전체 결과와 개별 분류](../tech/investigations/issue-8745/full-result-review.md)에 따라
RHWP 테스트 constructor에서 조판 진단으로 이어지는 46곳과 fixture의 본문 출력 2곳을
보정한다. 제품 코드·기대값은 유지하며 테스트 6개 파일의 인증 호출/실패 진단만 변경했다.
첫 전체 결과를 0건으로 보고하거나 PR 범위 107건과 직접 증감 비교하지 않는다.

같은 전체 소스 범위·query 설정으로 새 DB를 만들고 분석한다. 전체 회귀와 all-target Clippy도
다시 실행하며, 제품 소스가 같은 Native·최적화 WASM·실제 브라우저 결과는 재사용한다.
별도 vendor Python·qcms 결과를 이번 테스트 경계의 해결 완료에 포함하지 않는다.


둘째 전체 분석은 종료 코드 0이지만 logging 43건 중 RHWP 경로 7곳이 남았다.
HWP3 fixture와 차트 저장 검사의 다른 constructor 호출이 source였다. 남은 운영 목적의
테스트 호출을 함께 점검해 8개 파일의 열기·저장·재열기 경계를 연결했다. 기존 공개 constructor의
열기 자체와 feature-off 오류를 검사하는 세 호출은 유지한다. 기대값·제품 코드는 변경하지 않는다.
[최종 변경 목록](../tech/investigations/issue-8745/test-boundary-completion.json).
전체 회귀·lint와 전체 CodeQL을 다시 확인한 뒤 완료로 판정한다.

2026-11-07 23:39 KST에 최종 8개 테스트 파일의 전체 회귀 12,831건·기존 45건 skip,
fmt·workspace all-target Clippy·suite manifest 검사를 모두 통과했다. 해당 8개 파일의
49개 테스트도 실행 로그에서 개별 PASS를 확인했다. 입력·기대값은 유지했으며 분석 snapshot과
소스 해시가 같다. 제품·신규 #8745 회귀 소스도 기존 고정 해시와 같으므로 Native·WASM·브라우저
검증은 재사용한다. [최종 테스트 검증 원장](../tech/investigations/issue-8745/test-boundary-completion.json).

## 최종 분석 완료

2026-11-07 23:48 KST에 세 번째 전체 CodeQL 실행의 성공을 확인했다. RHWP logging은
48→7→0이며 별도 vendor Python logging 36건의 위치 집합은 그대로다. 하드코딩 결과
126→121은 같은 공개 PDF 대조군 5개의 호출 경계 변경이며 합성 값·검사 입력은 유지했다.
query·대상 파일·workflow·vendor 소스를 변경하거나 GitHub 경고를 재분류하지 않았다.

[최종 SARIF](../tech/investigations/issue-8745/candidate-full-final-analysis.json)와
[범위·실행 원장](../tech/investigations/issue-8745/full-codeql-validation.json)에 실제 결과를 보존했다.
4,730개 소스 파일 집합, 정상 추출 4,610개·문제 보고 85개, warning 115개의 경로·내용·중복 수가
전후 같았다. 기존 추출 한계와 별도 vendor·legacy 결과를 남긴 상태로 이번 로컬 작업을 완료한다.
PR·원격 CI·병합·이슈 종료는 수행하지 않았다.
