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
| JPEG / GIF / BMP / TIFF | 각각 100.00% | 각각 100.00% |

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

## 추가 형식·보안·성능 확인

JPEG/GIF/BMP/TIFF는 직접 만든 PNG 패턴을 해당 형식으로 인코딩한 공개 자원이다.
각 독립 기준은 기존 base의 above reference HWPX에서 이미지 payload·확장자와 OPF의
MIME/href만 바꾸었다. 새 이미지 builder 출력에서 기준 배치를 복사하지 않았다.
한컴 2024 Print 네 건과 Native/fresh WASM 전쪽 raw는 모두 100.00%다.
`codec-review.png`에서 3열의 그림 색·크기·종횡비와 문항 번호·본문·선택지 간격을 직접 판독했다.
모든 fixture·독립 PDF 해시는 `inputs.sha256`, 각 Print의 실제 실행 정보는
`print-provenance.json`에 보존한다.

`samples/`에는 추가·수정 문서가 없다. `tests/fixtures/issue665`의 HWP/HWPX 24개는 실제
CLI hidden-text·injection(`--include-fields --min-confidence low`)·unicode를 각각 실행했다.
72회 모두 exit 0 / clean이며 입력별 SHA는 `security-fixtures.json`에 있다.
`samples/` 전용 환경변수로 이 결과를 대신하거나 입력 없는 검사 성공을 24개 검사로 집계하지 않는다.
일반 회귀의 파일 시스템·글꼴 격리 결과는 `minimal-font-receipt.json`에 있다.

`benchmark-results.json`은 동일 호스트 Native CLI 생성 시간이다. 텍스트 100문항을 3회
예열한 뒤 base/head를 번갈아 21회씩 실행했다. 중앙값은 base 5.38ms / head 5.25ms이며
ZIP 안의 모든 비압축 entry가 같다. 이 짧은 측정으로 속도 개선을 주장하지 않는다.
같은 그림 100개는 7.05ms / peak RSS 15,256KiB, 1,000개는 25.67ms / 24,732KiB였다.
각각 7회 중앙값이며 RSS는 별도 `/usr/bin/time` 실행값이다. 자원은 두 경우 모두 1개다.
이 수치는 ingest 생성 성능이며 다페이지 편집·브라우저 로딩 성능 측정이 아니다.

## 공유 무효화 이동의 검사 보존

#2724 원 PR #2741 → 실제 통합 #2797(`f2c02b6669`) → 원 보고서의 독립 삭제 실증을
대조했다. 기존 Picture의 DocInfo 무효화 1곳을 model helper로 이동했으므로 밀도 원장은
Picture 7곳 + model 1곳을 각각 검사한다. 공통 helper 삭제를 허용하도록 하한만 내리지 않는다.
std 전용 원 검사 5개가 base에서 통과하고, 보완한 5개도 통과한다. 공통 helper의 무효화만
지운 ignored 복사본은 1 → 0으로 실패한다(`guard-proof.json`, `guard-*.txt`).
실제 기존 HWP를 연 뒤 다른 그림을 삽입하고 저장·재열기한 `--verify`도 IR 차이 0건이다
(`loaded-docinfo-verify.json`). 원 보고서는 `mydocs/report/archives/task_m100_2724_report.md`다.

## Rust API 호환 범위

기존 JSON과 `build_exam_paper`의 텍스트 projection은 유지된다. 새 이미지 API는 파일을
직접 읽지 않으며 호스트가 전달한 검증 바이트를 사용한다. 공개 enum `StemBlock`에는
`Paragraph { runs }`가 추가됐다. 이 enum을 빠짐없이 match하는 Rust 소비자는 새 variant의
처리를 추가해야 한다. CLI의 새 `paragraph.runs`를 구형 binary에서 읽는 호환성은 주장하지 않는다.
