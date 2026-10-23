# #8183 2단계 — 코덱 분리와 요청 자원 계약

Issue: [#8183](https://github.com/edwardkim/rhwp/issues/8183) ·
[수행 계획](../plans/task_m100_8183.md) · [구현 계획](../plans/task_m100_8183_impl.md)

상태: **구현·focused 검증 완료. 고정 source의 package 선별 확인을 마무리한다.
PR 제출·배포 단계가 아니다.**

작업지시자가 「계획과 2단계 구현 승인」으로 승인했다. 기준 base는
`d9fd85ed5bc3c4d9ce6b2981422e48bf9bb90867`, 선행 조사 커밋은
`2c304a1367703e142eeac07e780d8480b53fdc14`다. 작업 브랜치는
`feat/issue8183-pdf-codec-opt-in-20261023`이다. 새 원격 변경은 수행하지 않는다.

## 구현한 계약

| 공개 선택 | 활성 코덱 | backend feature |
| --- | --- | --- |
| 기존 기본 빌드 | 둘 다 비활성 | 기존 PDF 입력 유지 |
| `pdf-jbig2` | JBIG2와 내부 MMR용 CCITT | `jbig2` |
| `pdf-jpeg2000` | JPEG 2000 | `jpeg2000` |
| 두 feature | 두 코덱 | `jbig2`, `jpeg2000` |

네 구성의 metadata와 실제 Cargo tree를 대조했다. 한 코덱이 다른 코덱을 켜지 않으며,
전체 backend `rendering`·글꼴 검색은 선택하지 않는다. 기존 `rendering`은 `jbig2`를
포함하여 backend의 feature 관계를 보존한다. 기본 feature와 기존 byte/pixel 한도는 유지한다.

기존 hayro API는 RHWP 요청 예산을 받지 않았다. 공식 archive에서 내부 보정판
`rhwp-hayro-jbig2 0.3.0-rhwp.1`, `rhwp-hayro-jpeg2000 0.4.0-rhwp.1`,
`rhwp-hayro-ccitt 0.3.0-rhwp.1`을 구성했다. JBIG2 MMR이 CCITT의 changing-line Vec를
확장하므로 외부 JBIG2 bitmap만 제한하는 것으로 대신하지 않았다.
공식 archive SHA-256과 원본 32·40·11개 파일의 해시를 재계산해
각 `UPSTREAM_FILES.json`과 대조했다. 원저작자·라이선스·ICC 자료 고지를 유지한다.
version+path 의존성을 사용하며 root에만 적용되는 `[patch]`로 우회하지 않는다.

`rhwp-codec-resource`는 동기 호출의 thread-local scope로 원 요청을 전달한다.
guard는 다른 thread로 이동할 수 없고, 중첩 scope가 끝나면 이전 요청을 복원한다.
codec Vec의 데이터 크기 예약·복사·사전·tile/component·wavelet scratch는 checked 크기를
계산하고 실제 확장 전에 기존 `codec_allocation_bytes`·누적 decoded 예산을 소비한다.
실패 가능한 Vec 예약을 사용한다. 정렬은 명시적으로 예약한 index 저장소를 사용하며
같은 key의 원래 순서도 유지한다.

새 `codec_work_units`는 기본 **1,000,000,000**의 요청 누적 논리 작업량이다.
byte/bit 읽기·entropy/packet 반복은 실행 중 검사하고, 유한한 pixel 변환은 먼저 계상한다.
시간·CPU 명령 수가 아니며 decoder 수정 사이에 같은 값이 유지되는 ABI도 아니다.
한도 초과는 원 요청의 첫 sticky 실패를 보존하고 이후 EOF 등으로 덮어쓰지 않는다.
기존 operator·stream 예산을 재사용해 별도의 codec 작업량을 숨기거나 초기화하지 않는다.

## 실제 소비 경로

`src/parser/pdf/extract.rs`의 그림 선택 → `PdfImageHandle::decode`의 mask 읽기 →
`extract_image_from_xobject_at`의 codec 선택 → 요청 scope의 parser·예약·디코딩 →
실제 크기 대조 → mask/opacity/clip → `to_png_bytes_with_budget` → IR 그림 소유 경로다.
scope가 끝난 뒤의 PNG·RGBA 소비에도 기존 요청이 전달된다.

- 그림 목록과 materialization은 간접 참조와 배열 원소까지 해석하는 공통 Filter 결과를
  소비한다. 간접 Filter를 누락하던 반례의 수정 전 실패를 확인했다.
- JBIG2는 실제 header와 PDF 선언 크기를 대조한 뒤 collector를 한 번 예약한다.
  callback에서 Vec를 확장하지 않고 행·픽셀 수를 확인한다. 간접 DecodeParms와 filter 위치의
  JBIG2Globals를 해석하며, 명시한 공유 stream의 실패를 없는 것으로 처리하지 않는다.
- JPX는 실제 크기·성분 수를 pixel plane 예약 전에 검사한다. bounded 제품 경로는 기존
  원본 해상도·선언 크기 계약을 유지한다. legacy 무예산 경로의 축소 hint와 구분한다.
  source Indexed index, JP2 palette, 색 성분과 8-bit 저장 결과를 구분한다.
- `SMaskInData=0`만 opacity를 무시한다. 1/2의 Gray/RGB alpha는 RGBA로 보존하고
  premultiplied 색을 복원한다. 지원하지 않는 alpha 조합과 외부 SMask의 모순은 오류로 반환한다.
- codec source-mask adapter는 decoded sample을 packed 행으로 변환한다. 압축 byte를 raw
  mask로 읽지 않는다. 지원하는 sample depth가 다르거나 성분·행 길이가 맞지 않으면 거절한다.
  기존 Decode·Matte·보간·소유·최종 CTM 경로를 유지한다.

## focused 검증과 독립 근거

모든 Cargo 실행은 `CARGO_TARGET_DIR=/home/edward/mygithub/rhwp/target/pr-review`,
`CARGO_BUILD_JOBS=8`로 순차 수행한다. 실행 로그·입력·진단 harness는 ignored
`output/pr-review/issue8183/stage2/`에 있다. 최종 명령·종료 코드는
`final-validation.json`에 기록했다. 별도 fork lock도 보존하여 standalone 검사는
`--locked`로 재현한다.
최종 본체·manifest·vendor 등 107개 변경 파일의 SHA-256 목록은
`validated-source-files.json`이며 목록 자체의 SHA-256은
`bdf3d136d1f13fd93bda3e984cf54f0b0a6f196f779f8c03288c6761832ea388`다.
단계 계획·결과 기록은 이 source 목록에서 제외했다.
마지막 staging 검사에서 원본 README와 rustfmt-skip 표의 줄 끝 공백을 제거했다.
Rust 파일은 수정 전후 전체 leaf token이 같음을 parser로 확인했고
`whitespace-token-proof.json`에 해시를 기록했다. 이 변경에 실행 로직·literal 변경은 없다.

| 검사 | 실행·결과 |
| --- | --- |
| 네 feature의 실제 활성 graph | metadata + Cargo tree, partner codec·전체 rendering 미활성 확인 |
| 기본 Native library | `cargo check --locked -p rhwp --lib`: 통과 |
| 선택 Native library lint | `cargo clippy --locked -p rhwp --lib --no-default-features --features pdf-jbig2,pdf-jpeg2000 -- -D warnings`: 통과 |
| 선택 WASM32 library lint | 위 명령에 `--target wasm32-unknown-unknown`: 통과 |
| 독립 fork·backend lint | codec/resource 네 패키지와 backend의 선택 feature Clippy `-D warnings`: 통과 |
| scope/resource 단위 검사 | 4/4. 예약 거절·정확한 작업량·thread/중첩/다음 요청 분리·같은 key 정렬 순서 |
| 보정판의 기존 내장 단위 검사 | JBIG2 2/2, JPEG 2000 1/1. CCITT에는 내장 unit이 없으며 실제 MMR 제품 probe로 검사 |
| 실제 제품 경로의 codec probe | 9/9. 아래 정상·경계·손상·공유 자원·mask·palette·alpha·공개 Rust API 포함 |
| source policy 도구의 기존 계약 | Python unittest 18/18 |
| 형식·공백 | 네 fork `cargo fmt -- --check`, 변경 backend 파일의 rustfmt, `git diff --check`: 통과 |

Native/WASM 항목은 **library의 컴파일·lint**다. 실제 CLI 실행·fresh WASM binding/
브라우저 표시가 끝났다는 의미가 아니다. 그 필수 제품 검증은 승인할 3단계에 남는다.

실제 decoder의 개별 예약 한도를 작은 입력으로 탐색한 뒤 한도 바로 아래·정확한 값·바로 위를
실행했다. 이번 64-bit Native의 최소 허용값은 MMR 체크무늬 JBIG2 128 byte, grayscale JPX
720 byte였다. 이는 그 작은 입력의 관측값이며 실제 문서의 필요 메모리나 기본값이 아니다.
PNG 예약과 구분하여 `handle.decode()`까지 측정했다. 요청 누적 작업량도 정확한 값 전후에서
검사했고 같은 handle의 재디코딩으로 누적 한도를 넘기면 sticky 오류가 유지됐다.
새 요청에서는 정상 입력이 다시 성공했다.

간접 Filter·간접 DecodeParms·ASCIIHex→JBIG2·공유 dictionary 조합은
`indirect-filter-before.log`에서 수정 전 필터 목록 assertion 실패를 확인했다.
공통 필터 해석을 적용한 최종 `product-probe.log`에서는 통과했다.
이 probe는 진단 harness이며 3단계의 정식 rendering 회귀와 시각 증거를 대신하지 않는다.

작은 입력은 Pillow/libtiff의 T.6 strip과 libopenjp2의 lossless JP2로 생성했다.
TIFF의 Photometric 설정을 확인한 뒤 PDF용 JBIG2 segment로 포장했다.
Poppler `pdfimages`의 체크무늬·arithmetic JBIG2·wavelet tile 결과와 픽셀을 대조했다.
JP2 palette/Indexed 우선순위, straight/premultiplied alpha, explicit Mask·SMask도
제품 `PdfImageHandle` → PNG 경로에서 확인한다. Rust 공개 PDF API에서도 같은 요청의
codec 작업량과 자원 오류를 확인한다. 이는 전체 페이지의 렌더링 검증을 대신하지 않는다.

외부 JBIG2 입력은 hayro 공식 저장소의 blob을 해시로 확인하고 사용했다.
`symbol_global.jbig2`는 MuPDF와 픽셀이 같다. Poppler와는 33개 경계 픽셀이 달랐고
원본 `hayro-jbig2 0.3.0`도 RHWP 보정판과 같은 출력을 냈다.
`huffman_text_refine_global_tables.jbig2`는 MuPDF가 Huffman/잘림 오류를 내며
세 decoder 결과도 다르므로 정상 출력의 기준 입력으로 채택하지 않는다.
원본·출력·비교 실패·추가 조사의 근거를 보존하며 시각 통과나 제품 회귀로 바꾸어 보고하지 않는다.

JPX의 크기·색 공간·alpha 계약은 [ISO 32000-1:2008 §7.4.9](https://opensource.adobe.com/dc-acrobat-sdk-docs/pdfstandards/PDF32000_2008.pdf)에
대조했다. PDF 선언과 실제 크기 일치, ColorSpace의 우선순위와 부재 시 해석,
SMaskInData의 opacity 소비·외부 SMask 제약을 구분한다.

추가 standalone backend Clippy에서 base와 동일한 source의 진단 세 건을 확인했다.
`#7953`의 `b7a538803`와 `#7980`의 `f9d2ab17a`, 해당 정식 review를 추적했다.
불필요한 `return`만 제거하고, `Operator`의 40-byte 저장 계약 때문에 유지하는
`Box<String>` 복사 helper에는 그 이유를 명시했다. enum 표현이나 예약량을 바꾸지 않는다.
새 fork의 scalar lane 표현도 같은 계산 순서를 유지하면서 Clippy 경고를 보정했다.

## 완료 범위와 다음 단계

이 계약은 **입력 크기에 따라 늘어나는 codec 저장소와 계상한 반복 작업**의 제한이다.
고정 크기 Rc/Arc/iterator metadata는 일반 Rust 할당을 사용한다. legacy 무예산 호출과
선택하지 않은 upstream image/SIMD 통합 경로는 요청 제한의 근거가 아니다.
전체 allocator 실패 복구·process heap/RSS·wall-clock 상한을 증명하지 않았으며
`parserAllocationBoundsProven=false`를 유지한다.

원본 license/NOTICE와 선택 의존성을 `THIRD_PARTY_LICENSES.md`, source package policy와
ICC runtime include 목록에 연결했다. 패키지는 게시 전 로컬 후보다. 0.9.0 포함·실제 게시·
패키지 이름/MSRV의 미결정을 이 단계에서 자동 확정하지 않는다.

3단계에는 네 구성의 실제 Native CLI/Rust·fresh WASM/Studio Worker 소비, 실패·취소 뒤 복구,
저장·재열기와 기존 대조군, 전쪽 TSV·직접 판독, 성능·크기 관측과 사용자 가이드가 남는다.
신규 rendering 회귀·fixture/golden과 PR 제출은 그 선행 증거·전체 필수 gate를 충족한 뒤 진행한다.
기능 정보·CLI/설치/빌드 문서는 실제 제품 결과에 맞춰 3단계에서 완성한다.
