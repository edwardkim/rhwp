# #665 — ingest 이미지 삽입 검증

## 범위와 선행 분석

신규 중복 이슈 대신 [#665](https://github.com/edwardkim/rhwp/issues/665)를 재개했다.
원 PR #668은 직접 머지되지 않았으며 실제 선별 반영은 `80535a3be1`이다.
#666 보정 `f708a3cb9f`과 #667의 공유 지문·보기·머리말/꼬리말, #3355의 무테두리 기본값,
#3358의 미지 필드 거부를 보존한다. #8421은 placeholder를 알리는 문서 정정이었다.

기준 devel은 `ea86816b5316dfeeb47ecf20df97abfd4b02da73`, 구현 source는
`d78048a6f0e5f71aa3bdb7d32e703bd5971e3019`이다. 기존 `build_exam_paper`의
텍스트/placeholder API는 유지하고, CLI와 새 byte-only API는 실제 그림을 만든다.

## 입력과 독립 기준

입력은 이 작업에서 직접 만든 공개 합성 문서다. 실제 업무 문서 전체의 피델리티를 주장하지 않는다.
`tests/fixtures/issue665/*.json`과 `img/`는 문항·그림·치수·배치의 원본이다.
체커보드와 단색 PNG는 직접 생성한 테스트 패턴이며 외부 이미지나 글꼴 프로그램을 포함하지 않는다.
생성 HWPX/HWP와 별도 기준 HWPX를 목적별 `generated/`, `references/`에 보존했다.
`inputs.sha256`은 실제 검증한 파일의 동일성 확인용이다. image-first의 생성 HWPX는 above와
바이트가 같아 `generated/above.hwpx` 하나를 재사용하며 두 JSON 입력은 각각 보존한다.

독립 기준 HWPX는 새 builder 출력에서 복사하지 않고 **base의 텍스트 빌더와 기존
`edit insert-picture` → `edit set-picture`**로 따로 작성했다. 명시적 텍스트·문단 순서와
출력 폭(mm), 이미지 종횡비에서 기대 결과를 정했다. 문장 내 삽입 위치는 원문 글자 수다.
한컴 2024 `13.0.0.3901`로 Print했고 `pdf_print_method: 0`,
`pdf_output_mode: frame_print_to_pdf_ex_one_up`을 확인했다. engine·job·출력 방법은
`print-provenance.json`, PDF는 `pdf/issue665/`에 보존한다. 저장 환경이 알려지지 않은
외부 원본을 제품 연도만 추측해 선택한 사례가 아니라, 이 합성 검증의 대상 제품을 2024로 명시한 것이다.

이미지 원본을 자원 하나로 재사용하는 개선과 그림별 제어문자·문단 소유를 함께 검사한다.
`mixed`는 서로 다른 PNG 두 개·총 세 Picture와 머리말/꼬리말을 포함한다.
`image-first`는 첫 블록이 이미지여도 문항 번호가 보존되는 반례다.

## 실제 소비 경로

- CLI `ingest_media::load_images` → 검증한 `IngestImage` → `BuildState::picture`에서
  mm × 7200 / 25.4 및 원본 종횡비로 크기를 만든다. 근거는 HWP5 사양의 HWPUNIT 정의다.
- `Picture::from_embedded_image`는 기존 삽입 API의 display frame·crop·imgDim(px × 75)을 공유한다.
  BinData는 위치 ID와 storage ID를 구분하는 공통 등록 helper를 사용한다.
- 문장 내 순서는 `Paragraph.char_offsets`의 실제 UTF-16 위치와 8-unit 그림 슬롯으로 표현한다.
  임의의 저장 LineSeg를 만들거나 그림마다 DocumentCore 재조판을 호출하지 않는다.
- 기존 `composer::inline_picture_size_hu` → `tac_object_flow_height_hu` 및
  `inline_flow::picture_rows::picture_box`가 측정과 실제 줄 배치에서 같은 그림 치수를 소비한다.
  이미지 원본 좌표는 `image_geometry::picture_source_rect`에서 crop/imgDim을 읽는다.
  renderer·pagination·paint의 조건이나 원점/clamp 분기는 바꾸지 않았다.
- boxed 뒤 그림은 부모 컨테이너의 독립 문단으로 남는다. boxed 내부 그림만 boxed 문단 서식을 쓴다.

## 전쪽 측정과 판독

측정일 2026-11-02, 96 DPI, Print profile, compat 2024,
색상 경계 보정 전 2px `silhouette_raw_match_percent`다. 각 문서는 전부 1쪽이다.
Arial은 `/mnt/c/Windows/Fonts/arial.ttf`를 로컬에서 명시 공급했다. 글꼴은 커밋하지 않았다.
Chrome은 로컬 Chromium build 1660786(152.0.7946.0)을 사용했다.

| 대상 | Native | 실제 재컴파일한 fresh WASM |
| --- | ---: | ---: |
| above / between / below / inline | 각각 100.00% | 각각 100.00% |
| runs / mixed / image-first | 각각 100.00% | 각각 100.00% |
| shared-boxed | 91.04% | 91.00% |
| runs HWP 저장 / HWPX 재저장 | 각각 100.00% | 각각 100.00% |
| shared-boxed 생성본 자체의 추가 Print | 91.04% | 91.00% |

전쪽 TSV 원본은 각 producer 하위 폴더와 `all-pages.tsv`에 있다. 누락·90% 미만 쪽은 없다.
`placement-review.png`, `detail-review.png`를 직접 확대해 이미지 색·종횡비·내용,
문자/이미지 순서·앞뒤 간격·선택지·문항 번호·공유 지문 1회·보기 안팎 소유를 판독했다.
수식은 없는 입력이며 수식 IR 피델리티를 주장하지 않는다.

shared-boxed에는 base에도 있는 보기 border의 중복 선과 작은 세로 간격 차이가 있다.
같은 **생성 HWPX**를 base와 head Native로 렌더하면 모두 91.04%이고 그림 위치가 같다.
기존 편집 API로 따로 만든 reference의 base 점수 93.01%와 생성본의 91.04%는 서로 다른
입력 간 수치다. 그 차이를 engine 회귀라고 집계하지 않았다. 직접 판독에서 그림·문자 소유,
내용 순서·누락·중복에 새 결함은 없었다. shared-boxed는 px 좌표 핀을 추가하지 않는다.

## 빌드 provenance와 일반 검사

`base-producers.json`, `head-producers.json`에 실제 CLI/WASM SHA-256을 기록했다.
최초 head WASM 명령이 공유 target의 mtime 때문에 base cache를 0.17초에 재사용했으므로
그 산출물은 제외했다. 소스 내용 변경 없이 `src/lib.rs` mtime을 갱신하고 rhwp 재컴파일을
확인한 뒤 wasm-opt·패키징을 완료한 `1a55a703…` 산출물만 fresh WASM 측정에 사용했다.
기존 Studio의 배포 JS/WASM은 갱신하지 않았다.

현재 확인: Native/WASM Clippy, workspace build, 문서·스킬 계약 48개, 스킬 CI gate 3회,
103개 생성물 재생성 동일성, 변경 Markdown 40개 링크, 실제 CLI 오류 14개와 실패 시
기존 출력 보존, PNG/JPEG/GIF/BMP/TIFF의 실제 CLI 삽입·저장 확인.
전체 release-test·Native Skia·workspace all-target lint·새 정식 회귀의 최종 결과는
진행 기록과 최종 보고서에 추가한다. 이 문서만으로 미실행 검사를 완료 판정하지 않는다.

집중 회귀는 최소 글꼴 격리에서 새 검사 9개·기존 시험지 계약 14개·HTML 이미지 8개가 통과했다.
동일 생성 HWPX의 base WASM도 91.00%였다. SVG 실제 산출물은 base/head byte가 같고 두 원본
PNG를 모두 보존한다(`svg-consumption.json`).
