---
kind: snapshot
status: active
canonical: mydocs/working/task_m100_7911_stage3.md
last_verified: 2026-10-16
---

# #7911 3단계 — 제품 기능 경로 연결

## 1. 승인 범위와 동기화

- 사용자 지시: 원격 devel 동기화 후 다음 작업 진행. 구현 순서는 [구현계획](../plans/task_m100_7911_impl.md)의 F1–F4를 따른다.
- devel 승격은 기여자가 이어 만들 수 있는 실행 가능한 기반의 통합이다. 세부 보완은 [백로그](../plans/task_m100_7911_backlog.md)에 남기며 전체 보안 감사·배포 준비를 기능 연결의 선행 조건으로 되돌리지 않는다.
- 2단계 checkpoint `966af9575`: 기존 보완 코드·검사·계획을 보존했다. 기존 source/fixture/dependency 2,841개 해시는 보존된 baseline과 일치했다. vendor upstream의 기존 trailing whitespace 9줄은 그대로 보존했다.
- fetch한 `upstream/devel` = `f5251e7de4e31ee0fb939f352c6ef144bb340bdb`. 기본 작업공간 devel은 160개 커밋 fast-forward 후 ahead/behind `0/0`이다. `origin/devel`도 fetch해 같은 SHA와 `0/0`을 확인했다.
- 기본 작업공간의 수정된 `rhwp-studio/public/rhwp.js`는 별도 백업·stash로 보존하고 원래 바이트를 복원했다. 복원본이 최신 tracked 파일과 같아 작업공간은 clean이다. stash와 ignored 백업은 유지한다.
- 작업 브랜치 merge = `672acadd7d2309e5c0294a98c2142e240ed6f1a9`. 유일한 충돌 `mydocs/orders/20261015.md`는 #7911·#7661·#7923 기록을 모두 보존해 해결했다. 코드 충돌은 없었다.
- 원시 기록: `output/pr-review/issue7911/devel-sync-20261016/`. 이전 base의 성공 결과를 새 통합본의 최종 검증으로 재사용하지 않는다.

## 2. F1 공통 Parser/Core 연결

F1 구현·집중 검증 완료. `%PDF-` 감지 → 공통 ParsedDocument와 원본 쪽 대응 metadata → 공통 provenance 봉인 → Core의 기존 fixup·compose·paginate·재봉인을 연결한다. 선택 입력도 같은 초기화 함수를 소비한다. HWPX 직렬화·재파싱 우회나 PDF용 initializer 복사는 하지 않는다.

- 기본 feature-off: PDF 감지는 유지하고 `PDF_IMPORT_DISABLED`로 거부한다. 서비스 계층에서도 손상 문서 `Parse`가 아닌 `UnsupportedFormat`으로 분류한다.
- feature-on: 자동 진입점은 전체 쪽, 명시적 옵션은 증가하는 원본 쪽 선택을 처리한다.
- 임시 문단 정책·미지원 진단과 `parser_allocation_bounds_proven=false`를 보존한다.
- 새 합성 검사는 API/IR·내용 보존 계약이다. 한컴 시각 일치 또는 최종 제품 전체 검증으로 보고하지 않는다.
- F2 선택 가져오기 CLI·도움말/diagnostics, F3 공개 WASM/Worker, F4 실제 문서 편집·저장·재열기 및 제출 범위 검증은 다음 작업이다. 기존 CLI의 전체 PDF 입력은 공통 로더 연결에 따라 실행 확인했다.
- 새 `FileFormat::Pdf`는 feature-off에서도 존재한다. 공개 enum의 exhaustive match와 feature-on `ParsedDocument` struct literal 소비자는 새 항목을 처리해야 한다. 기존 함수 서명은 유지했다. PDF용 HWP 버전은 만들지 않고 info에서 null로 보고한다. DocLang·HWP 전용 roundtrip 진단은 PDF를 미지원/skip으로 명시하며 지원 범위를 넓히지 않는다.

## 3. 검증

실행 명령·결과는 `output/pr-review/issue7911/product-integration-20261016/`에 보존한다.
공통 Native target은 기본 작업공간의 `target/pr-review`, Docker는 기존 `rhwp_wasm-target`이다.

| 범위 | 결과 | 근거 |
| --- | --- | --- |
| PDF frontend·공통 Parser/Core·선택/저장·오류 복구 | 최종 오류 분류 보정 후 8건 통과 | `issue_7911_pdf_frontend.log`, `frontend-final.log` |
| feature-on 포맷 감지 | 1건 통과 | `issue_7911_pdf_detection.log` |
| 기존 PDF 자원·명시 여백 계약 | 111 + 1건 통과 | `issue_7911_pdf_resource_limits.log`, `issue_7911_explicit_text_margin.log` |
| 기존 HML 파서·Core 계약 | 38건 통과 | `hml_parser.log` |
| 기본 feature-off Parser/Core/Service | 2건 통과; 미지원 오류 분류와 안내 확인 | `default-detection-final.log` |
| 기본 CLI PDF 내보내기 거부 | exit 1, `PDF_IMPORT_DISABLED`, stdout 빈 값, 산출물 없음 | `default-cli.json` |
| 실제 12쪽 PDF → 제품 CLI info/HWPX/HWP | 세 명령 exit 0; 저장 전후 모두 12쪽. info는 `format=pdf`, `version=null` | `real-cli-smoke-final.json`, 각 final stdout/stderr |
| 같은 12쪽의 기존 frontend/새 제품 HWPX | SHA-256 `22bee8064e0d88223242f5ed61069ee49879229f6dee53add334596960750b82` 동일 | `real-hwpx-preservation.json`, `real-cli-smoke-final.json` |
| 기존 Core 초기화 순서 | PDF metadata 대입 외 기존 본문 동일 | `initializer-preservation.json` |
| Native lib Clippy | default/pdf-import 모두 `-D warnings` 통과 | `native-{default,feature}-clippy.log` |
| Docker WASM lib Clippy | default/pdf-import 모두 `-D warnings` 통과 | `docker-wasm-clippy-final.log` |
| fmt·integration manifest·unit tier | 통과, 정책 base `f5251e7de4e31ee0fb939f352c6ef144bb340bdb` | `fmt-check.log`, `manifest-check.log`, `unit-policy-check.log` |

집중 검사는 `node scripts/run-rust-test.mjs --cargo-test <case> -- --locked --profile release-test --target-dir /home/edward/mygithub/rhwp/target/pr-review [--features pdf-import]`로 실행했다. 기본 빌드의 오류 분류를 마지막에 보완한 후 해당 default 검사와 양쪽 target lint를 다시 실행했다. 변경되지 않은 자원·여백·HML 검사는 동기화 후 같은 단계에서 얻은 결과를 재사용한다. 전체 회귀·workspace/all-targets·새 제품 WASM/Worker 실행·전쪽 시각 판정은 F3–F4에 남아 있다.

SVG 내용 검사는 실제 출력이 글자별 `<text>`를 사용함을 확인한 뒤 XML 텍스트의 순서·내용을 검사한다. 원시 SVG에서 단어 한 덩어리를 찾던 첫 실패는 검사의 가정 오류였으며 렌더링 결함으로 집계하지 않았다. 새 합성 API 계약에 수정 전 제품 결함 FAIL 증거가 있다고 주장하지 않는다.

실물 입력은 저장소의 `pdf/issue5875/nested_table_text_caption-2020.pdf`, 제품 출력은 위 증적 디렉터리의 `real-all-final.hwpx`·`real-all-final.hwp`다. 이 검사는 제품 연결·내용 보존의 실행 확인이며 새로운 한컴 시각 gate 통과나 전체 보안 감사 완료가 아니다. 원격 push·PR 상태 변경은 수행하지 않았다.

문서 메타데이터 전체 검사에는 upstream/devel과 바이트가 같은 기존 문서 4개의 필수 필드 16개 누락이 있다. 이번 새 단계 보고의 메타데이터는 통과했다(`doc-metadata-scope.json`). F1 코드 결함과 구분하며 B11의 후속 제출 검증에서 이 기존 상태를 확인한다.

## 4. F1 완료 경계와 다음 작업

- 최종 source 변경 목록·SHA-256은 `source-final.json`, 이 소스와 같은 로컬 commit은 `completion.json`에 고정한다. 코드가 바뀌지 않은 자원·HML 계약을 포함해 feature 구성별 집중 검사 161건이 통과했으며, 마지막 오류 분류 변경의 영향 경로는 최종 Native 검사와 양쪽 target lint로 다시 확인했다.
- Native lib lint는 `cargo clippy --locked -p rhwp --lib [--features pdf-import] --target-dir /home/edward/mygithub/rhwp/target/pr-review -- -D warnings` 두 구성을 순차 실행했다. WASM lint는 `docker compose -p rhwp --env-file .env.docker run --rm --no-deps -e CARGO_BUILD_JOBS=4 wasm` 안에서 같은 두 구성에 `--target wasm32-unknown-unknown`을 적용했다. 실제 제품 WASM 생성·브라우저 실행 검증은 F3의 작업이다.
- 다음은 F2 `import-pdf` 선택 페이지 CLI·도움말·JSON 진단/종료/원자적 저장 계약이다. 이후 F3 공개 WASM/Worker와 F4 실제 편집·저장 및 제출 범위 검증을 연결한다. B01–B12 전체 소진을 이 순서의 선행 조건으로 삼지 않는다.
- 이번 완료는 F1이며 3단계 전체 완료·PR Ready·배포 준비 완료 판정이 아니다.


## 5. F2 선택 페이지 CLI (2026-10-16)

사용자의 다음 작업 승인으로 F2를 구현했다. `import-pdf <입력.pdf> <출력.hwpx|출력.hwp>`는
기존 F1 `DocumentCore::from_pdf_bytes`를 소비하며 PDF 추출·인식·lowering·조판 로직을
변경하지 않는다. frontend 상한 두 개만 공개해 CLI의 입력 읽기·범위 펼치기도 같은 값을 쓴다.

- `--source-pages 0,6-7`: 0-based 원본 쪽의 엄격히 증가하는 목록/포함 범위다. 생략하면 전체.
  중복·역순·빈 항목·정수/범위 오류는 exit 2이며, 펼치기 전 개수와 overflow를 검사한다.
  Core의 기존 `InvalidFile` 문자열에 봉인된 `PDF_PAGE_SELECTION` code만 사용법 오류로
  분류한다. 나머지 처리 실패는 exit 1이다. 기존 exit 3/4는 그대로다.
- 옵션 오타·중복·암호 옵션은 거부한다. 원본의 같은 경로·symlink·Unix hard link도 거부한다.
  전체 변환·직렬화 후 기존 `atomic_file`로 임시 파일을 교체한다. 실패 시 stdout과 부분 결과를
  남기지 않고 기존 출력 파일을 보존한다.
- JSON은 저장 후 한 번만 내며 원본 쪽/출력 구역 대응, 원본 좌표, 문단 정책·경고,
  `parserAllocationBoundsProven=false`를 유지한다. 원본 좌표의 provenance도 등록했다.
  입력 문서 전체는 현재 1~128쪽/16 MiB 한도를 유지한다.
- root/scoped 도움말·catalog·capabilities와 CLI 매뉴얼을 함께 갱신했다. feature-off에서도
  명령은 보이고 `available=false`, 실행은 `PDF_IMPORT_DISABLED`로 안내한다. PDF 읽기 형식은
  feature-on에서만 선언한다.
- 저장소의 JSON CLI/MCP 대응 규약에 따라 `hwp_import_pdf` 선언을 같은 CLI에 연결했다.
  선택 문자열 `sourcePages`는 기존 optionalArgs 경로를 사용하며 변환 구현을 복제하지 않는다.
  후속 F3 WASM의 `sourcePages` 배열 계약과는 다른 CLI 문자열 표면이다.

### 5.1 기능 실행 근거

원시 증적은 `output/pr-review/issue7911/cli-integration-20261016/`에 있다.

- 기존 F1 바이너리에서 새 정식 선택/저장 테스트가 미지원 명령으로 실패했고, F2에서 통과했다
  (`cli-before-red.log`, `cli-feature-final.log`). 컴파일/환경 실패를 red로 세지 않았다.
- 새 CLI 계약은 합성 PDF의 선택 내용·순서, HWP/HWPX 재열기·SVG 텍스트, 잘못된 옵션,
  과대 입력, 기존 출력 보존, rename 실패의 임시 파일 정리, 입력 별칭 보호를 검사한다.
  시각 배치 golden을 추가하거나 한컴 일치 증거로 해석하지 않는다.
- 실제 입력 `pdf/issue5875/nested_table_text_caption-2020.pdf`: 전체 12쪽 HWPX,
  원본 `0,6,7` → 구역 `0,1,2`의 선택 3쪽 HWPX/HWP를 저장·재열기했다. 선택 HWPX는
  기존 `export-svg`로 렌더했다. 모든 명령 exit 0이며 전체 HWPX가 F1 저장본과 바이트 동일하다.
  `real-smoke.json`에 입력·실행 바이너리 SHA-256, 명령, 저장본 hash를 기록했다.
- qpdf로 만든 암호화 PDF와 90도 회전 PDF는 각각 `PDF_ENCRYPTION_UNSUPPORTED`·
  `PDF_GEOMETRY_UNSUPPORTED`와 exit 1로 거부하며 stdout 비움·기존 출력 보존을 확인했다
  (`unsupported-inputs.json`). 미지원 내용을 조용히 생략해 성공 처리하지 않는다.
- 실제 `mcp-serve`에서 `6-7` 선택 가져오기 → `99` 범위 실패 → 선택 생략 전체 가져오기를
  연속 실행했다. 각각 2쪽 성공·오류(파일 없음)·12쪽 성공이다. `mcp.stdout`/`mcp.stderr`에
  응답을 보존했다. 이는 CLI subprocess 복구이며 F3의 브라우저 Worker 검증은 아니다.
- 포맷팅으로 새 테스트 source 크기와 파생 suite 배정이 바뀌어 첫 실행은 0건이었다.
  해당 실행은 통과에 포함하지 않았다. `--prepare`를 다시 수행해 manifest를 확인한 뒤
  실제 5건 실행을 확인했다. 이 상태를 `cli-feature-final.log`로 구분한다.

### 5.2 완료 경계

F2의 집중 검증 결과와 로컬 commit은 아래 후속 기록 및 ignored `completion.json`에 고정한다.
F3 공개 WASM/Worker·F4 실제 편집/저장·전쪽 시각 판정과 최종 제출 검증은 남는다.
공통 PDF 변환/조판 코드는 바뀌지 않아 이전 WASM을 다시 빌드하지 않았다. F2 결과를
새 제품 WASM 실행·전체 회귀·전체 parser 할당 감사 통과로 승격하지 않는다.


F2 정식 집중 검사는 **82건** 통과했다. CLI는 feature-on 5건/default 4건,
기존 catalog 20건·scoped help 11건·CLI JSON 31건·provenance 11건이다.
마지막 네 묶음은 pdf-import 활성 구성에서 새 명령과의 정합을 확인했다.
명령은 F1과 같은 `node scripts/run-rust-test.mjs --cargo-test <case> -- --locked
--profile release-test --target-dir /home/edward/mygithub/rhwp/target/pr-review
[--features pdf-import]`이며 실제 실행 건수/결과를 각 case 이름의 log에 보존했다.

`cargo fmt --all -- --check`, manifest 및 unit-tier의 base `f5251e7de` 비교도 통과했다.
Native CLI lint는 `cargo clippy --locked -p rhwp --bin rhwp [--features pdf-import]
--target-dir /home/edward/mygithub/rhwp/target/pr-review -- -D warnings` 두 구성 모두 통과했다.
새 source와 검사 hash는 `source-final.json`, 최종 로컬 commit 및 lint 결과는
`completion.json`에 기록한다. 기존 B11의 전체 workspace/회귀 gate를 이 focused 결과로
대체하지 않는다. 원격 push·PR 상태 변경은 수행하지 않는다.

확인할 출력은 위 증적 디렉터리의 `all.hwpx`(12쪽), `selected.hwpx`·`selected.hwp`
(원본 1·7·8쪽), `selected-svg/`다. 실행 바이너리는 같은 디렉터리의 `candidate/rhwp`다.
기능 경로 진행은 **F1·F2 완료 → F3 공개 WASM/Worker → F4 편집·저장·시각 검증**이다.
세부 보완 B01–B12의 완료를 이 순서의 선행 조건으로 되돌리지 않는다.

## 6. F3 공개 WASM API와 Worker (2026-10-16)

다음 작업 승인으로 F3를 구현·검증했다. 기존 제품에는 문서 소유 Worker가 없고
`tools/pdf-ir-poc/browser/worker.js`만 있었다. POC의 HWPX 변환·재파싱 경로를 옮기지 않고,
공통 frontend/Core를 소비하는 공개 API와 저장소 공용 `rhwp-shared/pdf/` 세션을 연결했다.

- `HwpDocument.importPdf(bytes, options)`는 JS 객체의 `sourcePages`를 엄격히 검사하고
  `DocumentCore::from_pdf_bytes`를 호출한다. 전체 생성자는 F1 경로를 그대로 소비한다.
  `getPdfImportDiagnostics()`는 Core metadata를 JSON으로 반환한다. 다른 형식·저장본
  재열기는 `null`이며 저장된 HWP/HWPX를 PDF 원본으로 가장하지 않는다.
- JS 옵션 경계는 객체/배열 구분, 알려진 키, 유한 정수·개수·증가 순서와 narrowing 전 범위를
  확인한다. 실제 원본 쪽 범위·전체 입력 제한은 기존 frontend가 다시 검사한다.
  오류 code는 JS로 전달하며 feature-off API는 `PDF_IMPORT_DISABLED`로 거부한다.
- Worker는 루트 `pkg/rhwp.js`를 직접 import하며 변환 IR을 직렬화하지 않고 문서를 소유한다.
  `PdfImportSession`은 현재 문서와 후보를 다른 Worker에 둔다. 후보의 가져오기·진단 생성
  성공 응답 뒤에만 교체하고 이전 Worker를 해제한다. 일반 입력 오류는 후보를 재사용한다.
- 소유자 스레드의 AbortSignal/deadline은 후보를 terminate하고 다음 요청에서 재생성한다.
  실패·취소·시간 초과가 현재 문서를 바꾸지 않는 것과, 이미 완료된 문서의 렌더/저장 요청을
  취소하면 그 Worker도 해제되는 것을 구분한다. 초기화 실패·WASM trap은 재사용하지 않는다.
  요청 id와 종료한 채널의 pending 정리로 늦은 응답이 문서를 교체하지 못하게 한다.
- 원본 Uint8Array는 유지한다. 길이를 제한한 view만 복사·transfer하여 큰 backing buffer를
  그대로 복제하지 않는다. 동기 WASM 중 cancel 메시지 처리나 전체 heap 상한을 주장하지 않는다.
- 사용법과 실행 검사는 `rhwp-shared/pdf/README.md`, 공개 API는 `npm/README.md`에 연결했다.
  세션 검사는 기존 Studio `npm test` 수집에 포함한다. npm 배포 export·Studio 파일 열기 UI를
  자동 활성화하지 않으며 패키지 소비/후속 채택은 B08에 남긴다.

### 6.1 실행 증적과 완료 판정

증적 디렉터리: `output/pr-review/issue7911/wasm-integration-20261016/`.
F3 완료. 최종 실행 결과·source/artifact hash는 아래 기록과 ignored JSON에 고정한다.
API/세션 계약 검사는 조판 golden이 아니며 전체 한컴 시각 일치를 뜻하지 않는다.


Native 집중 검사는 frontend 9건·feature-on 감지/기존 HWP 진단 2건·feature-off 3건,
총 14건 통과했다. 양쪽 feature의 Native lib Clippy와 Docker WASM lib Clippy도
`-D warnings`로 통과했다. `fmt`, 파생 manifest, source-unit 정책은 base `f5251e7de`로
통과했다. 명령·exit status는 `native-results.json`, Docker 명령 로그는 `docker-build.log`다.

Worker 채널 정식 검사는 6건 통과했다(`session-tests-final.log`). 후보 실패와 기존 문서
보존·같은 후보 재사용, abort/늦은 응답/재생성, deadline/fatal 오류, pre-abort/중복 요청/
dispose, load/clone 오류, 입력 view 복사 범위를 확인한다. 이 결과는 mock Worker의 소유권
검사이며 실제 브라우저 검증과 구분한다. 실제 제품 WASM 검사는 별도로 아래에 기록한다.


실제 CDP Chrome `155.0.8059.40`에서 **57개 확인 항목 통과**다(`browser.log`,
`browser-result.json`). 별도 탭에서 루트 제품 JS/WASM을 로드했으며 5개 실제 Worker로
전체 12쪽·선택 원본 0/6/7쪽·다음 정상 입력을 처리했다. 12쪽 모두 SVG를 산출했고
HWPX/HWP 저장본을 같은 제품 API로 재열어 12쪽을 확인했다. 합성 3쪽 입력은 선택 내용의
순서·누락과 저장 형식별 재열기를 검사했다. 옵션의 잘못된 타입·음수·소수·NaN·overflow·
중복·역순·빈 배열·원본 범위 초과, 손상/과대 입력의 code도 확인했다.

**F4 정정:** 위 실물 HWP 재열기 검사는 쪽수만 확인했다. 그림 보존이나 한컴 열기의
통과 근거가 아니며, 메인테이너가 발견한 실제 실패와 보정은 아래 7절에서 처리한다.

기본 Docker WASM은 새 API와 기존 생성자에서 `PDF_IMPORT_DISABLED`로 거부하고 기존
HWP는 열었다. 활성 Worker의 일반 실패 뒤 같은 후보 Worker가 정상 입력을 처리했다.
1ms deadline 및 워밍업한 후보의 1ms abort는 현재 3쪽 문서를 보존했고, 재생성 뒤 정상
1쪽 선택이 성공했다. 이 검사는 실제 가져오기 시도를 취소한 결과이며 WASM 내부의 특정
명령 시점·악성 CPU 소진 입력·총 heap 상한을 입증하지 않는다(B07 유지).

Docker 기본/활성 빌드는 각각 한 번 실행했다. 원본 Rust 소스는 빌드 중 변경하지 않았고
JS 세션/검사 수정만 반영해 최종 실행했다. 명령은 같은 Compose 서비스에서 두 구성의
WASM lib Clippy 뒤 `scripts/wasm-pack-locked.sh --target web --out-dir
output/pr-review/issue7911/wasm-integration-20261016/default-pkg`, 이어서
`scripts/wasm-pack-locked.sh --target web --out-dir pkg --features pdf-import`다.
기존 `rhwp_wasm-target`을 재사용했으며 cache 초기화는 하지 않았다.

- 제품 WASM SHA-256: `542dc26e412a78c5ce63521af040f10dbe75f0b53a36f9ae055e6213d6942ffc`
- 제품 glue SHA-256: `db6f74738d59e98ce74e22d1d30ee6e0436a291336912889e569dc9125f48989`
- 루트 pkg와 Studio public의 두 파일은 각각 동일하다(`artifact-sync.json`). generated public
  glue는 검증 산출물로 보존하며 이번 source commit에는 포함하지 않는다.
- 최적화 WASM: 기본 12,568,410 bytes, PDF 활성 17,108,303 bytes. 같은 제품 빌드 구성의
  이번 관측값이며 크기 축소 작업으로 범위를 넓히지 않는다.
- 생성 타입 선언/기존 js_name export 검사도 1건 통과했다(`bindings.log`). 새 테스트의
  브라우저 의존성은 동일 lock의 `npm ci --no-audit --prefer-offline`로 준비했다.
- Native F2와 Worker의 전체 HWPX는 바이트 동일하다. SHA-256은
  `22bee8064e0d88223242f5ed61069ee49879229f6dee53add334596960750b82`다. 공통 경로
  보존 근거이며 원본 PDF/한컴과의 시각 일치 증거로 사용하지 않는다.

실제 브라우저 명령은 `CHROME_CDP=http://localhost:19222`,
`PDF_IMPORT_DEFAULT_PKG=<위 증적/default-pkg>`, `PDF_IMPORT_EVIDENCE=<위 증적>`를
지정한 `node --test scripts/tests/pdf-import-wasm.test.mjs`다. 공개 실행 스크립트가
선택한 바이트만 HTTP로 제공하며 실제 로드한 JS/WASM·원본의 hash를 기록한다.

### 6.2 다음 경계

**F1·F2·F3 완료 → F4 실물 문서의 편집·저장·재열기 및 전쪽 시각 검증**이다.
이번 단계는 PDF frontend의 조판/추출 규칙을 변경하지 않았다. 전체 Rust/workspace/Skia
회귀와 최종 PR 게이트는 B11/F4 이후 제출 범위 검증에 남아 있다. B01–B12 전체 소진을
F4의 선행 조건으로 삼지 않으며, 기본 비활성·미입증 진단도 유지한다.

출력은 위 증적의 `worker-all.hwpx`, `worker-all.hwp`, `worker-svg/`다.
Windows 확인용 복사본은 `E:\tmp\issue7911-stage3-wasm-20261016\`에 둔다.
원격 push·PR 상태 변경은 수행하지 않았다. 최종 source hash와 로컬 commit은
`source-final.json`·`completion.json`에 연결한다.

## 7. F4 HWP 그림 참조 손상 보정 (2026-10-16)

메인테이너가 F3 산출 HWPX는 한컴에서 정상 열리지만 HWP는 손상 경고가 나고, Studio에서
HWP를 재열면 그림이 보이지 않는다고 보고했다. 현재 기능 경로의 저장 결함으로 먼저
처리한다. 후속 보안 백로그나 시각 허용 차이로 이관하지 않는다.

### 7.1 원인과 수정 전 증거

- 재현 source: `5eaccb0ed070348252a09d6b21c36635464a0317`.
- 원본 PDF: `pdf/issue5875/nested_table_text_caption-2020.pdf`.
- 실패 산출물: `output/pr-review/issue7911/wasm-integration-20261016/worker-all.hwp`,
  SHA-256 `90dba729fc1b310919c7e018737c0e71ccf90dd46fe334b7b7cba7e9700afc82`.
  기존 산출물과 Windows 복사본을 덮어쓰지 않는다.
- `parser/pdf/lowering.rs::lower`가 PNG payload와 `BinDataType::Embedding`을 만들면서
  wire 속성 `attr`는 기본값 0으로 남겼다. `serializer/doc_info.rs::serialize_bin_data`는
  `attr`를 그대로 쓰지만 본문은 `data_type`에 따라 storage ID·확장자를 쓴다. 결과는
  LINK 헤더 + EMBEDDING payload로 어긋난 레코드다. HWPX는 이 wire 속성을 쓰지 않는다.
- 독립 계약은 [HWP5 명세 표 17·18](../tech/한글문서파일형식_5.0_revision1.3.md)의
  포함 그림 유형 `0x0001`, 기본 압축·미접근 상태 0이다. PDF에서 생성하는 포함 PNG의
  모델·wire 속성을 일치시킨다. 기존 HWP 원본 raw 기록이나 공통 렌더 좌표를 변경하지 않는다.
- 실패 HWP의 CFB에는 PNG 13개가 있으나 13개 BIN_DATA 모두 attr=0이다. 재열기 SVG는
  12쪽 모두 그림 0개다. 같은 파일의 해당 속성만 1로 바꾼 진단본에서는 쪽별 그림 수가
  `2,1,3,1,2,2,0,0,0,0,2,0`으로 복원되고 저장 전과 각 쪽의 그림 바이트가 동일하다.
- 제품 Worker 검사를 쪽별 그림 내용·중복 수까지 보강한 뒤 기존 Docker WASM에서 실행해
  `Worker hwp page 1 preserves embedded image content and multiplicity` 실패를 확인했다.
  HWPX 대조 경로는 이 검사에서 통과했다. 새 합성 검사는 두 쪽의 서로 다른 RGB 이미지로
  저장 형식별 참조·PNG 바이트·픽셀 보존을 검사하며 조판 위치 golden을 만들지 않는다.

진단 증적은 `output/pr-review/issue7911/hwp-save-20261016/`의 `before.json`,
`image-diagnosis.json`, `before-wasm-single.log`다.

**메인테이너 재확인:** attr만 보정한 진단본은 Studio에서 그림이 보이지만 한컴의 손상
경고는 남는다. 그림 참조 보정의 효과와 HWP 상호운용 실패를 분리한다. 해당 보정만 반영한
Docker WASM의 실제 Worker 검사도 쪽별 그림 내용·중복 수를 포함해 79항목 통과했다
(`attr-wasm.log`). 이것으로 한컴 열기까지 완료라고 판정하지 않는다.

### 7.2 HWP 저장 전용 변환 누락 조사

PDF frontend는 HWPX scaffold를 바탕으로 일반 IR을 만들지만, HWP export snapshot의
출처 분기는 Hwpx/Hwp3/Hml만 처리했다. 이 때문에 그림 attr 보정 후에도 PDF의 임시
FileHeader 버전 `0.0.0.0`, 구역당 PAGE_BORDER_FILL 1개, 개체 local version 0 등이
그대로 HWP에 기록된다. 파일 버전 계약은 HWP5 명세 표 3, 구역/개체 materialization은
기존 HWP adapter와 독립 한컴 저장본을 대조한다. HWPX 직렬화·재파싱을 제품 경로에
넣거나 PDF 출처를 다른 형식으로 가장하지 않고, HWP 저장 스냅숏에서 처리한다.

- 정상 HWPX를 한컴 2020(`11.0.0.9136`)에서 HWP로 저장하는 대조군은 성공했다.
  job `62f153df-57dc-4edd-9b5b-1e9194950f9c`, `hancom/hwpx-control-hancom.hwp`,
  SHA-256 `b47ca4904a274f87b6ede429b05831c6c280b90422dc7bebaed3c9556d4f1f27`.
- 같은 HWPX를 rhwp의 기존 HWP adapter로 변환한 진단본도 한컴 열기·HWPX 저장에
  성공했다. job `454a9517-3e06-4e7f-9b31-52d9b88126e2`, `mcp-adapter-status.json`.
  이 경로는 원인 분리를 위한 대조군이며 PDF 직접 저장의 성공 증거가 아니다.
- 원래 HWP와 attr-only HWP의 자동 변환은 각각 180초 제한에 실패했다. 자동 timeout
  자체를 손상 확정으로 쓰지 않으며, 손상 경고는 메인테이너의 직접 확인 결과다.

PDF HWP export의 수정 경계는 `prepare_hwp_export_snapshot → convert_if_hwpx_source(Pdf)
→ convert_to_hwp_ir(false) → serializer`다. 버전 5.1, PBF 세 레코드, 구역 control과
문단/개체 전용 metadata를 기존 공통 변환으로 구성한다. HWPX 전용 그림 밝기/대비 교환과
원출처 marker는 적용하지 않는다. 어댑터 없는 public `export_hwp_native`도 PDF 출처에서는
같은 스냅숏을 사용한다. 기존 HWP/HWPX/HWP3/HML 경로의 분기·변환은 유지한다.

합성 파일 형식 검사는 수정 전 `native: HWP5 version`(0 != 5)로 실패했다
(`before-wire-contract.log`). 같은 결함은 실제 Worker 검사에서도 `Worker HWP save
declares HWP5 wire version`으로 검출했다(`before-wire-wasm.log`). 버전 바이트만 보정한
별도 진단본의 자동 변환도 180초 제한에 실패했으므로, 버전만 바꾸어 완료로 삼지 않는다.

### 7.3 직접 저장본의 독립 한컴 검증

수정 후 Native frontend 11건 및 CLI·감지 7건이 통과했다
(`after-wire-contract.log`, `cli-detection.log`). 파일 형식 회귀는 Native/adapter/snapshot
세 진입점의 HWP5 버전·구역 PBF·원문 내용 및 다른 출처 marker 부재를 검사하고, HWP 저장
전후 HWPX 바이트와 PDF live provenance 보존을 확인한다. 그림 회귀는 서로 다른 두 RGB
이미지의 참조·내용·픽셀을 검사한다. 조판 좌표나 page-count golden을 추가하지 않았다.

실물 직접 저장본 `wire-fixed.hwp` SHA-256은
`422a435ed4dc1dcc2f37995e8e854e5e584536b798537d5bbb5b0d7a562cab14`다.
한컴 2020 `11.0.0.9136` 열기·HWPX 재저장에 성공했다(job
`e97a488f-ead8-4021-8f44-4409520d1ec9`). 재저장 후 12개 구역의 텍스트 순서·내용,
그림 13개·표 13개를 보존했다(`hancom-content-result.json`; XML의 혼합 내용/tail 포함).
동일 HWP의 PDF 인쇄도 12쪽으로 성공했다(job `6c45a5d3-0449-4e92-91fd-860afadaa1ea`,
`pdf_print_method=0`, `hancom2020_pdf_driver_one_up`).

HWPX 저장본은 F3와 바이트 동일하다. Native 재열기 시 그림 13개는 쪽별 내용·중복 수가
보존된다. HWP 7·8쪽의 텍스트 y가 attr-only 진단본보다 약 6.39px 아래로 이동한 것은
저장 전 제품 IR/Worker 및 HWPX의 같은 위치로 복귀한 결과다(`wire-render-comparison.json`).

HWP/HWPX 각각을 한컴에서 Print한 12쪽 중 11쪽은 96dpi raster가 동일하다
(`hancom-format-print-comparison.json`). 8쪽의 “의하며6㎸접촉” 부분 간격 차이는 남는다.
원본 PDF와의 문단·셀 간격 차이도 관측되므로 **F4 전쪽 시각 일치 완료는 아니다**.
이번 파일 열기·내용 보존 결과를 Native/fresh WASM 90% 시각 gate로 승격하지 않는다.

직접 확인용 파일은 `E:\tmp\issue7911-hwp-fix-20261016\pdf-direct-save-fixed.hwp`,
같은 이름의 `-2020.pdf`다. 앞선 `worker-all-bindata-diagnostic.hwp`는 실패 증거로
보존하며, 메인테이너의 새 직접 확인과 최종 Docker WASM 검증 결과를 이어 기록한다.

메인테이너가 새 HWP의 정상 열기를 확인했으나 내용 보완을 요청했다. 같은 코드의 최종
Docker WASM 실제 Worker 검사는 80항목 통과했고, Worker HWP는 위 Native HWP와
SHA-256이 동일하다(`wire-wasm.log`, `wire-wasm/browser-result.json`). 이는 파일 구조·
그림 참조 보존 결과이며 아래에 드러난 한컴의 실제 배치 누락을 면제하지 않는다.

### 7.4 11쪽 그림 10의 한컴 표시 누락

메인테이너가 11쪽 `[그림 10]`이 한컴에서 보이지 않는다고 특정했다. 원본 PDF는 같은
쪽에 그림 9(107×158px)와 그림 10(81×225px)이 있지만, 직접 저장 HWP의 독립 Print에는
그림 9만 남는다. 같은 HWPX의 Print도 동일하다. 그림 10의 PNG·셀 소유·참조는 저장되어
있으므로 데이터 유실과 표시 누락을 구별한다. 원래 표 앞쪽 행들이 늘어나 해당 행이 용지
아래로 밀려 잘린다. `hwp-save-20261016/p11/{source,hwp,hwpx}.png`에 원본과 실패 출력을
보존했다.

원인은 `structured_lowering.rs`의 일반 표 분기가 물리 길이 `x`를 `hu(2*x)`로 반올림해
문단 여백 IR에 넣는 데 있다. 홀수 결과는 serializer의 HwpUnitChar `CHAR` 단위가 된다.
반면 중첩 표 분기는 기존 `physical_margin(x)`의 `2*hu(x)`를 사용해 물리 단위를 보존한다.
독립 한컴 재저장본에서 번호 26의 저장 줄 시작은 2,230에서 11,145 HWPUNIT으로 늘고,
번호 29는 15,943에서 79,710으로 늘었다. 반면 짝수로 기록된 번호 28은 9,086을 유지했다.

소비 경로는 `paragraph.margin_left / flow.spacing_before·after → PARA_SHAPE 직렬화 및
HWPX HwpUnitChar → 한컴 재조판 → 셀 요구 높이·뒤 행/그림 원점`이다. rhwp의 저장 LineSeg
재사용에서는 원래 좌표가 남아 이 오류를 가렸으므로 재열기 그림 개수만으로 검증할 수 없다.
페이지 번호·문서 ID 예외 없이 일반 표의 세 물리 여백 생산 지점도 같은 helper를 사용한다.
라인/그림 크기·행의 선언 높이·그림 payload·기존 문서 serializer는 변경하지 않는다.

원인 분리용 HWPX에서 `CHAR` 단위만 `HWPUNIT`으로 바꾼 진단본을 한컴 2020에서 Print했다
(job `d6f3fcc9-3237-4fc9-b59d-0fb90d26059e`, 12쪽). 11쪽에 그림 10과 캡션이 복원됐다.
이 수동 진단은 제품 출력 성공으로 대신하지 않으며 새 Native/Docker WASM 저장본을 이어
검증한다. 진단 Print와 원본 PDF의 전쪽 실루엣은 1·11쪽이 90% 미만이며, 그림 복원과 전체
조판 일치 완료를 구분한다. 실제 출력 보완 전 조판 golden/기대값을 새로 추가하지 않는다.


제품 코드가 생성한 `p11/flow-fixed.hwp`(SHA-256
`f13522a8725420486e1725220f14c3366bd23ff6c69698289f43f3687216ef9d`)의
한컴 2020 Print도 성공했다(job `fcf7d97e-ef11-4dd2-b58b-5de1645be0f7`, 12쪽).
11쪽의 그림 9·10과 두 캡션, 뒤의 번호 30 행이 모두 보인다. 원본의 그림 10 데이터
크기 81×225px에 해당하는 이미지의 전체 사각형이 Print 용지 안에 포함됨을 확인했다.
새 HWP/HWPX의 12쪽 모두 그림 내용·중복 수가 저장 전과 같다(`p11/flow-image-result.json`).
잘못된 CHAR 여백은 45개에서 0개로 바뀌었다(`p11/physical-unit-check.json`).
Windows 직접 확인본은 `E:\tmp\issue7911-hwp-fix-20261016\pdf-direct-save-p11-fixed.hwp`,
동명 `.hwpx`·`-2020.pdf`·`-page11.png`다. 이전 실패본은 유지한다.

메인테이너가 위 HWP를 한컴에서 열어 **그림 10 표시를 확인**했다. 이 판정은 해당 표시
누락의 해소이며 전쪽 조판 일치 완료를 의미하지 않는다.

### 7.5 3쪽 그림 4의 오른쪽 자르기 누락

메인테이너가 원본 HWP에서 그림 자르기 **오른쪽 43.01mm**를 직접 확인했다.
원본 `samples/issue5875/nested_table_text_caption.hwp`의 해당 그림은 BIN0006.jpg,
1924×1196px·300dpi이며 저장 crop은 `(0,0,106220,89700)`이다. 저장 crop의 right는
보존 영역의 오른쪽 좌표로, 속성창에 표시된 제거 길이와 구분한다. 75 HU/px로 환산한
보존 너비 비율은 73.6105%다. JPEG 해상도로 환산한 제거 길이는 약 42.99mm로,
사용자가 관측한 43.01mm와 약 0.02mm 차이가 있으며 그 표시 차이의 원인은 미검증이다.

원본 PDF의 세 이미지 조각은 전체 폭 360.526pt를 갖지만 `W* n`의 자르기 영역은
219.602–485.019pt, 폭 265.417pt다. 보존 너비 비율 73.6194%는 원본 HWP와 대응한다.
용지 위에서 숨겨진 폭 약 33.55mm와 속성창 43.01mm는 이미지 배율이 적용되는 서로 다른
좌표계다. HWP 변환본은 전체 이미지 프레임 크기는 거의 유지하면서 이 자르기를 누락해
원본 PDF에서 보이지 않던 오른쪽을 표시한다. 셀 너비에 맞춘 임의 축소로 처리할 사안이 아니다.

증적은 `hwp-save-20261016/p3/original-hwp-dump.txt`,
`original-crop-correspondence.json`, `content-stream.txt`, `image-geometry.json`이다.
PDF clip → 가시 그림 영역 → HWP/HWPX 그림 자르기 계약의 보완 대상으로 확인했으며,
사용자의 일시 중지 지시 이후 **제품 코드 수정·새 빌드 없이 진단만 수행**했다.


메인테이너가 코드 구현과 결과 확인 준비를 승인해 작업을 재개했다. 독립 근거는 원본 HWP의
자르기 속성·원본 PDF의 `W* n`과 [ISO 32000-1 §8.5.4](https://opensource.adobe.com/dc-acrobat-sdk-docs/pdfstandards/PDF32000_2008.pdf)다.
구현 경로는 `page/Form operator → image handle clip + 전체 CTM → extract의 visible bbox
→ table recognition의 소유 영역 / lowering의 그림 프레임·crop → 기존 Picture 직렬화·paint`다.
전체 PNG는 유지하며 한쪽에서 계산한 가시 영역을 소유 추론과 그림 프레임이 공유한다.
현재 path는 q/Q stack에 넣지 않고, clip은 path 종료에서 교차한 뒤 graphics state로 복원한다.
Form의 BBox·Matrix와 부모 clip도 같은 page 좌표로 합성한다. 사각형이 아닌 clip은 bbox로
대체하지 않고 명시적 미지원 오류로 처리한다. 텍스트·vector path clipping은 이번 보정 범위
밖으로 유지한다. 경계 검증은 q/Q 복원·누적 교차·빈 교차·Form 변환·비사각형 거부를 포함한다.

제품 Native 보정본의 HWP/HWPX를 한컴 2020 `11.0.0.9136`의 one-up Print로 검증했다.
HWP job `88b2e367-697c-498c-b9af-ed7e574dd6bb`, HWPX job
`ce3a0d75-cb00-44a5-be0a-4adc5cc4c454` 모두 12쪽이며 3쪽 raster는 동일하다.
메인테이너가 `E:\tmp\issue7911-hwp-fix-20261016\pdf-direct-save-p3-fixed.hwp`의
3쪽 그림 4에 대해 **시각 판정 통과**를 확인했다. PNG payload 13개와 페이지별 개수
`[2,1,3,1,2,2,0,0,0,0,2,0]`는 유지된다. 11쪽 그림 10의 표시도 보존했다.

원본 PDF 기준 Native Skia 전쪽 TSV는 최저 90.14609%(11쪽), 3쪽 96.84817%다
(`p3/native-source-scores/crop-hwp/silhouette.tsv`). 별도로 같은 HWP의 한컴 Print와
대조하면 3쪽은 98.48492%지만 1쪽 68.40217%, 11쪽 78.54899%의 기존 재조판 차이가
남는다(`p3/native-hancom-scores/silhouette.tsv`). 두 비교의 기준을 합치지 않으며
이번 3쪽 통과를 F4 전체 완료나 신규 조판 golden 등록의 근거로 확대하지 않는다.

진단 PDF의 자르기 경계 12건은 수정 후 모두 통과했고, 수정 전은 무자르기 대조군만
통과했다(`p3/boundaries/results.json`). 이는 ignored 진단 증거이며 정식 렌더링 회귀
추가와 구분한다. Native feature-on Clippy와 WASM feature-on Clippy는 통과했다.
정식 집중 회귀 130건(자원 111, CLI 5, frontend 11, 명시 여백 1, 형식 감지 2)도
통과했다(`p3/focused-tests.log`). 최종 전체 회귀·lint 묶음은 B11에 남는다.

**원본 그림과 PDF 조각의 구분:** 메인테이너가 변환 HWP에서 그림이 3개로 선택됨을
확인해 원본을 다시 대조했다. 원본 HWP의 BIN0006.jpg는 1924×1196px 하나지만,
독립 PDF 3쪽에는 이미 이미지 XObject 193·194·195가 각각 1924×399, 1924×399,
1924×398px로 위에서 아래로 연속 배치되어 있다(`pdfimages -f 3 -l 3 -list`).
현재 importer는 이 3개 paint 객체를 보존한다. 이번 보정은 각 조각의 clip을 복원한
것이며 원래의 단일 편집 객체를 추론·합성하지 않는다. 세 조각의 병합은 B09의 후속
편집 구조 복원 범위로 기록하며, 인접성만으로 무조건 합치는 규칙은 추가하지 않는다.

추가 대조군 `/mnt/e/tmp/rhwp_nested_table_text_caption.pdf`는 메인테이너가 원본 HWP를
rhwp에서 PDF로 출력한 파일이다(SHA-256
`eaa6bebee5241bf44eb94d3d36b15f19df8c51eb447e99ae218307fbdd7c4e17`,
Chrome 155 / Skia PDF m155, 12쪽). 3쪽에는 1924×1196px의 XObject 15 **하나**가 있다.
내장 JPEG 202,269 bytes의 SHA-256은 원본 BIN0006.jpg와 동일한
`b7da82df8c92b840c7a88d6850410abc9d8a37d0d4ae1c62b05a876a70dac42e`다.
clip의 page x 범위는 219.71968–485.26328pt이고 전체 이미지 폭은 360.74130pt다.
보존 비율은 원본 crop의 73.6105%와 대응하며 3쪽 PNG에서도 오른쪽 숨김이 유지된다.
같은 PDF를 보정 Native importer로 HWPX에 다시 저장해 3쪽 그림이 1개이고
`imgClip=(0,0,106220,89700)`임을 확인했다. 한컴 출력 PDF의 3분할과 rhwp 출력 PDF의
단일 객체를 구분한다. 이 추가 PDF는 출력 구조 대조군이며 독립 한컴 정답지로 사용하지 않는다.
증거: `p3/rhwp-export/{page3-content.txt,clips.json,reimport-pictures.json,page3.png}`.

최종 Docker WASM은 같은 source snapshot(`p3/source-state.json`)에서
`docker compose -p rhwp --env-file .env.docker run --rm --no-deps -e CARGO_BUILD_JOBS=4 wasm`
안의 feature-on WASM Clippy와 루트 `scripts/wasm-pack-locked.sh --target web --out-dir pkg
--features pdf-import`로 생성했다(`p3/docker-build.log`). WASM SHA-256은
`8e6089aec417c6097eabc8d01248fd8c608da8114dfff2a16a835476dbcb5c1d`이며
root pkg·Studio public·실제 Chrome 155의 WASM fetch가 일치한다. 공개 API/실제 Worker
검사 80항목·Worker 5개가 통과했다(`p3/wasm.log`, `p3/wasm/browser-result.json`).
Worker가 만든 HWP의 SHA-256은 Native와 같은
`ac245bae37b408b5c4af69ab39f6ef53505a47433034fc34288714704a107d99`다.
동일 바이트를 이유로 한컴 Print를 중복 생성하지 않았다.

`scripts/visual_sweep.py --silhouette-only --hwp p3/crop-fixed.hwp --pdf
pdf/issue5875/nested_table_text_caption-2020.pdf --key crop-hwp --rhwp-bin p3/candidate/rhwp
--wasm-pkg pkg --compat 2022 --dpi 96 --embed-fonts --font-path p11/fonts`로 12쪽을
새 Canvas 출력했다(여기서 p3/p11은 위 증적 디렉터리의 약칭). 전쪽 최저 90.16832%(11쪽),
3쪽 96.82916%이며 Native와 같은 자르기 영역·세 조각 연결·표 내부 포함을 PNG로 직접
확인했다(`p3/wasm-source-scores/crop-hwp/silhouette.tsv`,
`p3/wasm-review/crop-hwp/{review/review_003.png,overlay/overlay_003.png}`).
첫 실행의 Chrome 경로 탐색 실패는 증거로 보존하고 설치된 Chrome 154 경로를 명시해
성공했다. 제품 오류가 아니다. 저장 HWP의 한컴 Print 기준 WASM은 3쪽 97.84876%지만
1·11쪽은 각각 68.72149%·78.74329%이며 기존 F4 차이로 남는다.

Studio `http://localhost:7791/?review=issue7911-p3-image-clip&lang=ko`에 같은 저장본을
열고 3/12쪽 표시·브라우저 오류 없음·실제 WASM 해시를 확인했다(`p3/studio-result.json`,
`p3/studio-page3.png`). Windows `p3-review/index.html`에도 새 WASM 비교 PNG를 추가했다.
요약은 `p3/validation-result.json`이다. 이 결과는 3쪽 자르기 보정의 제품 검증이며
F4 전체·B11 최종 회귀·배포 준비 완료가 아니다. remote push·PR 상태 변경은 하지 않았다.

메인테이너가 rhwp 출력 PDF의 단일 그림·자르기 보존 대조까지 **시각 판정 통과**로
확인했다. 남은 순서는 저장본 한컴 재조판의 1·11쪽 차이 정리 → F4 보고·checkpoint →
최신 devel 정합 확인과 B11 통합 검증 → 기존 Draft #7913 갱신·CI·self-review·병합이다.
이번 판정을 미확인 1·11쪽의 예외 승인으로 확대하지 않는다. 그림 조각 병합·추가 자원
감사 등 후속 백로그 전체 소진은 기반 통합의 선행 조건이 아니며, 원격 반영은 별도 승인
범위에 따른다. 변경 없는 산출물과 유효한 검증은 재사용한다.

## 8. F4 저장·재열기 보정 checkpoint와 후속 이관 (2026-10-16)

메인테이너는 위 남은 절차 중 **1·11쪽 재조판 차이의 해결을 백로그에 기록해 나중에
구현하고, 2번인 현재 기능 검증 정리·로컬 커밋을 진행**하도록 지시했다. 이에 따라
[B10-1](../plans/task_m100_7911_backlog.md#21-b10-1--저장-hwp의-한컴-재조판-시-111쪽-배치-차이)에
원본·변환본·독립 Print·전쪽 TSV·관측값·진입 코드·완료 조건을 연결했다. 직전 7.5절의
“1·11쪽 차이 정리 후 checkpoint” 순서는 이 지시로 대체된다. 해당 두 쪽의 수치상
미달과 후속 구현 상태는 유지하며 다른 결함·미검증까지 면제하지 않는다.

이번 로컬 checkpoint는 다음 범위를 보존한다.

- PDF 포함 그림의 BIN_DATA 속성과 HWP 저장 스냅숏의 버전·구역·개체 레코드를 정합화해
  그림 참조와 한컴 열기를 복구했다. HWP/HWPX 저장·재열기의 데이터 계약을 검사했다.
- 일반 표의 물리 여백 단위를 일치시켜 11쪽 그림 10 표시를 복구했다.
- PDF 사각형 clip·Form 변환을 그림 프레임과 crop으로 전달해 3쪽의 숨긴 영역을 보존했다.
  원본 그림 payload와 PDF 조각 수는 유지하며 임의 병합은 후속으로 남겼다.
- 최종 Native 집중 130건·실제 Docker WASM/Worker 80항목, Native/WASM feature-on
  lib Clippy, 12쪽 원본 PDF 비교 및 사용자 시각 판정을 7.5절의 증거로 재사용한다.
  제품 source hash가 같음을 확인해 이번 문서·범위 정리 때문에 빌드나 시각 검사를 반복하지 않았다.

지원 제한은 기본 `pdf-import` 비활성, `parser_allocation_bounds_proven=false`, 비사각형
그림 clip 미지원, 텍스트/vector clipping·논리 문단/그림 병합의 불완전 복원이다.
1·11쪽 저장본 재조판 차이는 승인된 후속 범위다. 전체 편집·기존 포맷 호환성·전체
Rust 회귀·workspace/all-targets lint·최신 devel 병합은 다음 통합 검증에서 판정한다.
이번 결과를 전체 시각 일치·PR Ready·배포 준비 완료로 표시하지 않는다.

소스·정식 검사·backend patch 기록·계획/보고서를 함께 커밋한다. 검증용
`rhwp-studio/public/rhwp.js`와 ignored WASM·PDF·PNG·TSV·로그는 로컬에 그대로 보존하고
source commit에서 제외한다. `.env.local`과 generated suite도 포함하지 않는다.
최종 로컬 commit·검증 source의 일치·남은 작업공간 상태는
`output/pr-review/issue7911/hwp-save-20261016/p3/checkpoint-completion.json`에 기록한다.
이번 단계에서 remote push·PR 갱신·Ready·merge는 수행하지 않는다.
