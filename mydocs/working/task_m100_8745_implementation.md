# #8745 암호 문서 보안 경계 구현 기록

## 상태

- 이슈: [#8745](https://github.com/edwardkim/rhwp/issues/8745), 담당 edwardkim.
- 기준: `e4ffabea38062e716f6c333cc950fa31b42b4dd7`.
- 계획 커밋: `3e4e0c67b1`. 이슈 등록·구현·필수 로컬 검증을 승인받았다.
- 현재 구현과 검증 중이다. 제품 보정의 CodeQL 통과·PR 준비 완료를 아직 선언하지 않는다.

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
JS `openWithPassword`는 유지한다. 호환 wrapper와 전체 경로의 분석 효과는 아직 검증 중이다.

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

## 남은 검증

- [x] 집중 회귀 6건: Debug·실패 보존·재시도·constructor 호환·CLI 저장 정보·PDF R2–R6 인증·HWP5 지연 이미지. 기존 분류 가드 5건을 포함해 최소 글꼴 환경에서 11건이 통과했다.
- [ ] HWP3·HWP5·HWPX·PDF 기존 암호 회귀와 서비스·Native host 계약.
- [ ] 고정 CodeQL의 변경 전후 SARIF, 목표 logging flow 제거와 실제 비밀 양성 대조 유지.
- [x] 공개 벡터·작업 버퍼 16건 및 형식 관련 2건의 [개별 근거](../tech/investigations/issue-8745/crypto-result-review.md)를 정리했다. 분류 변경·알고리즘 변경은 하지 않았다.
- [ ] 필수 전체 회귀·Rust lint·Native 해당 검사·fresh WASM·실제 브라우저 열기와 저장.
- [ ] 현재 head의 결과·한계·호환 API 사용 가이드, 보고서·로컬 커밋.

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
