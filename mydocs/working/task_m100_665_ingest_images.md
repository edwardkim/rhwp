# #665 재개 — 시험지 이미지 구현 기록

Issue: #665

## 착수

- 신규 이슈 중복 등록을 피하고 #665를 open으로 재개했다. 담당자에 edwardkim을 추가했다.
- 원 PR #668 → 실제 선별 반영 `80535a3be1` → #666 보정 `f708a3cb9f` → 현행 placeholder를 추적했다.
- 사용자 승인 범위는 기존 조사 결과에 따른 로컬 구현·검증이다. 원격 코드 push·PR·merge는 수행하지 않는다.
- 계획: [구현 계획](../plans/task_m100_665_ingest_images.md).
- 착수 기록 시점에는 구현·빌드·시각 검증 전이었다. 아래에 후속 실제 결과를 추가한다.

## 구현 진행

- 공통 BinData 등록을 model helper로 공유하고 기존 native/HTML 그림 생성의 치수·crop·imgDim 구성을 공통 Picture factory로 연결했다.
- 이미지 byte를 검증·공유하는 `IngestImage`, 한도와 새 `build_exam_paper_with_images` API를 작성했다. 기존 text/placeholder API는 유지한다.
- `paragraph.runs`의 엄격한 text/image 역직렬화와 JSON schema 확장을 작성했다. 실제 문자 좌표는 UTF-16 stream과 그림의 8-unit 자리를 반영한다.
- CLI 로컬 이미지 로더, JSON 디렉터리 기본 기준 경로, 명시적 media root·한도 옵션을 연결했다. 파일 경로 이탈과 손상·누락을 실패 처리한다.
- 1차 `cargo check --locked --lib` 및 `--bin rhwp`가 공용 target에서 통과했다. 이후의 세부 보정은 최종 검증 때 다시 확인한다.
- 실험용 배치 4종·문장 내 두 이미지·공유 지문/보기 사례를 ignored output에 준비했다. 아직 정식 렌더링 회귀로 등록하지 않았다.
- 기준 devel의 Native/fresh WASM producer를 별도 clean worktree에서 빌드했다. source와 producer hash는 ignored `output/issue665/base-producers.json`에 기록했다. 구현 완료·시각 통과·전체 회귀 통과를 아직 판정하지 않는다.

## 재접속 후 진행

- 로그인 재접속 뒤 branch/WIP·실행 프로세스·이슈 담당자를 재확인했다. 중단된 기준 Native 빌드를 재개해 clean base `ea86816b53`의 Native와 최적화 fresh WASM을 확보했다.
- 수정본의 `cargo check --locked --bin rhwp` 최종 확인이 통과했다(34.71초). source compile과 시각·전체 회귀 완료를 구분한다.
- 본문 소유와 UTF-16 stream·반복 이미지·문항 번호를 점검하고, 본문 크기 초과·중복 지문 ID를 진단하도록 보완했다. picture의 raw 속성은 기존 공통 packer에서 산출한다.
- CLI 매뉴얼·스킬·생성기와 산출 안내를 현행 구현 계약으로 갱신했다. 기존 시험지·route/catalog/author 계약 48개와 주요 문서 상대 링크 검사가 통과했다.
- 기준 문서는 새 빌더에서 복사하지 않고 base의 텍스트 빌더와 기존 `edit insert-picture`/`set-picture`로 따로 작성했다. 6종의 한컴 2024 Print job이 성공했다. 실제 제품 `13.0.0.3901`, `pdf_print_method: 0`, `frame_print_to_pdf_ex_one_up`을 확인했다.
- 같은 기준 문서의 base Native 전쪽 TSV는 배치 4종·runs에서 각 100.00%, shared-boxed에서 93.01%였다. shared-boxed의 이미지·문자 순서와 보기 소유를 직접 확대 확인했다. 이 값은 새 빌더 후보의 검증 결과가 아니다.
- 다음 실행: 후보 Native/fresh WASM producer → 실제 CLI/저장 재열기 → 동일 독립 Print의 전쪽 TSV·직접 판독 → 조건 충족 뒤 정식 회귀와 필수 lint/전체 회귀.

## 후보 실행 검증 중

- 구현·가이드 로컬 커밋: `d78048a6f0e5f71aa3bdb7d32e703bd5971e3019`.
- 해당 source의 Native release 빌드는 16분에 완료했다. 실제 CLI 6종·mixed·image-first·JPEG/GIF/BMP/TIFF에서 Picture/BinData 생성을 확인했다. 기본 6종은 encoded PNG 원본 byte 동일성과 파서 재열기를 확인했다.
- Native 전쪽 TSV: above/between/below/inline/runs/mixed/image-first 각 100.00%, shared-boxed 91.04%(각 1쪽). HWP 저장 및 HWPX 재저장 runs도 각각 100.00%, 두 CLI `--verify` 모두 IR 차이 없음이다.
- shared-boxed를 같은 생성 HWPX로 base와 head에서 출력한 결과가 동일(91.04%)이다. 기존 편집 API로 따로 만든 reference의 93.01%와 다른 입력 간 수치 차이를 engine 회귀로 해석하지 않는다. 생성 HWPX 자체도 한컴 2024 Print를 추가 산출해 동일한 기준 표시임을 확인했다. 기존 보기 border의 중복 선 차이는 base에도 있고 이미지·문자 소유/순서·누락에 새 결함은 보이지 않았다.
- 공유 target의 mtime 때문에 최초 head WASM build가 base 산출물을 0.17초에 재사용했다. 해당 산출물은 검증에서 제외하고 작업 소유 최적화만 종료했다. `src/lib.rs` 내용 변경 없이 mtime을 갱신한 뒤 수정 패키지의 실제 재컴파일을 확인했다(`head-wasm-rebuild.log`). 최적화·브라우저 검증은 아직 진행 중이다.
- 최소 글꼴 환경은 bwrap 파일 시스템 격리로 준비했다. 개인 HFT와 Windows 글꼴을 숨기고 저장소 open-source 글꼴 5개만 Fontconfig에 노출했다. 시각 비교에 별도 공급한 로컬 Arial과 일반 회귀 환경을 구분한다.
- 변경 Markdown 40개 상대 링크, 스킬 계약 48개, 생성기 103개 파일의 byte 단위 재생성 동일성, 스킬 라우팅 CI gate 3회·부속 검사가 통과했다. 스킬 gate의 rhwp binary probe는 root 실행 파일 부재로 skip했으며 실제 CLI 검증으로 대체했다고 허위 집계하지 않는다.

## 정식 회귀와 남은 게이트

- 실제 재컴파일·wasm-opt를 마친 fresh WASM 전쪽 검증과 확대 판독을 완료했다. 기본 8종과 두 저장 왕복은 각 1쪽이며 전쪽 최저 Native 91.04% / WASM 91.00%다. shared-boxed 이외에는 양쪽 100.00%다.
- 같은 생성 HWPX의 base/head WASM도 91.00%로 동일했다. 실제 SVG byte와 이미지 payload도 두 빌드에서 동일했다.
- 이 선행 조건 뒤에만 정식 테스트·입력·독립 Print·TSV·대표 PNG를 커밋 `cba7849e30`으로 등록했다. 검사에서 저장소에 없는 tempfile helper 의존성을 사용한 오류는 기존 std 임시 디렉터리 패턴으로 보정했다. 테스트 source 변경 뒤 자동 suite를 재생성해 실제 9개 검사가 실행되는 것도 확인했다.
- 최소 글꼴 환경의 focused 결과: 신규 9/9, 기존 ingest 내용 계약 14/14, 기존 HTML 그림 8/8 통과.
- 이제 전체 release-test, Native Skia 3종, workspace all-target Clippy, 실제 새 문서 보안 검사, 성능 확인과 최종 기록을 완료한다. PR·push·merge·이슈 재종결은 승인 범위에 포함하지 않는다.

## 전체 검사에서 발견한 검증 경로 보완

- workspace all-target Clippy가 통과했다. 첫 전체 release-test는 12,378건 중 12,376건 통과, 2건 실패였다. 실패를 완료로 집계하지 않는다.
- #2724 원 PR #2741 → 실제 통합 #2797(`f2c02b6669`) → 원 보고서의 독립 삭제 실증을 확인했다. 기존 Picture 등록의 DocInfo 무효화 1곳은 `src/model/bin_data.rs`로 이동했으며 삭제되지 않았다. 밀도 검사의 범위를 그 파일까지 넓히고 기존 Picture 7곳 + 공통 helper 1곳을 각각 보호한다. 단순히 하한을 낮추지 않았다.
- std 전용 원 검사 5개는 base에서 통과했다. 보완한 검사 5개도 통과했고, ignored 복사본에서 공통 helper 무효화만 삭제하면 밀도 검사가 1 → 0으로 실패한다. 실제 기존 HWP를 연 뒤 다른 그림을 삽입·HWP 저장·재열기한 CLI `--verify`도 IR 차이 0건이었다.
- 두 번째 실패는 `samples/` 전용 `RHWP_SECURITY_SWEEP_SAMPLES_JSON`에 `tests/fixtures/` 경로를 넣은 실행 설정 오류였다. 이 작업은 `samples/`를 변경하지 않는다. 전체 검사는 해당 잘못된 입력 없이 다시 실행하고, 새 목적별 fixture는 실제 CLI의 hidden-text·injection·unicode 탐지를 각각 실행해 별도로 기록한다. 기존 보안 검사와 탐지 규칙은 변경하지 않는다.
