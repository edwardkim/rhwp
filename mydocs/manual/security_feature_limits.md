---
kind: reference
status: active
canonical: mydocs/manual/security_feature_limits.md
last_verified: 2026-11-01
---

# 보안 제한과 기능 사용 범위

이 문서는 보안 경계를 유지하기 위해 제한한 기능, 자원 한도에 따른 거부, 일반 미지원을
구분한다. [PDF 코덱 opt-in #8183](https://github.com/edwardkim/rhwp/issues/8183)은
[PR #8190](https://github.com/edwardkim/rhwp/pull/8190), 병합 commit
`e7a0dec55c6c6c6365e4ff50babe7583e1cf1c70`으로 `devel`에 반영됐다.
아래는 이 병합과 #8174 v2-S01·S02·S03·S04의 **로컬 source**를 구분해 2026-10-25 현행화한 상태이며,
0.9.0의 [crates.io 준비 #8174](../plans/crates_io_preparation_20261023.md)에서도 참조한다.
로컬 게시 후보 버전은 0.9.0이다. 실제 registry 게시·제품 릴리즈와 전체 통합 검증은 별도이며,
아래 목록만으로 이를 완료한 것으로 해석하지 않는다.
#8415의 PDF 열기 암호 지원은 현재 작업 브랜치의 로컬 구현·검증 상태다.
기존 배포본의 지원 여부와 공통 계획의 원본 127쪽 검증 완료를 의미하지 않는다.
공식 소비 계약과 환경별 지원표는 [crates.io 가이드](crates_io_guide.md)를 따른다.

## 현재 기본값

- **PDF 가져오기는 기본 포함**이다. `pdf-import`를 제외한 빌드만 `PDF_IMPORT_DISABLED`를
  반환한다. 기본 포함 전환은 [PR #7953](https://github.com/edwardkim/rhwp/pull/7953),
  통합 commit `b7a5388034ee66f58a7e1d5363974e83b6374f57`에서 확인한다.
- #8174 v2-S02의 기본 feature는 `pdf-import`·`pdf-jbig2`·`pdf-jpeg2000`·`pdf-icc`를
  포함한다. JBIG2·JPEG 2000·ICC를 쓰려고 별도 feature를 추가할 필요가 없다. backend는
  계속 `default-features = false`로 소비하며 본체가 필요한 기능을 명시적으로 연결한다.
  `pdf-import`만 선택한 개발용 제외 구성에서는 두 코덱과 ICC가 활성화되지 않는다.
- 실제 바이너리·WASM은 `pdfImportSupport`의 `enabled`·`jbig2`·`jpeg2000`·`icc`·
  `boundedXrefRecovery`로 확인한다. CLI·Rust·WASM은 backend의 실제 합산 feature를
  같은 원천에서 조회한다. 로컬 source의 기본값과 기존 배포 바이너리의 기능은 구분한다.
- `parserAllocationBoundsProven`은 계속 `false`다. PDF를 열 수 있다는 사실과
  codec·font·cache의 전체 내부 할당 상한을 입증했다는 주장은 구분한다.
- backend에는 예산을 받지 않는 legacy API도 남아 있다. 아래 PDF 자원 제한은 rhwp가
  사용하는 요청 예산 경로의 계약이며, backend 공개 API 전체의 제한으로 확대하지 않는다.
- 로컬 #8174 v2-S01은 CLI `import-pdf`의 `standard`·`constrained` 프로필과 개별 한도, 별도 작업자의
  실행 감독을 연결한다. 기존 Rust 직접 호출에는 프로세스 감독이 자동 적용되지 않으며
  S03은 같은 프로필·한도를 WASM API·공통 Worker에 연결하고 복사 전 입력 검사를 적용한다.
  S04는 Studio 실제 열기의 정책·Worker·후보 재열기와 실패 시 기존 문서 보존을 연결한다. [PDF 자원 정책](pdf_resource_policies.md)에서 사용법과
  OS별 적용 경계를 구분한다. 로컬 절편 완료와 `devel` 병합·F01–F08 전체·모든 제품의 완료는 구분한다.

근거: [본체 manifest](../../Cargo.toml), [PDF frontend](../../src/parser/pdf/mod.rs),
[backend manifest](../../vendor/rhwp-pdf-import-backend/Cargo.toml),
[보안 패치와 감사 범위](../../vendor/rhwp-pdf-import-backend/RHWP_PATCHES.md).

## #8183으로 해결한 항목

| 이전 제한·과제 | 완료한 처리 | 계속 적용되는 조건 |
| --- | --- | --- |
| JBIG2 그림을 본체 feature로 선택할 수 없었음 | `pdf-jbig2`로 Native·WASM의 실제 decoder를 선택한다. 공유 globals·지원되는 stencil/mask와 저장 후 이미지 보존을 검증했다. | #8174 v2-S02에서 기본 포함으로 전환했다. 기존 배포 산출물은 지원 조회로 확인한다. |
| JPEG 2000 그림을 개별 선택할 수 없었음 | `pdf-jpeg2000`으로 PDF의 `JPXDecode`를 선택한다. JP2/J2K, palette·PDF Indexed, 지원되는 Gray/RGB alpha·mask와 저장 후 보존을 검증했다. | 모든 색 공간·mask 조합의 지원을 뜻하지 않는다. 아래 지원 경계를 확인한다. |
| 새 decoder 내부의 데이터 크기별 예약·반복을 요청 예산에 연결해야 했음 | 실제 decoder의 입력 크기에 비례하는 저장소와 논리 작업량을 동일한 요청 예산에 연결했다. 한도 초과·손상 입력의 거부와 다음 정상 요청 복구를 검증했다. | 기존 한도를 유지하며 전체 parser·프로세스 자원 상한의 증명으로 확대하지 않는다. |
| 산출물별 지원 조회와 선택 빌드 안내가 필요했음 | CLI `capabilities`·가져오기 metadata·Rust API·WASM 사전 조회를 통일하고 Native CLI·Rust library·WASM/Studio 가이드를 보강했다. | Cargo feature 합산을 반영한다. 런타임 설정이나 플러그인 다운로드로 코덱을 켜는 구조는 아니다. |

빌드 명령·지원 조회·오류 대응은 [PDF 이미지 코덱 가이드](pdf_image_codecs.md),
실행 근거는 [#8183 3단계 기록](../working/task_m100_8183_stage3.md)과
[PR #8190 셀프 리뷰](../pr/archives/pr_8190_review.md)를 따른다.
두 4×2 반투명 사례는 [메인테이너 결정](../working/task_m100_8183_alpha_visual_review.md#메인테이너-결정--2026-10-24)에
따라 연속 alpha를 보존한 RHWP 출력을 정답으로 수용했다. 한컴 2024 Print의 1비트 mask 출력
차이는 해당 앱의 인쇄 경로 제한이며, RHWP의 미해결 코덱 결함이나 보안 차단으로 분류하지 않는다.

## 기능별 완료 범위와 남은 경계

| 항목 | #8174 v2의 로컬 처리 | 남은 경계·사용 안내 |
| --- | --- | --- |
| F01·F02 — JBIG2·JPEG 2000 | 기본 포함으로 전환했다. 기존 코덱 예산을 유지하고 PDF Indexed+내장 mask·CMYK+alpha의 실제 조합을 확대했다. | 모든 JP2 channel 정의·bit depth·subsampling 조합을 보장하지 않는다. 명시적인 개발용 제외 빌드와 실제 산출물 지원 조회를 구분한다. |
| F03 — ICC CMM | 기본 `pdf-icc`로 순수 Rust qcms의 Gray·RGB·CMYK 변환을 이미지·JPEG·Indexed palette·path에 적용하고 alpha를 보존한다. profile·compiled transform을 요청 안에서 cache하며 데이터 크기별 예약·작업을 계상한다. | 잘못되거나 CMM이 지원하지 않는 profile은 대체 색 변환을 사용한다. 예산 초과는 성공으로 숨기지 않는다. 고정 내부 할당을 포함한 전체 heap 증명은 아니다. |
| F04 — 손상 xref 복구 | 요청 예산 안에서 실제 객체를 문법적으로 scan해 복구한다. 문자열·주석·stream의 가짜 header를 제외하고 최신 incremental 객체를 소비한다. 복구 뒤 실제 object load도 stream Length를 확인한다. | 모든 손상을 복구하지 않는다. 모호한 stream 경계·한도 초과·암호화 보호를 무시해 성공시키지 않으며 무제한 legacy 복구를 호출하지 않는다. |
| F05 — SVG 글리프 | 공통 정규화·Native replay가 gradient·clip·opacity·내부 참조·내장 PNG/JPEG를 소비한다. S03의 WASM `prepareStaticSvgRaster`도 같은 공통 결과를 출력한다. OpenType에서 무시하도록 정한 script 등의 subtree를 제외하고 정상 artwork를 보존한다. | 외부 파일·네트워크 resolver와 실행 코드는 허용하지 않는다. XML·노드·깊이·참조 확장·내장 이미지 예산을 적용한다. 기존 proof·glyph mapping·geometry 계약은 유지한다. S04 CanvasKit도 같은 raster 결과를 소비한다. 입력 proof·자원 키·geometry와 대체 표시의 계약은 유지한다. |
| F06 — 링크 목적지 | 공통 생성·편집·조회가 웹·메일·내부 책갈피·상대/로컬 파일·등록 앱을 분류하고 원문을 보존한다. Native PDF에 URI·GoTo·파일 action과 실제 책갈피 위치를 연결한다. | 문서를 여는 동안 링크를 실행하지 않는다. JavaScript·data·제어문자·사용자 정보가 있는 웹 URL을 거부한다. S04는 실제 글자 영역의 클릭·책갈피 이동·host 위임과 브라우저의 파일/앱 안내를 연결한다. OS 앱 실행·메일 전송 성공은 host의 책임이다. |
| F07 — 양식 명령 단추 | 보존한 Click을 제한된 Worker runtime에서 계획으로 만들고 실제 양식 값·본문 편집·기존 저장/인쇄 dispatcher에 연결한다. HWP/HWPX 저장 시 원본 script와 결과를 보존한다. | 전체 JScript·임의 HAction·shell/DOM/네트워크는 지원하지 않는다. 시간·token·단계·문자열·명령 수를 제한한다. 읽기 전용 host는 단추 권한을 차단할 수 있다. [지원 문법·사용법](studio_document_features.md#양식-명령-단추)을 확인한다. |
| F08 — 보호 문서 복구 | UI의 암호 열기·저장에 세션 전용 키를 연결해 인증 암호화된 복구본을 만든다. 재시작 때 암호를 다시 받고 보호 저장 의도를 유지한다. | 키가 없으면 평문 exporter 이전 차단을 유지한다. 비밀번호·키를 영구 저장하지 않는다. 오입력·변조·삭제·문서 교체·저장 경합은 후보를 게시하지 않는다. [암호화·키 수명·host 경계](studio_document_features.md#보호-문서-자동-복구)를 확인한다. |

기본 기능도 요청 한도를 우회하지 않는다. decoder·CMM의 입력에 비례하는 저장소와 작업,
xref 복구 scan을 요청에 계상하지만 전체 parser·font·cache의 할당이나 process RSS를
증명하지 않는다. CLI 작업자의 wall timeout·Linux 주소 공간/CPU 제한은 별도의 실행 경계다.

구현 근거:

- PDF codec: [`decode_jbig2_image`·`decode_jpx_image`](../../vendor/rhwp-pdf-import-backend/src/extractors/images.rs).
  `UnsupportedFilter`는 frontend에서 `PDF_PARSE_FAILED`와 backend 진단으로 연결된다.
  일부 다른 mask/codec의 `Unsupported`는 `PDF_IMAGE_UNSUPPORTED`로 연결되므로 오류 코드를
  하나로 단정하지 않는다.
- PDF 색 변환: [CMM과 fallback 계약](../../vendor/rhwp-pdf-import-backend/src/color/mod.rs).
- PDF 복구: [예산 있는 constructor와 xref 검사](../../vendor/rhwp-pdf-import-backend/src/document.rs).
- SVG 글리프: [공통 정적 판정](../../src/renderer/static_svg.rs),
  [Native replay](../../src/renderer/skia/glyph_replay.rs),
  [CanvasKit 후보 판정](../../src/renderer/layer_renderer.rs),
  [Studio 정적 SVG parser](../../rhwp-studio/src/view/static-svg-path-layers.ts).
- 링크: [공통 생성](../../src/model/hyperlink.rs), [출력 URI](../../src/renderer/hyperlinks.rs),
  [Studio 클릭](../../rhwp-studio/src/engine/input-handler-hyperlink.ts).
  scheme 허용은 DNS·사설 주소·접속 대상 전체의 안전성 검사와 다르다.
- 양식 단추: [`handleFormObjectClick`](../../rhwp-studio/src/engine/input-handler.ts).
- 복구본: [자동 저장 관리자](../../rhwp-studio/src/recovery/autosave-manager.ts),
  [보호 상태 연결](../../rhwp-studio/src/main.ts).

## 자원 한도로 거부하는 입력

일반 문서 기능을 끈 것이 아니라, 요청의 작업량·압축 해제량·깊이를 제한한다.
한도는 프로세스 전체 heap이나 실행 시간의 상한과 같지 않다.

| 입력/작업 | 현재 기본 제한과 관측 가능한 결과 | 근거 |
| --- | --- | --- |
| PDF 입력·쪽 선택 | 입력 64 MiB, 선택한 원본 쪽 최대 1,024개. 표준 프로필은 파싱 객체 값 200,000개, stream/codec 단일 예약 256 MiB, 그림 하나 64,000,000픽셀·배치 누적 2,000,000,000픽셀 등 독립 한도도 적용한다. 자원 한도 초과는 `PDF_RESOURCE_LIMIT`이다. 작은 파일도 디코딩 작업량 때문에 거부될 수 있다. | [frontend](../../src/parser/pdf/mod.rs), [그림 추출](../../src/parser/pdf/extract.rs), [제품 요청 정책](../../src/parser/pdf/policy.rs), [backend 한도](../../vendor/rhwp-pdf-import-backend/src/import_limits.rs) |
| PDF 코덱·ICC·복구의 예약·작업량 | 표준 256 MiB·절약 32 MiB의 단일 예약과 요청의 누적 decoded 예산을 함께 적용한다. 표준 누적 decoded 예산은 8 GiB, 절약은 256 MiB이다. 누적량은 해제된 버퍼의 할당·복사도 포함하며 동시 점유 메모리가 아니다. `u64`로 WASM32에서도 4 GiB를 넘는 누적량을 계상한다. 논리 작업량은 표준 `codecWorkUnits = 2,000,000,000`, 절약 `1,000,000,000`이며 `resourcePolicy.backend.codecWorkUnits`·`resourceUsage.codecWorkUnits`로 확인한다. 밀리초나 CPU 명령 수가 아니다. | [요청 연결](../../vendor/rhwp-pdf-import-backend/src/codec_resource.rs), [할당·작업 계상](../../vendor/rhwp-pdf-import-backend/src/import_limits.rs) |
| PDF operator·참조·글꼴·decoded 누적량 | 선택 쪽수와 명시한 host 정책에 따라 유한한 요청 예산을 적용한다. backend의 단독 기본값과 본체가 덮어쓴 값은 다르다. `resourcePolicy`·`resourceUsage`에서 해당 요청의 정책과 사용량을 확인한다. | [제품 요청 정책](../../src/parser/pdf/policy.rs), [metadata](../../src/parser/pdf/mod.rs) |
| CLI PDF 작업자 수명·출력 | `standard`는 wall 120초·최종 출력 128 MiB·진단 32 MiB, `constrained`는 30초·32 MiB·8 MiB다. Linux 작업자에는 각각 주소 공간 4/1 GiB와 CPU 90/20초 제한을 적용한다. 초과·실패 시 작업자를 종료하고 기존 출력을 유지한다. RSS·부모·프로세스 트리 전체의 메모리 상한과 구분한다. | [자원 정책과 OS별 적용 범위](pdf_resource_policies.md), [Native 감독](../../src/resource_supervisor.rs), [CLI](../../src/cli/commands/pdf_import.rs) |
| Studio PDF·단추·보호 복구 | PDF UI 기본 deadline은 30초다. 단추 Worker 기본 1초·실행 20,000단계이며 복구 평문은 최대 64 MiB다. 각 경계의 실패는 부분 문서로 성공시키지 않는다. 전체 process RSS 상한이 아니다. | [Studio 가이드](studio_document_features.md) |
| HWP5 압축 stream | 문서 열기의 기본 정책에서 stream 출력 256 MiB·누적 출력 512 MiB를 적용한다. 파일 크기·암호화·BinData 등 다른 경계의 한도와 혼동하지 않는다. | [문서 열기 정책](../../src/parser/mod.rs) |
| HWPX ZIP/XML | XML 엔트리당 256 MiB, reader의 XML 누적 출력 512 MiB, BinData 엔트리당 512 MiB. 초과 시 읽기를 거부한다. | [HWPX reader](../../src/parser/hwpx/reader.rs) |
| HWPX 재귀 구조 | section 파싱 깊이 16, container 깊이 64, listParam 깊이 32 등 서로 다른 재귀 경계가 있다. 과도한 중첩은 오류가 된다. | [HWPX section parser](../../src/parser/hwpx/section.rs) |

PDF의 쪽 선택은 입력 파일 전체를 읽고 컨테이너·공유 자원을 검사하는 비용까지 없애지 않는다.
한도 오류를 정상적인 빈 출력이나 앞부분만 성공한 결과로 처리하지 않는다.
그림의 선언 픽셀 한도 외에 실제 디코딩·색 변환·mask·PNG 저장의 단일/누적 예약도 적용한다.
표준 한도를 늘려도 모든 코덱·채널 조합이 픽셀 상한까지 성공한다는 보장은 없다.
절약 프로필의 기존 한도는 유지한다. 기본 코덱 제공이 자동 축소나 한도 해제를 뜻하지 않는다.
공개 CLI 동작과 쪽 선택 방법은 [CLI 매뉴얼](cli_commands.md#pdf-가져오기-기본-포함)을 따른다.

## 기본 기능 확대 이후에도 남은 검증 과제

2026-10-24 메인테이너는 [MD05-B](../plans/task_m100_8174_maintainer_decisions.md#md05--pdf-입력의-신뢰-범위와-보안-조건)로
비신뢰 PDF 업로드도 공식 지원하는 것을 목표로 감사·격리 검증을 확대하기로 결정했다.
이어 MD06의 **F01–F08 전체 기본 탑재**, MD07의 **CLI → WASM → Studio 전체 적용**을
결정했다. [범위 v2의 구현 계획](../plans/task_m100_8174_impl.md#범위-v2--기본-기능과-환경별-자원-정책)에
기본 코덱·ICC·xref 복구·SVG 글리프·링크·명령 단추·보호 문서 복구와 환경별 정책을 연결했다.
CLI·공통 자원 정책은 [v2-S01](../working/task_m100_8174_v2_stage1.md), 기본 기능·Native는
[v2-S02](../working/task_m100_8174_v2_stage2.md)에 실제 구현·검증 범위를 기록한다.
[v2-S03](../working/task_m100_8174_v2_stage3.md)은 WASM API·공통 Worker의 실행 범위를 기록한다.
[v2-S04](../working/task_m100_8174_v2_stage4.md)는 Studio 소비 경로와 실제 브라우저 검증을 기록한다.
보호 복구의 제공은 일반 PDF 업로드의 전체 보안 감사 완료와 구분한다.
이 결정은 아래 미완료 경계의 구현·검증을 추가하는 방향이며, 현재 제품의 보안 보증 완료나
제한 해제를 뜻하지 않는다.

- **전체 할당 상한:** 새 코덱의 요청 예산 연결은 완료했지만 전체 parser·font·cache의 내부 할당과
  process RSS 상한은 입증하지 않았다. CLI의 별도 실행 감독을 이 증명으로 대체하지 않는다.
  `parserAllocationBoundsProven=false`와
  [#7911 B01–B05·B07](../plans/task_m100_7911_backlog.md)의 후속 검증을 유지한다.
  [v2-S06 조사](../working/task_m100_8174_v2_stage6.md)에서는 `fontEntries: 1`에서 Type1/CFF
  map 확장이 성공하는 적용 누락과 Unicode memo의 무계상 복사를 재현했다.
  [v2-S07](../working/task_m100_8174_v2_stage7.md)에서 해당 Type1/CFF·glyph name·Unicode
  memo의 생성·복사 전 계상과 sticky 오류를 보정해 Native/WASM 11개 계약 및 실제
  CLI·Worker·Studio의 거절·복구를 확인했다. 이후 [T03](../working/task_m100_8174_t03_font_fidelity.md)의
  폰트 출력, [T04](../working/task_m100_8174_t04_completion.md)의 초기 PDF/첫 저장 할당 경계와
  [T05](../working/task_m100_8174_t05_completion.md)의 입력·응답·편집 재열기/교체 경계를 보강했다.
  이는 전체 parser·opaque scratch·프로세스 heap/RSS 상한 증명과 구분한다.
  JPEG H resize의 실제 증가 예약·거절/복구 증거를 유지하며,
  [T06](../working/task_m100_8174_t06_completion.md)에서 mixed progressive 6건과 같은 원인의
  baseline 4건을 공통 sampling 기하에서 보정했다. 해당 PDF는 기본 가져오기 경로로 처리하며
  별도 활성화 옵션이 필요하지 않다. 독립 decoder와의 보간/IDCT 색상 차이와 실제 픽셀·alpha·
  저장 보존을 구분해 검증한다. T07·T08에서 실제 포장과 독립 소비까지 검증했으며, 최신 devel 통합 후보는 T09에서 같은 경계를 다시 확인한다. 실제 게시와 구분한다.
- **Native 실행 경계:** Linux/WSL2에서 작업자의 AS/CPU/wall 제한·종료와 실패 뒤 정상 요청을
  검증했다. 관측 RSS는 보증 상한이 아니다. Linux x86_64/aarch64의 `import-pdf`에는
  Landlock·seccomp 제한을 추가하며 성공한 작업자에서만 `filesystemNetworkSandbox=true`를
  보고한다. Linux에서 필요한 기능을 사용할 수 없으면 조용히 제한을 해제하지 않고 실패한다.
  [파일·네트워크 제한과 Rust host 예제](pdf_resource_policies.md#linux-pdf-작업자의-파일네트워크-제한)를
  따르며 다른 OS·부모·동시 요청의 보호 완료로 확대하지 않는다.
  `info`·`export-svg`·`convert` 등 PDF를 공통 로더로 여는 다른 CLI 명령은 현재 프로세스에서
  실행하므로 `import-pdf` 작업자의 감독 범위에 포함하지 않는다.
- **동기 호출의 중단:** 직접 `HwpDocument.importPdf()`는 동기 호출이다. 로컬 S04 Studio는
  공통 Worker의 취소·deadline·종료 후 재생성을 실제 열기에 연결한다. 성공한 편집 문서의
  main thread 재열기는 동기식이어서 같은 스레드의 timer로 선점하지 못한다. T05에서 입력/응답
  사본의 사전 예약, 실제 남은 credit으로 편집 HWPX 재열기, 예약/읽기 경계의 협력 시간 검사와
  이벤트 루프 양보 후 취소·deadline·generation 재검사를 연결했다. 후보 실패 시 기존 문서와
  저장 상태를 보존한다. XML 객체·opaque scratch·전체 heap/RSS 상한 증명과는 구분하며
  `parserAllocationBoundsProven=false`를 유지한다.
  [Studio 사용 계약](studio_document_features.md)을 따른다.
- **registry 배포 경계:** [v2-S05](../working/task_m100_8174_v2_stage5.md)에서 코덱을 포함한
  14개 실제 `.crate`와 로컬 검증 registry의 독립 Rust·CLI·WASM 소비를 확인했다.
  T07·T08에서 보정한 14개 archive의 strict gzip 무결성·공개 계약·독립 Rust/CLI/WASM 소비를
  다시 검증했다. 최신 devel 통합 후보도 [T09](../working/task_m100_8174_t09_completion.md)에서 고유 14개 actual archive의 strict gzip·독립 Rust/CLI·공개 계약 54개·WASM feature on/off 소비까지 통과했다.
  MD01–MD09는 결정 대장에 확정했으며 crates.io 게시 일정은 0.9.0과 별도로 정한다.
  source preflight의 `publish_ready=false`는 로컬 준비 검증과 실제 registry 게시·이름/소유권·인증을
  구분하는 표시다. 토큰 준비만으로 게시하거나 게시 완료를 선언하지 않는다.

## 보안 차단과 구분할 사항

| 항목 | 현재 분류 |
| --- | --- |
| 암호화 PDF 가져오기 | #8415에서 Standard Security R2–R6의 빈 암호·사용자·소유자 인증을 요청 예산 안에 연결했다. CLI 입력 암호와 Studio 재입력·취소를 지원하며 인증 후 복사·편집 플래그로 추가 차단하지 않는다. 공개키 방식과 혼합/Identity crypt filter 조합 등 미지원은 `PDF_ENCRYPTION_UNSUPPORTED`, 암호 누락/오입력은 `PDF_PASSWORD_REQUIRED`/`PDF_PASSWORD_INVALID`다. [입력 방법](studio_document_features.md#pdf-열기-암호). |
| Fasoo·SoftCamp 등 DRM 컨테이너 | 보호 컨테이너 감지는 하지만 복호화는 미지원이다. `DRM_PROTECTED` 안내에 따라 권한 있는 원본 프로그램에서 처리한다. 일반 배포용 HWP나 비밀번호 HWP 전체가 미지원이라는 뜻은 아니다. |
| PDF의 일부 mask·clip·paint, 원 저작 문단 복원 | #8413–#8416에서 Bézier 곡선·nonzero/even-odd 복합 clip·Form/아핀 이미지, 직교 축의 회전 텍스트와 서로 다른 축 배율, Flate predictor·Gray JPEG SMask를 지원한다. 곡선은 실제 alpha로 보존하고 변환한 이미지의 PDF 색을 HWP/HWPX 재열기까지 유지한다. 임의 shear/반사 텍스트, 모든 mask·색 공간 조합과 원 저작 문단 복원까지 지원한다는 뜻은 아니다. 표현·복원 지원의 경계이며 보안 때문에 전부 꺼진 기능으로 안내하지 않는다. [구현·검증 기록](../working/task_m100_8413_implementation.md). |
| 확대 후에도 남은 JPX 조합 | PDF Indexed+내장 mask·CMYK+alpha는 S02에서 지원을 확대했다. JP2의 모든 cdef 유형, 임의 bit depth·subsampling을 보장하지 않는다. 원 Lab 성분 범위를 잃은 Indexed Lab+preblended alpha는 명시적으로 거부하며 straight alpha와 구분한다. 기존 chroma 복원은 8-bit의 명확한 2×2(4:2:0) 조합으로 제한한다. 실제 지원 조합과 모든 JPEG 2000 표현 지원을 구분한다. |
| 스캔 문자의 OCR | 코덱을 디코딩해 스캔 그림을 보존하는 것과 검색·편집 가능한 텍스트를 인식하는 것은 별개다. 기본 코덱 제공으로 OCR이 추가되지는 않는다. |
| `wasm-api`·`native-skia` 미선택 | JS facade/자동 초기화와 Native 렌더러의 빌드 선택이다. 보안 결함 때문에 금지한 기능이라는 근거가 아니다. |
| 내부 crate의 `publish = false` | 외부 게시 승인·패키징 준비의 경계다. 실행 중인 제품 기능을 비활성화하는 설정이 아니다. |
| `inspect`·`threat-scan`·`armor` | 조사·출처 표시 기능이다. 문서를 자동 무해화하거나 모든 악성 입력을 차단하지 않는다. `threat-scan`은 신호가 있어도 exit 0일 수 있으므로 JSON 판정을 소비해야 한다. |

보호 형식은 [CLI 암호·DRM 지원 표](cli_commands.md),
검사 명령은 [에이전트 보안 매뉴얼](agent_codex/60_보안.md)을 참조한다.
문서 유래 문자열을 도구 실행 지시로 승격하지 않는 처리는 소비 에이전트의 책임도 포함한다.

## 0.9.0 배포 준비에 남기는 조건

지원 입력·제한 기능·자원 정책의 선택은 [메인테이너 결정 대장 MD05–MD07](../plans/task_m100_8174_maintainer_decisions.md)에
기록한다. 이 현황 문서의 제한 목록을 기능 해제 또는 게시 승인으로 해석하지 않는다.

1. source용 [코덱 빌드·사용 가이드](pdf_image_codecs.md)와 게시 후보 README·feature 표는
   S02의 기본 코덱·ICC·복구, 명시적 제외 빌드와 잔여 한도를 설명한다.
   최종 포장·registry 소비는 S05에서 다시 검증한다. backend의 upstream 기능 목록을
   검증 없이 rhwp의 제공 기능으로 복사하지 않는다.
2. #8183에서 Native·fresh WASM 네 코덱 구성과 source 설치 CLI·외부 Rust 소비 앱을 검증했다.
   registry 배포 후보에서는 기본 코덱·ICC·예산 있는 손상 xref 복구의 성공, 명시적 codec 제외
   빌드·암호 PDF·복구 불가 입력·자원 초과의 실패와 다음 정상 요청을 실제 실행한다.
   source 검증 결과만으로 registry 패키지의 경계 보존을 완료 처리하지 않는다.
   Native/WASM과 기본/제외 feature 조합을 구분해 기록한다.
3. [#7911 B01–B05·B07](../plans/task_m100_7911_backlog.md)의 font·cache·할당·제품 경계 검증을
   별도로 추적한다. 목록에는 미검증 경계도 있으므로 모두 재현된 취약점으로 부르지 않는다.
   반대로 package 생성 성공으로 이 검증이 끝났다고 판정하지 않는다.
4. F01–F08의 실제 기본 제공과 CLI → WASM → Studio의 자원 정책, MD05-B 지원 범위의
   보안 게이트를 완료한다. 메인테이너가 선택한 구현 목표와 실제 배포 준비 완료는 구분하고,
   남은 필수 구현·검증을 문서의 지원 표기로 대신하지 않는다.

## 확인 범위

기존 보안 경계의 최초 조사 source는 `e5086b4bab13b8267ee5b9d0104afd3c24a9a160`이다.
당시 코드·manifest와 기본 Native/WASM feature tree를 대조했고, 기존 Studio 검사인
`autosave-manager`, `hyperlink-target`, `static-svg-path-layers`는 합계 22개 통과했다.
이는 복구본·웹 주소·정적 SVG helper의 실행 근거이며 실제 브라우저 클릭, PDF codec 전수 실행,
전체 보안 감사나 저장소 밖 패키지 소비 검증의 결과로 확대하지 않는다.
명령·exit·로그는 [#8174 1단계 보완 기록](../working/task_m100_8174_stage1.md)에 연결한다.

PDF 코덱은 #8183의 제품 source `cfc5be8fe`·정식 테스트 source `2a58c5f37` 및 병합 commit
`e7a0dec55c6c6c6365e4ff50babe7583e1cf1c70`의 [실행 결과](../working/task_m100_8183_stage3.md)를 반영했다.
Native·최적화한 fresh WASM 네 구성 각각 19건, opt-in 정식 회귀 7건, 실제 Studio·Worker의
실패·복구와 source 설치 CLI를 검증했다. 원본 PDF 기준 17쪽의 Native/fresh WASM raw 최저값은
각각 100%이며, 저장본 한컴 Print의 두 alpha 미달은 위 메인테이너 결정과 함께 원래 수치를 보존한다.
합성 입력의 계약과 실제 앱·독립 PDF의 검증 범위를 구분하며 전체 보안 감사로 확대하지 않는다.

이번 현행화에서는 `3536e72819e165dbe959ee42cfd4e4cf8db33ce7`의 관련 코드·manifest와 위 실행
기록을 대조했다. 코덱 기능은 병합 뒤 변경되지 않았으며, 기존 실행 증거를 재사용했다.

#8174 v2-S02의 변경 경계와 새 실행 source·증적은 [S02 기록](../working/task_m100_8174_v2_stage2.md)을
따른다. 위 #8183 과거 실행을 S02의 새 head 검증으로 바꿔 표시하지 않는다.

## #8174 최종 로컬 검증

2026-10-29 [완료 보고](../report/task_m100_8174_report.md)에서 기본 기능·CLI/Native 격리·WASM/Studio 수명/오류/복구·최적화 제품·실제 포장 소비와 지원 경계를 대조했다. 이 작업의 필수 로컬 미완료 항목은 없으며 승인된 PR·CI·self-review·merge를 진행한다. 전체 parser/opaque scratch·프로세스 heap/RSS 보증, 다른 OS의 host 격리와 실제 registry 게시·owner 확보는 위 계약대로 구분한다.
