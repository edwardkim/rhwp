# #8174 crates.io 배포 준비 구현 계획

Issue: [#8174](https://github.com/edwardkim/rhwp/issues/8174)

- [수행 계획](crates_io_preparation_20261023.md)
- 기준 source: `def947c2149df5bc812d801e6e79be19a403b4d1`
- 현재 범위: v1을 완료했고, 2026-10-24 사용자 「v2 계획 승인·CLI부터 구현 진행」으로 아래 구현 계획을 승인받았다. [V2-S01 실행 기록](../working/task_m100_8174_v2_stage1.md)에 `import-pdf`·공통 정책의 로컬 구현, 전체 회귀 11,386건 통과와 이전 출력·성능 비교를 기록했다. 이후 승인한 S02–S05도 [S05 실행 기록](../working/task_m100_8174_v2_stage5.md)까지 로컬 구현·필수 검증을 완료했다. 현행 14개 후보의 실제 포장·독립 Rust/CLI/WASM 소비를 확인했으며, MD05의 나머지 감사·확정한 게시 계약의 최종 검증·실제 원격 게시는 계속 남아 있다.
- 2026-10-23 「PR 준비 하세요」 지시에 따라 제출 전 필수 로컬 검증·입력 보존·커밋과 PR 본문·생성 명령 초안을 준비했다. 실제 결과는 [2단계 보완 기록](../working/task_m100_8174_stage2_corrections.md#pr-제출-전-로컬-검증--2026-10-23)에 있으며 원격 push·PR 생성은 별도 승인 단계다.
- 추가 문서 작업: [메인테이너 결정 대장](task_m100_8174_maintainer_decisions.md)을 작성하고
  선택·조건·근거를 기록한다. MD02–MD04의 기술 보완 범위는 부분 확정이고 MD05–MD07은 아래
  2026-10-24 선택을 반영한다. 2026-10-25 추가 답변으로 MD01·MD02–MD04·MD08–MD09도
  확정했다. crates.io 일정은 0.9.0과 별도로 정하며, 14개·본체 0.9.0·핵심 API·Rust 1.93.1과
  실제 검증 target을 채택한다. edwardkim 책임의 초회 수동 게시 절차를 준비한다.
  실제 게시·태그·권한 변경의 실행 승인은 별도다.
- 2026-10-23 추가 착수 승인: 정책 선택과 무관한 C01–C04의 로컬 사전 검사 도구·실행 증적을
  먼저 구현한다. 제품 manifest·feature·실제 게시 설정은 이 절편에서 변경하지 않는다.
- 2026-10-23 구조 재조사 뒤 수정 지점·우선순위 착수 승인으로
  [로컬 보완 범위 v1](task_m100_8174_maintainer_decisions.md#결정-뒤-구현-범위-확정-방법)을 확정했다.
  [A01–A07 보완 실행](../working/task_m100_8174_stage2_corrections.md)을 기존 기본값과 보안 경계를
  유지하며 진행한다. 아래 일반 후보 목록은 범위 v1에서 승인한 부분부터 적용한다.

## 2026-10-24 재개 위치

- v1은 [PR #8181](https://github.com/edwardkim/rhwp/pull/8181), commit `d9fd85ed5`로 병합됐다.
  코덱 opt-in도 [PR #8190](https://github.com/edwardkim/rhwp/pull/8190), commit `e7a0dec55`로 병합됐다.
- 최신 base `3536e72819e165dbe959ee42cfd4e4cf8db33ce7` 위에서 기존 승인의 결정 비의존
  점검을 재개했다. [현재 후보 graph](../../packaging/crates-io/source-policy.json)는 12개다.
  source 검사·내장 참조·고지·archive readback은 재확인했고 제품·게시 manifest는 바꾸지 않았다.
- 다음 구현은 [결정 대장](task_m100_8174_maintainer_decisions.md)의 MD05-B 지원 목표,
  MD06의 F01–F08 기본 제공과 MD07의 CLI → WASM → Studio 전체 적용을 따른다.
  아래 v2 계획에서 제품 변경·한도·격리와 현행 graph의 실제 Cargo/독립 소비를 연결한다.
  8개 package의 과거 소비 결과를 추가 코덱까지 검증한 결과로 확대하지 않는다.
- [재개 점검과 남은 게이트](../working/task_m100_8174_stage2_corrections.md#2026-10-24-재개--코덱-opt-in-이후의-게시-후보)를
  현재 진행 위치로 사용한다. 아래 1·2단계의 기존 8개 설명은 최초 수행 범위의 이력이다.

## 2026-10-24 정책 선택과 보안 경계 조사

MD05=B는 **비신뢰 PDF 업로드의 공식 지원을 목표로 감사·격리 검증을 확대**하는 결정이다.
추가 답변으로 MD06의 F01–F08 전체 기본 탑재와 MD07의 CLI → WASM → Studio 전체 적용이
확정됐다. 이 문단은 구현 범위 v2의 조사 근거이며 제품 구현·보안 검증 완료를 뜻하지 않는다.

현재 code `3536e72819`에서 다음 진입점과 재사용 후보를 정적으로 확인했다.

| 계층 | 현재 경로·기존 근거 | 다음 범위에서 확인할 계약 |
| --- | --- | --- |
| Rust frontend | [`parse_pdf_with_policy`](../../src/parser/pdf/mod.rs), [`PdfImportPolicy`](../../src/parser/pdf/policy.rs)는 요청 한도와 사용량을 제공한다. 직접 policy를 받는 경로와 기본 policy 경로가 구분된다. | 새 프로필이 입력 복사 전 검사·실제 decoder·추출/수집까지 같은 요청 정책으로 소비되는지 확인한다. #7911 B01–B05의 적용 경계·미검증을 먼저 대조한다. |
| Core·CLI | [`DocumentCore::from_pdf_bytes`](../../src/document_core/commands/document.rs) → 공통 parser의 기본 정책 경로다. [`import-pdf`](../../src/cli/commands/pdf_import.rs)는 기존 입력 상수로 읽기를 제한하며 프로필 옵션은 없다. | API/CLI 선택이 실제 Core 소비까지 전달돼야 한다. parser 반환 후 Core 초기화·조판·저장도 지원하는 사용자 여정과 격리 수명에 포함한다. 오류·자원 초과 시 기존 출력 파일과 다음 요청을 보존한다. |
| WASM 옵션 | [`HwpDocument.importPdf`](../../src/wasm_api.rs), [JS options 경계](../../src/wasm_api/pdf_import.rs)는 동기 호출이며 `sourcePages`·`includeTableInference`만 받는다. 기본 policy getter는 별도다. | CLI 다음 필수 환경이다. 프로필 검증·실제 적용·입력 복사 전 상한을 같은 policy에 연결한다. 옵션 파싱 성공만으로 중단·격리를 입증하지 않는다. |
| 브라우저 Worker·Studio | [공통 Worker session](../../rhwp-shared/pdf/import-session.js)은 취소·deadline·후보/활성 세션 분리를 제공한다. [Studio 열기](../../rhwp-studio/src/core/wasm-bridge.ts)는 직접 동기 constructor를 사용한다. | 기존 #8183 실행 증거를 재사용하되 새 프로필과 자원 소진 입력의 종료·재생성·정상 후속 요청을 검증한다. Worker의 시간 제한을 프로세스 전체 메모리 상한으로 표시하지 않는다. |
| Native 감독 참고 | [font metric 검증용 supervisor](../../scripts/font_metric_coverage_supervisor.mjs)와 [계약 검사](../../scripts/tests/font_metric_coverage_supervisor.test.mjs)에 Linux의 별도 프로세스·주소 공간/CPU 제한·wall timeout 처리가 있다. | 제품 PDF 경로와 다른 도구이므로 제품 격리 검증 완료로 세지 않는다. 적용 OS·메모리 지표·입출력/진단의 경계·자식 종료·실패 후 복구를 정한 뒤 재사용 범위를 판단한다. |
| 포장 후 소비 | #8181의 기존 8개 Cargo package 소비, #8183의 source 기반 코덱 소비와 현행 12개 source 검사 근거가 있다. | 새 보안·프로필 계약을 실제 포장한 12개 후보의 Native/WASM/CLI에서 재검증한다. 서로 다른 source와 검증 종류를 합쳐 완료로 표시하지 않는다. |

위 표는 코드 조사 결과이며 새 결함을 실행으로 재현한 목록이 아니다. 다음 구현 범위에는
[기존 B01–B05·B07](task_m100_7911_backlog.md)의 원 해결·통합·후속 보정과 현재 호출 경로를
대조한 결과, 필수 정상/한도/실패 후 복구 사례, 환경별 격리 종료 조건을 연결한다.
기능·환경 범위의 결정은 끝났다. 프로필 수치·상한은 아래 절편에서 측정해 결정하며,
현재 실행 코드의 기본값·제한은 제품 구현이 반영될 때까지 그대로다.

## 범위 v2 — 기본 기능과 환경별 자원 정책

### 결정과 구현 계약

F01–F08을 표준 배포의 기본 기능으로 제공한다. 사용자가 추가 feature나 별도 빌드를 알아내야
기본 문서를 처리할 수 있는 상태를 해소한다. MD07은 **CLI → WASM → Studio** 순서이며,
Rust 라이브러리에 공통 정책과 기능 결과를 두고 각 제품이 소비한다. 기능별 opt-in 유지,
Studio 제외 또는 CLI만 완료하는 안으로 범위를 축소하지 않는다.

다음은 사용자 결정에 따른 **구체적인 구현 계획**이다. 기능·환경은 확정했고, 아래 API 이름과
프로필 구성·실행 계약은 2026-10-24 「v2 계획 승인·CLI부터 구현 진행」 답변으로 승인받았다.
[Issue Workflow](../manual/codex/docs_and_git_workflow.md#issue-workflow)의 계획 → 승인 → 구현·테스트
순서로 V2-S01을 진행한다. 계획 승인은 구현·보안 검증 완료와 구분한다.
각 절편의 변경·검증을 커밋하고 다음 단계 문서를 시작한다.

| 절편 | 포함 작업과 실제 변경 경계 | 완료 조건 |
| --- | --- | --- |
| V2-S01 — CLI·공통 정책 | `PdfImportPolicy`의 검증·프로필 선택, Core의 policy 전달 API, `import-pdf`의 입력 읽기 전 한도와 옵션·오류/출력 계약. #7911의 할당·font/cache 선행 분석과 제품용 Native 감독·중단 경계 | 정상 PDF 가져오기·조판·저장, 한도 직전/초과·중복/오류 옵션·시간 초과·자식 종료, 기존 출력 보존과 다음 정상 요청. 유효 정책과 실제 사용량·측정값 제시 |
| V2-S02 — 기본 기능·Native | F01·F02 기본 feature 전환, F03 ICC의 실제 변환 연결, F04 예산 있는 xref 복구, F05 SVG 글리프 공통 표현, F06 링크 보존·목적지 분류와 출력. 일반 입력의 성능·메모리 측정 | 별도 feature 없이 JBIG2/JPX/ICC/복구 입력 처리. 색·alpha·내용/목적지 보존, Native 출력과 독립 기준 대조, 정상·손상·초과 입력별 결과. F05·F06의 브라우저 소비는 후속 절편에서 완료 |
| V2-S03 — WASM | 같은 프로필·한도를 JS API/공통 Worker에 연결. F01–F06의 WASM 변환·렌더링·링크 결과와 실행 capability 계약 제공 | fresh WASM의 기본 빌드에서 기능 동작, 복사 전 한도, 취소·deadline·메모리 실패 후 Worker 종료/재생성과 정상 후속 요청. Native와 같은 입력의 의미·출력 대조 |
| V2-S04 — Studio | 직접 동기 PDF 열기를 중단 가능한 Worker 경로에 연결하고 프로필의 유효값·진행/취소/재시도를 UI에 반영. F05·F06 실제 표시·클릭, F07 명령 단추, F08 보호 문서 복구 | 일반 열기·편집·저장 보존, 무응답/실패 중 기존 문서 유지. 실제 단추 동작과 링크 결과, 암호화 draft·재시작·오입력/변조·삭제 경합을 브라우저에서 검증 |
| V2-S05 — 포장·독립 소비 | 변경 뒤의 전체 package graph·고지·기본 feature·빌드/사용 가이드, 실제 Cargo package·독립 Rust/CLI/WASM 소비 | 최종 source의 실제 포장·registry 소비와 지원 여정 통과. 과거 12개 source 검사나 기존 8개 소비로 대체하지 않음. 새 base의 JPEG encoder와 S02의 qcms를 포함한 14개 후보를 실제 검증 |

V2-S01부터 순서대로 진행한다. S02에서 공통 모델을 준비하더라도 WASM·Studio의 제품 적용
완료는 S03·S04의 실제 실행으로 판단한다. 각 절편의 제출 여부는 해당 절편의 필수 검증과
승인으로 정하며, F01–F08·세 환경·MD05의 필수 항목이 남으면 v2 전체를 완료하지 않는다.
작업량을 줄이려고 필수 기능을 후속 이슈로 옮기지 않는다.

### CLI부터 연결할 공통 자원 정책

1. 기존 `PdfImportPolicy`를 유일한 유효 정책으로 삼는다. 프로필 후보는 `standard`와
   `constrained`이며, 전자는 일반 문서 처리, 후자는 작은 자원 환경의 유한한 정책이다.
   프로필 이름으로 입력의 신뢰도를 판정하지 않는다. 입력/쪽/픽셀/decoded bytes/중첩/객체·참조/
   font·cache/codec 작업량의 단위·누적 범위를 표로 고정하고 누락된 실제 할당 경로를 감사한다.
2. CLI 옵션안은 `--resource-profile`과 `--resource-limits <JSON 파일>`이다. 기존 출력의
   `--profile print`와 구분한다. Rust는 policy를 받는 Core 진입점, WASM은 `resourceProfile`·
   `resourceLimits`를 소비하는 방향으로 연결한다. 기존 호출은 표준 기본값을 유지하고
   미지 키·0·음수·overflow·상한 초과와 중복 옵션은 변환 전에 거부한다.
3. 사용자별 수치는 프로필의 검증된 상한 안에서 조정한다. 새 한도를 무제한으로 두거나 기존
   상한을 근거 없이 올리지 않는다. 현재 64 MiB 입력·32 MiB codec 단일 예약과 쪽별로 좁히는
   decoded 예산은 측정의 시작점이다. 이 값만으로 정상 문서 수용·전체 RSS를 보증하지 않는다.
4. `CLI 읽기 → Core → parser → backend/decoder → IR 수집 → 조판 → 저장`에 같은 유효 정책과
   요청 수명을 전달한다. 입력을 먼저 전부 읽거나 복제한 뒤 한도를 검사하는 경로를 남기지 않는다.
   형식별 HWP5/HWPX의 기존 한도도 조사해 공통 조정 표면의 적용 범위와 단위를 명시한다.
5. 별도 프로세스 감독은 라이브러리 내부 카운터와 분리한다. CLI에서는 parse·조판·저장까지
   감독 수명에 포함하고 wall timeout·자식 프로세스 종료·출력 크기·임시 파일·원자적 교체를
   검증한다. 주소 공간 제한·RSS 관측·CPU 시간은 서로 다른 지표로 보고한다. 지원 OS별 수단을
   조사하고 실제 제한이 없는 환경을 격리 완료로 표시하지 않는다. 파일·네트워크·자격 증명
   접근 경계도 감사하며, 단순 자식 프로세스 생성만으로 보안 격리를 주장하지 않는다.
6. WASM/Studio는 복사 전 입력 검사와 Worker 종료·재생성을 연결한다. 전체 편집 API를 현재
   Worker의 inspect/render/export 세 메서드만으로 대체할 수 있는지 먼저 대조한다. 불가능하면
   가져오기 후보를 Worker에서 검증·직렬화하고 성공한 결과를 원자적으로 여는 경로를 설계한다.
   재직렬화·재열기의 조판과 할당도 검증 범위에 포함한다. Worker 종료는 실행 중단 수단이며
   브라우저 전체 메모리 상한을 의미하지 않는다.

수치는 정상·경계 입력의 변환 시간, peak RSS 또는 WASM 선형 메모리와 입력 복사량,
출력 크기를 고정 source·동일 조건의 base/후보에서 측정해 제안한다. 프로필 적용 때문에
기존 정상 여정이 실패하면 조용히 일부 내용만 가져오지 않고 원인·상한·실제 사용량을 비교해 보정한다.

### F01–F08의 구현과 검증 계약

- **F01·F02:** [#8183 결과](../working/task_m100_8183_stage3.md)와 실제 merge `e7a0dec55`를
  먼저 대조한다. 표준 `Cargo.toml`·CLI/WASM 빌드와 지원 조회를 함께 바꾸고 예산 연결을
  유지한다. 명시적인 개발용 제외 빌드의 거부도 보존한다. JPX Indexed+내장 mask·CMYK+alpha 등
  잔여 조합의 실제 입력·기대 출력을 조사해 지원 확대를 구현·검증하고, 불명확한 조합을 모든
  JPEG 2000 지원으로 표시하지 않는다. #8183의 두 alpha Print 결함에 대한 메인테이너 판단은 유지한다.
- **F03:** backend에는 `icc-qcms`·`icc-lcms2`와 이미지의 `build_icc_transform`이 이미 있다.
  CMM 후보는 Native/WASM의 동일 지원을 우선해 비교하되, feature를 켜는 것에 그치지 않고
  실제 예산 경로의 JPEG/raw/Indexed → 색 변환 → 저장 픽셀 소비까지 추적한다. profile 검증·
  transform cache·CLUT 할당·실패 전파와 alpha 보존을 감사하고 ICC 기준 출력·시간·메모리를 측정한다.
- **F04:** 현재 `xref_reconstruction`의 전체 `read_to_end`·regex scan·객체 수집을 그대로
  제한 경로로 연결하지 않는다. 유한한 scan/객체/참조/stream·예약 한도를 받는 복구 결과를
  기존 constructor가 소비하게 한다. 문자열·주석·stream 안의 거짓 object, 잘린 trailer,
  incremental update와 중복 참조, 한도 초과를 검증한다. 복구 성공 시 내용·쪽·진단을 보존한다.
- **F05:** 현재 공통 정적 판정·Native replay·CanvasKit 후보/Studio parser의 차이를 줄이고
  gradient·clip·opacity·내부 참조·내장 이미지 등 정상 SVG 글리프 표현을 확장한다.
  [OpenType 규격](https://learn.microsoft.com/en-us/typography/opentype/spec/svg#svg-capability-requirements-and-restrictions)은
  gradient·clip·opacity를 요구하고 `script` 등은 무시하도록 정한다. 규격의 금지와 현 helper의
  구현 부족을 구분해 기록한다. 글리프 전체를 버리는 처리 대신 유효한 artwork를 보존하며,
  외부 자원은 공통 resolver·예산·host 경계를 설계해 검증한다. 일반 SVG와 OpenType SVG의
  계약도 구분한다. 이 안을 임의 문서 JavaScript 실행의 승인으로 표시하지 않는다.
- **F06:** `command_uri`의 폭넓은 읽기와 `web_command`·`export_uri`·Studio 클릭의 좁은
  허용 범위를 공통 목적지 모델로 연결한다. 웹·메일·문서 내부 목적지·상대/로컬 파일·등록 앱
  링크를 원본 형식에 따라 분류하고 왕복·PDF 목적지·사용자 클릭을 검증한다. host의 파일/앱
  handler와 브라우저가 직접 열 수 없는 경우의 처리·권한·실패 안내를 제품 동작으로 구현한다.
  외부 실행은 실제 사용자 동작에서 요청하며, 링크 클릭을 문서 열기 시 자동 실행하지 않는다.
- **F07:** `FormObject.properties`와 보존한 script 원본에서 실제 button action을 추적한다.
  HWP/HWPX 파서 → IR action/원본 보존 → WASM 결과 → Studio dispatcher를 연결하고,
  실제 문서의 동작을 실행한다. 필요한 script 호환 runtime·host capability를 조사해 연결하며
  무제한 `eval`이나 shell 문자열 실행을 사용하지 않는다. 실행 시간·메모리·동시 요청 한도,
  readonly/보호 상태·undo·취소·실패 후 편집을 검증한다. 팝업 알림이나 성공 이벤트만으로
  원본 버튼의 업무 동작을 대체하지 않는다.
- **F08:** #5992의 평문 차단 원인·반영·검증을 먼저 추적한다. 보호 문서는 암호화된 복구본을
  기본 생성하고 다시 열 때 해제 절차를 제공한다. 기존 보호 저장 경로를 재사용할 수 있는지
  확인한 뒤, 별도 envelope가 필요하면 인증 암호화·암호 기반 키 도출·새 nonce와 버전·문서
  식별자를 설계한다. 키를 암호문과 함께 영구 저장하지 않고 재시작 때 사용자가 해제한다.
  [Web Crypto API](https://www.w3.org/TR/2017/REC-WebCryptoAPI-20170126/#aes-gcm)는 구현 후보의
  API 근거이며 완성된 키 관리의 증거가 아니다. 비밀번호 변경·잠금·폐기·진행 중 저장 경합,
  오입력·변조·재시작 복구에서 평문 잔류와 문서 누락이 없는지 검사한다.

### 공통 검증과 제출 조건

- 변경 전에 원 해결 PR → 실제 통합·보정 SHA → 독립 검증 → 현재 호출 경로를 각 기능별로
  연결한다. 위 정적 조사와 공식 규격 확인을 현재 코드의 실행 성공으로 기록하지 않는다.
  #7911 B01–B05·B07은 적용 경계별 감사를 완료하고 부족한 증거를 실제 실행으로 보완한다.
- 각 F 항목에 정상 사용자 여정, 정확한 한도/실패 반례, 실패 뒤 다음 정상 요청과 기존 기능
  보존을 둔다. 자동 수집되는 기존 suite와 Native/Studio 실행 경로를 재사용한다.
- 렌더링·색·배치 영향은 동일 입력·독립 기준 출력의 전쪽 Native/fresh WASM TSV, 쪽수·내용
  보존과 직접 확대 판독을 완료한다. 90% 미달·측정 불가·내용 누락은 제출 보류 조건이며
  이미 결정한 #8183 두 alpha 기준 결함은 해당 범위에서만 기존 근거를 재사용한다.
- 기본 feature·profile·격리·Worker 변경의 cold/warm 시간·메모리·산출물 크기를 구분해 측정한다.
  성능 근거 없이 기능을 비활성화해 기본 제공 결정을 바꾸지 않는다. 실패 시 기존 문서/파일
  보존과 재시도까지 완료한 뒤 사용자 가이드·보안 현황·package 지원표를 실행 결과에 맞춰 갱신한다.
- local_validation 4.3과 해당 보조 절차의 필수 lint·회귀·fresh WASM/Studio 검증을 절편별로
  수행한다. CI·게시·소유권·최종 버전 결정은 MD01–MD04·MD08–MD09의 경계를 따른다.

## 1단계 — 실제 source에서 계약과 패키지 범위 확정

1. Cargo manifest와 `src/lib.rs`의 공개 소비 경로에서 graph·feature·버전·소유 경계를 기록한다.
   내부 crate를 본체로 흡수하는 대안과 별도 게시를 비교하고, 기존 재사용·보안 경계를 보존하는
   별도 게시를 우선안으로 구체화한다. 이 단계에서는 제품 manifest·기능을 바꾸지 않는다.
2. 현재 Cargo 기본 패키지 목록을 재산출하고, 별도 게시 후보 8개 각각의 source 파일 목록을 선별한다.
   원본 경로·Git blob과 compile-time include를 대조한다. 실행용 문서·템플릿·메트릭·폰트와
   고지는 보존하며 대형 검증 corpus와 작업 기록은 제외한다.
3. 선별 source를 실제 gzip tar로 포장해 파일 수·원본/압축 크기·SHA-256을 측정한다.
   이 archive는 **정규화 전 source 후보**로 명시한다. unpublished dependency와 manifest
   전환을 마치지 않은 source archive를 Cargo 검증 완료 `.crate`로 부르지 않는다.
4. 현재 manifest의 `cargo package` 차단점을 확인한다. 임의로 버전·publish flag를 풀거나
   보안 patch를 제거하지 않고, 2단계에서 바꿀 정확한 파일과 이유를 남긴다.
5. [1단계 기록](../working/task_m100_8174_stage1.md)에 graph·사용자 여정·실측·미완료 게이트를
   기록한다. 이번 단계에서 실제 실행하지 않은 소비자 빌드를 PASS로 표시하지 않는다.
6. 추가 요청의 [보안 제한 현황](../manual/security_feature_limits.md)을 코드·feature tree와
   대조한다. 제한·자원 한도·일반 미지원을 구분하고 현황 정리 중에 feature를 활성화하지 않는다.

제품 소스·Cargo.lock·배포 workflow가 그대로이면 해당 변경의 전체 Rust/WASM 재빌드는
이 단계 문서·source 목록 검사의 완료 조건이 아니다. 이후 실제 package 소비는 생략하지 않는다.

## 2단계 — 게시용 manifest와 독립 소비

### 공통 선행 절편 — C01–C04

1단계 임시 측정을 정식 재실행 가능한 source 사전 검사로 옮긴다. Git source, 후보 graph,
필수 내장 파일·라이선스/고지, 제외 자료, archive 목록·크기·해시와 manifest 차단점을 확인한다.
지원 정책을 선택하지 않고도 도구와 진단을 준비하며, 실제 Cargo package·외부 소비 검증과
구분한다. 로컬 실패 경계 검사와 실제 저장소 실행을 완료하고 2단계 기록에 결과를 남긴다.
이 절편의 완료 결과와 남은 경계는 [2단계 공통 기록](../working/task_m100_8174_stage2.md)을 따른다.

### 정책 선택 뒤의 구현

아래 항목은 1단계 권고안에 따른 후보 목록이다. 관련 MD 결정과 범위 버전·단계 진행 승인을
기록한 뒤 선택된 항목을 구현한다. 미결정 항목에 의존하는 변경을 먼저 시작하지 않는다.

- 본체의 3개 path 의존성에 registry 버전을 병기하고 별도 PDF export fork의 package 이름을 연결한다.
  내부 crate 0.1.0, 보안 backend/decoder의 upstream 기반 보정 버전과 본체 0.9.0은 구분한다.
- 승인된 패키지별 include·README·metadata·라이선스 경로를 반영한다. publish flag 해제는
  패키지 검증과 승인된 범위를 확인하고 수행한다. 이 단계도 실제 업로드를 하지 않는다.
- 미게시 source는 저장소 밖 임시 registry에 실제 포장해 dependency를 검증한다.
  repository path fallback이나 사용자 Cargo cache 수정으로 registry 소비를 대신하지 않는다.
- core HWP/HWPX 왕복, 기본 PDF 입력, CLI 기본/Native 기능, WASM `wasm-api` on/off 소비,
  docs.rs와 feature 조합을 고정 source에서 실행한다. MSRV는 실제 검사한 버전만 선언한다.
- 게시용 README에 보안 제한을 반영하고 PDF 미지원 codec·암호·손상 xref·자원 초과의 거부,
  실패 후 정상 요청을 실제 소비자에서 검사한다. backend upstream 기능 목록을 그대로 지원표로 쓰지 않는다.
- 전체 소스·의존성을 바꾸는 후보는 local_validation 4.3의 해당 lint·회귀·fresh WASM 게이트를
  완료한다. PDF 보안 한도와 기존 수정 출력도 소비자 쪽에서 대조한다.
- 새 devel의 #7488 opt-in cdylib-only와 #7474 standard release 산출물 동일성 계약을 유지한다.
  package 검증용 임시 manifest가 root 개발 manifest를 덮어쓰지 않도록 한다.

## 3단계 — 게시 운영과 릴리즈 인계

MD01·MD08·MD09와 확정한 범위에 따라 수동 게시 준비 또는 자동화 구현을 선택한다.

- exact SHA/version/digest preflight, 자식→부모 게시·이미 게시된 항목 조회·부분 실패 복구를 마련한다.
- 실제 registry 대상 부모 dry-run은 자식이 아직 없으면 완료할 수 없음을 기록한다.
  사전 임시 registry 검증과 실제 게시 승인 뒤의 최종 dry-run을 구별한다.
- 초회 수동 게시와 이후 OIDC 연결, owner·권한·yank 절차를 준비한다. 실제 인증 설정과 게시를
  단계 검증의 이름으로 실행하지 않는다.
- workflow·운영 script 변경은 github_operations의 비례 검증을 적용한다. 기존 네 채널을
  성공으로 오판하거나 새 채널을 태그 생성만으로 활성화하지 않도록 계약을 확인한다.

PR 준비·실제 게시 완료·제품 릴리즈 완료는 각각 해당 검증과 승인 이후에 판정한다.


## 2026-10-25 현재 진행 위치

V2-S01–S05의 로컬 구현·절편별 필수 검증을 완료했다. [S04 실행 기록](../working/task_m100_8174_v2_stage4.md)에
제품 구현·전체 회귀·Studio·성능을, [S05 실행 기록](../working/task_m100_8174_v2_stage5.md)에 현행
14개 실제 `.crate`·독립 Rust/CLI/WASM 소비·새 98페이지 시각 대조·가이드를 연결한다.
S05의 검증 source는 `9c9c272fe1670baf305800dfc906349fa7933d4a`다.

사용자 「다음 절편 진행을 승인합니다.」에 따라
[S06 조사·실행 재현](../working/task_m100_8174_v2_stage6.md)을 진행했다. font cap 1에서
Type1/CFF map 확장의 성공, Unicode memo의 무계상 복사와 직접 API의 sticky hit를 재현했다.
JPEG worker H의 실제 capacity 증가 예약은 Native/WASM 각각 10회 실행해 거절·복구를
확인했고, 독립 decoder가 받는 mixed progressive 6개의 기존 호환 실패도 기록했다.

다음 보정 순서는 S06의 [우선순위와 완료 조건](../working/task_m100_8174_v2_stage6.md#보정-순서와-완료-조건)을
따른다. B01·B02 선계상/오류 전파 → B03·B04의 잔여 복사·index·frontend/Core/직렬화 →
B07 실제 confinement·재열기 경계 → 좁힌 의존성 갱신·progressive 호환 보정 순서다.
기존 MD05 지원 목표와 F01–F08·세 환경의 범위를 축소하지 않으며, 조사 완료를 제품 보정이나
전체 보안 감사 완료로 기록하지 않는다. MD01·MD02–MD04·MD08–MD09는 2026-10-25 답변으로 확정했다. 최종 graph·버전·소유권·게시 운영
계약의 실제 검증은 남아 있다. 절편별 로컬 완료를 전체 v2의 보안 지원·실제 crates.io 게시 준비 완료로
확대하지 않는다.

사용자의 다음 절편 승인으로 [S07](../working/task_m100_8174_v2_stage7.md)에서 B01·B02의
Type1/CFF·glyph name·Unicode memo 계상을 보정했다. code head `0ca4dac500`의 Native/WASM
11개 계약, 실제 CLI·Worker·Studio 거절·복구, 전체 회귀 11,616건과 Native Skia library
4,045건이 통과했다. 성능 비용과 원본 PDF의 기존 문자 폭·줄바꿈 결함도 실제 실행으로
확인했다. 전쪽 수치 통과를 시각 완료로 바꾸지 않으며 **시각 완료·PR 제출은 보류**한다.
다음 P0의 B03·B04, B07 및 기존 출력 결함 보정·재판독은 계속 필요하다.

이후 사용자가 제공한 PDF의 이미지 clip 오류를 기존 v2의 정상 입력 호환 경로에서 추적했다.
[S08](../working/task_m100_8174_v2_stage8.md)의 실제 원인은 연속 `m`의 앞선 빈 이동을
복합 clip으로 오판한 공통 tracker다. 같은 승인 범위에서 사양에 맞는 경로 구성과
사각·볼록·Form 정상 대조, 실제 복합 경로 거절을 보정·재검증한다. 미구현 곡선/복합 clip
전체 지원이나 기존 S07 시각 보류를 완료로 바꾸는 범위가 아니다.
clip 보정 뒤 드러난 operator의 단독/다중 쪽 예산 불일치도 MD07의 같은 공통 정책에서
보정한다. 1쪽당 허용량을 보존하고 profile·명시 host cap의 절대 상한을 유지하며, 실제
카운터와 정상 단독/batch·낮은 한도 거절을 함께 검증한다.

2026-10-25 사용자는 [operator 상한 검토](../working/task_m100_8174_operator_ceiling_review.md)의
권고안을 수용했다. 공통 operator Vec의 capacity 증가 전에 기존 단일 버퍼 한도를 적용하고,
그 뒤 standard operator만 8,000,000으로 확대한다. constrained 250,000·선택 쪽당 250,000·
명시적인 낮은 host cap과 다른 독립 한도는 유지한다. 진단 소비자의 800만 통과를 제품의
새 기본값 검증으로 재사용하지 않고 Native CLI·최적화 제품 WASM·Studio에서 실제 한도·
거절·기존 상태 보존·복구와 비용을 다시 확인한다. [PR 제출 전 잔여 작업](../working/task_m100_8174_pr_remaining_todo.md)에
S08·S07 보류, B03·B04·B07과 의존성·호환 보정, 최종 source 검증·게시 결정을 연결했다.
