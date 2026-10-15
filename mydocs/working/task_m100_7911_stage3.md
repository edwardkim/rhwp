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
