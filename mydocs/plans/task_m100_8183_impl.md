# #8183 — 코덱 opt-in 구현 계획

Issue: [#8183](https://github.com/edwardkim/rhwp/issues/8183) · [수행 계획](task_m100_8183.md)

기준 source는 `d9fd85ed5bc3c4d9ce6b2981422e48bf9bb90867`이다.
2026-10-23 작업지시자가 이 계획과 2단계 구현을 승인했다.
feature·자원 계약은 2단계에서 구현하고, 추가 기능 정보·설치 명령·사용자 안내는
3단계의 실제 제품 검증에 맞춰 확정한다. [2단계 기록](../working/task_m100_8183_stage2.md)을 따른다.

## 선택 빌드의 계약

본체의 공개 feature는 `pdf-jbig2`, `pdf-jpeg2000`을 제안한다.
각 feature는 기존 `pdf-import`와 해당 backend codec 하나만 활성화한다.
backend에는 `jbig2 = ["dep:hayro-jbig2"]`를 분리하고 기존 `rendering`이 이를 포함하도록 하여
backend의 기존 rendering 소비도 보존한다. `jpeg2000`의 기존 의미는 유지한다.

| 본체 선택 | PDF | JBIG2 | JPEG 2000 | backend 전체 rendering |
| --- | --- | --- | --- | --- |
| 기존 기본 빌드 | 현재대로 | 비활성 | 비활성 | 비활성 |
| `pdf-jbig2` 추가 | 포함 | 활성 | 비활성 | 비활성 |
| `pdf-jpeg2000` 추가 | 포함 | 비활성 | 활성 | 비활성 |
| 두 feature 추가 | 포함 | 활성 | 활성 | 비활성 |

`--no-default-features`에서도 codec 선택은 PDF 입력을 포함한다.
CLI가 필요로 하는 기존 `wasm-api`와 Native 출력의 `native-skia`는 별도 선택이다.
codec 기능만으로 PNG·Sixel·Kitty 지원이나 JS binding을 활성화했다고 설명하지 않는다.
미선택 decoder의 의존성이 실제 활성 graph에 들어오지 않는지 metadata/tree와 빌드로 확인한다.
추가 의존성의 정확한 선택 버전·license/NOTICE와 package 선별도 확인한다.

## 실제 호출 경로와 변경 지점

| 생산·소비 경계 | 현재 source와 관측 | 보완할 계약 |
| --- | --- | --- |
| 제품의 그림 선택 | `src/parser/pdf/extract.rs`: image handle 열거 → 선언 픽셀 계상 → `handle.decode()` → 실제 출력 픽셀 재계상 | 기존 선택 쪽·그림 누적 정책을 새 코덱에도 유지 |
| decode와 후처리 | backend `src/extractors/images.rs:PdfImageHandle::decode`: active color scope·mask 읽기 → extractor → 실제 크기 대조 → mask/opacity/clip → 최종 bbox/CTM | 코덱 helper 성공 뒤 mask·크기·최종 좌표 소비까지 검증. 원본 CTM/paint 소속 보존 |
| 코덱 진입 | 같은 파일의 `extract_image_from_xobject_at` → `decode_jbig2_image` / `decode_jpx_image` | legacy 무예산 호출과 제품의 예산 있는 decode를 구분하고 실제 예약 전 budget 검사 연결 |
| JBIG2 | `rendering` cfg, `hayro_jbig2::Image::new_embedded` → collector의 row/pixel Vec | 독립 `jbig2` cfg, 실제 header·사전·segment·bitmap·collector 확장의 checked 예약과 중단 |
| JPEG 2000 | `src/decoders/jpx.rs:decode_jpx_with`: `Image::new` → component decode → interleave/upsample | tile/component/scratch·누적 작업의 제한, 실제 source 크기·깊이·성분과 PDF 선언 대조 |
| source mask | `image_mask_limited.rs`, `image_mask_limited_extra.rs:samples`는 packed PDF sample과 codec sample을 구분 | 사양상 허용되는 codec mask에 감사 가능한 공통 sample adapter를 연결. encoded bytes를 raw mask로 취급하지 않음 |
| 최종 PNG·IR | `PdfImage::to_png_bytes_with_budget` → frontend lowering → 공통 그림 리소스 | 출력 인코딩·복사·RGBA 합성까지 기존 예산과 sample 의미를 유지 |

현재 JPX의 축소 해상도는 hint이며, 제품 handle은 최종 decoded 크기와 원 선언 크기가 다르면
거절한다. 이 제한을 끄고 전체 디코딩하는 방식으로 대체하지 않는다. 이번 기본 계약은 원본
해상도를 기존 한도 안에서 처리하는 것이다. 축소 복호화를 공식 소비하려면 원본 geometry·실제
sample 크기·mask·CTM이 함께 소비하는 별도 계약을 먼저 입증한다.

현재 `SMaskInData`, palette·색 성분·Decode, JBIG2Globals의 실제 분기도 확인한다.
지원한다고 정한 입력의 alpha·mask를 버리거나 픽셀을 잘라 통과시키지 않는다.
허용·거부할 조합은 PDF 사양·정상 생성본에서 정하며 합성 입력의 수동 속성만 근거로 확대하지 않는다.

## 자원 경계 구현 순서

1. `ImportBudget`의 기존 per-allocation·누적 decoded·operation·sticky 오류 소비를 실제 codec
   진입과 내부 예약 위치에 대조한다. 그림 출력 크기를 내부 임시 버퍼 상한으로 대신하지 않는다.
2. JBIG2의 공유 사전·참조 segment·중간 bitmap과 JPX의 tile/component·wavelet 작업을 추적한다.
   기존 decoder가 충분한 제한 API를 제공하지 않으면 필요한 source 보완과 upstream/fork 관리
   범위를 구체화한다. 실제 경계를 제한하기 전에 feature 연결만 완료로 처리하지 않는다.
3. 입력·출력 크기 곱셈, 32비트 WASM 변환, allocator 실패와 반복 작업 중단을 확인한다.
   Native host·기존 PDF Worker의 deadline/취소/재생성도 실제 제품 경로에서 대조한다.
   Worker 종료나 논리 예산을 전체 process heap/RSS의 보장으로 표현하지 않는다.
4. 기본·정확한 한도·초과·손상·반복 소비·오류 뒤 정상 요청을 작은 예산으로 실행한다.
   같은 helper 단위 검사만으로 Native/WASM 제품 실행을 대체하지 않는다.

## 기능 확인과 가이드 변경

공통 PDF 모듈에 compile-time codec 지원 정보를 한 번 정의하고 CLI와 WASM의 기존 지원·진단
표면이 이를 소비하게 한다. CLI의 `src/cli/metadata/capabilities/`와 PDF 진단 metadata에
추가할 최소 필드를 정한다. JSON/API를 확장하기 전에 에이전트 표면 플레이북과 기존 schema·
자동 수집을 확인하고, 기존 field·오류 코드·binding signature의 호환성을 유지한다.

| 문서 | 작성할 내용 |
| --- | --- |
| `mydocs/manual/dev_environment_guide.md` | 기본/codec 선택 Native·WASM 빌드, locked wrapper에 feature 전달, 실제 Studio JS/WASM 동기화·HTTP bytes 확인 |
| 새 `mydocs/manual/pdf_image_codecs.md` | 필요한 codec 식별 → 빌드/설치 경로 선택 → 기능 확인 → 정상 PDF 사용·실패 진단의 사용자 여정 |
| `mydocs/manual/cli_commands.md` | PDF 입력의 codec 선택과 현재 바이너리 지원 확인, 비활성·손상·한도 초과의 실제 진단 |
| `packaging/crates-io/README_PACKAGE.md` | Rust dependency와 CLI 설치의 개별 feature 예제, local source 명령과 실제 게시 버전의 구분 |
| `mydocs/manual/security_feature_limits.md` | 기본 미활성과 검증한 opt-in 지원을 구분하고 기존 자원·ICC·암호 PDF 제한 유지 |
| `mydocs/manual/README.md` 및 해당 에이전트 안내 | 새 가이드의 진입 링크와 필요한 최소 사용 설명 |

구현 전에는 가이드에 실행 가능한 설치 명령을 추가하지 않는다. 게시되지 않은 source의 기능을
현재 crates.io 버전으로 설치할 수 있다고 표시하지 않는다. WASM feature 선택은 npm/Studio의
런타임 설정이 아니라 해당 WASM을 다시 빌드하는 선택임을 설명한다.

## 실제 검증과 제출 조건

- 기존 `tests/cases/issue_7911_pdf_*`, `issue_7959_pdf_path_colour`,
  `issue_7974_pdf_source_masks`와 PDF 자원·공개 WASM Worker 검사의 수집·실행 경로를 먼저 확인한다.
  새 검사는 기존 자동 suite에 연결하며 CI의 중복 개별 호출을 추가하지 않는다.
- codec별 PDF를 출처·생성 방식·hash와 함께 확보한다. 정상 스캔, 공유 사전, 사양상 유효한
  palette/alpha/mask와 한도 반례를 구분한다. 필요한 실물 자료가 없으면 자료 확보를 먼저 한다.
- Native/WASM 네 조합, 실제 CLI/Rust 공개 API·Studio Worker, 실패 후 다음 정상 요청을 실행한다.
  기본·선택 조합의 기능 정보와 실제 지원이 같은지 검사한다.
- 출력 영향 문서의 전체 페이지를 원본 PDF 또는 적합한 독립 기준과 Native/fresh WASM에서
  Visual Sweep으로 대조한다. 페이지별 raw 90%·base 대비 하락 gate·직접 확대 판독과
  신규 회귀 선행 조건을 충족한다. 편집 저장본은 형식에 맞는 독립 Print도 확인한다.
- 기본/선택 빌드의 시간·메모리 관측·산출물 크기를 같은 입력·조건으로 비교하고 일반 상한과 구분한다.
- 제품 source 보정 뒤 변경 범위의 필수 lint·전체 회귀·Native/fresh WASM·시각 검증을 완료한다.
  코드는 모두 공용 `target/pr-review`를 사용하고 Cargo를 순차 실행한다.

단계 결과와 최종 보고를 커밋한 뒤 승인된 PR 절차를 적용한다. 현재는 설계·source 조사 결과이며
새 codec의 빌드·decode·자원 상한·시각 일치를 검증 완료로 기록하지 않는다.
