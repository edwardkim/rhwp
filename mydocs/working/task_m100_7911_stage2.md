# Task #7911 2단계 진행 기록: pdf_oxide 0.3.78 직접 IR 경계

- 일자: 2026-10-15 (Asia/Seoul)
- 상태: **2026-10-16 실행 순서 변경 — 2단계 checkpoint 정리, 제품 전체 경로 우선 구현으로 인계**.
  기존 집중 검증과 미완료를 보존한다. 전체 자원 안전성·제품 통합 완료 보고가 아니다.
- [수행계획](../plans/task_m100_7911.md), [승인된 구현계획](../plans/task_m100_7911_impl.md)
- 작업 브랜치 `task_m100_7911`, 고정 devel `a5e88ad682c805e1d845502ddc9521b94ea8eccd`
- 승인: 구현계획 승인 후 착수. 후속 지시 “0.3.78에 맞추어 코드를 수정해서 컴파일” 반영.
- 현재 순서: [구현계획 6절](../plans/task_m100_7911_impl.md#6-구현-순서와-승인).
  세부 미완료의 상태·종료 조건은 [백로그](../plans/task_m100_7911_backlog.md)를 따른다.
  3.1–3.22의 단계 유지 문구는 당시 기록이며, 후속 결정과 목적 정정은 3.23–3.24에 기록한다.

## 1. 반영 내용

- merge simulation 성공 뒤 `11a69d2b1`에서 POC `6bc3847e0`의 9개 commit을 보존했다.
  devel과 원격 branch는 변경하지 않았다.
- `src/parser/pdf/`에 optional 제품 frontend를 추가했다. `parse_pdf_with_options`는
  원본 PDF를 한 번 열고 선택 쪽을 하나의 Document/style/resource pool에 lowering한다.
  POC executable 호출이나 HWPX 직렬화·재파싱을 컴파일의 중간 경로로 사용하지 않는다.
- `pdf_oxide 0.3.78`의 bytes·page geometry·span/path·image handle API에 맞췄다.
  이미지 선언 크기/문서 누적 픽셀 수를 decode 전에 확인한다. 이로써 upstream 모든 할당이
  제한된다는 뜻은 아니다.
- 원본 쪽/출력 section 대응, span 관측 위치, 표·중첩·보류 개수, 손실 가능 경고를
  import metadata로 분리했다. 원본 전체 span JSON/POC report는 반환하지 않는다.
- 전체/명시적 선택 쪽, 빈 쪽, 전역 instance/style/image ID를 처리한다.
  `SourceFormat::Pdf`를 사용하고 HWP5/HWPX 레이아웃 출처로 가장하지 않는다.
- 글상자의 `text_margin_explicit`로 명시적 0과 부재를 구분한다. XML parser와 serializer가
  이를 보존하고 기존 생성기의 false 기본값은 유지한다. PDF 생성기는 true를 지정한다.
  **문단이 없는 글상자의 기존 `drawText` 생략 동작은 이번 변경에서 고치지 않았다.**
- POC 모듈은 비교 기준으로 보존하며 source-side unit test를 제품에 복제하지 않았다.
  새 검사는 `tests/cases/` 아래 합성 입력 계약이며 실물 렌더링의 golden이 아니다.

## 2. 의존성 호환성

최초 frontend는 `pdf_oxide =0.3.78`을 사용했다. 보안 patch 승인 뒤에는 동일 원본 기반의
`rhwp-pdf-import-backend =0.3.78-rhwp.1`로 변경했다. PDF 라이브러리 기반 버전을 낮추지 않았다.

자동 선택된 `office_oxide 0.1.13`에서 PDF crate 내부
`src/converters/pdf_to_ir.rs:194`가 `DocumentIR.defined_names` 초기화 누락 E0063으로 실패했다.
root adapter의 호출 변경만으로 제거되는 오류가 아니므로 검증된 `office_oxide =0.1.9`를
optional 호환성 제약으로 명시했다. Cargo registry source는 수정하지 않았다.
이 제약은 Cargo.lock을 채택하지 않는 rhwp 라이브러리 소비자에게도 적용된다.

기존 devel의 clap 4.6.7·zip 9.0.0을 유지했다. 새 PDF 의존성이 요구하는 smallvec 1.16 계열과
uuid 1.26 이상 조건에 따라 각각 1.16.3/1.28.0을 해석했다. office의 zip 8.6은 별도다.
POC의 불필요한 getrandom 0.2 alias는 제거했고 0.4 wasm_js를 재사용했다.
최초 의존성 반영/정리 명령을 제외한 build/lint/test는 `--locked`로 실행했다.

## 3. 실행 증거

공용 target: `/Users/edwardkim/mygithub/rhwp/target/pr-review`.
Cargo 빌드·검사는 순차 실행했다. 로그는 ignored `output/pr-review/issue7911/logs/`에 있다.

| 실행 | 결과 / 로그 |
| --- | --- |
| Native feature-on lib Clippy, `-D warnings` | 통과, `native-pdf-clippy.log` |
| Native feature-off lib Clippy, `-D warnings` | 통과, `default-lib-clippy.log` |
| wasm32 feature-on lib Clippy, `-D warnings` | 통과, `wasm-pdf-clippy.log` |
| Native feature-on lib build | 통과, `native-pdf-build.log`; 최종 manifest 재확인 `native-pdf-build-final.log` |
| wasm32 feature-on lib build | 통과, `wasm-pdf-build.log`; debug artifact이며 브라우저 실행/배포 증거는 아님 |
| PDF 합성 입력 계약 4개 | 실제 4건 통과, `pdf-four-contracts.log` |
| 글상자 부재/0/비영 여백 왕복 계약 | 실제 1건 통과, `margin-final-contract.log` |
| source-side unit tier, pinned base 비교 | 통과, 기존 4,103 tests / 296 modules; 신규 source-side 검사 없음 |

검사 이름:

- `all_pages_share_valid_style_pools_and_unique_instance_ids`
- `selection_is_explicit_and_keeps_source_page_mapping`
- `invalid_or_over_budget_input_does_not_poison_next_import`
- `blank_pages_are_not_silently_dropped`
- `absent_and_explicit_zero_margin_survive_distinctly`

`cargo-nextest`가 설치되지 않아 최초 nextest 명령은 실패했다. 전체 회귀를 cargo test로
대체하지 않고 정본이 허용하는 **정확한 두 test target과 issue_7911 filter의 focused cargo test**를 사용했다.
포맷 전후 신규 source weight 변경으로 첫 실행이 0건이었으며 통과 증거에서 제외했다.
fmt 후 harness 재생성과 실제 module 포함·검사 수를 확인해 다시 실행했다.

여백 검사의 최초 합성 입력에는 내부 문단이 없어 기존 serializer가 drawText를 생략했다.
저장 XML/재파싱 컨트롤로 이를 확인하고 PDF가 생성하는 문단 소유 글상자 범위로 입력을 정정했다.
구역/단 정의를 제외하고 소유 글상자가 정확히 하나인지 찾으며 XML 여백 중복도 검사한다.
실패·진단 로그는 삭제하지 않았다. 위 다섯 건은 실물 배치/한컴 재조판 일치 증거가 아니다.

### 3.1 PDF 포함/제외 WASM 크기 실측

메인테이너의 추가 크기 측정 요청에 따라 비용 검증 일부를 앞당겼다.
ignored `output/pr-review/issue7911/size-probe/`에 동일한 `compile_input` 진입점과
HWPX 출력 경로를 가진 측정 crate를 만들었다. PDF variant는 실제 제품 frontend를 호출해
사용하지 않는 PDF 코드가 LTO에서 제거되는 잘못된 비교를 피했다.

동일 source·lock, wasm32, opt-level 3, LTO=true, codegen-units=1,
strip=debuginfo, 공식 wasm-bindgen 0.2.127의 web 후처리를 사용했다. wasm-opt는 미적용이다.
도구는 공식 release archive와 게시 SHA-256을 검증해 output 안에서만 실행했고 전역 설치하지 않았다.
기존 제품 `pkg/`는 덮어쓰지 않았다.

| 항목 | PDF 제외 | PDF 포함 | 증가 |
| --- | ---: | ---: | ---: |
| 후처리된 WASM | 15,687,986 B (14.96 MiB) | 20,440,106 B (19.49 MiB) | 4,752,120 B (4.53 MiB), 30.29% |
| WASM gzip level 9 | 5,221,058 B (4.98 MiB) | 6,974,683 B (6.65 MiB) | 1,753,625 B (1.67 MiB), 33.59% |
| JS glue | 양쪽 457,395 B, gzip 48,906 B | byte-identical | 0 B |

양쪽 모두 기존 HwpDocument를 포함한 함수 export 496개와 실제 compile_input export를 확인했다.
새 PDF variant의 합성 PDF 변환, 잘못된 PDF 뒤 정상 변환, baseline의 생성 HWPX 재파싱/저장을
web binding을 로드한 Node에서 실행했고 양쪽 6,447 B HWPX를 생성했다.
이는 브라우저 조판/한컴 일치가 아니라 **측정 경로가 실제 실행됨**을 확인하는 smoke다.

PDF crate 자체의 WASM release `.rlib`는 62,330,194 B (59.44 MiB), `.a`는
100,598,214 B (95.94 MiB)였다. Native debug `.rlib`는 약 237.18 MiB다.
서로 다른 profile/target의 중간 파일을 최종 WASM 증가분에 더하거나 메모리 요구량으로 해석하지 않는다.
최종 Studio/확장 bundle은 실제 제품 진입점·wasm-opt·packaging 연결 후 다시 측정해야 한다.

증거는 `output/pr-review/issue7911/size/metrics.json`, `smoke.json`,
`raw/`, `baseline/`, `pdf/`와 `logs/size-*.log`에 보존했다.
metrics는 working-tree source SHA, 도구 버전, source 변경 시각·lock 동일성 검사,
산출 SHA·압축 byte·증가율을 포함한다. 원격 게시·PR 생성은 하지 않았다.
이 크기 수치는 보안 patch 도입 전 source의 측정이다. 아래 새 보안 runtime probe의
산출물을 같은 측정으로 바꾸거나 이전 metrics를 덮어쓰지 않았다.

### 3.2 크기 최적화 제외 결정

2026-10-15 메인테이너가 크기 최적화를 이번 범위에서 제외하고 현재 방식으로 진행하도록 지시했다.
당시 `pdf_oxide =0.3.78` Cargo dependency와 `office_oxide =0.1.9` 호환성 제약을 유지하기로 했다.
이후 별도 승인된 보안 전용 vendoring은 3.4에 기록했다.
크기 축소 목적의 소스 추출·vendoring·추출 전용 fork·별도 WASM 분리는 수행하지 않는다.
위 측정값은 유지하고 최종 제품 bundle 비용은 제품 연결 뒤 기록하되 감축 목표는 두지 않는다.
자원 안전성 게이트와 미검토 dependency 변경의 별도 승인 조건은 그대로다.
이번 반영은 계획·진행 기록에 한정하며 제품 소스와 의존성은 변경하지 않았다.

### 3.3 자원 제한 보완안 조사

2026-10-15 “순서대로 진행” 승인을 받아 자원 제한 보완안부터 진행했다.
[구현계획 4.1](../plans/task_m100_7911_impl.md#41-자원-제한-보완안-도입-승인-구현검증-진행-중)에
실제 호출 경로, 요청별 budget·bounded decode·fatal 오류·이미지의 지연 decode,
초기 정책과 작은 계약을 추가했다. 조사 당시 추가 승인 대기였고, 이후 승인은 3.4에 기록했다.

- constructor의 xref/recovery부터 context가 필요하며, `from_bytes_with_options` 추가만으로
  해결되지 않는다. 일반 object stream과 xref bootstrap은 options 없는 decoder를 사용한다.
- `extract_paths`의 빈 결과 복구와 image handle의 `if let Ok`는 새 예산 오류를 삼키지 않도록
  별도 fatal 전파가 필요하다. 기존 동작을 조사한 것이며 수정·보안 통과를 주장하지 않는다.
- 선언 이미지 픽셀만으로 충분하지 않다. 실제 JPEG decode가 `to_png_bytes`까지 미뤄질 수 있어
  codec header·색 변환·인코딩까지 포함해야 한다. 구문 깊이·객체 수·predictor 산술도 범위에 포함했다.
- 0.3.78 기반 최소 보안 patch를 권고하되 공식 source/Cargo 캐시는 수정하지 않는다.
  수정 dependency의 명칭·NOTICE·library 소비자/package 경로를 별도로 확정한다.
  크기 최적화를 다시 범위에 넣거나 미검토 fork를 이미 도입한 것이 아니다.

정적 관측 14개를 파일별 source SHA와 함께
`output/pr-review/issue7911/resource-audit/source-audit.json`에 기록했다.
진단 스크립트는 같은 폴더의 `source-audit.mjs`다. 최초 실행은 로컬에 없는
`.cargo-checksum.json` 조회에서 실패했으며, Cargo.lock의 식별자와 실제 파일 SHA를 기록하도록
정정해 재실행했다. 이 감사는 archive 무결성 검증·공격 재현·런타임 보안 검사와 구분한다.
이번 보완안 조사 중에는 제품 코드·dependency 변경이나 Cargo build/test를 실행하지 않았다.

### 3.4 승인된 최소 보안 patch 적용

아래는 최초 보안 patch source의 검증 기록이다. 후속 승인에 따른 새 변경과 재검증은 3.5에
구분하며, 이전 source의 빌드·패키지·WASM 결과를 새 source의 완료 증거로 재사용하지 않는다.

메인테이너가 “0.3.78 기반 최소 보안 패치 도입 승인”을 선택한 뒤 적용했다.
공식 registry source/Cargo 캐시는 수정하지 않았다. root의 `pdf_oxide` alias는
`vendor/rhwp-pdf-import-backend`의 `rhwp-pdf-import-backend =0.3.78-rhwp.1`을 소비한다.

- 원본 `.crate` SHA-256 `95b1ecc1a60a314fbd1aea5fe8f700ec32af1efa70fef5c6879162444da3437d`가
  기존 Cargo.lock checksum과 일치했다. archive의 401개 파일 hash를 `UPSTREAM_FILES.json`에 보존했다.
  source Rust 변경은 19개 기존 파일과 2개 새 모듈이다. 쓰기/rendering 파일 8개 안팎의 변경은
  budget을 보존하는 stream byte type의 기계적 호환성 수정이며 기능을 제거하지 않았다.
- 동일 upstream commit `ad49c4cb3638dc950a29350ef882161372fde473`의 rustfmt 설정을 gh로 조회해
  vendor 안에 보존했다. 초기 root 포맷 적용의 250개 format-only 변경은 이번 생성분에서 제거했다.
  루트 포맷 정책은 변경하지 않았다. 원본과 다른 파일은 실제 hash로 추적한다.
- root의 명시적 workspace member `.`가 vendor 제외를 무시하게 하는 현상을 별도 최소 manifest로
  재현했다. 중복 `.`만 제거하고 implicit root·기존 default-members를 유지했다.
  metadata member 집합은 원래 10개와 같으며 backend는 독립 workspace다.
  임시로 유입됐던 vendor 개발용 lock 항목은 분리 후 root Cargo.lock에서 정리됐다.
- `from_bytes_with_limits`에서 요청별 budget을 만들고 xref bootstrap·일반 object stream까지 전달한다.
  `StreamData` clone이 동일 context를 보존한다. 제한된 경로는 암호 PDF와 손상 xref의 전체 복구를 거부한다.
- bounded writer/read, Flate/Brotli/LZW/RunLength/ASCII, 다단 filter와 predictor 산술을 적용했다.
  출력 확장·명시적 예약 전에 검사하며 정확한 cap의 정상 입력을 수용한다. 잘린 Flate prefix는 성공으로 만들지 않는다.
- 구문 깊이·객체/컬렉션·xref 항목과 `/N`, 순차 content operator·graphics stack·문자열·표시 텍스트·span을 제한한다.
  기존 global cap이나 thread-local budget을 새 안전장치로 사용하지 않는다.
- 이미지 추출의 선언/최대 버퍼, 실제 JPEG header·decoder 한계, PNG bounded writer를 연결했다.
  정상 RGB JPEG에는 기존 image decoder를 유지한다. 선언/헤더 불일치는 거부하고 합성 정상 PNG는
  legacy 출력과 byte-identical임을 검사한다. 별도 이미지 문서 누적 128 MiB 정책을 이미 구현했다고 쓰지 않는다.
- fatal 상태를 backend·제품 frontend 경계에서 검사하고 `PDF_RESOURCE_LIMIT`로 매핑한다.
  다음 문서는 독립 budget을 사용한다. 공통 renderer에 PDF 분기를 추가하지 않았다.
- `THIRD_PARTY_LICENSES.md`를 실제 내부 package명·버전·변경 경계로 갱신했다.
  MIT/Apache license·NOTICE·상표 정책·폰트 고지 6개가 원본 hash와 일치한다.
  `publish=false`이며 외부 게시·배포는 수행하지 않았다.

#### 적용 범위와 미입증 범위

현재 decoded 예산은 stream 단계 32 MiB / 문서 누적 256 MiB다. 원본 복사·filter 중간 출력·재시도·
predictor scratch·이미지 변환 작업도 해당 경로의 실제 charge에 포함한다. 입력 16 MiB와 기존 쪽/표
예산은 유지한다. 구문 깊이 16, 객체 50,000개/컬렉션·xref 합계 100,000개,
operator 250,000개/graphics stack 128, 단일 문자열 64 KiB/표시 텍스트 2 MiB/span 40,000개를 적용했다.
이미지는 아직 별도 64/128 MiB 정책이 아니라 공통 stream·decoded 예산을 사용한다.

codec 내부 scratch, 폰트/CMap 확장·공유 cache, reference/Form/page traversal의 제안된
32/16-depth 정책, 합성 content/cache clone·metadata의 총 byte 예산, Worker deadline은 전체 입증 전이다.
실물 전체 페이지의 Native/fresh WASM 시각·총 메모리 한계·배포된 rhwp 소비자 검증도 남아 있다.
**metadata의 `parser_allocation_bounds_proven=false`와 배포 보류는 유지한다.**

#### 이번 작은 계약 결과

`tests/cases/issue_7911_pdf_resource_limits.rs`의 19개 검사 실제 실행이 모두 통과했다.
로그는 `security-resource-contracts-final.log`다. 최초 실행은 fmt 이후 suite 배정과 harness가
어긋나 0건이었고 통과 증거에서 제외했다. 최종 fmt 뒤 prepare하여 실제 module 포함과 19건을 확인했다.
새 검사는 합성 parser/자원 계약이며 실물 배치의 golden 또는 전체 보안 통과가 아니다.

- Flate/LZW/RunLength/ASCII의 cap-1/cap/cap+1, Brotli 127/128/129 byte 경계
- 여러 stream의 누적·sticky 실패·문서 독립성, filter 중간 출력, predictor의 0/overflow
- 구문 깊이/객체/컬렉션, stream clone의 원 context, object stream N의 capacity 전 거부
- traditional/xref-stream bootstrap의 예산 전달, decode 오류를 빈 paths 성공으로 만들지 않음
- content operator/graphics stack와 중첩 operand, 제품 오류 코드와 다음 정상 import
- 컨테이너 객체 quota와 content operand 계상 분리, operator당 64/65개 경계, 최초 fatal 사유 보존
- 실제 JPEG header 불일치·정상 legacy PNG 일치, 거대 이미지 선언의 픽셀 할당 전 거부

이 API 도입 전의 동등한 실행 FAIL 증거와 모든 codec의 allocation instrumentation을
확보했다고 주장하지 않는다. 정적 source 관측, Native 실행 계약, WASM compile/runtime는 구분한다.

#### 실물 PDF와 계상 오류 수정

`pdf/issue5875/nested_table_text_caption-2020.pdf`(525,733 bytes, 12쪽)를 Native에서 실행했다.
초기 실행에서는 생성자 객체 7개·page tree 탐색 후 33개였으나, content 수치 operand까지
컨테이너 객체에 계상해 spans 추출 중 객체 50,000개 한도를 소진했다.
최초 fatal 사유를 보존하도록 수정해 `parsed object count`임을 확인했다.
객체 quota를 늘리지 않고 content operand를 operator 영역으로 분리했다.
operator당 operand 64개와 operand 내부 중첩·컬렉션·문자열 제한은 유지했다.

수정 후 전체 12쪽 HWPX 생성 약 4.54초, 7쪽 단독 0.56초, 8쪽 단독 0.56초였다.
7·8쪽은 각각 표 5개 중 nested 4개를 보고하며 실제 HWPX를 생성했다.
전체 12쪽 중 7쪽은 semantic table 복원을 보류한 것으로 metadata에 남고 선 기반 fallback을 사용한다.
해당 실행의 변환 성공은 한컴 조판·시각 일치·모든 표 인식 완료를 뜻하지 않는다.
전후 진단 로그는 `security-real-pdf-{compile,reason,phase-trace,domain-fix}.log`,
결과는 `output/pr-review/issue7911/resource-real-pdf/summary.json`과 각 하위 폴더에 있다.
probe의 프로세스 exit code만이 아니라 세 selection의 `ok=true`를 확인했다.

기존 frontend 합성 계약 4건과 명시적 여백 왕복 계약 1건도 보안 패치 뒤 재실행해 통과했다.
로그는 `security-frontend-contracts-final.log`, `security-margin-contract-final.log`다.

#### 최종 소스의 제한된 빌드·패키징 검증

| 실행 범위 | 결과 / 로그 |
| --- | --- |
| Native feature-on lib Clippy | 통과, `security-native-clippy-final.log` |
| Native default lib Clippy | 통과, `security-default-clippy-final.log` |
| wasm32 feature-on lib Clippy | 통과, `security-wasm-clippy-final.log` |
| backend 독립 workspace·default-off lib Clippy | 통과, `security-backend-clippy-final.log` |
| backend 독립 workspace·default-off wasm32 lib Clippy | 통과, `security-backend-wasm-clippy-retry.log` |
| Native / wasm32 feature-on lib build | 통과, `security-{native,wasm}-build-final.log` |
| 로컬 backend `cargo package --no-verify` | 통과, `security-package-local.log` |
| 압축 해제한 package를 소비한 경계 실행 | 통과, 초기 해석 `security-package-consumer-resolve.log`, locked 재실행 `security-package-consumer-locked.log` |
| 압축 해제한 package 소비자의 wasm32 check | 통과, `security-package-wasm-consumer-final.log` |

Clippy는 모두 `-D warnings`, 위 lib build/lint는 `--locked`를 사용했다.
backend 독립 WASM lint의 첫 offline 실행은 캐시에 없는 js-sys 0.3.105 다운로드에서 중단됐다
(`security-backend-wasm-clippy-final.log`). 이 환경 실패를 코드 실패로 세지 않았다.
locked 상태에서 필요한 공개 crate를 다운로드하도록 offline만 해제한 재실행이 통과했다.
그 외 표의 lib build/lint와 package 소비자 check는 offline이었다.
backend 독립 Clippy는 자체 lock을 사용하므로 root 제품 lock과 동일한 transitive graph 검사로
표시하지 않는다. 이 lock의 wasm-bindgen 0.2.128은 독립 compile/lint에만 사용했다.
실제 root 연결·fresh binding 실행은 root/probe lock의 0.2.127이며 이를 업그레이드하지 않았다.
소비자 probe는 초기 해석 뒤 lock을 고정해 재검사했다.
backend는 workspace member에 포함하지 않아 개발용 default feature가 제품 기본값을 바꾸지 않는다.

`cargo package`는 407개 파일, 21.6 MiB 원문 / 6.5 MiB 압축의 로컬 `.crate`를 생성했다.
이는 전송용 WASM 비용이 아니며 크기 최적화 목표를 다시 도입한 것이 아니다.
압축 해제한 391개 소스·고지·설정 파일은 현재 vendor와 byte-identical이었다.
Cargo가 루트 `Cargo.toml.orig`를 재생성하므로 upstream 원본은 별도
`upstream/Cargo.toml.orig`로 포함하고 원본 SHA와 대조했다.
증거는 `output/pr-review/issue7911/package-validation/integrity.json`에 있다.
소비자 실행은 이 **로컬 backend package**의 API/경계 확인이며,
crates.io에 게시된 rhwp 소비자가 이 dependency를 해석할 수 있음을 입증하지 않는다.
backend는 계속 `publish=false`이고 외부 배포 모델은 별도 승인 전이다.

#### fresh WASM 실행과 Native 대조

새 ignored `output/pr-review/issue7911/security-wasm-probe/`에서 현행 root frontend를 직접
링크한 release WASM을 빌드하고 공식 wasm-bindgen 0.2.127로 **web binding**을 생성했다.
기존 제품 `pkg/`·Studio public bundle과 앞선 size probe는 변경하지 않았다.
초기 lock 해석 뒤 `--locked --offline` 재빌드가 통과했다.
로그는 `security-wasm-probe-build-{resolve,locked}.log`, `security-wasm-bindgen-final.log`다.

Node에서 web binding을 실행한 `security-wasm-runtime-final.log`와 `summary.json` 결과:

- 잘못된 bytes는 `PDF_INVALID_INPUT`, 과도한 구문 중첩은 `PDF_RESOURCE_LIMIT`로 거부했다.
- 같은 instance에서 위 두 실패 뒤 정상 합성 PDF를 변환했다.
- 실물 전체 12쪽과 7·8쪽을 변환했고 HWPX ZIP의 section XML을 xmllint로 파싱했다.
  전체 12개 section/표 13개, 선택 쪽 각각 1개 section/표 5개를 확인했다.
- 전체 변환 약 0.347초, 선택 쪽 각각 약 0.026초였다. Native는 debug, WASM은 release이므로
  이 숫자로 두 backend의 상대 성능을 판정하지 않는다.
- 관측된 WASM 선형 메모리는 전체 실행 후 23,199,744 bytes(22.125 MiB)였다.
  이는 현재 메모리 용량 관측이며 JS heap/RSS·임의 입력 상한·live allocation 증명이 아니다.
- 세 selection 모두 Native와 HWPX SHA-256이 같았다.
  `native-comparison.json`에 전체 checksum을 보존했다. 이 동일성은 시각 일치나 임의 문서의
  Native/WASM 동등성 증명이 아니다.

이 검증용 export `compile_input`은 3단계 제품 API의 선행 대조 수단이다.
Studio·브라우저 확장 UI 연결, 실제 브라우저 Worker 취소·재생성, 한컴 Print 및 시각 gate를
통과했다고 표시하지 않는다.

통합 suite 수집과 source-side unit tier 정책을 고정 base와 비교해 통과했다.
로그는 `security-integration-policy-final.log`, `security-unit-policy-final.log`다.
`cargo fmt --all -- --check`도 통과했다(`security-fmt-final.log`).
증적은 미커밋 작업 트리에서 생성했으며 HEAD만으로 변경분을 식별하지 않는다.
`output/pr-review/issue7911/resource-audit/security-source-snapshot.json`에 관련 426개 파일의
SHA-256과 HEAD, 원본 대비 21개 변경 파일(기존 Rust 19개 + manifest/lock)을 기록했다.
추가된 두 Rust 모듈·고지·upstream manifest 사본은 파일별 목록에 별도로 포함한다.

### 3.5 후속 승인: 폰트·CMap·content cache 보완

2026-10-15 메인테이너가 “다음 절차 진행을 승인합니다”로 후속 자원 감사·보완을 승인했다.
2단계를 계속하며, 3단계 제품 API 연결이나 배포 준비 완료로 전환하지 않았다.

#### 확인한 결함과 적용 경로

- `LazyCMap::get`의 전역 cache는 다른 문서에서 만든 결과를 재사용한다. 제한된 import의
  budget을 거치지 않는 재사용을 막기 위해 `new_with_budget` 경로는 전역 cache를 우회한다.
  raw source와 local parsed 결과는 Arc로 clone 간 공유하고 원래 요청 budget을 보존한다.
- CMap source 1 MiB, 개별 line은 기존 문자열 한도 64 KiB, 요청 누적 매핑 작업 100,000개,
  보수적 scan 작업 64 MiB를 검사한다. 범위의 산술·개수는 확장 전에 확인하고,
  중복/덮어쓰기·배열의 임시 항목도 계상한다. 제한된 sequential 범위는 기존 10,001개
  묵시적 truncation 대신 전체 수용 또는 명시적 실패로 처리한다.
- CMap hex가 비ASCII이면 byte 경계 slice 전에 거부해 UTF-8 경계 panic을 막는다.
  유효한 다문자/보조평면 매핑 처리는 기존 parser를 사용하며 별도 PDF/CMap parser를 복제하지 않는다.
- CID `/W`·`/W2` 배열/범위는 요청별 별도 100,000-entry 예산을 확장·예약 전에 소비한다.
  CMap quota와 다른 counter다. FontInfo가 descendant parsing 오류를 fallback 성공으로 바꿀 수 있어
  공개 `from_dict`의 진입·반환 경계에서 sticky fatal 상태를 다시 확인한다.
- `cached_page_content`의 복수 stream 결합에는 separator까지 개별 buffer cap과 누적 예산을
  적용한다. `get_page_content_data`의 반환 복사도 같은 예산을 소비하며 fatal 뒤 cache hit는 거부한다.

CMap source의 24배 scratch 단위 및 mapping/metric당 128-byte 단위는 보수적 작업 정책이다.
실측 allocator 크기·총 heap/RSS 한계나 전체 parser의 선행 할당 입증으로 해석하지 않는다.
기존 32/256 MiB byte budget에는 이 구조 작업 계상도 포함하므로 실제 decoded output 합계와
동일한 수치라고 부르지 않는다. font program/대체 인코딩·Unicode memo와 나머지 cache는 미입증이다.

#### 수정 전 실패와 수정 후 대조

직전 `.crate`의 Rust source를 보존한 대조군을 별도 작은 프로세스에서 실행했다.
`security2-before-probe/integrity.json`은 385개 Rust 파일이 원 패키지와 동일함을 기록한다.
package/lib 이름과 crate type만 진단용으로 구분했으며 기존 패키지를 변경하지 않았다.

| 입력/경계 | 수정 전 관측 | 수정 후 계약 |
| --- | --- | --- |
| 비ASCII CMap hex `AAA` + invalid UTF-8 + `AAAAA` | byte index 4 char-boundary panic, exit 101 | legacy/limited parser 모두 panic 없이 malformed 항목 생략 |
| 4-byte content, 누적 16-byte 예산, 3회 반환 복사 | 세 번째도 성공해 계약 assertion 실패, exit 101 | 세 번째 backend `ImportResourceLimit`, 이후 cache hit도 실패, 새 문서는 정상 |
| CID 3-entry 범위와 2-entry 예산 | 최초 신규 검사에서 fatal은 설정되나 FontInfo가 Ok 반환; 28/29 통과 | 공개 반환 경계 보완 뒤 29/29 통과 |

로그는 `security2-before-unicode-isolated.log`, `security2-before-cache-isolated.log`,
`security2-resource-contracts.log`, `security2-resource-contracts-final.log`다.
CMap quota 도입 자체는 동일 API의 수정 전 실행 증거가 없으므로 위 세 재현과 구분한다.

첫 비격리 대조 실행은 예상과 달리 성공했다(`security2-before-unicode.log`).
같은 lib 이름의 비해시 산출물을 공유 target에서 재사용하는 영향을 배제할 수 없어 증거에서 제외했다.
진단 이름을 분리해 실제 수정 전 source를 새로 컴파일한 뒤 두 실패를 확인했다.
내부 제품 backend도 Rust 의존성용 `rlib`만 생성하도록 바꾸어 Cargo의 package/source별 hashed
artifact를 사용하게 했다. 원본 C API·writer 코드는 보존했으며 독립 cdylib/staticlib 배포는
이 내부 package의 계약이 아니다. 크기 최적화나 새 target directory 생성·cache 삭제는 하지 않았다.

#### 재검증 현황

rlib 변경 전 29개 자원 계약과 기존 frontend 4개·여백 1개가 실제 통과했다.
rlib 변경 후의 같은 34개도 재실행해 실제 통과했다. 로그는
`security2-resource-rlib.log`, `security2-frontend-rlib.log`, `security2-margin-rlib.log`다.
Native PDF-on/default lib Clippy, wasm32 PDF-on lib Clippy, backend 독립 Native/WASM
default-off lib Clippy가 모두 `--locked --offline`, `-D warnings`로 통과했다.
로그는 `security2-{native,default,wasm}-clippy.log`, `security2-backend-{native,wasm}-clippy.log`다.
실행용 fresh WASM과 package 재검증도 아래와 같이 통과했다. 기존 3.4의 artifact로 대신하지 않았다.

실물 PDF는 rlib 변경 뒤 전체 12쪽 및 7·8쪽 변환이 통과했다.
전체 HWPX 생성 약 4.44초, 선택 쪽 각각 약 0.50/0.51초였다(Native debug).
각 선택 쪽의 표 5개/nested 4개와 12개 section을 확인했다.
로그 `security2-native-real.log`, 결과 `output/pr-review/issue7911/security2-real-pdf/summary.json`.
시각·한컴 Print·Worker 격리 검증은 아니다. 원래 7개 쪽의 semantic table 보류도 유지한다.
세 selection의 HWPX SHA-256은 직전 보완 전 Native 생성물과 동일했다
(`security2-real-pdf/before-comparison.json`). 이 동일성은 해당 샘플의 직렬화 대조이며,
다른 PDF/기존 형식의 전체 무회귀나 독립 정답 PDF의 시각 일치 판정이 아니다.

고정 base 대비 integration suite 수집·source-side unit tier 정책도 통과했다
(`security2-integration-policy.log`, `security2-unit-policy.log`). 새 검사는 source-side unit이
아닌 `tests/cases/`의 합성 자원 계약이며 렌더링 golden을 추가하지 않았다.
새 미커밋 source 426개 파일 SHA와 원본 대비 23개 변경 파일(기존 Rust 21개 + manifest/lock)을
`resource-audit/security2-source-snapshot.json`에 기록했다. 초기 source fingerprint는 보존했다.

#### fresh WASM·패키지 최종 확인

새 `security2-wasm-probe/`는 현재 제품 Rust frontend를 직접 링크한다.
WASM release build와 locked/offline 재빌드, 공식 wasm-bindgen 0.2.127 web 후처리 뒤
Node 실행이 통과했다. `security2-wasm-{release,release-locked,bindgen,runtime}.log`에 기록했다.
기존 제품 `pkg/`·Studio public bundle은 변경하지 않았다.

- 잘못된 bytes, 과도한 object 중첩, CMap 범위 초과를 예상 코드로 거부했다.
- 같은 instance에서 위 실패 뒤 정상 PDF를 처리했고 비ASCII CMap 입력도 panic 없이 처리했다.
- 실물 전체 12개 section/표 13개, 7·8쪽 각각 1개 section/표 5개를 ZIP·XML 파싱으로 확인했다.
- 전체 변환 약 0.330초, 선택 쪽 약 0.026초였다(release; Native debug와 속도 비율을 계산하지 않음).
- 관측 선형 메모리 용량은 전체 변환 후 22,347,776 bytes였다. live heap/RSS 또는 상한 증명이 아니다.
- 세 selection의 HWPX가 최신 Native 결과와 byte-identical이었다.
  `security2-wasm-probe/{summary,native-comparison}.json`에 보존했다.

새 로컬 `.crate` 생성 뒤 압축 해제한 소비자에서 decode 경계·CMap 제한/새 요청 정상 처리·
비ASCII 입력 안전성을 실행했고, locked 재실행과 독립 wasm32 check도 통과했다.
로그는 `security2-package.log`, `security2-package-consumer-{resolve,locked}.log`,
`security2-package-wasm-consumer.log`다. 소비자 compile 로그에서 실제 압축 해제 경로의
backend가 새로 컴파일된 것을 확인했다.

`security2-package-validation/integrity.json`은 패키지의 391개 source/고지/설정과 현재 vendor의
동일성, 6개 license/NOTICE/상표 고지 및 upstream manifest 사본의 원본 hash 일치를 기록한다.
직전 `.crate`는 `previous.crate`로 별도 보존했다. 외부 게시·배포나 게시된 rhwp 소비자 지원을
검증한 것은 아니다. `publish=false`와 별도 배포 승인 경계는 유지한다.

포맷 검사도 통과했다(`security2-fmt.log`). 수정 전 진단 실패와 환경/캐시 영향 배제,
최종 source의 Native 계약·WASM 실행·package 소비 검증을 서로 구분했다.

#### 남은 자원 게이트

- page/reference 탐색의 요청별 depth 32 및 방문량, Form 경로의 depth/fatal 전파는 미완료다.
  특히 `collect_all_pages`의 visited 검사는 같은 노드 순환을 막지만 깊은 비순환 트리의
  요청별 depth 계약을 대신하지 않는다. 기존 `process_xobject`의 조용한 depth/개수 skip도
  새 제한 계약의 명시적 실패 증거로 세지 않는다. 이 항목은 정적 관측이며 공격 실행 증거는 아니다.
- codec 내부 scratch·mask/palette/색 공간·별도 이미지 64/128 MiB 정책,
  embedded font/대체 인코딩·Unicode memo·나머지 object/span/postprocess cache는 전체 감사 전이다.
- 실제 브라우저 Worker deadline·종료/재생성·제품 UI 복구는 실행하지 않았다.
  Node에서 같은 instance의 정상 반환형 오류 뒤 재사용한 결과와 구분한다.

따라서 `parser_allocation_bounds_proven=false`, 2단계 진행 중·비신뢰 입력 배포 보류를 유지한다.
이번 후속 승인으로 3단계 제품 공개 API 연결·전체 회귀·commit/PR/merge/close를 완료했다고
보고하지 않는다.

### 3.6 중간 결과 커밋·PR 요청 (2026-10-15)

메인테이너가 “커밋 후 PR 오픈 진행합니다”로 현재 변경의 커밋과 PR 생성을 요청했다.
이번 커밋은 2단계 중간 결과 보존이며 단계 완료·3단계 전환·merge·이슈 종료 승인이 아니다.
426개 source/설정/고지 파일이 3.5의 최종 검증 fingerprint와 동일함을 커밋 준비 시 확인했다.
기존 34개 계약과 Native/fresh WASM·로컬 package 결과를 재사용하되 전체 회귀·시각 검증
통과로 확대 해석하지 않는다. 생성 harness·로그·변환물·환경 파일은 커밋하지 않는다.

현재는 아래 미완료 항목 때문에 Open PR의 준비 완료 조건을 충족하지 않는다.
WIP 공유용 Draft PR로 진행할지, 남은 구현·검증 후 Open PR로 진행할지 확인을 요청했다.
이 기록 시점에는 원격 push·PR 생성은 미실행이며, 기존 배포 보류와 보안 게이트를 유지한다.
PR 번호를 예측한 review·오늘할일 trailing 문서는 만들지 않는다.

staged 430개 파일에서 환경 파일·output·파생 suite가 제외됐고, 426개 검증 fingerprint와
index의 내용도 일치했다. `git diff --cached --check`는 새로 보존하는 upstream 원본
4개 파일의 기존 trailing whitespace 6곳을 보고했다. 네 파일 모두 원본 hash와 동일하므로
보안 patch와 무관한 포맷 변경은 하지 않았다. vendor를 제외한 staged 변경 검사는 통과했으며
전체 검사가 통과했다고 기록하지 않는다. upstream 대비 실제 보안 변경의 검사는 3.5와 구분한다.

### 3.7 Open PR 전 완료 선택과 탐색 제한 보완 (2026-10-15)

메인테이너는 3.6의 질문에 **“남은 작업 완료 후 Open PR”**을 선택했다.
중간 결과는 `9a5c6603f661287c50ff766f42329177dd4bc34a`에 커밋했고 작업 트리가 clean함을
확인했다. `gh pr list --head task_m100_7911 --state all` 결과는 빈 목록이었다.
Draft PR을 만들거나 미완료 게이트를 생략하지 않으며 원격 push·PR 생성은 아직 하지 않았다.

#### 이번 보완의 경계

- 요청별 `reference_depth=32`, `reference_visits=100000`, `form_depth=16`,
  `xobject_visits=10000` 정책을 추가했다. 0은 제한 해제가 아니라 오류다.
- 페이지 트리의 실제 깊이는 root=0인 edge 수다. 단일 쪽·bulk cache·쪽수 scan·참조 열거에
  같은 깊이/방문량 예산을 적용한다. 제한된 쪽수 계산은 `/Count`나 조용한 enumeration 복구 대신
  제한된 실제 트리를 순회한다. 참조 열거의 flat-tree count hint도 제한된 경로에서는 쓰지 않는다.
- `load_object`는 cache hit를 포함해 방문량을 소비하고, 요청별 활성 객체 로드 수를 진입 전에
  검사한다. RAII로 반환 시 활성 수를 복구한다. 활성 동시 로드 수의 상한은 모든 논리적 참조
  helper의 재귀 깊이를 감사했다는 뜻이 아니다. 기존 thread-local counter는 원본 경로에 유지한다.
- cache hit를 반환하는 `get_page`와 `load_object` 경계도 sticky fatal 상태를 확인한다.
  tree recursion·bulk fallback에서 자원 실패를 branch skip이나 부분 성공으로 바꾸지 않는다.
- 제품 frontend가 소비하는 텍스트·path·image-handle의 Form 경로에 요청별 방문량과 깊이 정책을
  적용했다. 텍스트 경로의 기존 depth 10/500회 조용한 생략은 제한된 경로에서 사용하지 않는다.
  텍스트 Form의 span/stream cache 우회와 fatal 확인도 추가했다. path/image의 전체 cache 복사,
  별도 materialized-image API 및 기타 참조 helper의 전수 감사 완료를 주장하지 않는다.

#### 수정 전 재현과 환경 중단

먼저 기존 29개 자원 계약에 새로운 5개 계약만 추가하고 제품 source는 `9a5c6603f` 상태로
실행했다. `traversal-before-contracts.log`는 실제 **29개 통과 / 5개 실패**, exit 101이다.

| 추가 계약 | 수정 전 관측 |
| --- | --- |
| 페이지 tree 깊이 31/32/33 | 한도를 넘은 33도 성공 반환 |
| 비정상 Count의 깊은 tree scan | 자원 오류가 아닌 성공 반환 |
| fatal 이후 cached page/object | cached page가 성공 반환 |
| Form 깊이 15/16/17 | 허용할 깊이의 `VISIBLE` 텍스트도 기존 depth 10에서 조용히 누락 |
| 실제 PDF frontend의 깊은 입력/새 요청 | 깊은 입력이 오류 없이 성공해 계약 실패 |

보완 후 같은 명령을 실행했으나 `librhwp.rlib` 생성에서 **No space left on device**로 실패했다
(`traversal-after-contracts.log`, exit 101, 약 63.3초). 테스트 본문은 실행하지 않았으며
**수정 후 PASS 또는 코드 결함 재현으로 세지 않는다**. 이후 각 추출 mode의 Form 한도,
cache-hit 방문량, bulk tree cache, 실제 leaf 쪽수와 zero 정책의 계약 5개를 더 보완했다.
현재 자원 계약은 총 39개이며 이번 최종 수정본의 실행 결과는 아직 없다.
내부 `all_page_refs` 직접 호출용 공개 test API는 추가하지 않았다. 참조 열거 변경의 실제 소비자
경로 실행은 별도로 미검증이며 공개 쪽수 API의 계약과 구분한다.

확인 시 디스크 여유 약 527 MiB, 공유 `target/pr-review` 약 58 GiB, 그중 Native debug
증분 cache 약 25 GiB였다. 실행 중인 Cargo가 없음을 확인했으나 공유 target은 정리하지 않았다.
메인테이너가 **“직접 공간 확보 후 알림”**을 선택해 추가 빌드·테스트를 중단했다.
소스 포맷 정리만 수행했으며 `traversal-final-fmt.log`는 root format 명령 결과다.
vendor 3개 수정 파일도 원래 vendor rustfmt 설정으로 정리했다.

공간 확보 알림 뒤 최종 source의 39개 자원 계약·기존 frontend/여백 계약부터 재실행한다.
작은 계약이 통과한 뒤 Native/WASM·실물/패키지 검증을 수행하고 나머지 자원 게이트를 계속한다.
`parser_allocation_bounds_proven=false`, 2단계 진행 중·비신뢰 입력 배포 보류는 유지한다.
이번 보완 source와 기록은 재검증 전 미커밋이며, 기존 커밋의 34개 통과 결과를 적용하지 않는다.

### 3.8 공간 확보 후 탐색 계약 재검증 (2026-10-15)

메인테이너의 “공간 확보했습니다” 알림을 받아 빌드·테스트를 재개했다.
최초 여유는 약 4.2 GiB였고 다른 Cargo 작업이 없음을 확인했다. 공유 target·cache·기존 증적은
삭제하지 않았다. 아래 기록은 `9a5c6603f` 위 이번 탐색 보완 source의 실행 결과다.

#### 재개 중 추가 확인과 보완

최초 실행은 39개 중 37개 통과/2개 실패였다
(`traversal-resume-contracts.log`, 진단 보강 `traversal-resume-diagnostic.log`).

- 실제 leaf가 2개지만 `/Count=1`인 합성 tree에서 `page_count`는 2를 반환했으나
  `get_page(1)`은 여전히 `/Count`로 subtree를 건너뛰었다. 제한된 페이지 선택에서도 이 hint를
  사용하지 않도록 수정했다. 정상 tree뿐 아니라 쪽수 계산/선택의 같은 소비 계약을 확인한다.
- Form 방문량 계약의 실패 mode는 text였다. 공개 span 추출은 후처리의 glyph-origin stamp에서
  character 해석을 한 번 더 실행한다. 3개 Do의 두 해석을 합쳐 6회가 정확한 작업량이며,
  path/image-handle은 각각 3회다. 기대값을 각 작업량의 한도-1/한도/한도+1로 정정했다.
  제품의 문서 한도 10,000은 변경하지 않았고 반복 작업 계상을 우회하지 않았다.

#### 실제 통과 결과

| 검증 | 결과 | 로그 |
| --- | --- | --- |
| 자원 계약 | 39/39 통과, 테스트 본문 실행 | `traversal-resume-contracts-fixed.log` |
| 기존 frontend 계약 | 4/4 통과 | `traversal-frontend-contracts.log` |
| HWPX 여백 존재성 계약 | 1/1 통과 | `traversal-margin-contract.log` |
| 제품 PDF-on Native/WASM32 lib Clippy | 각각 locked/offline·`-D warnings` 통과 | `traversal-{native,wasm}-clippy.log` |
| backend 독립 Native/WASM32 default-off lib Clippy | 각각 locked/offline·`-D warnings` 통과 | `traversal-backend-{native,wasm}-clippy.log` |
| 실물 전체 12쪽·선택 7/8쪽 Native 변환 | 3건 모두 실제 성공 | `traversal-native-real.log` |
| fresh WASM release·web binding·실제 실행 | locked/offline build 약 6분 53초, smoke 통과 | `traversal-wasm-{release,bindgen,runtime}.log` |

총 44개 계약이 통과했다. 3.7의 최초 5개 실패 계약도 동일 이름으로 통과했다.
이 수치는 전체 nextest·workspace/all-targets·시각 게이트의 실행 결과가 아니다.

실물 출력은 `output/pr-review/issue7911/traversal-real-pdf/`에 보존했다.
전체 약 4.41초·선택 쪽 약 0.508/0.539초(Native debug), 각 선택 쪽 표 5개/nested 4개,
전체 12개 section을 확인했다. 3건의 `ok=true`를 JSON에서 별도로 확인했고,
`before-comparison.json`에서 직전 Native 결과와 HWPX가 byte-identical임을 확인했다.
7개 쪽의 semantic table 보류는 유지한다. 한컴 Print·시각 판정·모든 문서의 무회귀 증거는 아니다.

fresh WASM은 기존 probe의 Rust frontend를 현재 source로 다시 빌드해
`traversal-wasm-runtime/`에 새 binding·결과를 보존했다. 이전 `security2-wasm-probe/pkg/`와
제품 `pkg/`·Studio public bundle은 변경하지 않았다.

- 페이지 깊이 33·Form 깊이 17은 `PDF_RESOURCE_LIMIT`로 거부했다.
  깊이 32·16은 성공했고 후자는 HWPX section XML에 `VISIBLE` 텍스트가 보존됐다.
- 잘못된 bytes·lexical 깊이·CMap 범위 초과까지 포함한 5건의 실패 뒤, 같은 instance에서
  정상 입력을 처리했다. 비ASCII CMap도 panic 없이 처리했다.
- 실물 전체 변환 약 0.329초, 선택 쪽 약 0.026초였으며 전체 12개 section/표 13개와 선택 쪽
  각각 표 5개를 XML 파싱으로 확인했다. `native-comparison.json`은 최신 Native와 3건 모두
  byte-identical이며 raw/후처리 WASM·JS의 크기·SHA도 함께 기록한다.
- 관측 선형 메모리 용량은 실물 처리 뒤 22,347,776 bytes였다. live heap·RSS 상한이나
  실제 브라우저 Worker/UI의 격리·취소·재생성 증거는 아니다.

`resource-audit/traversal-source-snapshot.json`에 이번 source/설정/고지 426개 파일 hash를
고정했다. 빌드 중에도 변경 0개를 확인했다. 원본 대비 수정 파일은 기존과 같은 23개이며,
기존 upstream integrity 기록을 덮어쓰지 않았다.

#### 파생 harness와 공유 산출물의 정합

테스트 보완 후 첫 정책 검사에서 harness 2개의 drift가 나왔다.
`traversal-final-prepare.log`로 파생물을 재생성한 뒤 고정 기준
`a5e88ad682c805e1d845502ddc9521b94ea8eccd` 대비 integration 정책이 통과했다
(`traversal-final-integration-policy.log`: 1,467 sources, 6,281 static attrs, 48 targets).
source-side unit 정책도 4,103 tests/296 modules로 통과했다(`traversal-unit-policy.log`).
이 기준은 계획에 고정한 SHA이며 새로 fetch한 최신 devel의 최종 PR gate라고 표시하지 않는다.

재생성 뒤 추가 focused build는 같은 버전 `serde_core`/`serde_json`이 서로 다른 타입이라는
컴파일 오류로 중단됐다(`traversal-final-resource-contracts.log`). 현재 Cargo graph에는 각 버전이
하나뿐이고, 별도 Native probe와 root workspace가 공유하는 비해시 `librhwp.rlib`가 확인됐다.
소스 내용/의존성/검사 기준을 변경하거나 target을 지우지 않고 `src/lib.rs`의 mtime만 갱신해
현재 workspace graph로 재빌드를 유도했다. `git diff`는 해당 source의 내용 변경 0을 확인했다.
같은 정상 명령을 재실행하자 39개 자원 계약이 모두 통과했다
(`traversal-final-resource-contracts-refreshed.log`). 이 결과는 source/lockfile/검사 기준 변경 없이
workspace 재빌드만으로 얻었으며, 앞 컴파일 실패와 수정 전 결함 재현을 혼동하지 않는다.
같은 최종 graph에서 frontend 4개도 다시 통과했다(`traversal-final-frontend-contracts.log`).
여백 1개도 재실행해 통과했다(`traversal-final-margin-contract.log`).
root fmt check, vendor 변경 Rust 3개 파일의 원래 설정 fmt check, diff check와 변경 문서 3개의
내부 링크 검사가 통과했다. `traversal-final-source-integrity.json`은 결과 기록용
`RHWP_PATCHES.md` 이외의 425개 fingerprint가 그대로이며 Rust/Cargo/테스트 변경이 없음을 확인한다.
이 검증한 탐색 보완과 진행 기록을 중간 checkpoint로 보존하며 2단계 완료로 승격하지 않는다.

#### 남은 조건

현재 재검증으로 모든 참조 helper·codec scratch·font/cache·이미지 버퍼 정책·Worker 실행의
감사를 완료한 것은 아니다. `parser_allocation_bounds_proven=false`와 비신뢰 입력 배포 보류,
2단계 진행 중 상태를 유지한다. 제품 CLI/WASM 공개 연결·전체 검증·PR 생성은 여전히 미실행이다.
이번 탐색 source의 로컬 package 소비자 재검증은 아직 하지 않았으며 3.5의 이전 package 결과를
그대로 적용하지 않는다. 게시된 rhwp 소비자의 의존성 배포 모델도 아직 종료 조건으로 남아 있다.

### 3.9 캐시 반환·이미지 원본 복사 보완 (2026-10-15)

메인테이너의 “남은 작업 진행” 승인으로 자원 감사를 이어갔다. 시작 head는
`f3bb52c0631cbce13b4a007422bef6b4c40102da`, 작업 트리는 clean이었다.
현재 `origin/devel` 대비 ahead 14/behind 19를 확인했으며 이번 보완 중 fetch·merge·rebase는
수행하지 않았다. 기존 계획의 고정 base와 최종 PR 시 최신 devel 기준을 구별한다.

#### 원인과 보완

- `extract_spans`의 cache hit와 `cached_page_chars`는 이미 실패한 요청에도 데이터를
  반환할 수 있었다. 진입/생성 후 fatal 검사를 추가했다.
- span cache 삽입과 반환 복사, 공개 `extract_chars`의 복사는 요청의 decoded 예산에 잡히지
  않았다. flat struct 배열과 소유하는 text/font-name/glyph-width/origin 배열의 요청 bytes를
  checked arithmetic으로 합산해 **복사 전에** 개별 32 MiB·누적 256 MiB 제한을 적용했다.
  이는 요청한 payload 작업량이며 allocator overhead·live heap·RSS 측정이나 모든
  postprocessing/문자 생성 버퍼의 선행 할당 입증은 아니다.
- path·image-handle·기존 materialized image의 Form stream 소비는 cache 반환과 삽입 시
  payload Vec을 복제했다. 공통 `cached_form_stream`에서 decoded Vec의 소유권을 Arc로
  옮겨 세 소비자가 공유하도록 바꿨다. 기존 50 MiB cache 정책과 legacy 동작은 유지했다.
  materialized **이미지 자체**의 cache clone을 감사 완료했다는 뜻은 아니다.
- `PdfImageHandle::raw_compressed_bytes`의 inline/XObject 복사를 요청 budget에 연결했다.
  raw 반환과 decode 진입/성공 반환에도 fatal 검사를 적용했다. 공개 함수 서명이나
  `PdfImportLimits` 필드, 기본 feature·의존성 버전·렌더 배치는 바꾸지 않았다.

#### 작은 계약과 정상 대조군

`tests/cases/issue_7911_pdf_resource_limits.rs`에 6개 계약을 추가했다.
첫 4개는 보완 전 source에서 기존 39개 PASS/신규 4개 FAIL로 결함을 재현했다
(`logs/cache-before-contracts.log`). fatal cache 반환은 성공으로 나왔고 span/char/raw 복사의
decoded 증가량은 모두 0이었다. 보완 후 이 4개를 포함한 43개가 통과했다
(`cache-after-contracts.log`).

추가 계약은 개별 복사의 한도 직전/동일/초과, Form payload를 여러 추출 모드에서 세 번씩
재사용하는 대조군이다. 초기 대조군의 비공개 field 접근은 공개 getter로 정정했다.
동일 좌표의 선을 두 번 그린 입력은 정상 dedup에 걸렸고, 2x1 이미지는 기존 materialized
API의 8x8 미만 제외 정책에 걸렸다. 입력을 서로 떨어진 두 Form/8x8 이미지로 정정해
legacy·limited 양쪽에서 선 2개/handle 2개/materialized image 2개와 원본 pixel payload가
보존됨을 확인했다. 기대값이나 기존 제품 정책을 낮추어 통과시키지 않았다.
앞 실패 로그도 `cache-final-contracts*.log`에 남겨 source 결함과 테스트 API/입력 오류를 구별한다.

최종 `cache-final-workspace-resource-contracts.log`는 **45/45** PASS다.
`cache-frontend-contracts.log`의 기존 frontend **4/4**, `cache-margin-contract.log`의
명시적 여백 왕복 **1/1**을 합쳐 **50개 계약**이 통과했다. 새 합성 자원 계약이며
한컴 조판 golden/시각 완료 증거가 아니다. Native probe 후 비해시 root rlib를 현재 workspace
graph로 다시 만들도록 `src/lib.rs`의 내용 변경 없이 mtime만 갱신한 뒤 최종 검사했다.
별도 target·cache 삭제·source/lockfile 우회는 하지 않았다.

| 확인 | 결과/증적 (`output/pr-review/issue7911/` 아래) |
| --- | --- |
| root feature-on Native/WASM lib Clippy | `logs/cache-native-clippy.log`, `cache-wasm-clippy.log`: PASS |
| backend no-default-features Native/WASM lib Clippy | `logs/cache-backend-native-clippy.log`, `cache-backend-wasm-clippy.log`: PASS |
| root fmt | `logs/cache-fmt.log`: PASS |
| integration 정책 | `logs/cache-integration-policy.log`: 1,467 sources/6,287 static attrs/48 targets PASS; 고정 base `a5e88ad6` |
| source unit 정책 | `logs/cache-unit-policy.log`: 4,103 tests/296 modules PASS |
| Native 실물 전체/선택 | `logs/cache-native-real.log`, `cache-real-pdf/summary.json`: 3건 모두 `ok=true` |
| Native 보완 전후 | `cache-real-pdf/before-comparison.json`: HWPX 3건 byte-identical |

실물은 기존 `pdf/issue5875/nested_table_text_caption-2020.pdf`다.
전체 12개 section/표 13개, 선택 7·8쪽 각각 표 5개/중첩 4개를 유지했다.
Native debug 전체 약 4.602초·선택 약 0.518/0.513초였다. 7개 쪽의 semantic 보류와
`parser_allocation_bounds_proven=false`는 유지한다. 해시 동일은 이 표본의 저장 결과가
유지된 증거이며 한컴 Print·확대 판독이나 모든 문서의 무회귀 증거는 아니다.

fresh WASM은 `logs/cache-wasm-release.log`에서 release 빌드 약 413.3초로 완료했다.
같은 공식 `wasm-bindgen 0.2.127`로 `cache-wasm-runtime/pkg/`의 새 web binding을 만들고
Node에서 실제 실행했다(`cache-wasm-bindgen.log`, `cache-wasm-runtime.log`).
이전 진단 bundle과 root `pkg/`·Studio public bundle은 덮어쓰지 않았다.

- 기존 오류 5건에 비활성 JBIG2/JPX 이미지 2건을 추가했다. 전자는 기존 안정 code,
  후자는 현재 frontend의 `PDF_PARSE_FAILED`로 거부했다. unsupported stub 경로의 실제
  오류 반환을 확인했으며 두 codec의 pixel decode가 지원된다는 뜻은 아니다.
- 같은 instance에서 오류 7건 뒤 정상 문서, 페이지 깊이 32/Form 깊이 16,
  비ASCII CMap과 실물 전체/선택을 처리했다. 실물 전체 약 0.372초·선택 약 0.031/0.029초,
  12개 section/표 13개 및 선택 쪽 각각 표 5개를 확인했다.
- `cache-wasm-runtime/native-comparison.json`은 최신 Native와 실물 HWPX 3건의
  byte-identical 및 raw WASM 22,685,898 bytes/후처리 20,817,886 bytes/JS 457,493 bytes의
  SHA-256을 보존한다. 선형 메모리 관측 용량은 실물 뒤 22,347,776 bytes였다.
- Rust 자원 계약 45개는 Native 테스트 실행이다. 위 Node 검사는 frontend의 실제 WASM
  진단 실행이며, 45개 계약 자체의 WASM 실행·제품 공개 import API·Studio UI·browser
  Worker 격리/취소·live heap/RSS 입증·시각 검증으로 승격하지 않는다.

#### 실제 의존 graph와 남은 경로

`cache-native-feature-tree.log`, `cache-wasm-feature-tree.log`의 backend features는
양쪽 모두 빈 집합이다. 따라서 현재 graph의 JBIG2/JPEG 2000은 각각 `rendering`/
`jpeg2000` cfg의 decoder가 아니라 명시적 unsupported stub이다. qcms/lcms2의 ICC
engine도 비활성이다. 이 사실을 활성 codec 내부 할당 미입증과 혼동하지 않는다.
feature를 추가하는 소비자의 조합에는 이 결론을 그대로 적용할 수 없다. ICC profile bytes,
palette/색 변환 fallback 자체의 처리는 별도다.

`decoders/bounded.rs`의 Brotli는 출력 read/append를 제한하지만 `brotli-decompressor 6.0.1`
내부 allocator를 제한하지 않는다. 해당 version의 custom allocator API와 실패 반환 경로는
읽었으나 안전한 교체·모든 할당 실패 처리 계약을 구현/검증하지 않았다.
CCITT는 `to_png_bytes_with_budget`에서 legacy `to_dynamic_image`를 거쳐 in-house/fax
decoder를 사용한다. 이미지 width/height와 별개인 `DecodeParms` Columns/Rows, recovery/
padding·fallback 작업량까지 선행/누적 제한을 증명하지 못했다. 단순한 fallible reserve는
요청 예산 제한과 다르다. embedded font/대체 encoding·추출 char 생성·나머지 cache와 참조
helper, 이미지당 64 MiB/문서 128 MiB 작업 정책·Worker 격리도 종료 조건으로 남긴다.

`resource-audit/cache-source-snapshot.json`에 root·내부 crate·backend source/고지·설정·
관련 계약 1,816개 hash를 고정했다. `cache-final-source-integrity.json`에서 결과 기록용
`RHWP_PATCHES.md` 이외 1,815개가 그대로이며 예상 밖 변경 0개임을 확인했다.
`cache-upstream-integrity.json`은 원본 archive의 401개 파일 누락 0개와 원본 대비 변경
23개를 구별하며 라이선스·고지 6개의 byte-identical 보존을 확인한다. 새 registry 다운로드나
현재 source의 package 소비자 검증으로 표시하지 않는다.
vendor 수정 Rust의 원래 설정 fmt check, diff check와 변경 문서 2개의 링크 검사가 통과했다.
문서 검사 첫 명령의 존재하지 않는 `--files` 인자는 실제 positional 경로로 정정해 재실행했다
(`logs/cache-doc-links-corrected.log`).
이번 결과만으로 2단계 완료, 제품 CLI/WASM 공개 연결, 전체 lint/회귀/시각 검증,
최신 source의 packaged consumer·외부 의존 배포 모델이나 Open PR 준비 완료를 선언하지 않는다.

### 3.10 Brotli private scratch 보완과 디스크 중단 이력 (2026-10-15)

메인테이너가 PR Open까지 후속 진행을 지시했고, 별도 확인에
**별도 crate 배포 준비 포함**을 승인했다. 실제 `cargo publish`/외부 게시 승인은 아니며
backend `publish=false`와 안전성/시각/단계 승인 게이트를 유지한다.

시작 head는 `36dbafc0b78b07e444c8a1e71514bcb7ce5ad730`이며 시작 worktree는 clean이었다.
기존 자원 계약 45개에 작은 Brotli 반례 2개를 추가해 보완 전 **45 PASS/2 FAIL**을 확인했다
(`output/pr-review/issue7911/logs/brotli-before-contracts.log`).
Node `zlib.brotliCompressSync`로 확인한 `[59]`는 decoded 출력이 비어 있는데도 내부 Huffman
할당이 필요하다. 128-byte 출력의 동일 Brotli fixture도 출력 예산만으로 성공했다.
이전 카운터가 private decoder 할당을 계상하지 않는 반례이며 큰 입력/OOM 시험은 아니다.

첫 보완 시점의 미커밋 변경은 다음과 같다. 후속 재실행은 3.11에 구분한다.

- 직접 `brotli-decompressor =6.0.1` 제약을 추가하고 감사한 allocator API를 사용한다.
  기존 Brotli 9 legacy decoder는 보존한다. 별도 package 소비자가 다른 decoder version을
  선택할 수 있었던 문제를 구별하고 root/standalone lock에 같은 제약을 반영했다.
- private 할당은 요소 수×크기를 checked 계산해 `codec_allocation_bytes`(기본 32 MiB)와
  기존 누적 decoded 작업 예산을 **할당 전에** 소비한다. 메모리를 해제해도 작업량은 반환하지 않는다.
  fallible reserve 실패는 요청 fatal로 기록한다. 초기 Huffman table 실패도 decoder 실행 전에 확인한다.
- 고정 4 KiB 출력 chunk와 기존 bounded writer를 사용하고, 표준 window의 strict 상태로
  시작한다. decoder 내부 실패 뒤 부분 출력 성공을 반환하지 않는다.
- 기존 출력 한도 127/128/129 계약은 유지하면서 새로 계상하는 private scratch를 누적량에
  포함하도록 검사했다. scratch 개별/누적 한도 경계와 부분 할당 실패 반복 계약 2개도 추가했다.
  자원 계약은 현재 49개이나 **보완 후 실제 PASS를 확인한 상태가 아니다**.

`brotli-after-contracts.log`에서 새 backend 컴파일 이후 root library metadata 기록 중
`No space left on device (os error 28)`로 중단됐다. 종료 코드 101은 컴파일 산출물 기록 실패이며
새 계약 실행 실패/성공으로 해석하지 않는다. 확인한 여유는 약 **125 MiB**였다.
작업 소유 Cargo/rustc 프로세스가 남지 않았음을 확인했고, 캐시/기존 산출물을 삭제하지 않았다.
추가 공간 확보 또는 이번 작업의 캐시 식별·정리안 확인을 메인테이너에게 요청했다.

root update가 함께 변경한 tempfile/getrandom 항목과 standalone update의 무관한 optional
dependency churn은 이 batch에서 제거했다. 남긴 것은 직접 decoder 의존 추가와 standalone
6.0.0→6.0.1/checksum 변경이다. 이 좁힌 lockfile의 `--locked` 재검증도 공간 확보 뒤 수행한다.
직전 3.9의 50개 계약/fresh WASM 결과는 `36dbafc0b` source의 증거이며 새 Brotli 변경의
통과 증거로 재사용하지 않는다.

공간 확보 후 순서는 새 allocator 계약/정상 Brotli 대조군 → CCITT/이미지/폰트·참조 잔여 감사 →
동일 source Native/WASM 재검증·package 소비자 준비다. 이후 2단계 승인, 제품 CLI/WASM 직접
연결, 전쪽 시각·기존 형식 비교와 승인된 전체 검증을 마쳐야 Open PR을 만든다.
필수 게이트를 생략하거나 미완료 상태를 Draft로 바꾸어 우회하지 않는다.

### 3.11 디스크 재개와 Brotli 계약 재검증 (2026-10-15)

첫 공간 확보 후 약 1.9 GiB로 재시도했으나 `brotli-resume-contracts.log`의
`rhwp-agent`/CLI 링크와 suite query cache 기록에서 다시 ENOSPC가 발생했다.
이는 테스트 실행 결과가 아니다. 두 번째 공간 확보 알림 직후 여유는 약 2.6 GiB였다.
메인테이너가 이어서 **정리안의 증분 캐시 9개 삭제**를 명시 승인했다.

`resource-audit/cache-cleanup-proposal.json`의 정확한 9개 경로에 대해
실행 중 Cargo/rustc 부재, incremental 직계 하위 realpath, 컴파일 metadata의 #7911
전용 소스 경로를 다시 확인했다. 승인된 오래된 증분 캐시만 삭제했고 source·실행파일·
WASM·검증 증적·최신 캐시·공유 target 자체는 보존했다. 삭제 전 allocated 합계는
10.1318 GiB이며 직후 가용 공간은 약 9.8 GiB였다. APFS의 실제 가용 공간과 `du`
합계를 동일시하지 않는다. 경로별 재확인/실행 내역은
`resource-audit/cache-cleanup-execution.json`에 보존했다.

재개 후 Cargo는 공용 target에서 `-j 1`, `--locked --offline`로 순차 실행했고
디스크 512 MiB 미만이면 자신의 Cargo에 SIGINT를 보내는 감시를 적용했다.
첫 실행은 **48 PASS/1 FAIL**이었다 (`brotli-resume2-contracts.log`).
실패는 빈 `[59]`의 scratch가 초기 Huffman table 하나뿐이라는 테스트 가정 오류였다.
감사한 6.0.1의 `state.rs:530`은 1080개, `decode.rs:3312/3318`은 각각 그 세 배의
Huffman entry를 할당한다. repr(C) entry는 4 bytes라 최대 개별 요청은 12,960 B,
누적은 30,240 B다. 구현 결과에서 기대값을 역산하지 않고 이 독립적인 의존성 소스에
근거해 개별/누적 cap-1/cap/cap+1 검사를 정정했다. 제품 한도를 올리지 않았다.

추가 정상 대조군은 Node zlib로 독립 압축한 12,288-byte 반복 패턴이다.
3개 이상 출력 chunk, 기존 legacy decoder와 bytes 일치, 출력 cap-1/cap/cap+1,
모든 잘린 prefix의 오류, 여러 개별 할당 cap에서의 fatal/정상 결과와 새 요청 복구를 검사했다.

| 현재 source의 집중 검증 | 결과 / 로그 |
| --- | --- |
| 자원 계약 50개 | PASS, `brotli-final-contracts.log` |
| PDF frontend 계약 4개 | PASS, `brotli-frontend-contracts.log` |
| 부재/명시적 0 여백 왕복 계약 1개 | PASS, `brotli-margin-contract.log` |
| root Native/WASM library Clippy | PASS, `brotli-{native,wasm}-lib-clippy.log` |
| backend 독립 Native/WASM library Clippy | PASS, `brotli-backend-{native,wasm}-lib-clippy.log`; 좁힌 standalone lock 포함 |
| Native/WASM feature tree | `brotli-decompressor 6.0.1`의 std 구성, unsafe/FFI 비활성 |
| suite 수집/source unit tier 정책 | PASS, `brotli-{suite,unit-tier}-policy.log` |

제품/backend Clippy는 **library 범위**다. workspace/all-targets/full regression gate를
통과했다고 쓰지 않는다. 이 단계에서 native 55개 계약을 WASM에서도 실행했다고 표시하지 않는다.
현재 source 1,811개 파일 hash는 `resource-audit/brotli-source-snapshot.json`에 보존했다.
fresh WASM release는 6분 48.8초, 공식 wasm-bindgen 0.2.127 web 후처리는 1.35초였다.
`brotli-wasm-runtime/smoke.mjs`는 13,643 B content를 독립 압축한 PDF에 대해
잘린 입력 4개를 `PDF_PARSE_FAILED`로 거부한 뒤 정상 변환을 수행했다.
압축/비압축 content 및 빈 Brotli/빈 비압축 content의 HWPX가 각각 byte-identical이었다.
기존 오류 7개와 page/Form 깊이 정상 경계·비ASCII CMap·실물 12쪽/7·8쪽 변환도 통과했다.
이는 fresh web binding의 **Node 진단**이지 제품 공개 API·브라우저 Worker·시각 검증이 아니다.

같은 source의 Native 실물 전체/7쪽/8쪽 import는 각각 약 3.81/0.61/0.51초였고,
HWPX는 1,607,950/15,175/15,777 B였다. 세 결과 모두 직전 정상 source 및 이번 fresh
WASM과 byte-identical이다. 7·8쪽 각각 표 5개/nested 4개를 보존한다.
해시 비교는 변환 결과 보존의 증거일 뿐 원본 PDF/한컴과의 조판 일치 증거가 아니다.
이전 POC/시각 결과를 현재 source 검증으로 승격하지 않는다.

| fresh WASM artifact | bytes / SHA-256 |
| --- | --- |
| raw probe WASM | 22,766,345 / `b5ceb36624ca4a0e0e095bb13dee39aeb5941f4cf7a7c52f8bdc4098d6ee24b1` |
| web 후처리 WASM | 20,894,104 / `ee97af5f79853141e3c7285485feaa347a9e114225e405be8b9723dca3c66cd6` |
| JS glue | 457,493 / `b7feac4608455aebf697d64d365e0700b372877a3df26843f66d4859c0fa1443` |

실물 실행은 `brotli-real-pdf/summary.json`, 비교는 `before-comparison.json`,
fresh WASM 실행/대조는 `brotli-wasm-runtime/{summary,native-comparison}.json`에 보존했다.
관측한 WASM linear memory capacity는 live heap/RSS 상한으로 해석하지 않는다.

현재 backend를 로컬 `cargo package --allow-dirty --no-verify --locked --offline`로 만들었다.
407 files / 압축 약 6.5 MiB이며 별도 `brotli-package-validation/`에 archive와 unpacked source를
보존했다. archive SHA-256은 `d00cdc043ea6cf901c584d58242a2108a0899ac07e8bc86f32815e048029fb58`다.
source와 고지 등 387개 파일이 일치했고 원본 라이선스/고지 6개도 일치했다.
이 아카이브만 소비하는 독립 Native 실행에서 private 한도·sticky 실패·새 요청의 빈/정상
Brotli 변환을 확인했다. 이는 `--no-verify` 또는 `--list`만 실행한 결과와 구별한다.
독립 소비자의 `--locked --offline` WASM 컴파일도 9.9초에 통과했다
(`brotli-package-consumer-{native,wasm}.log`). 이는 registry의 root rhwp 소비자 해석·실제
publish와 별도 crate 배포 준비 완료를 의미하지 않는다. 외부 게시·배포는 하지 않았다.

최종 source/lock 1,811개는 실행 전 snapshot과 동일했다
(`resource-audit/brotli-final-source-integrity.json`). root workspace fmt, 변경 backend Rust
두 파일의 원래 설정 fmt, diff check가 통과했다. 이번 기록은 Brotli 보완의 중간 checkpoint다.
CCITT/이미지·문자 생성/폰트·postprocess의 나머지 선행 할당 감사가 끝나지 않아
`parser_allocation_bounds_proven=false`, 2단계 진행 중 및 Open PR 보류를 유지한다.

### 3.12 CCITT DeviceGray 제한 경로 보완 (2026-10-15)

메인테이너의 다음 절차 진행 지시에 따라 2단계 잔여 이미지 할당 경로를 조사했다.
시작 head는 `769ff849d158f2457748856c0f3d51a236e1c7ee`, 시작 worktree는 clean이었다.
새 dependency·버전·feature는 추가하지 않았다. 공용 target에서 `-j 1`, `--locked --offline`,
자기 Cargo의 디스크 감시를 유지하고 임의 캐시 정리를 하지 않았다.

#### 원인과 수정 전 반례

`lowering::prepare_images` → `PdfImage::to_png_bytes_with_budget` → `encode_png`의 Raw
경로가 `to_dynamic_image`를 호출해 예산 없는 CCITT decoder/`fax` fallback/흰 대체로 이어졌다.
이때 별도 `/Rows`가 packed output 예약·패딩 크기를 결정해 작은 `/Height`의 픽셀 검사로는
할당량을 제한하지 못했다. 또한 압축 byte 수가 회색 픽셀 수와 같으면 PNG 직접 쓰기 shortcut이
CCITT decode 자체를 생략했다. 1x1의 `[0x80]`은 실제 흰 V(0) 행인데 회색 128로 출력됐다.

32,768-row 선언과 1-byte codestream 등 작은 입력만으로 기존 50개 PASS/새 4개 FAIL을
확인했다 (`ccitt-before-contracts.log`). OOM/거대한 실제 할당 공격을 실행하지 않았다.
네 반례는 행 수 불일치, private 전이 버퍼의 한도 우회, 압축/픽셀 길이 동일 shortcut,
디코딩 불가 데이터를 흰 이미지 성공으로 반환하는 경우다.

#### 생산·소비 경계와 지원 범위

- 제한 문서의 `extract_image_from_xobject_at`는 CCITT filter의 위치에 맞는 DecodeParms를
  선택하고 outer/entry 참조를 해석한다. 정수→u32 변환을 검사하여 음수·절단된 큰 정수로
  바뀌지 않게 한다. 직접 dictionary는 다시 clone하지 않는다.
- `Rows` 부재/0은 이미 검증한 이미지 높이로 유한화한다. 명시한 Columns/양수 Rows가
  이미지 geometry와 다르면 제한 경로가 거부한다. 이는 모든 유효 PDF를 수용하는 정책이
  아니다. EndOfBlock에 따라 Rows를 무시하는 입력도 존재하므로 그 조합의 완전 지원을
  주장하지 않는다. 실제 크기를 임의 clamp하거나 일부 행만 복원했다고 표시하지 않는다.
- PNG 쓰기에서 CCITT DeviceGray는 압축 크기와 관계없이 bounded decoder를 사용한다.
  `decode_ccitt_with_budget`만 공개 재노출하고 CCITT 내부 모듈 전체는 공개하지 않는다.
- 기존 Group 3/4 행 해석기를 재사용한다. 제한된 호출의 전이 버퍼 초기/성장·packed row
  scratch·grayscale은 예약 전에 개별 codec/문서 누적 예산을 소비한다. packed 출력은
  알려진 전체 크기를 검사하고 기존 bounded writer를 사용한다. 압축 입력은 출력 한도가
  아니라 입력 예산을 적용한다. 기본 입력 16 MiB/개별 codec 32 MiB/출력 32 MiB/
  누적 decoded 작업량 256 MiB를 유지한다. 이 수치는 총 heap/RSS 상한이 아니다.
- 내부 u16 폭을 절단하지 않으며 제한 경로의 65,536 이상 폭은 명시적 미지원 오류다.
  모든 요구 행을 읽을 수 없으면 부분/흰 대체 성공 대신 오류를 반환하고 예산 없는 fax
  fallback을 호출하지 않는다. 비제한 API는 기존 fallback/부분 복구를 보존한다.
- decoder/PNG의 자원 실패는 같은 budget에 sticky하게 남는다. frontend는 lowering 뒤
  budget을 다시 확인해 `PDF_RESOURCE_LIMIT`로 전파한다. 일반 decode 실패와 resource
  실패를 구별하고 실패한 builder를 다음 문서에 설치하지 않는다.

#### 현재 검증 결과

초기 수정 후 반례 포함 54개가 PASS였다. 확장 검사 첫 실행은 내부 모듈 접근과
BinDataBytes 직접 비교의 컴파일 오류였고, 새 제한 API 재노출/기존 `load()` 사용으로 고쳤다.
그다음 59 PASS/1 FAIL은 입력 크기에 출력 cap을 잘못 적용한 구현 오류였으며,
입력 예산으로 정정했다. 최종 자원 계약 **60개 PASS**다 (`ccitt-final-contracts.log`).

- Group 4/Group 3 1-D/혼합, byte alignment 두 값, polarity 두 값의 12개 조합을 검사했다.
  T.4/T.6의 독립 bit code가 나타내는 `[255,255,231]` packed 행과 grayscale sample을
  비교하며 정상 이미지의 PNG는 기존 비제한 API와 byte-identical이다.
- 압축 입력·packed 출력·개별 scratch·누적 작업량의 cap-1/cap/cap+1, 같은 요청 반복,
  실패 후 sticky 거부/새 요청 복구를 검사한다. 압축 입력 7 B/출력 3 B를 구별한다.
- zero-run 반복의 작은 스트림으로 전이 Vec의 초기 크기 이후 성장도 같은 quota가
  제어하는지 확인했다. 잘린 Group 3/4 prefix를 부분 이미지 성공으로 바꾸지 않는다.
- 제품 frontend에서 Rows/Columns 음수·u32 범위 밖·geometry 불일치를 거부하고 정상
  간접 DecodeParms 및 ASCIIHex→CCITT chain의 index와 `/Decode [1 0]`을 보존한다.
- root/backend의 Native/WASM **library** Clippy 네 구성이 통과했다. 전체 workspace/
  all-targets lint 통과로 표시하지 않는다. suite/source-side unit tier 정책도 통과했다.

실행 전 source/lock 1,811개 hash는 `resource-audit/ccitt-source-snapshot.json`에 보존했다.
frontend/여백 **5개 PASS**도 확인했다 (`ccitt-frontend-margin-contracts.log`).
이번 codec pixel 계약은 합성 입력의 decode 계약이며 실물 페이지의 조판 golden이 아니다.

fresh WASM release는 421.1초, 공식 wasm-bindgen 0.2.127 web 후처리는 1.36초였다.
`ccitt-wasm-runtime/smoke.mjs`를 Node에서 실행해 다음을 확인했다.

- CCITT 오류 6개를 `PDF_PARSE_FAILED` 또는 `PDF_LOWERING_FAILED`로 거부한 뒤 정상 입력을 처리했다.
- Group 3/4·alignment·polarity의 12개 조합은 기존 설치된 독립 `pngjs` decoder로 PNG의
  RGBA 24개 픽셀을 T.4/T.6 독립 기대값과 비교했다. ASCIIHex→CCITT chain의 HWPX도 동일했다.
- 압축 길이=픽셀 수인 1x1의 실제 PNG가 회색 128이 아니라 흰 sample 255임을 확인했다.
- 기존 Brotli 정상/손상·자원/CMap/page/Form 경계·비활성 codec 오류·실물 12쪽/7·8쪽도 실행했다.

이는 **fresh web binding의 Node 진단**이다. Native 계약 65개를 WASM에서 그대로 실행했다는
뜻이 아니며 제품 공개 API·브라우저 Worker/Studio·시각 검증이나 총 메모리 상한의 증거도 아니다.

| fresh WASM artifact | bytes / SHA-256 |
| --- | --- |
| raw probe WASM | 22,778,589 / `4e8afbc963f50ab6532685b83cd33a266b1f5d0db243872194d2cd74ef13111b` |
| web 후처리 WASM | 20,905,746 / `a1189ee5405899bf7252722afa4297d58a230d80ec56f1c4c3785e0e7ada432d` |
| JS glue | 457,493 / `b7feac4608455aebf697d64d365e0700b372877a3df26843f66d4859c0fa1443` |

같은 source의 Native 전체/7쪽/8쪽 import는 약 3.90/0.48/0.48초였다.
HWPX는 1,607,950/15,175/15,777 B이며 세 결과가 직전 정상 source 및 새 WASM과
byte-identical이다. 7·8쪽 각각 표 5개/nested 4개를 보존했다. 결과·비교는
`ccitt-real-pdf/{summary,before-comparison}.json`,
`ccitt-wasm-runtime/{summary,native-comparison}.json`에 보존했다.
바이트 보존을 한컴/원본과의 시각 일치나 전체 회귀 통과로 승격하지 않는다.

현재 source를 로컬 package로 만들고 `ccitt-package-validation/`에 archive/unpacked source를
보존했다. 407 files, 압축 약 6.5 MiB이며 archive SHA-256은
`e0b819ce5e468a09d13d57e031122dc81b78ee171b72d861570ed0cdc1c329f7`이다.
source·고지 등 387개와 원본 license/notice 6개의 일치를 확인했다.
아카이브만 소비하는 독립 Native 실행에서 Brotli/CCITT의 한도·sticky 실패·새 예산의
정상 복구가 통과했다. 독립 WASM 소비자의 `--locked --offline` 컴파일도 10.4초에 통과했다
(`ccitt-package-consumer-{native,wasm}.log`). Native 실행과 WASM 컴파일을 구별한다.
이는 registry의 root package 소비자 해석/별도 crate 배포 모델 완료와 구별한다.
외부 publish·push·PR 생성은 하지 않았다.

최종 source/lock 1,811개가 실행 전 snapshot과 같았다
(`resource-audit/ccitt-final-source-integrity.json`). 이번 batch에서 캐시는 삭제하지 않았다.
root workspace fmt, 변경 backend 세 파일의 원래 설정 fmt와 diff check도 통과했다.

#### 미입증 범위

이 보완은 **CCITT DeviceGray 경로**다. 저장소에서 발견한 실물 CCITT 입력
`pdf/hwpx/[2027] 온새미로 1 본교재.pdf`(46쪽)의 3쪽 CCITT는 Indexed 색 공간이므로
이 경로의 실물 시각 통과 증거로 쓰지 않는다. Indexed CCITT/다른 색 공간·mask/palette와
나머지 image 경로 감사는 유지한다. 글자 생성/폰트·postprocess와 Worker 종료도 미완료다.
`parser_allocation_bounds_proven=false`, 2단계 진행 중과 Open PR 보류를 유지한다.

### 3.13 명시 승인에 따른 WIP Draft PR 생성 (2026-10-15)

메인테이너가 "draft 로 pr 을 open 합니다"라고 지시하여, 이전의 남은 작업 완료 후
Open PR 방침을 WIP 공유에 한해 변경했다. `gh` CLI로 같은 head branch의 기존 PR이
없음과 #7911의 OPEN 상태를 확인했다. 원본 저장소 remote는 `origin`이며 다른 remote를
추가하거나 기존 이름을 변경하지 않았다.

코드 head `43863fb753018d6b044cc73e42fa6549af8e2bc9`를
`origin/task_m100_7911`에 일반 push한 뒤 `gh pr create --draft --base devel`로
[Draft PR #7913](https://github.com/edwardkim/rhwp/pull/7913)을 생성했다.
생성 직후 API 재조회에서 `state=OPEN`, `isDraft=true`, `baseRefName=devel`,
`headRefName=task_m100_7911`과 동일 코드 SHA 및 한국어 본문 보존을 확인했다.
원격 devel의 조회 SHA는 `463c48cc228170b576c832591b76826116e86a49`다.

PR 본문은 현재 65개 집중 계약 및 library lint/fresh WASM probe의 검증 범위와
제품 CLI/WASM 연결·할당 감사·전체 lint/회귀·전쪽 시각 검증 등의 미완료를 구분한다.
전체 branch diff에는 보존한 upstream 소스의 trailing whitespace 6곳이 남아 있음을
명시했다. 기존 upstream 파일을 일괄 포맷하거나 산출물을 추가 커밋하지 않았다.

이는 명시 승인된 WIP 공유이며 자원 안전성·PR 준비 완료·정식 review·Ready 전환·merge·
외부 crate 게시·#7911 종료의 승인이 아니다. 생성 직후 CI는 진행 중이며 성공으로
판정하지 않았다. 이 기록은 작업 문서일 뿐 CI 성공 뒤 작성하는 정식 PR review나
오늘할일 trailing 기록을 대신하지 않는다. 아래 미완료 게이트는 그대로 유지한다.

### 3.14 WSL 재개: Indexed·인라인 이미지 경계 보완 (2026-10-15)

메인테이너의 “순서에 따라 진행을 시작합니다”에 따라 #7913의 2단계 자원 감사를 재개했다.
작업트리는 `/home/edward/mygithub/rhwp-issue7911`, 시작 head는 `bc5ff5690`이며 이번 변경은
로컬 WIP다. Native target은 기본 작업공간의 `/home/edward/mygithub/rhwp/target/pr-review`를
재사용했다. 기본 작업공간의 기존 WIP·Studio·환경 파일은 보존했다.

**적용한 계약과 실제 경로**

- `page_image_handles → PdfImageHandle::decode → extract_image_from_xobject_at →
  decode_stream_with_encryption → Indexed expansion → to_png_bytes_with_budget`를 추적했다.
  일반 CCITT filter가 압축 bytes를 그대로 반환하는데 Indexed 분기는 이를 즉시 palette index로
  해석했다. 제한된 경로는 기존 bounded Group 3/4 decoder를 먼저 호출하고
  `BlackIs1 → /Decode → palette → RGB` 결과를 소비하도록 보정했다.
- 직접 palette는 필요한 prefix를 요청 예산으로 복사하고, 참조 stream palette는 같은 문서의
  decoder를 사용한다. hival·palette 길이를 검사하며 RGB·Lab palette의 예약 전 예산을 적용한다.
  기대값은 [ISO 32000-1:2008](https://opensource.adobe.com/dc-acrobat-sdk-docs/pdfstandards/PDF32000_2008.pdf)
  7.4.6, 8.6.6.3, 8.9.5.2에서 정했다. index 반올림·제한은 색상표 규칙이며 배치 좌표 보정이 아니다.
- 인라인의 `image_handle_from_inline → StreamData::from(data) → decode_stream_data`는
  원래 요청 context를 잃었다. `StreamData::bounded(data, doc.import_budget())`로 바꿔
  합성 XObject도 filter·누적·fatal 예산을 보존한다. 비제한 문서는 기존 None context를 유지한다.
- Indexed `/Decode` 적용 결과를 `decode_folded_in()`에도 반영했다. 원래 sample 여부·저장 bpc와
  별도로 검증한다. 기존 분판 renderer는 Indexed를 직접 소비하지 않으므로 이 표시 누락을
  현재 frontend의 픽셀 오류와 혼동하지 않았다. 확장과 상태 표시가 같은 해석 결과를 사용한다.

**수정 전후 실행 증거** (`output/pr-review/issue7911/` 아래)

| 검증 | 결과 | 증거 |
| --- | --- | --- |
| Indexed 수정 전 | 기존 60 PASS, 신규 반례 3 FAIL | `logs/indexed-before-contracts.log` |
| 인라인 수정 전 | cap 4,096에서 4,097-byte 출력을 허용해 정식 검사 FAIL | `logs/inline-before-contract.log` |
| Decode 상태 수정 전 | 비기본 Decode 적용 뒤 false/true assertion FAIL | `logs/indexed-metadata-before.log` |
| 최종 Native 자원 계약 | **67 PASS / 0 FAIL** | `logs/images-final-metadata-contracts.log` |
| 최종 Native frontend·여백 | **4 + 1 PASS / 0 FAIL** | `logs/images-frontend-margin-final.log` |
| backend Native/WASM lib Clippy | 둘 다 `-D warnings` PASS | `logs/images-backend-native-clippy-final.log`, `logs/images-docker-wasm-final.log` |
| 최종 Docker web WASM 진단 | CCITT 조합·상태 24, codec cap 3, 오류 후 정상 2, inline 경계 4 PASS | `logs/images-docker-wasm-smoke-final.log`, `indexed-wasm-probe/results/smoke.json` |
| 실물 3쪽 이미지 9회 | Native/WASM 모두 Poppler와 RGB 동일; 기존 정상 5 PNG byte 동일 | `indexed-real-control/final-pixel-comparison.json`, WASM smoke JSON |
| 실물 전체 12쪽·선택 7/8쪽 | Native/WASM HWPX 3건 byte 동일 | `indexed-wasm-probe/results/final-comparison.json` |

실물 이미지 입력은 `pdf/hwpx/[2027] 온새미로 1 본교재.pdf`이며 Poppler 24.02.0의
`pdfimages -f 3 -l 3 -png`로 독립 기준을 만들었다. 수정 전 CCITT 4회가 실패했고 보정 후
9회 모두 추출된다. frontend 실물 입력은 `pdf/issue5875/nested_table_text_caption-2020.pdf`다.
합성 CCITT 24개 및 인라인 정상 입력은 rhwp frontend가 저장한 HWPX의 BinData PNG도
독립 디코더로 다시 읽어 명시적 기대 픽셀과 비교했다. 이는 codec/IR 계약 검증이며
전쪽 조판·한컴 Print·Studio 시각 통과 증거로 사용하지 않는다.

Native 실행은 suite prepare 뒤 공용 target의 `--locked --profile release-test --features pdf-import`로
수행했다. `node scripts/run-rust-test.mjs --cargo-test issue_7911_pdf_resource_limits -- ...`가
선택한 source를 실행했다. frontend/여백의 최종 suite는 012/008이다. 재분배 전 suite 006의
0건 실행이나 최초 중복 필터 CLI 오류는 통과·실패 재현 건수에 포함하지 않았다.

Docker는 기존 `rhwp-pr7704-review-20261011-wasm:latest`와 `-p rhwp`의 named volumes를
재사용했다(Rust 1.93.0, wasm-pack 0.15.0; host Native Rust 1.93.1).
진단 crate가 현재 root frontend/backend를 path로 소비하고 root의 svg2pdf patch를 명시한다.
루트에서 `scripts/wasm-pack-locked.sh output/pr-review/issue7911/indexed-wasm-probe --target web
--out-dir /app/output/pr-review/issue7911/indexed-wasm-probe/pkg`를 실행했다.
진단 빌드는 wasm-opt를 끄며 제품 root pkg·Studio public을 덮어쓰지 않는다.
독립 backend Clippy는 standalone lock을, 제품 probe는 제품 그래프를 사용하므로 구분한다.
root/backend manifest·lock은 이번 재개에서 변경하지 않았다.

최종 source 식별자는 `indexed-wasm-probe/source-final.json`, 실제 JS/WASM hash는
`indexed-wasm-probe/results/final-comparison.json`에 있다. 변경 이미지 source SHA-256은
`73c8ebd2336465260c25c52a73b23b55c3a7555be558bc6e9c6513e39e8ac0c5`다.
`cargo fmt --all -- --check`, `git diff --check`, suite policy의 고정 base
`463c48cc228170b576c832591b76826116e86a49` 비교도 통과했다.

**남은 범위:** 다른 색 공간의 샘플/LUT 변환·mask 의미·PNG private scratch,
객체/색 공간 map cache의 소유 복사, embedded font·glyph-name·후처리 cache의 선행 할당과
CPU 격리는 미완료다. 정적 관측은 `resource-audit/remaining-allocation-paths-ko.md`에 기록했다.
`parser_allocation_bounds_proven=false`를 유지하며 제품 CLI/WASM API·Worker·독립 package 소비·
전체 lint/회귀·전쪽 시각 게이트의 완료로 확대하지 않는다. 2단계 중간 보완이며 원격 #7913은
기존 head의 Draft 상태다. 이번 변경의 commit·push·Ready 전환·merge는 수행하지 않았다.

### 3.15 이미지 샘플·색 변환 예산과 마스크 오류 경계 (2026-10-15)

메인테이너의 “다음 절편 진행을 승인합니다”에 따라 3.14의 로컬 변경을 보존하고
이미지 변환의 후행 소비 경로를 보완했다. 시작 head `bc5ff5690` 위의 로컬 WIP이며,
이번 절편의 구현·검증은 원격 push·Ready·merge·외부 package 게시를 포함하지 않는다.

**구현 주장과 적용 경로**

| 경로 | 독립 계약과 보완 | 비적용·남은 범위 |
| --- | --- | --- |
| `extract_image_from_xobject_at → samples_to_decoded_bytes → ImageData::Raw` | ISO 32000-1 Table 89, 8.9.5.2의 bit depth·행 단위 byte alignment·Decode를 따른다. LUT·Decode range·8-bit 변환 출력의 예약 전에 `sample_buffer`가 요청 codec/누적 예산을 적용한다. 제한된 변환은 잘린 sample을 0으로 만들지 않고 오류를 반환한다. | legacy 추출의 짧은 stream 복구는 유지한다. Lab 등 기존 keep-raw 색 변환의 완전한 의미는 이번 통과 범위가 아니다. |
| 16-bit reduction → 후행 Decode → PNG | big-endian sample의 기존 반올림을 유지한다. 검증한 geometry로 읽을 prefix와 변환 크기를 정하고 할당 전에 같은 예산을 소비한다. 잘못된 정수 bit depth가 `as u8`로 유효한 값으로 바뀌지 않게 검사한다. | 별도의 16-bit 고정밀 IR 도입이 아니다. |
| `PdfImage::encode_png → raw_rgb_with_budget / into_rgb_with_budget` | CMYK/Gray→RGB의 입력 길이를 검사하고 RGB 버퍼를 예약 전에 계상한다. 이미 RGB인 소유 버퍼는 복제하지 않고 소비한다. 기존 색 정책은 보존 대조로 확인한다. | PNG encoder private scratch와 optional CMM 내부 메모리는 미입증이다. |
| `decode_jpeg_with_budget → RGB 변환 → PNG` | 실제 header geometry 확인 후 JPEG decoder 출력 payload와 후행 RGB payload를 개별·누적 한도로 계상한다. CMYK 반전은 기존 규칙을 유지한다. | JPEG decoder private scratch의 전체 선행 할당 한계를 입증한 것은 아니다. |
| 제한된 `PdfImageHandle::decode → frontend extract` | Table 89, 8.9.6 및 11.6.5.3의 `/Mask`·`/SMask`·stencil을 소비할 compositor가 없으므로 `PDF_IMAGE_UNSUPPORTED`로 명시적으로 거부한다. 뒤쪽 선택 페이지의 오류도 전체 builder 실패로 전파한다. | mask를 구현해 보존한 결과가 아니다. renderer가 mask를 직접 적용하는 legacy 경로는 유지하며 graphics-state soft mask/alpha/blend는 별도 미검증이다. |

`sample_buffer`는 요청 payload bytes와 누적 작업량을 계상한다. 기존 보수적인 선행
이미지 작업량 계상도 유지한다. 이 합계를 실제 peak RSS나 전체 allocator overhead로
표시하지 않는다. 직접·참조 Decode, 2/4-bit의 행 끝 padding, null Mask/SMask와 false
ImageMask, 마스크 없는 페이지만 선택하는 경우를 반례의 정상 대조군으로 사용한다.
참조 사양은 3.14의 독립 ISO PDF이며 로컬 사본은 `resource-audit/PDF32000_2008.pdf`다.

**실행 증거** (`output/pr-review/issue7911/` 아래)

- 첫 수정 전 기존 67 PASS·신규 4 FAIL: `logs/samples-masks-before.log`.
  sample 출력 1,025 bytes 대비 cap 1,024, 3-component LUT 768 bytes 대비 cap 767,
  잘린 sample의 조용한 0 채움, colour-key mask의 무시를 각각 검출했다.
- PNG 소비 경계의 추가 반례는 기존 71 PASS·신규 1 FAIL:
  `logs/color-output-before.log`. 2-pixel RGB에 필요한 6 bytes보다 작은 cap 5에서도
  CMYK 변환이 성공했던 경로다. 보정 후 **72 PASS / 0 FAIL**:
  `logs/color-output-after.log`.
- Native backend/root lib Clippy는 `logs/color-output-clippy.log`,
  `logs/color-output-root-clippy-final.log`에서 통과했다. Docker backend/root WASM lib
  Clippy도 `logs/samples-masks-docker-wasm-final.log`에서 통과했다.
- 최종 Native frontend/여백 **5 PASS**: `logs/samples-masks-focused-final.log`.
  테스트 추가 뒤 파생 배정이 바뀌어 이전 0건 실행은 제외했다. 최종 prepare 후 두 source가
  속한 suite 016에서 실제 5건을 실행했다. 자원 72건은 suite 002에서 실행했다.
- fresh Docker WASM의 이번 경계 검사 **40 PASS**: `samples-masks-probe/results.json`,
  `logs/samples-masks-wasm-final.log`. 24개 예산 경계, 7개 잘못된 입력, 정상 픽셀 4개,
  mask/선택 페이지 오류 계약 5개다. 이전 bundle에서는 이 중 19개가 실패했다
  (`before-results-expanded.json`). CMYK를 단순 RGB로 계산한 초기 진단 기대값은 결함
  건수로 세지 않고 기존 변환의 픽셀 보존 대조로 정정했다.
- 기존 Docker WASM CCITT 24조합·codec 경계 3·오류 후 정상 2·inline 4도 통과:
  `logs/samples-masks-wasm-controls-final.log`,
  `indexed-wasm-probe/results-samples-masks/smoke.json`.
- Native CMYK/Gray/RGB JPEG의 실제 handle→PNG 경계 12개 통과. 실물 이미지 9개는
  Native/WASM 모두 독립 Poppler RGB와 같다. 실물 전체 12쪽·선택 7쪽·선택 8쪽
  HWPX 3개는 이전 정상 Native 및 새 Native/WASM 사이에서 byte 동일하다:
  `samples-masks-probe/native-results.json`, `logs/samples-masks-native-controls-final.log`.
- fmt/check·변경 diff whitespace·고정 base `463c48cc228170b576c832591b76826116e86a49`
  대비 suite 정책 검사가 통과했다. `logs/samples-masks-fmt-final.log`,
  `logs/samples-masks-final-suite-policy.log`를 따른다. workspace all-targets를 포함한
  최종 PR lint 묶음이나 전체 회귀는 이번 결과로 대체하지 않는다.

실행 source/manifest/input/JS/WASM SHA-256은 `samples-masks-probe/source-final.json`에
고정했다. Native와 이번 Docker는 Rust 1.93.1이며, 기존 이미지에서 저장소 toolchain pin을
따라 준비했다. 3.14의 Docker Rust 1.93.0 결과와 구분한다. Docker 프로젝트 `rhwp`와
기존 `/build-target` volume을 재사용했고, 저장소 루트의 locked wrapper로 진단 web
bundle을 만들었다. 제품 root pkg·Studio public·기본 작업공간의 기존 WIP는 변경하지 않았다.

핵심 코드 위치는 `vendor/rhwp-pdf-import-backend/src/extractors/images.rs`의
`raw_rgb_with_budget:518`, `into_rgb_with_budget:556`, `sample_buffer:1058`,
`limited_decode_ranges:1077`, `samples_to_decoded_bytes:1118`, mask 오류 반환 `:4018` 및
`src/parser/pdf/extract.rs:113`이다. 정상 출력 보존 대조는 codec/IR 계약이며 한컴 Print
전쪽 Visual Sweep·제품 Studio/Worker의 실행 증거가 아니다.

이번 절편은 `tests/cases/issue_7911_pdf_resource_limits.rs`의 합성 codec/자원/오류
계약이며, 새 페이지 조판 golden이나 한컴 시각 판정을 추가하지 않았다. 정확한 경계,
실패 후 같은 요청의 fatal 유지·다음 요청 정상 처리, 정상 픽셀과 all-or-error를 검사한다.
기존 3.14 WASM bundle은 `samples-masks-probe/pkg-before/`에 보존했다.

**계속 남은 게이트:** PNG/JPEG private scratch, 객체·색 공간 map cache의 소유 복사,
embedded font/glyph-name/후처리, graphics-state 투명도와 미지원 색 공간의 의미,
제품 CLI/WASM 공개 API·Worker 종료/취소·패키지 소비·전체 회귀·전쪽 시각 검증이다.
`parser_allocation_bounds_proven=false`를 유지하며 이번 절편을 2단계 전체 완료나
외부 비신뢰 입력 배포 준비 완료로 표시하지 않는다.

### 3.16 PNG 제한 출력과 JPEG 입력 복사 선계상 (2026-10-15)

메인테이너의 “다음 절편 진행을 승인합니다”에 따라 기존 WIP 위에서 PNG/JPEG 내부
할당을 추적했다. 시작 head `bc5ff5690`의 로컬 작업이며 기본 작업공간의 WIP·Studio,
공용 Native target과 Docker `rhwp`의 기존 `/build-target`을 보존했다.

**실제 경로와 구현 범위**

- `PdfImage::to_png_bytes_with_budget → encode_png → image::PngEncoder →
  png 0.18.1 write_image_data`는 외부 bounded writer 호출 전에 행 버퍼·압축기 내부
  공간·압축 결과 전체 Vec을 할당했다. 기존 출력 Vec 제한만으로 이 중간 할당은 막지 못했다.
- 제한 경로는 `extractors/png_limited.rs::encode`로 연결했다. 8-bit Gray/RGB의 원래
  샘플에 filter 0을 붙이고 최대 16 KiB의 블록 하나를 재사용한다. 블록을 codec/누적
  예산으로 선계상한 뒤 `try_reserve_exact`하며, 저장 DEFLATE 블록을 bounded writer에
  직접 쓴다. PNG 전체 길이의 checked 사전 검사와 누적 출력·fatal 처리는 유지한다.
  압축 전체 Vec·별도 PNG 압축기 heap은 이 제한 경로에서 생성하지 않는다.
  비제한 API의 기존 adaptive 압축 경로는 유지했다.
- 계약 근거는 [PNG §5.5/10/11.2](https://www.w3.org/TR/png-3/) 및
  [RFC 1951 §3.2.4](https://www.rfc-editor.org/rfc/rfc1951)이다. IDAT 경계가 행과
  filter byte를 가를 수 있음을 반영하며 CRC/Adler와 종료 블록을 독립 디코더로 검증했다.
  픽셀 해석과 조판 좌표를 변경한 것이 아니라 무손실 container 표현의 변경이다.
- `decode_jpeg_with_budget → image 0.25.10 JpegDecoder::new`가 limits 적용 전에
  전체 input을 복사한다. reader 생성 전에 알려진 input payload를 codec/누적 예산으로
  계상했다. private Vec capacity/OOM·header metadata·JPEG coefficient/upsample buffer까지
  입증한 것은 아니다. CMYK의 별도 jpeg-decoder 경로와 함께 후속 감사 대상으로 남긴다.
- 넓은 사용자 지정 한도에서도 PNG 입력의 `width * height * channels`가 u32로
  overflow하지 않도록 checked usize 연산을 사용한다. 잘못된 길이를 출력하지 않는다.

**실행 결과와 증거** — 아래 경로는 `output/pr-review/issue7911/` 기준이다.

| 검증 | 결과 | 로그·자료 |
| --- | --- | --- |
| 수정 전 Native | 기존 72 PASS, PNG/JPEG 새 반례 **2 FAIL** | `logs/png-codec-before.log` |
| 수정 전 WASM | 27 경계 중 **8 FAIL**, 한도 미적용 재현 | `png-codec-probe/before-wasm.json` |
| 최종 Native 자원 계약 | **77 PASS / 0 FAIL** | `logs/png-codec-final-contracts.log` |
| 기존 Native frontend·여백 | **4 + 1 PASS** | `logs/png-codec-frontend.log`, `logs/png-codec-margin.log` |
| fresh Docker WASM 추가 경계 | **27 + 14 PASS** | `logs/png-codec-wasm-boundaries.log`, `logs/png-codec-wasm-raw.log` |
| 기존 WASM 대조군 | samples/masks 40, CCITT 24, 한도 5, 오류 뒤 정상 2, inline 4 PASS | `logs/png-codec-wasm-samples-masks.log`, `logs/png-codec-wasm-existing-controls.log` |
| 실물 이미지 9회 | Native/WASM 모두 독립 Poppler와 RGB 동일 | `png-codec-probe/native-results.json`, `indexed-wasm-probe/results-png-codec/smoke.json` |
| 비제한 PNG 대조군 9회 | 기존 압축 PNG와 byte 동일 | `png-codec-probe/legacy-results.json` |
| 실물 12쪽 전체·선택 7/8쪽 | Native/WASM HWPX byte 동일, 변경 전후 비-PNG entry 동일·PNG 픽셀 동일 | `png-codec-probe/native-results.json` |
| backend/root lib Clippy | Native와 Docker WASM 모두 `-D warnings` PASS | `logs/png-codec-native-*-clippy.log`, `logs/png-codec-docker-wasm.log` |
| fmt·suite 정책·diff | PASS, base `463c48cc228170b576c832591b76826116e86a49` | `logs/png-codec-fmt-check.log`, `logs/png-codec-policy-final.log` |

정식 source는 `tests/cases/issue_7911_pdf_resource_limits.rs`이며 5개 계약을 추가했다.
2개는 수정 전 실패를 확인했고, 나머지는 block/row 분할·개별 출력·누적 반복·정수 경계의
추가 검사다. geometry 검사를 수정 전 실행한 결함 검출 증거로 세지 않는다.
기존 검사 4개에서 PNG 압축 byte 비교를 decoded pixel 비교로 바꾸고, 변환 버퍼와
추가 PNG scratch를 구분했다. Indexed RGB는 72 bytes, PNG는 filter 3 bytes를 포함한
75 bytes이며 각각의 미만·정확 경계·초과와 fatal 상태를 검사한다. 시각 baseline 완화가 아니다.
최종 suite는 자원 002, frontend 027, 여백 028이다. prepare 후 실제 77/4/1건 실행을 확인했다.

Native/Docker Rust 1.93.1, 루트 locked WASM wrapper와 기존 진단 crate를 사용했다.
실행 source·manifest·입력·JS/WASM SHA-256은 `png-codec-probe/source-final.json`에 고정했다.
PNG source SHA-256은 `66ea4aadc5d3f8a81626cac8f8f70f0d348dea2671d5542be9df3a0dec8a0137`이다.
새 dependency/lock 변경은 없으며 제품 root pkg·Studio public을 갱신한 결과가 아니다.

**확인된 크기 비용:** 제한 PNG는 압축하지 않는다. 실물 이미지 9회는 각각 기존의
115.22–766.27배이며 합계 **36,323 → 15,252,683 bytes**다. 동일 encoded-byte 한도에
더 빨리 도달할 수 있다. 별도 12쪽 대조군의 HWPX는 ZIP 재압축으로
**1,607,950 → 1,457,545 bytes**였고, 선택 7/8쪽 HWPX는 byte 동일했다. 최종 ZIP 크기가
줄었다고 내부 PNG 보관 비용을 상쇄한 것으로 판단하지 않는다. 제품 통합 전 제한 가능한
압축 경로와 이 비용을 함께 검토해야 한다. 과거 PNG/HWPX 해시를 억지로 유지하지 않았다.

**남은 범위:** JPEG metadata·progressive/MCU/upsampling/worker의 private allocation,
일반 Flate 라이브러리의 내부 scratch, 객체/색 공간 map cache·embedded font/glyph-name,
graphics-state 투명도 의미, 공개 CLI/WASM API·Worker·패키지 소비와 최종 전체 검증이다.
상세 위치·dependency hash는 `png-codec-probe/allocation-audit-ko.md`에 있다.
`parser_allocation_bounds_proven=false`를 유지한다. 이번 결과는 codec/IR 자원 계약이며
2단계 전체 완료·전쪽 한컴 Visual Sweep·외부 비신뢰 입력 배포 준비 완료가 아니다.

### 3.17 요청별 할당 제한을 유지한 PNG 압축 복구 (2026-10-15)

메인테이너의 다음 절편 승인으로 3.16의 PNG 보관 비용을 보정하고 JPEG 입력 복사의
실제 예약 지점을 보완했다. 시작 head `bc5ff569022bd296c3bfe2f904118b9e81d488e4`와
기존 WIP를 유지한 같은 2단계 작업이다. 공유 Native target·Docker volume을 재사용했다.

**구현과 비용**

- `to_png_bytes_with_budget → encode_png → png_limited::encode → deflate::init/feed/end`를
  연결했다. 기존 전이 의존성 `zlib-rs =0.6.7`의 internal allocator API를 exact pin하여
  사용한다. state/window/hash/pending/symbols와 alignment 공간을 요청 예산으로 선계상하고
  `try_reserve_exact`로 예약한다. 원본 행은 빌려 쓰며 16 KiB 고정 stack 출력 버퍼를
  bounded writer에 직접 전송한다. 전체 filtered/compressed Vec을 만들지 않는다.
  정상 종료·출력 중 오류에서 RAII로 codec allocation을 해제한다.
- 이 버전에서 해당 압축 경로는 단일 private allocation이다. Native **380,041 bytes**,
  wasm32 **379,845 bytes**이며 각각 미만/정확/초과 한도를 실제 실행했다. 크기 차이는
  내부 layout·pointer 폭의 차이이며 출력 PNG/HWPX byte 일치와 별개다. 다른 codec의
  private heap·malloc overhead·전체 RSS를 이 값으로 대표하지 않는다.
- `image 0.25.10 JpegDecoder::new`가 limits 적용 전 `read_to_end`로 input을 소유 복사하는
  지점에 `jpeg_input::Input`을 연결했다. 이전 외부 payload 선계상을 실제 복사 경계의
  예산 확인 → fallible exact reservation → 복사 순서로 옮겼다. RGB/Gray 픽셀 정책과
  CMYK의 별도 decoder 경로는 유지했다. JPEG metadata·계수·upsampling의 제한 완료는 아니다.
- root/backend lock의 변경은 기존 zlib-rs dependency edge 한 줄씩이다. 새 crate 버전이나
  C/native zlib를 도입하지 않았다. `std`, `rust-allocator`, `__internal-api`를 확인했다.
  internal API라 exact version을 갱신할 때 allocation/lifetime 경로를 다시 감사해야 한다.

실물 이미지 9회의 제한 PNG 합계는 **15,252,683 → 35,939 bytes**로 줄었다.
이전 일반 압축 경로는 36,323 bytes였다. 이미지별 이전 일반 압축 대비 비율은
0.8023–1.3082이며, 모든 입력의 압축률 개선을 주장하지 않는다. 제한 경로는 filter None,
비제한 경로는 기존 adaptive filtering이다. 독립 Poppler RGB와 9개 모두 동일했다.
실물 12쪽 HWPX는 1,457,545 → **1,431,027 bytes**, 선택 7/8쪽은 기존과 byte 동일하다.
전체 HWPX의 바뀐 PNG entry는 decoded pixels가 같고 다른 ZIP entry는 byte 동일하다.

**최종 실행 결과** — `output/pr-review/issue7911/` 기준.

| 검증 | 결과 | 증거 |
| --- | --- | --- |
| 압축 비용 반례, 수정 전 | 기존 77 PASS / 새 반복 scanline 검사 **1 FAIL**, PNG 393,972 bytes | `logs/png-compression-before.log` |
| 최종 Native 자원 계약 | **78 PASS / 0 FAIL** | `logs/png-compression-jpeg-final-contracts.log` |
| 기존 Native frontend·명시 여백 | **4 + 1 PASS** | `logs/png-compression-frontend.log`, `logs/png-compression-margin.log` |
| fresh Docker WASM raw PNG | **24 PASS**: codec/output/누적·반복·geometry·여러 IDAT·픽셀·회복 | `logs/png-compression-wasm-raw-final.log`, `png-compression-probe/raw-results.json` |
| fresh WASM samples/masks/JPEG | **48 PASS** | `logs/png-compression-wasm-samples-final.log` |
| 기존 fresh WASM CCITT·inline·오류 | **24 + 6 + 2 + 4 PASS**, 실물 이미지 9회 PASS | `logs/png-compression-wasm-existing-final.log` |
| Native 추가 작은 한도·정상 복귀 | **36 PASS**, 실물 이미지 9회 Poppler 동일 | `png-compression-probe/native-results.json` |
| 실물 frontend 전체/선택 | **3건 Native/WASM HWPX byte 동일**, 비-PNG entry 보존·PNG 픽셀 동일 | `png-compression-probe/native-results.json` |
| 비제한 기존 PNG | **9건 byte 동일** | `png-compression-probe/legacy-results.json` |
| root/backend lib Clippy | Native/Docker WASM 모두 `-D warnings` PASS | `logs/png-compression-native-*-clippy.log`, `logs/png-compression-jpeg-docker-wasm.log` |
| fmt·suite policy·diff | PASS, base `463c48cc228170b576c832591b76826116e86a49` | `logs/png-compression-fmt-check.log`, `logs/png-compression-policy.log`, `logs/png-compression-diff-check.log` |

정식 test source는 `tests/cases/issue_7911_pdf_resource_limits.rs`다. 새 반복 행 검사는
수정 전 실패를 확인했다. 기존 stored-container 99-byte 기대값은 실제 encoded 길이의
미만/정확/초과 한도와 픽셀 검사로 바꿨다. decoder의 작은 한도 계약은 유지하며,
정상 PNG 출력 대조군 4개의 누적 8 KiB 예산은 새 private compressor 공간을 수용하도록
2 MiB로 분리했다. 중간 4건 실패는 이 정상 대조군의 예산 부족이었고 이미지 차이는 아니었다.
PNG scratch는 별도 정확 경계와 누적 반복 검사로 보호한다. 조판 golden/시각 기준을
완화한 것이 아니다. 긴 행·압축이 어려운 입력으로 16 KiB 출력 버퍼를 넘는 경우도 실행했다.

Docker Rust 1.93.1과 루트 locked wrapper로 진단 web package를 새로 생성했다.
WASM은 **21,462,527 bytes**, SHA-256
`9e92466a9b460449c2f68a1c84610c29483bed552996f3486f90d2d741591fcb`다.
소스·manifest·입력·JS/WASM 26개 hash는 `png-compression-probe/source-final.json`,
allocator 실제 호출과 unsafe lifetime·dependency hash는 같은 폴더의
`allocation-audit-ko.md`, `dependency-source.json`에 있다. 제품 root pkg·Studio의
사용자 WIP를 갱신하지 않았다. synthetic codec 계약은 전쪽 시각 검증의 대체물이 아니다.

**다음 경계:** JPEG metadata/progressive/MCU/upsampling/worker의 실제 예약 지점에
요청 context를 전달하는 보안 보완, 일반 Flate 내부 scratch, 객체/색 공간 cache와 font
할당 감사가 남는다. 공개 API/Worker·패키지 소비·전체 회귀·전쪽 한컴 시각 검증은 이후
게이트다. `parser_allocation_bounds_proven=false`를 유지하며 이번 절편을 2단계 전체 완료나
비신뢰 입력 배포 준비 완료로 표시하지 않는다. remote push·PR 상태 변경은 하지 않았다.

### 3.18 일반 Flate 내부 할당의 요청 예산 연결 (2026-10-15)

메인테이너의 다음 절편 승인으로 일반 Flate의 내부 작업 공간을 먼저 보완했다.
시작 head `bc5ff569022bd296c3bfe2f904118b9e81d488e4`와 기존 WIP를 유지한 같은
2단계 절편이다. JPEG private allocation은 실제 호출 경로를 다시 감사했으며 이번
절편에서 제한 완료로 표시하지 않는다.

**원인과 구현:** 기존 `bounded::decode`는 Flate 출력만 `BoundedOutput`으로 제한했다.
`flate2::read::ZlibDecoder/DeflateDecoder`의 state/window 예약에는 요청 context가
없어, codec 한도가 **1 byte여도 출력이 빈 정상 zlib 스트림이 성공**했다. 정식 검사로
이 실패를 확인하고 `flate_limited → zlib_limited::Stream → inflate::init → allocator`
경로를 연결했다. 실제 예약량을 `charge_codec_allocation`으로 개별·누적 예산에 먼저
반영한 뒤 `try_reserve_exact`를 사용한다. PNG 압축도 같은 allocator/RAII 수명 관리를
공유하며 기존 압축 정책은 유지한다. dependency 버전·lock은 3.17에서 추가 변경하지 않았다.

- pinned zlib-rs 0.6.7의 inflate init은 state와 32 KiB window/padding을 한 번에
  예약한다. 실제 요청은 Native **47,561 bytes**, wasm32 **47,365 bytes**였다.
  고정 구조체 크기를 테스트 기대값으로 쓰지 않고 실제 정상 예약량의 미만/정확/초과
  한도와 누적·반복 호출을 검사한다. 두 ABI의 차이와 전체 RSS는 구분한다.
- zlib 실패 후 raw Deflate 대체 경로는 유지한다. 첫 시도의 해제와 별개로 비용은 같은
  요청 누적값에 남으며, resource 실패에는 대체 경로를 실행하지 않는다. 출력은 4 KiB
  stack buffer에서 bounded writer로 보낸다. 정상 종료·checksum 실패·잘린 입력·뒤따르는
  PDF 바이트의 기존 동작을 보호했다. 잘린 입력 검사는 수정 전에도 통과했으므로 새 결함
  검출로 세지 않는다.
- context는 mutable lifetime으로 빌려 해제까지 이동하지 못하게 하고, 생성 때 고정한
  압축/해제 mode로 init/step/end를 호출한다. input/output 길이는 u32 경계에서 나누고
  반환 시 pointer를 지운다. dependency 예약·alignment·해제와 callback의 실제 호출은
  `flate-scratch-probe/allocation-audit-ko.md`, dependency hash는 `dependency-source.json`에 있다.
- 정식 소비 경로 검사는 작은 codec 한도를 content/path, constructor xref, inline Image와
  Image XObject에 전달해 fatal 오류 및 재시도 거부를 확인했다. 정상 content/image 대조군은
  별도 요청으로 성공하며 픽셀도 보존한다. 실제 PDF의 정상 xref/본문 경로는 아래 실물
  frontend 검사로 대조했다.

**최종 실행 결과** — `output/pr-review/issue7911/` 기준.

| 검증 | 결과 | 증거 |
| --- | --- | --- |
| 수정 전 빈 Flate의 private limit | 기존/대조 **79 PASS**, 새 guard **1 FAIL** | `logs/flate-before.log` |
| 정식 Native 자원 계약 | **84 PASS / 0 FAIL** | `logs/flate-contracts-final.log` |
| 기존 frontend·명시 여백 | **4 + 1 PASS** | `logs/flate-frontend.log`, `logs/flate-margin.log` |
| 추가 Native Flate 경계·복귀 | **35 PASS** | `flate-scratch-probe/native-results.json` |
| fresh Docker WASM Flate 경계·복귀 | **34 PASS** | `flate-scratch-probe/wasm-results.json` |
| shared allocator의 fresh WASM PNG | **24 PASS**, 이전 PNG 8종 byte 동일 | `logs/flate-wasm-png.log`, `flate-scratch-probe/png-bytes.json` |
| fresh WASM samples/masks/JPEG | **48 PASS** | `logs/flate-wasm-samples.log` |
| 기존 WASM CCITT·codec·잘못된 입력·inline | **24 + 6 + 2 + 4 PASS** | `logs/flate-wasm-existing.log` |
| Native 추가 작은 한도·정상 복귀 | **36 PASS** | `flate-scratch-probe/native-controls-results.json` |
| 실물 PNG·frontend | **9개 PNG** 독립 Poppler 픽셀 동일·3.17 PNG byte 동일; **3개 HWPX** Native/WASM 및 3.17 byte 동일 | `flate-scratch-probe/native-controls-results.json`, `logs/flate-native-controls.log` |
| root/backend lib Clippy | Native/Docker WASM 모두 `-D warnings` PASS | `logs/flate-native-*-clippy.log`, `logs/flate-docker-wasm.log` |
| fmt·suite policy·diff | PASS, policy base `463c48cc228170b576c832591b76826116e86a49` | `logs/flate-fmt-check.log`, `logs/flate-policy.log`, `logs/flate-diff-check.log` |

정식 tests/cases에는 자원 계약 6건을 추가했다. 기존 필터 출력 경계 검사의 4 KiB
누적 예산은 출력 한도와 codec 내부 한도를 분리하도록 1 MiB로 조정하고, 누적값을
`출력 + 계상된 codec 예약`으로 검사했다. 낮은 codec·누적 한도는 새 검사에서 직접
실행한다. 조판 검사·golden·시각 gate를 완화한 것이 아니다. Native 35건에는 정상 크기를
측정하는 1건이 포함되어 WASM 34건과 수가 다르다.

루트 locked wrapper와 Docker Rust 1.93.1로 진단용 fresh WASM을 생성했다. 크기는
**21,461,770 bytes**, SHA-256은
`4fc48759f76b69b2885ba8031be563c564742ca30762f4ae2497a4bd38456440`다.
소스·test·manifest·입력·JS/WASM 25개 hash는 `flate-scratch-probe/source-final.json`에
고정했다. 실제 Rust frontend 검증이며 제품 공개 API/Worker·Studio 검증과 구분한다.
기존 제품 pkg/Studio 사용자 WIP는 변경하지 않았다.

**다음 경계:** JPEG의 image adapter는 limits 적용 전 header를 해석하고 실제 decode 때
다시 해석한다. zune-jpeg metadata·progressive/MCU·row/upsampling·worker 예약 및 CMYK의
jpeg-decoder 별도 plane/metadata 예약에 요청 context를 전달하는 보안 패치가 남는다.
extended XMP는 연속 offset·완전성 확인 후 재조립하므로 선언 길이만으로 무조건 예약한다고
단정하지 않는다. 객체/색 공간 cache·font 할당 감사, 공개 API/Worker·패키지 소비·전체
회귀·전쪽 한컴 시각 검증도 남아 있다. `parser_allocation_bounds_proven=false`를 유지한다.
이번 결과는 일반 Flate의 할당 경계 완료이며 2단계 전체 완료나 비신뢰 입력 배포 준비
완료가 아니다. remote push·PR 상태 변경은 수행하지 않았다.

### 3.19 RGB/Gray JPEG의 실제 예약에 요청 예산 연결 (2026-10-16)

다음 절편 승인에 따라 `bc5ff569022bd296c3bfe2f904118b9e81d488e4`와 기존 WIP를
유지하고 JPEG의 zune 경로를 보완했다. 일반 이미지 소비자·CMYK 별도 디코더와
구분한 같은 2단계의 보안 절편이며 backend 기능 축소는 하지 않았다.

**위반 계약과 수정 전 증거:** 기존 제한 경로는 JPEG 입력·출력만 계상했다. libjpeg로
생성한 19×17 RGB JPEG에 32 KiB EXIF payload 네 개를 추가하면, 입력 증가량
131,112 bytes만 codec 누적값에 반영됐다. 교체되며 해제된 metadata 131,072 bytes가
빠져 있었다. 동일 검사의 수정 전 실행은 의도한 assertion에서 FAIL했고, 보정 후
누적 증가량은 **262,184 bytes**로 PASS했다. `jpeg-alloc-probe/before.log`, `after.log`와
정식 `replaced_jpeg_metadata_reservations_remain_in_the_request_budget`에 연결했다.
고정 codec 구조체 크기를 이 기대값으로 사용하지 않았다.

**구현 경로:** `PdfImage::to_png_bytes_with_budget → decode_jpeg_with_budget →
jpeg_limited::decode → JpegDecoder::set_allocation_budget → decode_headers/decode_into →
Allocator::reserve → ImportBudget::charge_codec_allocation → try_reserve_exact`.

- 기존 zune-jpeg **0.5.15** 소스를 별도 `rhwp-zune-jpeg 0.5.15-rhwp.1` local dependency로
  고정했다. `publish=false`, zune-core **0.5.3**이며 보안 patch·상류 source hash·license는
  `vendor/rhwp-zune-jpeg/`에 보존한다. `image`의 registry zune와 CMYK jpeg-decoder는
  교체하지 않는다. 현재 workspace rustfmt에 의한 상류 소스 서식 차이는 있다.
- header 전에 빌린 요청 context를 metadata·component·MCU·baseline 별도 scan 계수·
  progressive plane·row/upsampling·worker resize까지 전달했다. 크기 곱·증가를 검사하고
  요청 한도 승인 후 fallible reserve한다. Vec 증가 시 전체 새 예약량을 계상한다.
  해제된 메모리도 누적 비용에서 빼지 않으며 allocator overhead/RSS와 구분한다.
- EXIF/IPTC/XMP/ICC/MPF/gain-map payload와 collection 증가를 제한한다. Extended XMP는
  GUID·전체 크기·연속 offset·완전성을 확인한 뒤 조립한다. 큰 미완성 선언은 조립 공간을
  예약하지 않는다. sort는 추가 할당이 없는 unstable sort이며 offset 합산은 checked다.
- 동일한 non-strict·색 공간 선택을 유지한다. 기존 owned input copy도 남겨서 제한하며,
  출력은 별도 fallible 예약 후 decode_into로 채운다. 각 호출 뒤 sticky budget을 먼저
  확인해 한도 초과가 일반 이미지 오류나 부분 성공으로 바뀌지 않게 한다.
- 원래 CMYK Adobe 반전 경로는 유지한다. 일반 JPEG의 `info()` clone·`icc_profile()`
  조립 accessor는 이 소비 경로에서 호출하지 않는다. 임의 reader 구현·오류 문자열·고정
  구조체·CPU/전체 heap 격리까지 검증한 것으로 해석하지 않는다.

**집중 검증 결과** — `output/pr-review/issue7911/` 기준.

| 검증 | 결과 | 증거 |
| --- | --- | --- |
| 정식 자원 계약 | **87 PASS / 0 FAIL** (새 계약 3건 포함) | `logs/jpeg-contracts-final.log` |
| frontend·명시 여백 | **4 + 1 PASS** | `logs/jpeg-frontend.log`, `logs/jpeg-margin.log` |
| Native 실제 예약 거부 | 14 입력의 **229개 예약**을 하나씩 거부, 모두 AllocationLimit | `jpeg-alloc-probe/native-traces.json` |
| fresh Docker WASM 실제 예약·한도 | 동일 **229개 예약**, 개별/누적/반복 **80건 PASS** | `jpeg-alloc-probe/wasm-jpeg-results.json` |
| JPEG 픽셀 보존 | Native 16개가 기존 decoder 픽셀 동일; WASM 16개가 3.18 WASM과 PNG byte 동일, 실제 PDF XObject 소비 경로도 동일 | `jpeg-alloc-probe/native-pixels.json`, `wasm-jpeg-results.json` |
| 기존 WASM PNG·Flate | PNG **24**, Flate **34 PASS**, 이전 PNG 8종 byte 동일 | `logs/jpeg-wasm-png.log`, `jpeg-wasm-flate.log`, `jpeg-wasm-png-bytes.log` |
| 기존 WASM samples/masks/JPEG | **48 PASS** | `logs/jpeg-wasm-controls.log` |
| 기존 WASM CCITT·codec·invalid·inline | **24 + 6 + 2 + 4 PASS** | `logs/jpeg-wasm-existing.log` |
| Native 추가 작은 한도·정상 복귀 | **36 PASS** | `jpeg-alloc-probe/native-controls-results.json` |
| 실물 PNG·frontend 보존 | **9 PNG** 독립 Poppler 픽셀·기존 PNG byte 동일; **3 HWPX** Native/WASM 및 이전 결과 byte 동일 | `logs/jpeg-native-controls.log`, `jpeg-alloc-probe/native-controls-results.json` |
| root/backend lib Clippy | Native·Docker WASM `-D warnings` PASS | `logs/jpeg-native-*-clippy.log`, `logs/jpeg-docker-clippy-final.log` |
| fmt·suite policy·diff | PASS; base `463c48cc228170b576c832591b76826116e86a49` | `logs/jpeg-fmt.log`, `jpeg-policy.log`, `jpeg-diff-check.log` |

예약 진단에는 4:4:4·4:2:2·4:2:0, baseline/progressive, Gray, 비인터리브 baseline,
혼합 sampling, 여러 metadata, 완전/미완성 extended XMP가 포함된다. horizontal sampling
대조군은 통과했으나, worker resize가 기존 capacity를 넘어 **새 예약이 생기는 분기**는 이 입력에서
관측되지 않았다. 이 증가 분기는 정적 연결 확인이며 실행 검증 완료로 표시하지 않는다.
16개 JPEG 중 CMYK 2개에는 Native와 WASM 사이 채널값 **최대 1, 각 23개 채널** 차이가
있다. 각 플랫폼의 수정 전후 출력은 동일하며 이번 RGB/Gray 보정의 회귀가 아니다.

정식 새 검사는 반복 metadata 비용·sticky/request 격리·10개 기본 JPEG 픽셀 보존이다.
`tests/fixtures/issue7911_jpeg/README.md`에 입력 생성 방식과 버전을 기록했다. 합성 codec
계약이며 조판 golden을 등록한 것이 아니다. 기존 입력-copy 검사의 중간 한도에서는
이제 private component 예약도 먼저 거절될 수 있어 `input+output`을 반드시 계상했다는
가정을 `input` 선행 예약 보존으로 수정했다. 큰 한도에서는 픽셀 보존을 계속 검사한다.
첫 84건 실행에는 새 3건이 누락된 준비 오류가 있었으므로 최종 87건 로그만 최종 결과로
사용한다. 검증용 WASM harness의 닫는 괄호 오류도 수정 후 새 wrapper 빌드로 확인했다.

**별도 lint 제한:** 추가로 실행한 vendor 단독 strict Clippy는 통과하지 않았다. 동일
1.93.1·manifest 조건의 상류 원본은 129건, patch는 128건을 진단했다. 상류 SIMD cast·
MSRV·문서/style 경고가 남고, 보정으로 worker의 함수 길이 경고 1건이 추가됐다.
`jpeg-alloc-probe/lint-comparison.json`에 대조했다. 이를 root/backend lib PASS에 섞지
않으며 최종 제출 전 vendor lint 정책·정리 범위 검토가 남는다. 경고를 숨기는 allow는
추가하지 않았다.

최종 Docker locked wrapper 산출물은 **21,389,097 bytes**, SHA-256
`afc3d22b3415df5371c889f3acc59b6091caf9756b38c475accf4d4cd51b48b1`이다.
source·manifest·fixture·probe·WASM hash는 `jpeg-alloc-probe/source-final.json`에 고정한다.
제품 pkg/Studio의 사용자 WIP는 변경하지 않았다. 자료는 optional Rust PDF frontend와
codec 검증이며 공개 API·Worker·Studio·한컴 전쪽 시각 검증을 대신하지 않는다.

메인테이너 직접 확인용 복사본은 `/mnt/e/tmp/issue7911-stage2-review/`에 준비했다.
`wasm/`의 전체 12쪽·원본 7/8쪽 HWPX, `native/` 대조본, `source/`의 입력 PDF와
`jpeg/`의 입력·양쪽 PNG를 포함하며 55개 파일의 동일 hash를 복사 후 확인했다.

**다음 경계:** CMYK의 jpeg-decoder metadata·progressive/plane·worker 실제 예약을
요청 context에 연결하는 보안 보완이 우선이다. worker resize 증가 분기의 실행 증거,
객체/색 공간 cache·font 할당 감사, 공개 API/Worker·패키지 소비·전체 회귀·전쪽 한컴
시각 검증도 남는다. `parser_allocation_bounds_proven=false`를 유지하며 2단계 전체 완료나
비신뢰 입력 배포 준비 완료로 표시하지 않는다. remote push·PR 상태 변경은 하지 않았다.

### 3.20 CMYK JPEG의 실제 예약에 요청 예산 연결 (2026-10-16)

작업 계속 승인에 따라 같은 HEAD `bc5ff569022bd296c3bfe2f904118b9e81d488e4`와
기존 WIP를 보존하고 CMYK의 별도 jpeg-decoder 경로를 보완했다. 이번 절편은 보안
예약 계약이며 색상 변환·backend 기능 축소나 공개 API 연결은 하지 않았다.

**독립 기대값과 수정 전후:** 19×17 CMYK JPEG의 APP1에 32 KiB EXIF payload를
네 번 넣었다. 원래 CMYK 경로는 입력을 빌리고 출력만 계상하므로 입력 증가
131,112 bytes에도 codec 누적 증가가 **0 bytes**였다. EXIF payload 131,072 bytes가
누적값에 포함되어야 한다는 검사가 의도한 assertion에서 FAIL했다. 보정 후에는
marker 임시 버퍼와 EXIF 복사 예약을 모두 포함해 **262,168 bytes 증가**, PNG는 동일하다.
`cmyk-alloc-probe/before.log`, `after.log`와 정식
`cmyk_jpeg_replaced_metadata_is_charged_before_reservation`에 연결했다.

**실제 소비 경로:** `PdfImage::to_png_bytes_with_budget → decode_jpeg_with_budget →
Decoder::new_with_allocation_budget → read_info/decode → Allocator::reserve/charge →
ImportBudget::charge_codec_allocation → try_reserve_exact`.

- jpeg-decoder **0.3.2**를 별도 `rhwp-jpeg-decoder 0.3.2-rhwp.1` local dependency로
  고정했다. 상류 checksum·source hash·revision·license·patch 설명은
  `vendor/rhwp-jpeg-decoder/`에 보존한다. 기존 registry decoder·일반 이미지 경로,
  Adobe 반전·색 변환·IDCT·기본 Rayon 기능과 worker 선택은 유지한다.
- 생성자 table과 header 이전부터 요청 context를 전달한다. APP/COM metadata,
  반복 EXIF/XMP/PSIR·ICC chunk, Huffman scratch/table, frame/scan collection,
  progressive/MCU/lossless plane·row, interleaved 출력과 행별 upsampling을 계상한다.
  크기 연산을 검사하고 Vec는 fallible reserve한다. 교체·해제한 예약도 누적 비용에 남긴다.
- worker plane은 호출 스레드에서 예약한 뒤 Immediate·Rayon·std-thread에 소유권을
  넘긴다. Immediate의 고정 4-slot은 배열과 계상된 scope Box로 표현한다. Rayon의
  행별 실패는 `try_for_each`, 미완료 progressive의 row 복사 실패는 scoped iterator의
  Result로 전달한다. 한도 초과 뒤 일반 이미지 오류나 부분 성공으로 바뀌지 않도록
  각 decoder 호출 후 sticky budget을 먼저 확인한다.
- 고정 크기 quantization Arc·worker Box·generic upsampler Box는 payload를 먼저
  계상하되 stable Rust의 infallible 생성자는 유지한다. allocator overhead, thread/channel·
  스케줄러, 오류 문자열, 임의 reader, RSS·CPU 전체 제한은 이번 입증 범위가 아니다.
  제한 PDF 경로가 호출하지 않는 `icc_profile()` accessor의 추가 조립 예약도 제외한다.

**최종 집중 검증** — `output/pr-review/issue7911/` 기준.

| 검증 | 결과 | 증거 |
| --- | --- | --- |
| 정식 자원 계약 | **91 PASS / 0 FAIL**, 새 계약 4건 포함 | `logs/cmyk-contracts-final.log` |
| frontend·명시 여백 | **4 + 1 PASS** | `logs/cmyk-issue_7911_pdf_frontend.log`, `cmyk-issue_7911_explicit_text_margin.log` |
| Native 예약 거부 | 23개 입력 **3,607건 PASS**; 큰 병렬 입력은 위치 표본 외에 종류별 **50건** 추가 | `cmyk-alloc-probe/native-traces.json`, `native-large-kind-denials.json` |
| std-thread 경로 | 3개 입력의 예약 거부 **76건**, 기존 픽셀 동일 | `cmyk-alloc-probe/native-thread-results.json` |
| fresh Docker WASM | 23개 입력 예약 거부 **3,607건**, 한도·반복 **110건 PASS** | `cmyk-alloc-probe/wasm-cmyk-results.json` |
| Native 출력 보존 | raw sample **23개** registry decoder와 동일; PNG **22개**, 정상 CMYK legacy PNG 대조 **14개** 동일, 누적 경계 **66건** | `cmyk-alloc-probe/native-pixels.json` |
| WASM 출력 보존 | PNG **22개** 직전 WASM과 byte 동일, 실제 PDF XObject 소비 경로와도 동일 | `cmyk-alloc-probe/wasm-cmyk-results.json` |
| 기존 WASM 대조군 | samples/masks **48건**, CCITT·codec·invalid·inline **24+6+2+4건 PASS** | `logs/cmyk-wasm-masks.log`, `cmyk-wasm-existing.log` |
| 기존 Native·실물 | 작은 한도 **36건**, 실물 PNG **9개** Poppler 픽셀·이전 byte 동일, HWPX **3개** 이전 및 Native/WASM byte 동일 | `cmyk-alloc-probe/native-controls-results.json`, `logs/cmyk-native-controls.log` |
| Clippy | root/backend lib Native·Docker WASM `-D warnings` PASS; 새 fork lib Native 기본·no-default 및 Docker WASM PASS | `logs/cmyk-*-clippy.log`, `cmyk-docker-final.log` |
| fmt·정책·diff | PASS, 정책 base `463c48cc228170b576c832591b76826116e86a49` | `logs/cmyk-fmt.log`, `cmyk-policy-final.log`, `cmyk-diff-check.log` |

예약 입력은 CMYK/YCCK baseline·progressive·분리 scan·미완료 progressive·sampling·
metadata와 8/12-bit lossless를 포함한다. CMYK 선언과 실제 RGB/Gray JPEG가 다른
경계 대조군도 포함하므로 전부를 정상 CMYK 문서라고 표현하지 않는다. 12-bit lossless는
decoder sample을 독립 midpoint **2048**과 대조했고, 기존 제품 PNG 경로의 미지원 상태는
유지했다. 8-bit lossless midpoint **128**도 확인했다. 큰 Native 병렬 입력은 실행 순서가
바뀔 수 있어 예약 번호 표본만으로 종류별 분기를 입증하지 않고 이름으로 거부하는
추가 검사를 수행했다. Native/WASM 간에는 기존 IDCT·색 변환의 채널값 최대 **2** 차이가
있으며 같은 플랫폼의 수정 전후 결과가 동일하다는 보존 근거와 구분한다.

정식 검사는 반복 metadata 비용·sticky/request 격리·실제 PDF XObject 오류 전파·
큰 worker와 lossless의 출력 보존이다. 입력 생성 근거는
`tests/fixtures/issue7911_jpeg/README.md`와 `make_cmyk_scans.c`에 기록했다.
조판 fixture/golden 추가가 아니다. 초기 시험의 private field 접근 컴파일 오류와 새 fork의
상류 WASM cfg/lint 오류는 수정 후 위 최종 검사로 확인했다. 마지막 테스트 source 변경 뒤
생긴 generated harness drift는 파생 suite를 재준비해 정책 검사를 통과시켰으며, 정식 source
91건의 성공을 정책 drift로 바꾸거나 파생 파일을 stage하지 않았다.

최종 Docker WASM은 **21,489,783 bytes**, SHA-256
`e4c4739fb6d83f25c328ef9ca7b641ff194adb07a8b1a2790380be8718713ac2`이다.
소스·lock·fixture·probe·출력 hash는 `cmyk-alloc-probe/source-final.json`에 고정한다.
직접 확인용 JPEG/Native PNG/WASM PNG **67개**는
`/mnt/e/tmp/issue7911-stage2-review/cmyk/`에 복사하고 hash 일치를 확인했다.
상위 `wasm/`의 HWPX 3개는 새 결과와 동일하므로 기존 확인 파일을 덮어쓰지 않았다.
제품 pkg/Studio 및 다른 worktree의 사용자 변경은 보존했다.

**다음 경계:** 객체/색 공간 cache·font 할당 감사, 3.19 zune worker resize 증가 분기의
실행 증거와 vendor 단독 lint 정리가 남는다. 공개 API/Worker·패키지 소비·전체 회귀·
전쪽 한컴 시각 검증은 이후 게이트다. `parser_allocation_bounds_proven=false`를 유지하며
2단계 전체 완료·비신뢰 입력 배포 준비 완료로 표시하지 않는다. remote push·PR 변경·
커밋·merge는 수행하지 않았다.

### 3.21 객체·색 공간의 소유 데이터 복사 예산 (2026-10-16)

메인테이너의 “계속 작업 진행하세요” 지시에 따라 같은 2단계를 이어갔다. 시작 HEAD는
`bc5ff569022bd296c3bfe2f904118b9e81d488e4`, 정책 base는
`463c48cc228170b576c832591b76826116e86a49`다. 이전 JPEG/Flate 보완 WIP와
공유 `target/pr-review`를 보존했다. 커밋·원격 push·PR 상태 변경은 수행하지 않았다.

#### 실제 경로와 수정

- `document.rs::load_object` → `load_object_inner`의 일반·double-check·recovery cache hit,
  `load_uncompressed_object_impl`의 cache 보관, `load_compressed_object`의 반환 복사에
  `object_copy::{clone_object,clone_dictionary}`를 연결했다. `String`·name·배열 slot·
  사전 key/value slot과 내부 소유 payload를 checked arithmetic으로 합산하고, 요청의
  **남은 누적 decoded 예산을 먼저 소비한 뒤** fallible reserve와 복사를 수행한다.
  원본 스트림 bytes와 요청 budget은 공유하며, 읽기 전용 Form subtype 조회는 전체 복사를 제거했다.
- `page_image_handles` → `collect_handles_from_do`/`collect_image_handles_from_form_xobject`의
  resources·XObject 사전, `build_color_space_map`, 두 image-handle builder의 사전 복사도
  같은 경계를 사용한다. 색 공간의 읽기 전용 조회는 빌린 객체를 사용하고 Indexed base의
  수정 가능한 사본은 계상한다. 제한 오류가 빈 사전·부분 handle 성공으로 숨겨지지 않는다.
- 객체 메타데이터는 스트림 출력 버퍼와 구분한다. 첫 통합 실행은 새 객체 복사에
  `stream_bytes` cap까지 적용해 기존 작은 스트림/텍스트 cap의 관측을 먼저 막았다.
  해당 적용을 제거하고 누적 요청 예산과 fallible reserve는 유지했다. 스트림/텍스트/PNG
  출력 cap과 codec cap, 실제 색 변환·샘플 해석·Document IR 배치는 변경하지 않았다.
- 계상 대상은 **요청한 payload/collection slot의 누적 작업량**이다. HashMap bucket의 여유,
  allocator overhead, cache index HashMap/VecDeque 성장, 전체 live heap/RSS의 증명이 아니다.
  legacy 비제한 API는 기존 Object clone을 사용한다.

#### 결함 재현과 검사 변경

`object-cache-probe/before.log`: 4,097-byte 문자열을 가진 중첩 객체의 warm load에서
기존 계상 증가량이 **0 bytes**여서 assertion이 실패했다. 새 정식 검사 6개 중 5개는
수정 전 실패, 1개는 기존에도 통과한 오류 전파 대조군이다
(`contracts-before.log`: 5 FAIL/1 PASS). 보완 후 6개와 기존 출력 경계 4개를 묶은
실행은 10 PASS였다(`ten-contracts.log`).

첫 전체 집중 실행은 93 PASS/4 FAIL이었다(`logs/object-cache-contracts.log`).
그중 stream/텍스트 cap 검사 2개는 구현의 메타데이터 cap 분리로 복구했다. 기존 검사 2개는
계상 범위가 넓어진 것을 반영했다: content 반환은 초기 parse/cache 준비 비용을 분리한 뒤
매번 **정확히 4 bytes**, XObject raw 반환은 같은 객체의 metadata-copy 비용과 실제
**6 bytes** 반환을 합쳐 미만·정확·초과 경계를 검사한다. inline raw는 여전히 6 bytes다.
누적 한도·sticky 실패·새 요청 복구 기대값을 완화하지 않았다. 조판 baseline 변경은 없다.

#### 최종 검증

| 검사 | 최종 결과 | 증거 (`output/pr-review/issue7911/` 기준) |
| --- | --- | --- |
| Native 정식 자원 계약 | **97 PASS / 0 FAIL** | `logs/object-cache-contracts-final.log` |
| frontend·명시 여백 계약 | **4 + 1 PASS** | `logs/object-cache-issue_7911_*.log` |
| 실제 작은 이미지 예산 경계 | **36 PASS** | `object-cache-probe/native-controls-results.json` |
| fresh Docker WASM 계약 | 신규 6 + 기존 출력 경계 4 = **10 PASS** | `object-cache-probe/wasm-results.json` |
| 실물 Indexed 이미지 9개 | Native/WASM 모두 Poppler 기준 픽셀 동일, 이전 출력 보존 | `object-cache-probe/{native-controls-results,wasm-results}.json` |
| 실물 12쪽 PDF 전체/7쪽/8쪽 | Native/WASM HWPX 3개가 이전 Native/WASM과 byte-identical | 같은 JSON, `native-real-frontend/`, `wasm/` |
| Native root/backend lib Clippy | 모두 `-D warnings` PASS | `logs/object-cache-root-clippy.log`, `object-cache-backend-clippy-final.log` |
| Docker WASM root/backend lib Clippy | 모두 `-D warnings` PASS | `logs/object-cache-docker-final.log` |
| fmt·suite 정책·diff | 모두 PASS, 위 고정 base와 비교 | `logs/object-cache-{fmt,policy,diff-check}.log` |

정식 검사는 `tests/cases/issue_7911_pdf_resource_limits.rs`에 추가했다. synthetic 객체·색 공간
데이터는 자원 계약이며 한컴 조판 golden이 아니다. 최종 WASM은 Docker에서 루트
`scripts/wasm-pack-locked.sh`로 **기존 진단 crate**를 빌드한 결과다. 제품 root pkg/Studio
공개 API를 갱신하거나 전체 Worker/시각 gate를 통과했다는 의미가 아니다.

WASM SHA-256: `ca6e86e7965404fda69c3b0d3cfa62a3dc83db1d417c5f63a2fca2b94c7a2a01`, 크기 **21,565,064 bytes**.
source/테스트·입력·증거 목록은 `object-cache-probe/source-final.json`에 고정했다.
첫 Native 4개 실패 로그와 초기 WASM 빌드 로그도 보존했으며 최종 source 증거와 구별한다.
실물 `nested_table_text_caption-2020.pdf`의 전체/선택 변환은 각각
**1,431,027 / 15,175 / 15,777 bytes**다. 기존에 전달한
`E:\tmp\issue7911-stage2-review\{native,wasm}\`의 6개 HWPX도 현재 결과와
byte 동일함을 재확인했다(`delivery-verification.json`). 기존 사용자 파일은 덮어쓰지 않았다.

**다음 경계:** `load_fonts`의 전역 cross-document cache는 현재 요청 budget과의 격리를
추가 검증해야 한다. `FontInfo::truetype_cmap`의 lazy 파싱, TrueType format12의 선언 범위
확장, Type1/CFF encoding·Unicode memo와 font/cache index 예약도 남아 있다. 실제 코드
지점은 `object-cache-probe/next-font-audit.md`에 기록했으며 아직 실행 재현/보완 완료로
판정하지 않았다. 3.19 zune worker 증가 분기·vendor lint, 공개 API/Worker·패키지 소비·
전체 회귀·전쪽 한컴 시각 검증도 이후 게이트다. `parser_allocation_bounds_proven=false`를
유지한다. 이번 절편을 전체 객체/cache/font 할당 안전성 또는 2단계 전체 완료로 표시하지 않는다.

### 3.22 폰트 전역 캐시 격리와 TrueType cmap 예약 경계 (2026-10-16)

메인테이너의 “계속 작업 진행하세요” 지시로 같은 2단계를 이어갔다. HEAD와 정책 base는
3.21과 동일하다. 이전 WIP·공용 review target을 보존했으며 커밋·원격 push·PR 상태 변경은
수행하지 않았다. 새 public PDF API나 Studio 연결을 완료한 절편은 아니다.

#### 재현·구현 경로

- `document.rs::load_fonts → load_fonts_impl`의 global FontInfo cache에서 제한 없는
  이전 문서의 폰트를 재사용하면 새 요청의 CID metric 한도를 우회했다. 제한된 요청은
  global get/insert를 모두 우회하고 문서 내부 cache만 공유한다. legacy 간 공유는 유지한다.
  resources/font 직접 객체 복사는 기존 bounded copy를 사용하며 lazy/fallback 뒤 sticky
  오류를 확인해 실패를 빈 폰트·부분 성공으로 숨기지 않는다.
- `FontInfo::from_dict → import_budget`이 원래 요청 context를 보관한다. 소비 위치는
  simple font의 `truetype_cmap` lazy parsing과 Type0의
  `extract_truetype_cmap_from_descriptor`다. 후자는 별도 eager 경로임을 확인해 함께 연결했다.
  이미 성공한 lazy 결과도 원래 요청이 실패하면 다시 성공으로 반환하지 않는다.
- `TrueTypeCMap::from_font_data_with_budget`는 sfnt table/subtable offset·길이 범위를
  확인한다. format 4가 뒤 table bytes까지 glyph 배열로 읽던 경계를 닫았다. format
  0/4/6/12·symbol map의 확장 작업을 `font_entries`와 누적 decoded 예산으로 선계상하고
  fallible reserve한다. format 4 scratch는 별도로 계상하며 format 12의 역구간·Unicode/
  glyph overflow를 거부한다. 중복/덮어쓰기 항목도 작업량에 포함한다.
- 두 TrueType map은 immutable Arc로 공유해 `FontInfo` clone과 cmap donation에서
  무계상 deep copy를 만들지 않는다. 모든 FontInfo 필드·cache index·allocator overhead나
  live heap/RSS를 증명한 것은 아니다. 기존 한도 수치나 조판 baseline을 완화하지 않았다.
- 독립 근거는 [OpenType cmap 명세](https://learn.microsoft.com/en-us/typography/opentype/spec/cmap)의
  table/subtable 길이, format 4의 상대 offset, format 12의 연속 code/GID 관계다.
  새 검사는 synthetic 자원 계약이며 한컴 조판 fixture/golden이 아니다.

#### 검증과 입력 수정의 구분

`font-cmap-probe/before.log`의 신규 반례 7개는 수정 전 모두 실패했다. 전역 cache의
요청 한도 우회·요청 간 Arc 공유, lazy mapping 무계상·오류 뒤 재사용, GID wrap을 검출했다.
보완 후 같은 반례와 실제 FontFile2 simple/CID 경로, format 0/4/6/12·symbol 경계,
큰 선언·table 밖 glyph 읽기를 합친 **14개**가 Native와 새 Docker WASM에서 통과했다.

추가 경로 검사의 첫 입력은 simple font의 FontDescriptor를 직접 dictionary로 만들었으나,
현재 backend가 이 형태를 소비하지 않아 embedded font가 없었다(`diagnosis.log`). 정상
간접 FontDescriptor로 입력을 고쳐 실제 FontFile2 경로를 검증했다. 이를 구현 결함 해결
실적으로 세지 않았다. 별도 TrueType 기존 unit 실행은 5 PASS/18 FAIL이었으며, 원인은
합성 생성기가 directory length를 0으로 써 놓은 것이었다(`module-tests-initial.log`).
실제 직렬화 길이로 생성기를 보정한 뒤 **23 PASS**였다. 매핑 기대값은 변경하지 않았다.
첫 Docker Clippy의 `manual_is_multiple_of` 1건은 수정 후 다시 통과했다.
정식 검사를 추가한 뒤 source weight가 바뀌면서 이전 generated suite와 runner의 동적
배정이 달라졌다. 보조 검사 두 묶음의 최초 0건 실행과 suite drift는 통과로 세지 않고
`logs/font-cmap-stale-suites-*`에 보존했다. 최종 source 고정 후 `--prepare`를 다시 실행해
frontend **4건**, 여백 **1건**을 실제 실행하고 base 정책 검사도 통과했다. 자원 **111건**은
처음부터 올바른 suite에서 실제 실행됐으므로 생성 파일 갱신만을 이유로 반복하지 않았다.

| 최종 검사 | 결과 | 증거 (`output/pr-review/issue7911/` 기준) |
| --- | --- | --- |
| Native 정식 자원 계약 | **111 PASS / 0 FAIL** (기존 97 + 신규 14) | `logs/font-cmap-final-issue_7911_pdf_resource_limits.log` |
| frontend·명시 여백 | **4 + 1 PASS** | `logs/font-cmap-final-issue_7911_pdf_*.log`, `logs/font-cmap-final-issue_7911_explicit_text_margin.log` |
| 기존 TrueType parser unit | **23 PASS**, source module 직접 컴파일 | `font-cmap-probe/module-tests-final.log` |
| fresh Docker WASM 새 계약 | **14 PASS** | `font-cmap-probe/wasm-results.json` |
| 작은 이미지 budget 대조군 | **36 PASS** | `font-cmap-probe/native-controls-results.json` |
| 실물 Indexed 이미지 9개 | Native/WASM 모두 Poppler 픽셀 일치·기존 출력 보존 | 두 controls/results JSON |
| 실물 12쪽 PDF 전체/7쪽/8쪽 | Native/WASM HWPX **3개 모두 기존 bytes 보존** | `font-cmap-probe/{native-real-frontend,wasm}/`, 두 JSON |
| Native·Docker WASM root/backend lib Clippy | **모두 PASS**, `-D warnings` | `logs/font-cmap-{root-clippy,backend-clippy,docker-final}.log` |
| fmt·suite/unit 정책·diff | **PASS**, 고정 base 비교 | `logs/font-cmap-{fmt,policy,unit-policy,diff-check}.log` |

최종 WASM SHA-256: `145c93230bfc9526008163267452508bb2cd2578efd5084d869cd86bb6492249`, **23,050,719 bytes**.
Docker에서 기존 진단 crate를 루트 `scripts/wasm-pack-locked.sh`로 빌드했다. 제품 root
pkg/Studio의 공개 import 경로는 아직 연결하지 않았다. source·입력·명령·로그·산출물
해시는 `font-cmap-probe/source-final.json`에 고정했다. 최종 Native 묶음 명령은
`font-cmap-probe/native-final.sh`, WASM 실행은 `font-cmap-probe/smoke.mjs`다.
이전 `E:\tmp\issue7911-stage2-review\{native,wasm}\`의 6개 HWPX와 현재 결과도
byte 동일함을 확인했다(`delivery-verification.json`). 사용자 전달 파일은 덮어쓰지 않았다.

**다음 경계:** Type1/CFF encoding·glyph name·Unicode memo·font/cache index 예약과
FontInfo의 나머지 payload 복사가 남는다(`font-cmap-probe/next-font-audit.md`).
3.19 zune worker 증가 분기/vendor lint, 공개 API/Worker·패키지 소비·전체 회귀·전쪽
한컴 시각 검증도 남아 있다. `parser_allocation_bounds_proven=false`를 유지하며
이 결과를 전체 font 할당 안전성·2단계 완료·비신뢰 입력 배포 준비 완료로 표시하지 않는다.

### 3.23 중간 점검과 제품 전체 경로 우선 결정 (2026-10-16)

메인테이너는 현재 작업의 분할정복 흐름을 점검한 뒤 “전체적인 구현 완료부터 한 후,
백로그의 세부 항목들을 구현해나가는 방식”을 지시했다. 보안 보완은 실제 목표에 필요하지만
제품 CLI/WASM 연결을 계속 뒤로 미루는 선행 조건으로 확장하지 않도록 순서를 변경한다.

- 수행·구현계획에 **전체 기능 구현 → 세부 백로그 → 최종 통합 검증**을 반영했다.
- 미완료 폰트·캐시·JPEG 실행 경계·vendor lint·패키지·시각·최종 검증은
  [B01–B12](../plans/task_m100_7911_backlog.md)로 이관했다. 이관은 해결이나 검증 통과가 아니다.
- 다음 구현은 F1 Parser/Core → F2 CLI → F3 제품 WASM/Worker → F4 실제 렌더·저장·재열기다.
  기능을 막는 결함은 해당 경로에서 처리하고 나머지 세부 발견은 백로그에 기록한다.
- 3.22에서 이미 보완한 전역 폰트 cache 요청 격리·TrueType cmap 공유를 다시 미완료로
  나열하던 4절을 정정했다. 남은 font/cache index·payload는 B03에 구분했다.
- 기존 자원 제한·정상 대조군·미지원 정책을 유지한다. `parser_allocation_bounds_proven=false`와
  비신뢰 입력 배포 보류는 로컬 기능 구현 순서 변경과 별개다.
- 이번 변경은 계획·백로그·진행 기록에 한정한다. Rust/WASM·fixture·dependency·원본 증거는
  바꾸지 않으며 코드 변경이 없다는 이유로 같은 빌드·검사를 반복하지 않는다. 문서의 변경을
  과거 `source-final.json` 실행 결과에 포함된 것처럼 표시하지 않는다.
- 현재 단계의 구현·기록을 로컬 checkpoint로 보존한 뒤 다음 단계 문서를 시작한다.
  이 결정만으로 새 단계의 기능 구현이나 원격 push·Ready·merge·close를 수행했다고 쓰지 않는다.
- 문서 4개의 내부 상대 링크·변경분 whitespace와 신규 백로그의 절 anchor를 확인했다.
  사전 기록한 소스·fixture·의존성 2,841개 파일의 해시 변경은 0개다. 실행 기록은
  `output/pr-review/issue7911/implementation-flow-20261016/`의 `doc-links.log`와
  `verification.json`에 보존한다. 소스 빌드·회귀는 재실행하지 않았다.

### 3.24 devel 승격의 공동 구현 목적 정정 (2026-10-16)

메인테이너는 POC의 devel 승격이 완전한 코드의 배포를 위한 것이 아니라, 구현할 빈자리를
코드에 마련하여 기여자들이 참여하고 함께 만드는 방식임을 명확히 했다. 3.23에 이어
백로그 전체 완료를 통합 선행 조건으로 남긴 해석을 정정한다.

- 기반 기능 연결 → 현재 범위 검증·devel 통합 → 기여자와 후속 구현 순서를 계획에 반영했다.
- [구현계획 6.3](../plans/task_m100_7911_impl.md#63-devel-통합과-배포의-판정-분리)에서
  devel 통합과 배포 판정을 분리했다. 현재 주장한 동작·기존 기능·적용 검증은 확인하고,
  명시된 미완성 후속 기능의 존재만으로 기반 통합을 막지 않는다.
- 백로그에는 통합 전 검증/후속 기여/배포 준비를 구분하고 기여 진입 코드·기존 검사·
  공개 재현 자료의 준비 조건을 연결했다. 모두를 메인테이너가 완료해야 하는 목록으로 쓰지 않는다.
- 현재 제품 연결·통합 검증·공개 인계 자료는 여전히 준비 중이다. 이번 정정은 기능 완성이나
  개별 검증 통과·원격 merge 승인·릴리즈 준비 완료 판정이 아니다.
- 계획·백로그·보고만 수정하며 기존 코드·증거를 유지한다. 일반 Markdown 보완에 대해
  변경분과 새 상대 링크를 확인하고 Rust/WASM 빌드·회귀는 반복하지 않는다.

### 3.25 원격 devel 동기화 전 checkpoint (2026-10-16)

메인테이너가 원격 devel 동기화 후 다음 작업 진행을 지시했다. `upstream/devel`을 fetch해
`f5251e7de4`를 확인했다. 로컬 devel은 160개, 현재 작업 HEAD는 공통 조상 기준 원격 측
179개 커밋이 뒤에 있다. 후속 실제 병합·정책 base·검증은 3단계 기록에서 고정한다.

3.14–3.22의 구현과 집중 검증, 3.23–3.24의 계획·백로그 정정을 로컬 checkpoint에 함께
보존한다. 문서 정리 전 기록한 소스·fixture·의존성 2,841개 해시 변경은 0개다.
이 커밋은 미완료 백로그의 해결이나 전체 보안·회귀·시각 검증 완료를 뜻하지 않는다.
기본 작업공간의 수정된 Studio JS는 별도로 보존하고, 새 제품 WASM으로 교체하지 않는다.

## 4. 현재 미완료와 다음 작업

- **우선 구현:** 공통 format detect/`ParsedDocument`/Core·CLI·제품 WASM API는 아직
  PDF에 연결하지 않았다. 다음 시작점은 [F1](../plans/task_m100_7911_impl.md#61-제품-기능을-완결하는-실행-순서)이다.
- **이후 세부 보완:** 폰트·캐시·추가 자원 감사와 JPEG 미관측 경계·vendor lint는 B01–B07,
  최신 보정판 패키지 소비는 B08이다. 이미 확인한 라이브러리/probe 결과와 제품 경로를 구분한다.
- **통합 범위 검증과 후속 분리:** 현재 제출 범위의 출력·회귀·CI는 B06/B09–B12에서
  적용 범위를 확인한다. 나머지 기능·안전성·패키지 보완은 기여 일감으로 이어가며
  상세 분류는 백로그 1절을 따른다.
- 제품 연결·현재 범위 검증이 남아 PR 준비는 아직 미완료다. 전체 백로그 미완료 자체를
  통합 차단 사유로 쓰지 않는다. 기본 비활성·외부 비신뢰 입력 배포 보류는 별도로 유지한다.
  #7913은 승인된 WIP Draft이며 로컬 후속 구현과 원격 checkpoint가 다르다.
- 세부 일감의 현재 상태·완료 증거는 백로그를 정본으로 삼고 이곳에 중복 목록을 누적하지 않는다.
