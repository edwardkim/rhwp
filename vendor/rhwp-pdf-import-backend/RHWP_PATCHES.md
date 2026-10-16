# rhwp PDF Import Backend

이 패키지는 PDFOxide의 공식 release가 아니다. `pdf_oxide 0.3.78`을 기반으로 한
rhwp 내부 보안 패치 dependency이며, 원 저작권·라이선스·NOTICE·상표 정책을 보존한다.
`README.md`와 `upstream/Cargo.toml.orig`는 upstream 배포본의 기록이다.
소스 루트의 `Cargo.toml.orig`도 원문이지만 Cargo는 패키징 시 이 경로를 현재 manifest로
다시 생성하므로, 패키지 안의 원본 확인에는 `upstream/Cargo.toml.orig`를 사용한다.
코드 import 호환성을 위한 lib 이름 `pdf_oxide`와 별개로 package 이름은
`rhwp-pdf-import-backend`, 내부 버전은 `0.3.78-rhwp.1`이다.

## 원본과 추적

- 출처: <https://crates.io/crates/pdf_oxide/0.3.78>
- upstream repository: <https://github.com/yfedoseev/pdf_oxide>
- `.crate` SHA-256: `95b1ecc1a60a314fbd1aea5fe8f700ec32af1efa70fef5c6879162444da3437d`
- 위 hash를 기존 rhwp `Cargo.lock`과 대조한 archive에서 복제했다.
- archive에 없는 `rustfmt.toml`은 같은 upstream commit
  `ad49c4cb3638dc950a29350ef882161372fde473`에서 읽어 보존했다.
  루트 포맷 정책은 변경하지 않았다.
- `UPSTREAM_FILES.json`은 원본 archive의 파일별 SHA-256 목록이다.
  변경·추가 파일과 원문 보존을 이 목록으로 구별한다. upstream 갱신 시 보안 diff를 다시 감사한다.
- MIT 선택지로 사용하며 `LICENSE-MIT`, `LICENSE-APACHE`, `NOTICE`, `TRADEMARKS.md`를 보존한다.
- 외부 배포 승인이 없으므로 `publish = false`다. 별도 게시·배포는 수행하지 않았다.

## 내장 폰트 고지 보완

- 폰트 바이너리 네 개는 upstream 원본 그대로 유지한다. 공개 저장소의
  `assets/fonts/licenses/font-notices.json`에 SHA-256과 각 폰트의 name table 저작권을 등록한다.
- `src/fonts/assets/LICENSE-DejaVu`는 두 DejaVu 폰트에 내장된 Bitstream Vera·Arev 고지
  전문으로 보완한다. Noto Emoji에는 저작권과 OFL 1.1 전문을 `LICENSE-NotoEmoji`로 동봉하고,
  Droid Sans Fallback에는 기존 패키지 루트의 Apache 2.0 전문을 연결한다.
- 원본 archive의 파일 해시 목록은 수정하지 않으며, 이 고지 변경은 원본과의 차이로 남긴다.

## 보안 패치 경계

- `PdfDocument::from_bytes_with_limits`에서 요청별 `ImportBudget`을 생성한다.
  bytes 입력과 xref bootstrap부터 같은 context를 사용한다.
- 파싱된 `StreamData`는 객체/캐시 clone을 지나도 원래 context를 공유한다.
  legacy writer가 생성하는 unbound bytes와 한 문서의 요청 budget을 구별한다.
- stream 단계와 문서 누적량을 append/reserve 전에 검사한다. Flate/Brotli/LZW,
  RunLength/ASCII, 다단 filter와 predictor를 처리하며 제한 초과를 부분 성공으로 만들지 않는다.
  Flate의 제한된 경로는 잘린 prefix 복구를 하지 않는다.
- 객체/배열/사전 깊이·개수, xref `/W`·`/Index`, object stream `/N`을 검사한다.
  제한된 constructor는 손상 xref의 전체 파일 재구성과 암호 PDF를 명시적으로 거부한다.
- content stream은 기존 operator parser를 요청 context로 실행한다.
  process-global cap을 바꾸거나 text prescan의 묵시적 truncation을 안전장치로 사용하지 않는다.
  수치 등 content operand는 컨테이너 객체 quota와 분리하고 operator당 64개로 제한한다.
  operand 내부의 배열/사전에는 동일한 중첩·컬렉션·문자열 제한을 적용한다.
- operator·graphics stack·문자열·표시 텍스트·span 예산을 검사한다.
- 실제 JPEG header/decoder 한계와 PNG writer를 검사하며, 정상 RGB JPEG에는 기존
  image decoder를 유지한다. CMYK 경로의 기존 반전·색 변환 의미를 보존한다.
- typed `ImportResourceLimit`과 sticky fatal 상태를 사용한다. rhwp frontend는 작업 경계에서
  이 상태를 다시 확인하고 최초의 정적 실패 사유를 보존한다. 다음 문서는 새 budget을 사용한다.
- `office_oxide =0.1.9`를 호환성 제약으로 고정한다.
- 제한된 ToUnicode CMap은 전역 cache를 우회하고 요청별 source/scan/확장 예산을 사용한다.
  lazy clone은 raw bytes와 parsed 결과를 Arc로 공유하며 원래 budget을 보존한다.
  비ASCII hex 입력을 byte 경계로 잘라 panic하는 경로를 제거했다.
- CID `/W`·`/W2`의 배열/범위 확장 전에 별도 font metric 예산을 소비하고,
  공개 FontInfo 반환 경계에서 fallback이 fatal 오류를 삼키지 못하도록 검사한다.
- page content 결합의 separator와 cache 반환 복사도 확장 전에 같은 요청 예산을 소비한다.
- 내부 Rust 의존성은 `rlib`만 생성한다. Cargo가 package/source별 hash로 산출물을 구별하게 하여
  공유 target의 비해시 `libpdf_oxide.rlib` 충돌을 피한다. C API·writer 소스는 보존하지만
  이 내부 package에서 독립 cdylib/staticlib 제품을 배포하는 계약은 제공하지 않는다.
- Rust 1.93에서 알려지지 않은 upstream lint 한 항목을 제거하고, 기존 boolean 식 한 곳을
  동치 식으로 정리했다. backend 자체의 `-D warnings` 검사를 위한 호환성 변경이다.
- 후속 미검증 보완: 요청별 페이지 트리 깊이 32, 페이지 노드/객체 로드 방문량 100,000,
  활성 객체 로드 수 32, Form 깊이 16, XObject 방문량 10,000 정책을 추가했다.
  page/object cache hit의 fatal 상태 확인과 tree fallback의 오류 전파를 보완했다.
  제한된 page count는 실제 tree를 순회하며 `/Count`와 flat-tree hint를 신뢰하지 않는다.
  텍스트 Form의 조용한 depth/개수 skip과 예산을 우회하는 span/stream cache 재사용도 막았다.
  수정 전 5개 계약 실패를 재현한 뒤 최초 재빌드는 디스크 부족으로 중단됐다.
  공간 확보 후 Count 기반 subtree skip도 보완하고, span/glyph의 반복 해석 작업량을 반영한
  총 44개 계약과 Native/WASM lib lint를 통과했다. 실물 Native HWPX도 보완 전과 동일했다.
  전체 탐색/cache 감사·제품 공개 API·비신뢰 입력 배포 준비 완료를 뜻하지 않는다.

## 제한과 남은 게이트

### 후속 캐시 보완

- span/character cache hit의 fatal 확인, span 보관·반환과 character 반환의 소유 payload
  복사 전 checked 크기 계상을 추가했다. 한 복사의 요청 payload 32 MiB와 문서 누적 decoded
  256 MiB에 포함하며 allocator overhead나 전체 heap 측정값으로 표시하지 않는다.
- Form stream은 path·image handle·legacy materialized image 소비자 간 Arc로 공유하여
  cache hit/삽입 때 decoded Vec을 복제하지 않는다. 기존 50 MiB 보관 정책은 유지한다.
- inline/XObject image handle의 raw compressed copy와 decode 진입/반환은 같은 요청 budget을
  확인한다. legacy 무제한 API의 함수 서명과 기본 동작은 유지했다.
- 수정 전 신규 4개 실패를 확인했고, 개별 복사 경계와 Form 반복 소비 대조군까지 포함한
  자원 계약 45개 및 기존 frontend/여백 5개가 통과했다. Native 실물 HWPX 3건도 동일했다.
  Native/WASM root·backend library Clippy가 통과했다. 전체 cache/codec/Worker 종료 증거가 아니다.
- 현재 rhwp Native/WASM feature tree에서 이 backend의 활성 feature는 없다.
  JBIG2/JPEG 2000은 unsupported stub이고 qcms/lcms2는 연결되지 않는다.
  추가 feature를 켠 소비자에게 이 감사 범위를 일반화하지 않는다.
- 최신 source의 fresh web WASM 진단 실행에서도 실물 HWPX 3건이 Native와 동일했고,
  비활성 JBIG2/JPX 오류를 포함한 실패 뒤 정상 입력을 처리했다. 제품 공개 API·browser
  Worker·시각 검증이나 자원 계약 45개의 WASM 실행 결과로 표시하지 않는다.

### 남은 검증

후속 CCITT DeviceGray 제한 경로는 기존 Group 3/4 행 해석기를 재사용하면서 전이 Vec의
초기/성장·packed row·grayscale 예약에 요청 예산을 적용한다. 압축/픽셀 길이가 같은
경우에도 압축을 해석한다. 알려진 이미지 높이의 모든 행을 읽어야 성공이며 예산 없는
fax fallback/흰 대체/부분 복구를 호출하지 않는다. 비제한 API의 기존 복구는 보존한다.
DecodeParms filter index와 참조, checked 정수 변환을 적용한다. 제한 경로는
Columns/양수 Rows와 이미지 geometry의 일치를 요구하며 폭 65,535 초과를 지원하지 않는다.
Rows 부재/0은 검증한 이미지 높이를 사용한다. 모든 유효 CCITT PDF의 수용을 뜻하지 않는다.
작은 반례 4개는 수정 전 FAIL, 보완 후 Native 자원 계약 60개/frontend·여백 5개와
Native/WASM의 root/backend library Clippy가 PASS다. 새 release/web WASM의 Node 실행도
CCITT 오류 6개를 거부하고 정상 Group 3/4의 12개 조합 및 1x1 사례를 독립 PNG 디코더로
픽셀 대조했다. Indexed CCITT/색 공간·mask/palette, 제품 공개 API·Worker·시각 검증과
전체 이미지/메모리 안전성 완료로 표시하지 않는다. 현재 아카이브의 독립 소비자 결과는
2단계 기록에서 구분한다.

### WSL 후속: Indexed CCITT와 인라인 이미지

- 제한된 Indexed CCITT는 pass-through된 압축 bytes를 palette index로 해석하지 않는다.
  기존 bounded Group 3/4 decoder로 먼저 풀고, PDF `BlackIs1`의 sample polarity와
  Indexed `/Decode`의 반올림·유효 index 범위를 적용한 뒤 palette를 조회한다.
  같은 Decode 해석 결과로 `decode_folded_in` 상태도 기록해 변환 여부를 보존한다.
- 직접 색상표는 필요한 prefix만 요청 예산으로 복사하며, 참조 stream palette는 같은
  문서의 decode context를 사용한다. 잘못된 hival·짧은 palette를 거부하고 RGB 확장·Lab
  palette 변환의 할당 전 예산을 계상한다. 기본 feature와 비제한 API 계약은 유지한다.
- 인라인 `BI/ID/EI`에서 만든 합성 XObject stream에도 문서의 `ImportBudget`을 보존한다.
  내용 stream에서 추출한 뒤 예산 없는 일반 filter 경로로 우회하던 연결을 고쳤다.
- 수정 전 Indexed 반례 3개와 인라인 stream 상한 반례 1개의 실패를 확인했다.
  정상 Group 3/4·색상표·누적량·filter chain·오류 후 재사용 경계를 작은 입력으로 검증했다.
  Native 자원 67개·frontend/여백 5개와 backend Native/Docker WASM lib Clippy가 통과했다.
  fresh Docker WASM에서도 CCITT 24조합·이미지 상태·개별/인라인 한도·오류 후 정상 입력,
  실물 이미지 9회 및 frontend 3건을 확인했다.
  실행 결과·명령·source 식별자는 `mydocs/working/task_m100_7911_stage2.md` 3.14를 따른다.
  독립 Poppler와의 이미지 픽셀 대조는 페이지 조판·한컴 Print Visual Sweep이 아니다.
- 남은 색 공간·mask·객체/cache 복사·embedded font·PNG private scratch/CPU 감사와
  제품 공개 API·Worker 게이트는 유지한다. 전체 parser allocation 검증 완료로 표시하지 않는다.

### WSL 후속: 샘플·색 변환과 mask 오류 경계

- 제한된 샘플 경로는 Decode range·component LUT·unpack 출력·16-bit reduction을
  예약 전에 같은 요청 codec/누적 예산으로 계상한다. 행 단위 padding과 참조 Decode를
  보존하고 잘못된 bit depth 및 잘린 변환 입력을 오류로 반환한다. legacy 복구는 유지한다.
- CMYK/Gray→RGB의 최종 출력과 JPEG decoded pixel payload에도 같은 예산을 연결했다.
  RGB 소유 버퍼는 불필요한 복제 없이 소비한다. 기존 반전/색 정책은 그대로 대조한다.
  PNG/JPEG private scratch·optional CMM 내부 할당은 이 결과로 검증 완료하지 않는다.
- 제한된 image handle은 독립 그림으로 가져올 때 적용할 수 없는 `/Mask`, `/SMask`,
  stencil을 명시적으로 거부한다. frontend는 `PDF_IMAGE_UNSUPPORTED`를 반환하며
  선택된 뒤쪽 페이지의 오류를 부분 성공으로 반환하지 않는다. null·false의 정상 의미와
  renderer가 mask를 직접 적용하는 legacy 경로는 유지한다. mask 보존 구현이 아니다.
- 수정 전 새 반례 5개가 각각 실패했고 최종 Native 자원 72개, frontend·여백 5개,
  Native/Docker WASM backend·root library Clippy가 통과했다. fresh Docker WASM의
  추가 경계 40개·기존 CCITT/inline 대조군과 실물 이미지 9개·frontend 3건도 통과했다.
  명령·범위·실행 소스와 남은 게이트는 `mydocs/working/task_m100_7911_stage2.md` 3.15를
  따른다. 전쪽 시각 검증이나 전체 parser allocation 검증 완료로 표시하지 않는다.

### WSL 후속: PNG 제한 출력과 JPEG 입력 복사

- 제한 PNG 변환은 `png_limited`의 8-bit Gray/RGB 저장 DEFLATE 블록 writer를 쓴다.
  최대 16 KiB 단일 scratch를 요청 예산으로 선계상·fallible 예약하며 출력 길이·누적량·
  CRC/Adler·최종 블록을 생성한다. 일반 PNG encoder의 압축 전체 Vec을 만들지 않는다.
  비제한 API는 기존 adaptive 압축을 유지한다. 새 dependency는 추가하지 않았다.
- PNG 픽셀은 보존되지만 압축을 하지 않아 크기가 늘어난다. 실물 9회는 합계
  36,323 → 15,252,683 bytes이며 제품 통합 전 보관 비용과 제한 가능한 압축 경로를
  검토해야 한다. ZIP/HWPX의 재압축 효과를 PNG 자체의 비용과 혼동하지 않는다.
- `image` JPEG adapter의 input 복사 payload를 header parse 전에 계상했다.
  JPEG metadata·progressive/MCU/upsampling 및 CMYK worker의 private scratch는
  여전히 미검증이다. 이 보정을 전체 JPEG 할당 제한으로 설명하지 않는다.
- 수정 전 Native 2개/WASM 8개 실패를 확인했다. 보정 후 Native 자원 77개·frontend/여백
  5개, 추가 fresh Docker WASM 경계 41개와 기존 이미지/mask 대조군이 통과했다.
  Native/WASM 실물 이미지 9회는 Poppler와 같고, HWPX 3건은 비-PNG entry와 픽셀을
  보존했다. library Clippy와 source·증거는 2단계 기록 3.16을 따른다.
  공개 제품 API·Worker·전쪽 시각·전체 parser allocation 검증 완료는 아니다.

후속 Brotli 보완은 `brotli-decompressor =6.0.1`의 custom allocator/stream API로
private scratch를 할당 전 계상한다. 별도 개별 할당 32 MiB와 문서 누적 작업 예산을
같이 적용한다. 초기 상태 할당 실패 확인·표준 window·부분 출력 실패 처리를 추가했다.
수정 전 작은 반례 2개는 FAIL이었다. 두 차례 ENOSPC 중단 뒤 승인된 캐시 9개만 정리했고,
현재 source에서 Native 자원 계약 50개/frontend 4개/여백 왕복 1개와
root/backend Native·WASM library Clippy가 통과했다. 빈 입력의 scratch는 의존성 소스에
따라 초기 table과 블록 table 두 개를 모두 포함하도록 검사 가정을 정정했다.
추가 다중 chunk·전체 잘린 prefix·부분 할당 실패 경계도 통과했다.

새 release/web WASM의 Node 실행에서 손상 Brotli 4개를 거부한 뒤 정상 변환을 수행했다.
정상/빈 Brotli 내용은 비압축 내용과 HWPX가 동일했고 실물 12쪽/7·8쪽 변환도 이전 정상
결과와 일치했다. 이는 직접 Rust frontend의 진단 probe다. Native 55개 계약의 WASM 실행,
공개 제품 API·Studio·Worker·시각 검증이나 총 메모리 상한 입증으로 표시하지 않는다.
현재 source의 packaged consumer 검증과 별도 crate 배포 준비 완료는 구분한다.

이 변경은 크기 최적화가 아니며 일반 PDF 쓰기/변환 기능을 삭제하지 않는다.
decoded/copy/구조 작업 예산은 총 RSS/heap 상한이 아니다. CMap source 1 MiB,
요청 CMap scan 작업량 64 MiB/매핑 100,000개, CID metric 확장 100,000개를 적용한다.
128-byte entry 작업 단위와 source 정규화 가중치는 allocator 측정값이나 RSS 증명이 아니다.
모든 codec의 내부 scratch, embedded font/대체 인코딩, 참조/Form 탐색 및
나머지 postprocess/cache의 선행 할당·누적 작업량 감사는 별도 종료 조건이다.
일부 경로만 통과한 결과로 안전성을 보장하거나 비신뢰 입력 배포 준비 완료로 표시하지 않는다.
rhwp import metadata의 `parser_allocation_bounds_proven`는 이 게이트가 끝날 때까지 false다.

루트 workspace는 이 package를 제외하고 optional path dependency로만 소비한다.
workspace patch에만 의존하지 않지만, crates.io 소비자를 위해서는 이 내부 dependency의
별도 배포 모델이 확정되어야 한다. `cargo package --list` 확인만으로 packaged consumer의
빌드나 배포가 검증됐다고 판단하지 않는다.

### PNG 압축 복구와 JPEG 입력 예약 경계

- 제한 PNG의 저장 DEFLATE 블록은 안전한 scratch 경로였으나 실물 9회 합계가 15,252,683
  bytes로 증가했다. `png_limited`를 streaming zlib 압축으로 교체했다. 원본 행을 빌리고
  고정 16 KiB stack 출력 버퍼를 IDAT에 직접 쓴다. 압축 전체 Vec은 만들지 않는다.
- 이미 flate2가 사용하던 `zlib-rs =0.6.7`의 `__internal-api`에 exact 제약을 두고, 동일
  `ImportBudget`을 custom allocator에 연결했다. state/window/hash/pending/symbols와 내부
  alignment 여유 전체를 선계상한 뒤 `try_reserve_exact`로 예약한다. 단일 allocation·해제,
  pointer lifetime과 적용 API는 해당 버전 소스에서 감사했으며 버전 변경 시 재감사한다.
  C allocator/native zlib를 추가하지 않았다. 고정 stack·allocator overhead·전체 RSS는
  요청 payload counter와 구분한다. 일반 Flate decoder의 후속 allocator 보완은 아래 절을 따른다.
- 실물 PNG 9회는 35,939 bytes로 줄고 독립 Poppler와 픽셀이 일치했다. 비제한 API의 기존
  adaptive encoder는 유지한다. 제한 경로는 filter None이므로 모든 입력의 압축률이나
  legacy container byte 일치를 보장하지 않는다.
- `jpeg_input::Input`은 image adapter의 `read_to_end`에서 원본 payload를 선계상한 뒤
  fallible exact reservation·복사를 수행한다. RGB/Gray JPEG pixel 정책은 유지하며 CMYK의
  별도 decoder는 변경하지 않는다. JPEG metadata/progressive/MCU/worker private allocation
  감사 완료가 아니다. 세부 검증과 source hash는 2단계 보고서 3.17에 연결한다.


### 일반 Flate 내부 예약 경계

- 제한 Flate는 `flate_limited`에서 `zlib_limited`의 요청 allocator를 사용한다. PNG도 같은
  allocator/RAII를 공유한다. pinned 0.6.7 init의 단일 state/window 예약을 개별·누적 예산으로
  먼저 계상하고 fallible 예약한다. 4 KiB stack 출력은 기존 bounded writer를 거친다.
- zlib→raw Deflate 호환 경로는 유지하며 실패한 첫 시도의 비용을 되돌리지 않는다.
  resource 오류는 즉시 전파하고, 완전 종료·checksum 확인 전에 부분 성공을 반환하지 않는다.
  비제한 Flate API는 변경하지 않았다. dependency 변경 시 allocator·pointer 경로 재감사가 필요하다.
- 수정 전 codec 한도 1 byte에서 빈 스트림이 성공하는 반례를 확인했다. 최종 Native 84개
  자원 계약, fresh Docker WASM 경계, PNG/HWPX byte 보존은 2단계 보고서 3.18을 따른다.
  JPEG private allocation·나머지 font/cache·Worker·전체 자원 상한은 여전히 미완료다.


## Request-bounded RGB/Gray JPEG internals (#7911, Stage2 slice 3.19)

`extractors/jpeg_limited.rs` installs the request context before JPEG headers and
retains it through decode. The separate local `rhwp-zune-jpeg` 0.5.15-rhwp.1
security patch gates metadata/component/MCU/progressive/upsampling reservations;
see `../rhwp-zune-jpeg/RHWP_PATCHES.md`. Ordinary `image` consumers and the
Adobe-aware CMYK decoder remain unchanged. Output color policy and PNG pixels
are compared against those independent legacy consumers. CMYK internals and
non-codec font/cache allocations remain unproven; the global completion flag
remains false.

## CMYK JPEG 요청 예약 보완 (#7911, 2단계 3.20)

`decode_jpeg_with_budget`의 4-component 분기에만 별도 local `rhwp-jpeg-decoder`
0.3.2-rhwp.1과 `jpeg_cmyk_limited::Request`를 연결했다. context는 생성자 table·header
이전에 설치하며 metadata·Huffman·progressive/MCU/lossless·worker plane·행별 upsampling과
출력의 실제 예약을 계상한다. 외부에서 추정 출력 크기를 미리 계상하던 호출은 실제 decoder
예약으로 대체했고 stream cap·PDF/header 크기 대조·Adobe 반전·색 변환은 유지한다.
호출 후 sticky budget을 우선 확인해 자원 오류를 일반 이미지 오류로 숨기지 않는다.

일반 registry jpeg-decoder와 zune 경로는 유지한다. 자세한 변경·고정 Arc/Box 및 scheduler,
accessor 등 입증 경계는 `../rhwp-jpeg-decoder/RHWP_PATCHES.md`와 단계 보고 3.20을 따른다.
이 보완도 전체 parser 할당 입증이나 공개 API/Worker·전쪽 시각 검증 완료가 아니다.

### 객체·색 공간의 소유 payload 복사 (2026-10-16)

- 제한된 `load_object`의 일반/재확인/recovery cache hit, uncompressed cache 보관,
  object-stream 반환 복사는 동일한 `object_copy` 경계를 사용한다. 문자열·이름·배열 slot·
  사전 key/value slot의 checked 합계를 요청 누적 예산에 먼저 계상하고,
  실제 String/Vec/HashMap 복사는 fallible reserve 이후 수행한다. 메타데이터 복사에
  스트림 출력의 개별 cap을 적용하지 않는다. StreamData는 원래처럼
  bytes와 요청 context를 공유한다. legacy 무제한 경로의 Object clone 의미는 유지한다.
- Form subtype 조회는 객체 전체 복사 대신 빌린 사전에서 판정한다. lazy image enumeration의
  page/Form resources·XObject 사전·색 공간 사전·inline 사전 및 handle 소유 사전도 복사
  경계를 사용한다. 색 공간 읽기 전용 조회는 직접 객체를 빌리고, 실패한 요청이 빈 사전이나
  부분 handle 성공으로 바뀌지 않도록 상위 경계의 sticky 검사를 유지한다.
- 이 계상은 요청한 payload와 collection slot 작업량이다. HashMap bucket 여유·allocator
  overhead·cache index의 HashMap/VecDeque 성장·RSS·폰트 내부 할당 전체의 상한을
  입증하지 않는다. `parser_allocation_bounds_proven=false`를 유지한다.

### WSL 후속: 요청별 폰트 캐시와 TrueType cmap

- 제한된 `PdfDocument`는 전역 FontInfo cache를 읽거나 채우지 않는다. 문서 내부 캐시와
  legacy 문서 간 공유는 유지한다. `load_fonts`는 lazy mapping/fallback 후 sticky 오류를
  다시 확인하며, 직접 resources/font 객체 복사는 기존 bounded copy를 사용한다.
- FontInfo가 원래 요청의 budget을 보관한다. simple TrueType lazy 파싱과 CID descendant
  FontFile2 파싱에 같은 context를 전달한다. cmap/table 길이·offset을 확인한 뒤 format
  0/4/6/12와 symbol mapping의 확장 작업·예약을 선계상하고 fallible reserve한다.
  format 4 scratch도 계상하며 format 12의 역구간·Unicode/GID overflow를 거부한다.
- TrueType map은 immutable Arc로 공유해 clone/donation 시 전체 map을 다시 할당하지
  않는다. 다른 FontInfo 필드의 clone·Type1/CFF encoding·Unicode memo·cache index까지
  제한했다고 주장하지 않는다. 요청 예산은 누적 작업량이며 전체 live heap/RSS가 아니다.
- OpenType `cmap`의 table/subtable 길이를 사용하는 변경에 맞춰 기존 synthetic unit
  builder의 0-byte directory length를 실제 직렬화 길이로 보정했다. 기대 매핑은 유지한다.
  근거: https://learn.microsoft.com/en-us/typography/opentype/spec/cmap
- 재현·검증·남은 범위는 `mydocs/working/task_m100_7911_stage2.md` 3.22를 따른다.

### 편집용 이미지 추출의 사각형 clip 보존 (#7911, F4)

- `page_image_handles`의 page/Form 경로는 CTM과 clip을 함께 추적한다. W/W*는
  path 종료 시 교차하며 q/Q는 graphics state를 복원한다. 현재 path는 별도로 유지한다.
  Form의 Matrix·BBox와 부모 clip도 같은 page 좌표계에서 합성한다.
- handle은 전체 이미지 CTM·payload와 가시 영역을 구분한다. frontend는 가시 영역을
  소유권·프레임·HWP/HWPX crop에 사용하며 원본 bitmap을 임의 축소하거나 합치지 않는다.
  빈 교차는 paint하지 않고, 비사각형 clip은 사각형 bbox로 근사하지 않고 미지원으로 알린다.
- 기존 legacy 이미지 추출 API, 텍스트/vector clipping은 이번 수정 범위 밖이다.
  전체 할당 경계 입증을 의미하지 않으며 `parser_allocation_bounds_proven=false`를 유지한다.
  명세·경계 실행·실물 한컴 출력 근거는 3단계 보고서 7.5를 따른다.
