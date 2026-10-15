# Task #7911 PDF2IR 제품 통합 구현계획

- Issue: [#7911](https://github.com/edwardkim/rhwp/issues/7911)
- 작성일: 2026-10-15 (Asia/Seoul)
- 상태: **2026-10-16 공동 구현 목적 반영 — 제품 연결 기반의 devel 통합과 배포 준비를 분리**.
- [수행계획](task_m100_7911.md), [1단계 조사 보고](../working/task_m100_7911_stage1.md)
- 브랜치: `task_m100_7911`; 조사 head `92df2f3cbdc7012f5e91635512ceda68debecfac`
- 고정 base: `origin/devel` `a5e88ad682c805e1d845502ddc9521b94ea8eccd`
- POC: `poc/pdf-ir` `6bc3847e0ff3e1302736914859903f3c9c51bb69`

현재 순서·완료 경계는 [6절](#6-구현-순서와-승인), 세부 일감·상태·종료 증거의 정본은
[백로그](task_m100_7911_backlog.md)다. 2026-10-15의 단계 전환 제한은 6절의 후속 지시로
변경됐다. 이미 구현한 제한과 검증 결과는 보존하며, 백로그 등록을 해결 완료로 표시하지 않는다.

## 1. 설계 결정

승인된 설계는 **공통 파서에 opt-in PDF frontend를 연결하는 직접 IR 컴파일**이다.

이 기반을 devel에 두어 기여자가 추출·인식·lowering·제품 연결의 명확한 경계에서 함께
구현하도록 한다. 미완료 지점은 지원 여부·진단·재현·수용 조건으로 드러내고 후속 기여를
받는다. “전체 경로 구현”은 이 연결을 실제로 동작시키는 뜻이며 모든 PDF 기능·보안 감사·
배포 패키지를 먼저 완성한다는 뜻이 아니다.

```text
PDF bytes + PdfImportOptions
  -> parser/pdf (추출, 좌표 정규화, 구조 인식, lowering)
  -> ParsedDocument { Document, PDF import metadata }
  -> DocumentCore 공통 로드/compose/paginate
  -> 기존 Native/WASM 렌더, HWPX/HWP 저장
```

제품이 POC executable이나 HWPX 직렬화·재파싱을 호출하지 않는다.
PDF 특유의 추론은 frontend 소유이고 renderer에 PDF 전용 좌표 보정 분기를 만들지 않는다.
질문/보기 중심의 `parser/ingest` 스키마는 범용 PDF frontend로 사용하지 않는다.

### 1.1 POC 이력과 모듈 소유

- 구현 승인 후 작업 브랜치에서 고정 POC head를 `--no-ff` merge하여 9개 이력을 보존한다.
  먼저 merge simulation과 변경 경로를 확인한다. devel에는 직접 merge하지 않는다.
- 원 POC `tools/pdf-ir-poc/`는 별도 workspace로 보존한다. root에 의존하는 POC를
  제품 dependency로 추가하면 순환이므로 제품 컴파일러는 `src/parser/pdf/`에 이관한다.
- 제품용 모듈은 `mod`, `extract`, `recognition`, `table_detection`, `table_hierarchy`,
  `table_topology`, `lowering`, `structured_lowering`으로 책임을 구분한다.
  POC의 probe·JSON snapshot·독립 table extraction 진단은 제품 hot path에 복제하지 않는다.
- 과거 실패 checkpoint와 출력 기록은 성공 증거로 승격하지 않는다. POC 동결 복제는
  비교 기준이고 제품과 두 구현을 계속 병행 개발하는 구조는 이번 목표가 아니다.

### 1.2 Feature와 의존성

- 새 `pdf-import` feature는 **기본 비활성**으로 시작한다. CLI/WASM 모두 명시해 빌드한다.
  활성 제품 경로를 실제 검증해야 하며, feature-off 빌드 성공만으로 완료하지 않는다.
- `pdf_oxide = "=0.3.78"`, `default-features = false`, optional dependency를 출발점으로 한다.
  root `Cargo.lock`에 의도적으로 반영하고 이후 검증은 `--locked`로 수행한다.
- `office_oxide`는 전이 의존성이다. POC의 직접 pin을 이유 없이 root에 추가하지 않는다.
  해석된 버전·라이선스·NOTICE와 불필요한 OCR/ML/Python/native feature 유입을 확인한다.
- 2단계 확인: 자동 선택 `office_oxide 0.1.13`은 `pdf_oxide 0.3.78` 내부의
  `DocumentIR` initializer와 호환되지 않아 E0063을 재현했다. 검증된 `=0.1.9`를
  optional 직접 제약으로 명시한다. lockfile을 사용하지 않는 라이브러리 소비자에도 필요한 제약이다.
- WASM의 구버전 getrandom `js` feature 필요 여부는 feature tree와 실제 빌드로 확인한다.
  서버·Python·native PDF runtime·원격 OCR은 필수 의존성이 아니다.
- 2단계 확인: POC의 `getrandom 0.2` alias는 실제 전이 의존성이 사용하지 않았다.
  제품에서 제거하고 기존 getrandom 0.4 `wasm_js`로 wasm32 빌드를 확인했다.
- 기본 활성화, 외부 비신뢰 PDF에 대한 안전성 보장, 배포 번들 자동 채택은 별도 판단이다.
  opt-in은 자원 검증을 면제하지 않는다. 확인되지 않은 안전성을 홍보하지 않는다.
- 2026-10-15 자원 보완안 추가 승인 후 `pdf_oxide` import alias의 실체를
  `vendor/rhwp-pdf-import-backend`의 `rhwp-pdf-import-backend =0.3.78-rhwp.1`로 전환했다.
  0.3.78 원본은 보존하며 크기 축소가 아닌 보안 전용 patch다. 변경·고지는
  [backend 기록](../../vendor/rhwp-pdf-import-backend/RHWP_PATCHES.md)을 따른다.

#### 1.2.1 크기 최적화 비범위 확정 (2026-10-15)

메인테이너 지시로 **크기 최적화는 이번 이슈 범위에서 제외**한다.
현재의 `pdf_oxide =0.3.78` Cargo 의존 방식과 `office_oxide =0.1.9` 호환성 제약을 유지한다.
크기 축소를 위한 소스 추출·vendoring·추출 전용 fork·별도 WASM 분리는 도입하지 않는다.
기존 실측 증가분은 [2단계 기록](../working/task_m100_7911_stage2.md#31-pdf-포함제외-wasm-크기-실측)에 보존한다.
최종 제품 bundle 비용 측정은 유지하되 이번 완료 조건에 크기 감축 목표를 추가하지 않는다.
이 결정은 아래 자원·오류 게이트의 면제가 아니며, 안전성 보완에 별도 dependency 변경이 필요하면
그 범위를 보고하고 구현계획 변경 승인을 받는 절차는 유지한다.
이후 별도 승인된 0.3.78 기반 보안 patch 도입은 크기 최적화 제외 결정의 변경이 아니다.

## 2. 파일별 변경과 API

| 경계 | 예정 변경 | 보존할 계약 |
| --- | --- | --- |
| `Cargo.toml`, `Cargo.lock` | optional PDF dependency·feature | 기존 기본 feature와 workspace 유지 |
| `src/parser/mod.rs` | `FileFormat::Pdf`, `%PDF-` 감지, PDF metadata 및 분기 | 기존 parse API의 기본 동작·raw 봉인 |
| `src/parser/pdf/` | `PdfImportOptions`, 직접 IR 생성, 제한·오류·진단 | 공통 Document 모델만 반환 |
| `src/model/provenance.rs`, `document.rs` | `SourceFormat::Pdf`와 올바른 profile 분류 | PDF를 Hwp5/Hwpx 출처로 가장하지 않음 |
| `src/document_core/commands/document.rs`, Core 필드 정의 | PDF 옵션 진입점, 공통 parsed-document 로드 helper | fixup·compose·paginate·봉인 순서 유지 |
| `src/wasm_api.rs` | PDF source 이름, 명시적 import·diagnostics | 기존 생성자와 HWP/HWPX 입력 유지 |
| CLI dispatch/commands/document_io/metadata | `import-pdf`, 기존 입력 경로의 PDF 지원·안내 | 종료 코드·stdout JSON·원자적 저장 규약 |
| 모델/파서/`serializer/hwpx/shape.rs` | 필요한 경우 명시적 0 글상자 여백 보존 | 원본 부재와 0을 일괄 동일시하지 않음 |
| `tests/cases/`, 기존 WASM harness | 제품 경로 계약·회귀 | 기존 suite 수집 정책 유지 |
| CLI·빌드·지원 형식 문서 | 사용 예, 제한, opt-in 상태 | 미지원 범위를 지원으로 표시하지 않음 |

새 enum variant/public struct field가 외부 exhaustive match/struct literal에 미치는 영향을
확인하고 변경 기록에 알린다. 기존 함수 서명을 불필요하게 바꾸지 않는다.

### 2.1 Parser와 Core

- 기존 `parse_document*`와 `DocumentCore::from_bytes`는 feature-on PDF의 **전체 쪽**을 연다.
  feature-off에서도 PDF를 감지하고 `PDF_IMPORT_DISABLED`로 명시적으로 거부한다.
- 옵션 진입점은 `parser::pdf::parse_pdf_with_options(bytes, &PdfImportOptions)`와
  `DocumentCore::from_pdf_bytes(bytes, &PdfImportOptions)`를 추가하는 안이다.
  옵션 경로도 기존 경로와 동일한 provenance 봉인·Core 초기화 helper를 소비한다.
  기존 Core의 큰 initializer를 PDF용으로 복사하지 않는다.
- `ParsedDocument`와 Core에 별도 PDF import metadata를 둔다. HML의 메타데이터 분리 패턴을 따른다.
  공통 Paragraph에 PDF parser 객체를 삽입하거나 원본 PDF 전체를 진단 JSON에 복사하지 않는다.
- 공통 IR이 요구하는 UTF-16 char count·control mask·참조 무결성을 producer에서 맞춘다.
  HWPX 전용 로드 fixup을 얻으려고 PDF를 HWPX로 표시하지 않는다.

### 2.2 CLI와 WASM

- 기존 `export-hwpx input.pdf output.hwpx`, `convert input.pdf output.hwp`, `info`/렌더 명령은
  공통 document loader를 통해 전체 PDF를 처리한다. feature-off 안내를 함께 갱신한다.
- 선택 가져오기는 `rhwp import-pdf input.pdf output.hwpx --source-pages 0,6-7 --json`을 제안한다.
  기존 CLI의 0-based 페이지 규약을 따른다. 이 옵션은 입력 쪽 선택이며 렌더 `--page`와 다르다.
  미지정은 전체 쪽, 잘못된 문법·중복·내림차순·범위 초과는 exit 2, 처리 실패는 exit 1이다.
  출력·검증의 기존 코드 3/4 의미를 재정의하지 않는다.
- WASM은 기존 생성자의 PDF 전체 열기와 정적 `HwpDocument.importPdf(bytes, options)`를 제공한다.
  `sourcePages`는 0-based 증가 배열, 생략은 전체 쪽이다. 결과 객체는 기존 렌더·저장 API를 쓴다.
  `getPdfImportDiagnostics()`로 source/output page mapping·경고·임시 문단 정책을 조회한다.
- 오류는 안정적인 PDF code를 포함하고 JS 오류에도 전파한다. 실패한 새 문서를 기존 세션에
  부분 설치하지 않는다. 한 Worker에서 실패 뒤 정상 입력을 여는 검사를 실행한다.
- 확장/Studio UI 전면 개편은 제외한다. 실제 제품 JS import로 실행하고 POC WASM을 대신 쓰지 않는다.

## 3. 페이지·문단·구조 계약

- 하나의 Document builder가 모든 선택 쪽의 font/CharShape/ParaShape/TabDef/BorderFill,
  BinData, instance ID를 소유한다. 쪽별 Document를 단순 concatenate하지 않는다.
- 초기안은 PDF 물리 쪽마다 원본 크기·원점에 맞는 Section을 생성한다. 작성 당시 section의 복원이 아니다.
  서로 다른 크기·빈 쪽·비연속 선택의 순서와 쪽 대응을 보존하고 의도치 않은 빈 페이지를 검사한다.
- 전체 가져오기는 atomic all-or-error다. 회전·CropBox 등 구현하지 않은 변환은 안정적인
  unsupported code로 거부하고 첫 쪽만 반환하거나 조용히 빠뜨리지 않는다.
- 표 확정은 기존 EFSM과 canonical boundary contributor, 위상 검증, 부모 셀 포함·소유권 검증을
  함께 사용한다. 원본 획은 소유한 표 테두리 또는 fallback paint 중 한 번만 출력한다.
- 모호한 가족은 `Deferred`로 진단하고 원래 visual content를 보존한다. table 검출 실패와
  PDF page import 실패를 구분한다. 3쪽의 Deferred를 임의로 Complete로 바꾸지 않는다.
- 자식 materialization 실패 시 가족의 스타일·탭·이미지 풀을 rollback한다. 원본 내용의 중복·누락을 검사한다.
- 문단은 일단 **관측한 줄 단위 임시 Paragraph**로 만든다. 원본 논리 문단/읽기 순서를 완전 복원했다고
  주장하지 않는다. source page·bbox·관측 줄 소속·추론 사유를 bounded metadata에 남긴다.
- LineSeg는 PDF 물리 관측에서 생성한 layout 결과이며 한컴 저장 캐시가 아니다.
  좌하단 PDF 좌표를 페이지/셀 기준 좌상단으로 한 번 정규화하고 pt→HWPUNIT·baseline/height를 함께 생성한다.
  측정과 실제 배치가 같은 줄/객체 점유 결과를 소비하는지 최종 원점 선택까지 추적한다.
- 물리 문단 여백은 POC R3의 `2 * hu(points)` 계약을 보존해 CHAR 플래그가 섞이지 않도록 한다.
  탭은 TabDef와 실제 저장 폭을 함께 기록한다. inline 표 호스트의 baseline·여백을 별도 추측하지 않는다.
- 임의 clamp·겹친 텍스트 숨김·저장 LineSeg 일괄 삭제로 출력만 맞추지 않는다.

### 3.1 HWPX 후처리 제거

현재 serializer는 글상자 네 여백이 0이면 `textMargin`을 생략한다. POC는 ZIP XML 후처리로
명시적 0을 넣어 한컴의 기본 여백 적용을 막았다. 직접 IR만 반환하면 이 처리가 빠진다.

제품은 부재와 명시적 0을 구분하는 모델 표현을 도입하고 parser/serializer에서 보존하는 안을 따른다.
구체 필드 이름은 `text_margin_explicit` 등 기존 모델과 가장 작은 변경으로 정한다.
기존 생성·파싱 지점의 기본값 의미를 먼저 확인한다. 모든 기존 글상자에 무조건 0을 쓰지 않는다.
이 공통 변경은 별도 commit과 HWP/HWPX/HML 대조군으로 검증한다.
공통 모델 확장이 예상보다 넓거나 저장 의미를 바꾸면 구현계획을 보완하고 승인을 받는다.

## 4. 자원·오류 게이트

현재 추출 후 카운터는 parser 내부 선행 할당의 안전장치가 아니다.
검토한 0.3.78은 decoder 호출 후 크기 검사 경로와 Brotli `read_to_end`가 있으며,
Flate 개별 제한이 있어도 누적 decoded budget을 증명하지 못한다.
`open_with_config`의 `_config`도 이 경로의 유효한 bytes/options 제한 API가 아니다.

| 범위 | 초기 예산/정책 | 검증 위치 |
| --- | --- | --- |
| 원본 | 최대 16 MiB, 전체 128쪽 | bytes 복사 전 / page count 확인 직후 |
| 페이지 객체 | spans 20,000, images 32, paths 20,000 | 가능하면 추출 생성 중, 추출 후 검사와 구분 |
| 구조 | table 후보 128, 셀 4,096; 중첩 깊이 상한 | 후보 확장·격자/행렬/재귀 처리 전 |
| decoded 내용·이미지 | 문서 누적 budget, 이미지 width*height 및 bytes checked arithmetic | decoder/이미지 대규모 할당 전 |
| 진단 | 항목 수·문자열 길이·총 bytes 상한 | 수집 전, 잘린 경우 별도 표시 |
| CPU/취소 | 단계별 시간 측정, Worker deadline/취소 | browser Worker; heap 한계와 별도 |

위 숫자는 초기 POC 정책을 계승하는 제안이지 성능·안전성의 실측 보장이 아니다.
구현 단계에서 누적 decoded bytes/이미지 픽셀/깊이의 수치와 실제 적용 지점을 확정해 기록한다.
상한에 맞는 오류를 반환하고 일부 결과를 성공으로 반환하지 않는다.
크기 산술 overflow, 다단 filter, 압축 확장, 과다 객체·후보·깊이를 경계 입력으로 검사한다.

외부 crate의 할당 전 제한이 공개 API로 불가능하면 **배포 준비 판정을 보류**한다.
필요한 최소 upstream/fork patch의 범위·라이선스·회귀를 먼저 보고하고 구현계획 변경 승인을 받는다.
opt-in 표시나 Worker 종료를 메모리 안전성 입증의 대체로 삼지 않는다.
프로세스 전역 환경변수를 요청마다 바꿔 제한을 구현하지 않는다.
PDF JavaScript·첨부 실행·외부 링크 fetch·시스템 글꼴 설치는 하지 않는다.
암호 PDF는 이번 범위에서 명시적으로 미지원 처리하고 HWP 암호 오류로 오분류하지 않는다.

### 4.1 자원 제한 보완안: 도입 승인, 구현·검증 진행 중

2026-10-15 메인테이너의 “순서대로 진행” 승인에 따라 첫 순서인 자원 제한 보완안을 조사했다.
첫 조사에서는 dependency source를 변경하지 않았고, 이후 메인테이너가
**“0.3.78 기반 최소 보안 패치 도입 승인”**으로 별도 이름의 dependency 변경을 승인했다.
이 승인에 따라 구현·작은 계약 검증을 진행한다. 크기 최적화 제외 결정과 외부 게시·배포,
단계 전환·최종 회귀의 승인 경계는 유지한다.

조사 대상은 로컬 `pdf_oxide 0.3.78` source와 현재 rhwp frontend다.
ignored `output/pr-review/issue7911/resource-audit/source-audit.json`에 14개 정적 관측,
파일별 SHA-256·위치와 Cargo.lock 패키지 식별자를 기록했다.
이는 **실행 보안 시험이나 upstream 배포 archive 무결성 재검증이 아니다**.

#### 실제 우회 경로와 변경 책임

아래 경로는 모두 crate `src/` 기준이다. rhwp 위치는 별도로 표기한다.

| 생산/소비 경로 | 확인한 현재 동작 | 필요한 계약 |
| --- | --- | --- |
| `document.rs:1087` → `open_from_reader:1138` | bytes 복제와 xref/recovery 후 `ParserOptions::default()` 설정 | constructor 진입 전부터 요청별 context 전달 |
| `document.rs:1615` | `open_with_config`는 Native-only stub | 설정 함수 이름만으로 안전성을 주장하지 않음 |
| `xref.rs:924`, `objstm.rs:135` → `object.rs:280` | 생성·객체 로드 중 options 없는 decode 사용 | bootstrap, 복구, lazy load도 동일 context 소비 |
| `decoders/mod.rs:212` → `StreamDecoder::decode` | 출력 Vec 생성 뒤 크기 검사; predictor는 이후 별도 실행 | 모든 filter 중간 출력·predictor를 확장 전에 제한 |
| `decoders/flate.rs:159`, `brotli.rs:12` | Flate는 자체 개별 cap, Brotli는 무제한 `read_to_end` | 문서 잔여 예산과 filter cap을 명시 전달; 환경변수 금지 |
| `parser.rs:423`, `:485`; `objstm.rs:202` | 배열/사전 재귀에 depth 인자 없음; 선언 N으로 capacity 예약 | 큰 parser frame 진입·push/insert/reserve 전에 깊이·개수 검사 |
| `document.rs:19310`, `:23742` | paths는 decode 오류 시 빈 결과, image Do 순회는 `if let Ok`로 오류 생략 | resource failure는 복구·빈 결과로 바꾸지 않고 fatal 유지 |
| `images.rs:3437` → `to_dynamic_image:477`; rhwp `lowering.rs:254` | handle은 선언 크기를 제공하지만 JPEG 픽셀 decode가 PNG 변환까지 지연 가능 | 실제 codec header·픽셀/색 변환·인코딩 버퍼도 같은 예산 소비 |
| `document.rs:12636` → rhwp `extract.rs:36` | span Vec·cache clone 생성 뒤 rhwp 카운터 검사 | 추출 push와 cache 보관/clone 전 제한; 폰트/CMap·Form 재귀도 추적 |

일부 decoder만 고치거나 `from_bytes_with_options`만 추가해서는 위 경로가 닫히지 않는다.
PDF 원본 bytes를 따로 스캔하는 preflight 파서를 복제해 해결했다고 판단하지 않는다.

#### 권고하는 변경 단위

1. **요청별 context:** 가칭 `PdfImportLimits`와 `ImportBudget`을 PDF 생성 전에 만들고
   constructor/xref/object stream/일반 stream/폰트·이미지/추출 경로에 전달한다.
   전역·thread-local·환경변수 대신 실제 문서 소유 context를 사용한다. 기존 public API와
   제한된 새 API의 차이를 명시하고, rhwp는 제한된 경로만 사용한다.
2. **bounded decode:** Flate/Brotli/LZW/RunLength/ASCII 계열, 다단 filter의 각 중간 출력,
   predictor의 산술과 버퍼 증가를 같은 한계로 제한한다. 압축률 검사는 보조값이다.
   출력 한계에 정확히 맞는 정상 입력과 초과 입력을 구별하고, 잘린 prefix를 성공으로 반환하지 않는다.
   누적량은 재시도·반복 decode의 실제 작업도 계상하고 실패 후 한도를 되돌려 우회하지 못하게 한다.
3. **컨테이너와 추출:** xref 항목·`/Index`·`/W`·`/N`·`/First`·객체 깊이·Form/페이지/폰트
   참조 순환, content operator·graphics stack·span/path/image 누적량을 확장 전에 확인한다.
   `checked_*`와 실패 가능한 reserve를 사용한다. 토큰/사전/배열과 cache clone도 포함한다.
4. **이미지:** 선언값뿐 아니라 JPEG 등 실제 codec header를 픽셀 할당 전에 검사한다.
   색 공간·bit depth·행 pitch·mask/palette·CMYK→RGB·PNG 변환의 복수 버퍼를 포함한다.
   bounded API를 적용할 수 없는 codec은 이번 제한된 경로에서 명시적으로 거부한다.
   이미지 핸들 개수 검사만으로 선행 decode 안전성을 주장하지 않는다.
5. **fatal 오류:** 예산 초과/취소를 별도 typed error와 요청별 fatal 상태로 보존한다.
   기존 lenient recovery·`ok()`·빈 결과 반환이 이를 삼킬 수 없도록 전파하고 public 작업 종료 시
   fatal 상태를 확인한다. rhwp는 `PDF_RESOURCE_LIMIT` 등 안정적 코드로 매핑한다.
   한 쪽 실패 시 builder 전체를 버리며 다음 문서에는 새 context를 만든다.
6. **검증·고지:** 변경 diff·원 upstream 버전/출처·NOTICE·상표 구분·후속 보안 수정 추적을 남긴다.
   해당 경로 밖의 쓰기·변환 기능 삭제나 크기 최적화는 하지 않는다.

#### 초기 정책 제안과 한계

기존 입력/페이지/표/선언 픽셀 상한은 유지한다. 아래 수치는 추가 승인 후 검증할 초기 정책이지
모든 정상 PDF 처리나 브라우저 총 메모리의 안전한 상한이라는 보장이 아니다.

| 추가 예산 | 초기 제안 | 선행 검사/측정 |
| --- | --- | --- |
| decoded 출력 | stream/filter 단계당 32 MiB, 문서 누적 256 MiB | 출력 append와 predictor/복구 재시도마다 실제 소비량 계상 |
| 객체/컬렉션 | 객체 50,000개, 배열·사전·xref 항목 합계 100,000개 | 선언 크기의 reserve/insert 전, xref bootstrap 포함 |
| 깊이 | 구문 중첩 16, 참조 탐색 32, Form 중첩 16 | 큰 frame 진입 전; 순환 검사는 깊이와 별도 |
| 이미지 작업량 | 이미지당 decoded 64 MiB, 문서 누적 128 MiB | 실제 header 및 변환 버퍼 계상; 기존 픽셀 상한도 병행 |
| content 작업량 | 문서 operator 250,000개, graphics stack 128 | token/operator 생성 중; 기존 span/path 한계도 병행 |
| CPU/취소 | 결정적 작업량 카운터와 상위 Worker deadline | 동기 WASM 실행 중 cancel 메시지가 처리된다고 가정하지 않음 |

원본 전체 12쪽/7·8쪽과 정상 대조군의 실측 소비량으로 정책을 검토한다. 변경이 필요하면 일반적인
처리 요구와 근거를 기록하며 특정 문서 ID로 예산을 완화하지 않는다. 합산 decode bytes는 전체
RSS/heap과 다르다. codec 내부 scratch·동시 버퍼·allocator 여유분까지 전부 입증한 것으로
표시하지 않으며, 미확인 경로가 있으면 배포 게이트를 유지한다.
Worker 종료·재생성은 시간 초과/치명적 실패의 운영 복구이고 선행 할당 제한의 대체가 아니다.

초기 구현의 정상 PDF 검증에서 content 수치 operand가 컨테이너 객체 quota를 소진하는
계상 오류를 확인했다. 객체 50,000개 한도는 유지하고 content operand를 별도 영역으로 분리했다.
operator당 operand 64개, 내부 배열/사전의 중첩·컬렉션·문자열 제한을 병행한다.
자원 초과의 최초 정적 사유를 fatal 상태에 보존하고 다음 문서는 독립 context로 시작한다.
실제 적용값·검사 수·실물 PDF 결과는 2단계 진행 기록을 정본으로 삼는다.

#### 승인된 dependency 선택과 배포 경계

현재 공개 API만으로 필요한 제한을 주입할 수 없다. **0.3.78을 기준으로 하는 보안 전용 patch**를
권고했고 메인테이너가 승인했다. 공식 registry source나 Cargo 캐시를 직접 수정하지 않는다.
비교한 선택지는 다음과 같으며 두 번째 경로를 채택했다.

- upstream에 변경을 제안하고 지원 API가 포함된 공식 버전으로 전환한다. 외부 게시와
  `=0.3.78` 버전 변경은 각각 별도 승인 대상이며 즉시 가능한 해법으로 가정하지 않는다.
- **0.3.78 유지 권고안:** 원본을 보존한 별도 내부 이름의 dependency에서 최소 보안 diff만
  관리한다. 수정판을 공식 PDFOxide라고 표시하지 않고 라이선스/NOTICE와 변경 이력을 유지한다.
  이는 크기 축소용 fork가 아니라 배포 안전성 보완이다. 도입 승인을 받아 적용 중이다.

단순 workspace `[patch.crates-io]`는 라이브러리 소비자에게 자동 전파되지 않는다.
내부 dependency를 택하면 배포 모델도 함께 확정한다. Native/WASM repository build뿐 아니라
`cargo package`와 packaged 소비자가 동일한 보안 코드를 쓰는지 확인한다. 별도 crate 게시가
필요하면 이름·버전·배포 권한을 승인받으며, 게시하지 않은 dependency를 정상 배포 가능하다고
표시하지 않는다. 현재 backend는 `publish = false`로 외부 게시를 차단한다.

초기 연결에서 root의 명시적 workspace member `.` 때문에 vendor 제외가 무시되는 현상을
작은 독립 manifest로 재현했다. 이 중복 항목만 제거하고 implicit root·기존 default-members를
유지했다. metadata의 실제 member 집합이 기존 10개와 같고 backend는 제외됨을 확인했다.
임시로 유입된 vendor 개발용 lock 항목은 이 분리 후 Cargo가 정리했다. 실제 제품의
기본 비활성 정책을 vendor workspace의 default feature로 우회하지 않는다.

2026-10-15 메인테이너가 PR Open까지 후속 진행과 **별도 crate 배포 준비 포함**을 승인했다.
내부 backend의 별도 package 이름·0.3.78 기반 version·저작권/NOTICE를 유지하고,
package 내용과 의존성의 정확한 version, 압축 해제한 소비자의 Native/WASM 빌드 및
root package의 의존 해석을 검증한다. 검증 전 `publish=false`는 유지한다.
이 승인은 외부 게시 승인이 아니며 `cargo publish`/registry 업로드는 검증 완료 뒤 별도 승인받는다.
namespace/게시 권한과 실제 배포 경로가 확인되지 않은 상태를 배포 가능으로 표시하지 않는다.

#### 작은 계약과 종료 조건

- 작은 주입 예산의 한도-1/한도/한도+1, 압축률이 낮아도 누적량이 초과하는 여러 stream,
  다단 filter 중간 출력·predictor, xref bootstrap·object stream의 제한 전달을 검사한다.
- Flate/Brotli/LZW/RunLength와 이미지 header/선언 불일치·CMYK/PNG 변환·mask,
  거대한 선언값/overflow·깊이/순환·과다 operator/cache 경계를 검사한다.
  위험한 입력의 수정 전 관측은 메인 프로세스가 아닌 제한된 자식 프로세스에서 작은 상한으로 수행한다.
- recovery가 제한 오류를 삼키지 않는지 실제 parser→rhwp import 경로로 확인한다.
  동일 프로세스/Worker에서 실패 뒤 정상 입력, 다른 문서 간 예산 독립성을 검사한다.
- 최종 count만 비교하지 않고 append/예약 전의 거부 및 bounded reader/codec 호출을 검사한다.
  수정 전 FAIL / 수정 후 PASS와 정상 대조군을 구분한다. 14개 정적 관측을 실행 검사로 대신하지 않는다.
- 제품 검사는 `tests/cases/`와 정식 suite 수집 정책을 따른다. Native/WASM feature-on/off
  빌드·작은 계약 뒤 실물 전쪽 시각과 기존 포맷 비교를 수행한다. 긴 전체 회귀 승인은 별도다.
- 소스 경로 감사와 실행 증거가 부족하면 `parser_allocation_bounds_proven=false`를 유지한다.
  2026-10-16 지시에 따라 미완료를 백로그로 인계하고 제품 CLI/WASM 연결을 먼저 구현한다.
  제품 연결의 착수·기능 완료와 자원 안전성 검증 완료를 구분한다.

구현 적용 범위와 아직 적용하지 않은 제안값·검증 결과는
[2단계 진행 기록](../working/task_m100_7911_stage2.md)에 구분한다.

2026-10-15 후속 진행 승인에 따라 CMap source/scan/확장과 CID metric 확장,
요청별 lazy cache/clone, page content 결합·반환 복사의 예산 적용을 보완한다.
내부 backend는 shared target의 비해시 산출물 충돌을 피하도록 Rust 의존성용 `rlib`만 생성한다.
원본 기능 소스·고지는 보존하며 독립 C ABI package 배포나 크기 최적화는 범위에 추가하지 않는다.
적용 수치·수정 전 실패·최신 source 재검증·남은 경계는 위 진행 기록 3.5를 따른다.

## 5. 검증과 증거

정본은 [local validation 4.3](../manual/pr_review/local_validation.md#43-변경-범위별-기본-검증),
[시각 증거](../manual/pr_review/visual_fixture_evidence.md),
[한컴 Print 계약](../manual/mcp_hwp2024Convert_usage.md)이다.

1. **작은 계약:** format 감지/feature-off, 손상·암호·예산·실패 후 정상 열기, 빈/다중/선택 쪽,
   resource ID 충돌, 실제 표·중첩 소유권, 원본 괘선 단일 소비를 제품 경로로 실행한다.
2. **실물 선행 시각:** 원본 PDF 전체 12쪽과 7/8쪽 분리본, B본/단일 표 대조군을
   Native/fresh 제품 WASM으로 재출력한다. 원본 PDF는 PDF frontend의 물리 관측 기준이며,
   생성 HWPX/HWP의 실제 재조판은 버전에 맞는 독립 한컴 Print로 별도 비교한다.
3. **회귀 수집:** 전쪽 90% 이상·쪽수·직접 판독 선행 뒤 실물 관계 검사를 `tests/cases/`에 이관한다.
   논리적 소유·내용 순서·누락/중복·포함/겹침을 검사한다. POC 71개 결과를 제품 통과로 복사하지 않는다.
   source-side cfg test나 자동생성 suite 파일을 그대로 이관하지 않는다.
4. **최종 Rust 게이트:** suite manifest prepare/check, fmt/check, default 및 `pdf-import`의
   Native/WASM Clippy, workspace build와 all-targets Clippy를 실행한다. feature-off도 유지한다.
   긴 전체 회귀는 focused/시각 결과를 보고하고 별도 승인 뒤 정본의 nextest release-test,
   unit tiers, 관련 Native Skia 게이트를 실행한다.
5. **fresh WASM:** 저장소 루트의 locked wrapper로 web bundle을 만들고 `--features pdf-import`를
   전달한다. 공용 target을 사용하고 실제 JS/WASM SHA·Worker 로드·다중 페이지 렌더/저장을 기록한다.
6. **회귀 비교:** 기준 devel과 head의 같은 기존 HWP/HWPX/HML 대조군을 비교한다.
   Native/WASM 어느 쪽이든 페이지별 0.01%p 이상 하락하면 90% 이상이어도 재검토한다.
   base가 PDF를 지원하지 않는 항목은 N/A로 기록하고 0점·통과로 꾸미지 않는다.
7. **비용:** feature-on/off WASM bytes와 gzip 크기, cold/warm 추출·인식·lowering·Core·저장 시간,
   측정 가능한 Native peak RSS/브라우저 메모리를 따로 기록한다. 메모리 측정 불가를 0으로 쓰지 않는다.

현재 WSL host의 Cargo 공용 target은 `/home/edward/mygithub/rhwp/target/pr-review`이며 순차 실행한다.
macOS 작성 당시 경로는 해당 host의 기록이며 현재 worktree별 target을 새로 만들지 않는다.
출력/명령 로그·입출력 SHA·전쪽 TSV·대표 PNG·Print job은
`output/pr-review/issue7911/`에 보존하고 commit하지 않는다.
기준 입력과 재사용 가능한 독립 PDF의 Git 포함은 별도로 확인한다.
회귀의 절대 px pin은 완료 페이지 Native/WASM 각각 98% 이상·직접 판독·±2px 조건 없이 추가하지 않는다.
회귀 실패를 보정하기 위해 baseline/ignore를 조용히 바꾸지 않는다.

## 6. 구현 순서와 승인

2026-10-16 메인테이너는 **전체 기능 구현을 먼저 완료하고, 세부 처리 대상은 백로그에
기록하면서 이후 순차 구현**하도록 지시했다. 이 결정이 이전의 “보안 세부 감사 완료 후
3단계 착수” 순서를 대체한다. 단계별 기록·commit, 최종 품질·배포 게이트는 유지한다.

이후 같은 날 목적을 명확히 했다. **devel 승격은 공동 구현의 기반을 만드는 단계**다.
직전 계획에서 백로그 전체 완료를 Ready·merge·배포의 공통 선행 조건으로 묶은 부분은
이 지시와 맞지 않아 정정한다. 현재 통합 범위의 검증과 후속 완성·배포 검증을 분리한다.

| 단계 | 산출물 | 다음 단계 조건 |
| --- | --- | --- |
| 1 (완료) | 호출 경로 조사, 이 구현계획, stage1 보고·commit | 메인테이너 구현계획 승인 완료 |
| 2 (checkpoint 정리) | POC·frontend·기존 보안 보정과 집중 증거 보존, 미완료 ID 인계 | stage2 기록·commit 후 3단계; 잔여 보안 항목은 미완료 유지 |
| 3 (다음 구현) | F1–F4 제품 전체 경로 및 기본 오류 복구 | 실제 CLI·제품 WASM 실행, 렌더·저장·재열기와 정상 대조군 확인, 보고·commit |
| 4 | 현재 범위의 통합 검증·기여 진입점·미완료 인계 | 적용 검증·보고와 원격 통합 승인; 전체 백로그 완료를 전제하지 않음 |
| 후속 | 기여자·메인테이너의 백로그별 구현·검증, 배포 준비 | 개별 변경의 수용 조건과 배포할 지원 범위의 검증 |

### 6.1 제품 기능을 완결하는 실행 순서

| 순서 | 구현 범위 | 기능 완료에 필요한 관측 |
| --- | --- | --- |
| F1 | 포맷 감지·`ParsedDocument`·Core 공통 초기화, 전체/선택 PDF 옵션 | feature-on에서 직접 IR·원본 쪽 대응, feature-off에서 안정적인 거부; 기존 HWP/HWPX 진입점 보존 |
| F2 | CLI `import-pdf`, 기존 info·렌더·저장 명령, 도움말·JSON | 실제 CLI로 전체/선택 쪽 렌더·저장·재열기, 잘못된 옵션·미지원·예산 오류 및 출력 원자성 |
| F3 | 제품 WASM 생성자·import/diagnostics, Worker deadline·취소·재생성 | 루트 Docker WASM과 실제 JS/Worker에서 열기·렌더·저장; 실패/취소가 현재 세션을 교체하지 않으며 다음 정상 입력 복구 |
| F4 | 같은 실물 PDF를 CLI와 제품 WASM에서 끝까지 처리 | 전체 12쪽·선택 7/8쪽의 원본 대응·내용·표 소속, 출력 생성과 재열기; 단순 byte 동일성을 시각 일치로 대체하지 않음 |

각 경로를 실제 실행하고 기능을 막는 결함은 그 자리에서 해결한다. 기능 경로의 기본 오류
처리·원자성·Worker 복구를 후속 백로그로 미뤄 빈 구현을 기능 완료로 판정하지 않는다.
진단 probe만 성공한 상태도 F3/F4 완료가 아니다. 기존 테스트와 이미 적용한 예산은 유지한다.

### 6.2 발견 사항과 백로그 운영

- 세부 일감은 [백로그](task_m100_7911_backlog.md) 한 곳에서 ID·범위·근거·우선순위·
  종료 증거·상태를 관리한다. 보고서는 해당 ID와 실행 결과를 연결한다.
- 진행 중인 F1–F4를 막지 않는 폰트·코덱·캐시의 추가 보완은 등록 후 제품 연결을 계속한다.
  의존성 전체의 일반 개선·비활성 기능 감사로 현재 범위를 자동 확장하지 않는다.
- F4 이후 현재 기반의 통합 검증을 수행하고, 나머지 백로그는 기여자가 맡을 수 있도록
  진입 코드·재현 입력·독립 기대값·수용 조건을 연결한다. 통합 준비와 후속 구현의 분류는
  백로그 1절을 따른다. 새 발견도 같은 기준으로 분류한다.
- 제품 경로를 완성해도 `parser_allocation_bounds_proven=false`, 기본 비활성, 비신뢰 입력
  배포 보류를 유지한다. 안전성 주장은 감사한 실제 경로와 예산의 범위로 한정한다.
- PR Ready·merge에는 **현재 제출 범위**의 동작·회귀·시각 검증과 미완료 공개가 필요하다.
  후속 세부 보완·비신뢰 입력 배포 검증의 미완료를 일괄적인 devel 통합 차단 사유로 쓰지 않는다.
  기존 기능 회귀·현재 주장과 다른 실행·검증 실패는 해당 범위에서 해결한다. 현행 시각 기준의
  적용/미검증/예외도 명시하며, 공동 구현 취지를 개별 검사 통과로 바꾸지 않는다.
  크기 최적화·OCR·원본 논리 문단 완전 복원은 이번 범위에 자동 추가하지 않는다.
- 이미 고정한 소스·명령·입력의 유효한 증거는 재사용하고, 변경이 영향을 준 검사만 재실행한다.
  최종 제출 때 필요한 제품 head 검증은 별도로 수행한다.

### 6.3 devel 통합과 배포의 판정 분리

| 판정 | 필요한 결과 | 남겨 둘 수 있는 일감 |
| --- | --- | --- |
| 공동 구현 기반의 devel 통합 | F1–F4 실행 가능한 연결, 적용 build/lint·회귀·시각 검증, 기존 HWP/HWPX 보존, 미지원·미검증의 명시, 기여 진입점과 수용 조건 | 기본 비활성 PDF의 추가 자원 감사·폰트/codec 완성·지원 확장·외부 package 게시 준비 |
| 지원 기능의 배포 | 배포할 feature·입력·지원 범위에 필요한 자원·실패 복구·호환성·package 소비 검증과 배포 승인 | 배포 범위 밖으로 명시된 후속 기능 |

알려진 미지원·진행 중·검증 완료 상태를 API 진단과 문서에서 구별한다. 기본 비활성 정책을
유지하며 devel에 코드가 있다는 이유로 자동 배포나 비신뢰 입력 수용을 활성화하지 않는다.
원격 통합 자체는 별도 승인을 따른다. 현재는 제품 연결 전이며 이 문서 수정으로 통합 준비가
완료되었다고 판정하지 않는다.

PR 경로는 `collaborator_self_merge`를 기본으로 `intake_and_review`, `local_validation`,
`visual_fixture_evidence`, `rework_and_exceptions`를 적용한다. 9,337줄 POC 도입을
작은 변경으로 취급하지 않으며 CI 통과만으로 merge하지 않는다.
현재 #7913은 승인된 WIP Draft다. 로컬 후속 변경과 원격 checkpoint를 구분한다.
코드 head CI 성공 뒤에만 review·오늘할일 trailing 기록을 추가한다.
이미 생성한 오늘할일은 보존하되 이번 설계 단계에서 더 갱신하지 않는다.

2026-10-16 지시는 위 순서에 따른 로컬 제품 구현과 후속 백로그 처리를 승인한다.
원격 push·Ready 전환·merge·close·외부 crate 게시·기본 feature 활성화의 포괄 승인은 아니다.
현재 **기능 구현 및 PR 준비 완료 상태는 아니다**.
