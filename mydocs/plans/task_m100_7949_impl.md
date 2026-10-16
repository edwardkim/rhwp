# Task #7949·#7950 — PDF 기본 빌드와 명시적 이미지 마스크 공동 구현계획

- Issue: [#7949](https://github.com/edwardkim/rhwp/issues/7949)
- 공동 처리: [#7950](https://github.com/edwardkim/rhwp/issues/7950)
- 작성일: 2026-10-16 (Asia/Seoul)
- 기준 SHA: `0953598d8e08af5e9149b3e4cdbbbf2f5f2bae64`
- 상태: 2026-10-16 사용자 승인. 명시적 Mask·기본 빌드 구현 뒤, 새 샘플의 SMask·Matte까지 포함하도록 추가 승인
- 상위 계획: [공동 수행계획](task_m100_7949.md)

## 1. 변경 값의 실제 소비 경로

| 범위 | 생산 → 소비 → 최종 결과 | 구현·확인 대상 |
| --- | --- | --- |
| 기본 지원 | `Cargo.toml` default → Native/WASM cfg → 실제 capability → Studio 파일 형식/loader | 기본에 `pdf-import` 추가. `--no-default-features` 제외와 기존 feature 조회 API 유지 |
| 명시적 마스크 | `PdfImageHandle::decode()`의 이미지·Mask 참조 → 제한된 stream decode·합성 → `src/parser/pdf/extract.rs`의 `PdfImage` → lowering의 그림 데이터/범위 → 기존 backend | 합성 RGBA가 실제 저장 그림과 paint까지 전달되는지 추적. bbox·CTM·visible bounds를 새로 추측하거나 덮어쓰지 않음 |
| 자원 제한 | `ImportBudget` → 객체 참조·압축 해제·RGBA 생성·마스크 적용 → 오류의 sticky 상태 | 버퍼 요청 전 checked 산술과 예산 계상. 완료 후 한도 확인만으로 대신하지 않음 |

현재 renderer의 `page_renderer.rs::render_image`에도 명시적 Mask 처리가 있지만,
그 경로의 실패 시 base image 유지나 codec 추정 fallback은 제한된 제품 import에 복사하지 않는다.
필요한 의미와 이미 존재하는 제한된 stream decoder를 확인해 공통 해석 또는 작은 전용 합성을 정한다.
현재는 `images.rs`의 guard와 제품 `extract.rs`의 오류 변환까지 확인했으며,
합성 이후 lowering·저장·각 backend의 실제 소비 추적은 구현 착수 시 완료할 항목이다.

## 2. #7950 마스크 계약

1. `/Mask` 스트림과 `/ImageMask true`·1비트·크기를 해석한다. 참조 해석은 기존 요청 context를
   사용하고 dangling·순환 참조, 잘못된 타입·크기·길이를 임의 기본값으로 바꾸지 않는다.
2. PDF 명시적 마스크의 Decode 기본값/반전, MSB 우선과 행 단위 바이트 패딩을 독립 사양과
   실행 기준에 맞춰 적용한다. 기본 극성과 반전의 기대 투명 영역을 서로 구분한다.
3. 마스크 크기가 base와 다른 경우의 PDF 의미를 먼저 확인한다. 샘플과 같은 크기뿐 아니라
   지원 범위의 크기 정책을 명시하며, 미구현 보간/스케일은 허용한 것으로 가장하지 않는다.
4. 실제 신규 합성 경로는 제한된 stream decode를 사용한다. 길이 산술·출력 버퍼·반복 계산과
   참조 방문을 요청 예산/시간 확인에 연결한다. 색상·기존 alpha와 곱해진 최종 alpha를 보존한다.
5. 색상 키 배열·독립 stencil 등 이번에 구현하지 않는 분기는 기존 명시적 거절을 유지한다.
   입력 오류와 미지원 종류, 자원 초과의 오류 코드를 혼동하지 않는다.

작은 검증은 Decode 기본값·반전, 8의 배수가 아닌 너비와 행 패딩, 길이 부족·잘못된 참조·
중첩/순환·지나친 크기·예산 초과, 무마스크/Null Mask 정상 대조군을 사용한다.
실물 시각 선행 조건을 충족하기 전에는 렌더링 fixture/golden을 추가하지 않는다.

### 2.1 추가 승인: SMask·Matte

사용자가 제공한 `issue-issue7949-type-002.pdf`(PowerPoint, 19쪽)는 DeviceGray 1·8비트
SMask와 RGB 그림의 Matte를 사용한다. 기존 구현은 이 유형을 명시적으로 거절한다.
2026-10-16 사용자가 “이번 작업에 포함”을 승인했다.

- PDF Reference 1.7 §7.5.4, 표 7.11·7.12의 회색 alpha와 Decode를 적용한다.
- Matte는 색 변환 전에 `c = m + (c′ - m) / alpha`로 되돌린다. 우선 DeviceRGB·DeviceGray
  원색 공간을 지원하고, 별도 색 변환이 필요한 Matte는 명시적으로 미지원 처리한다.
- Matte가 있으면 mask/base 크기가 같아야 한다. 없으면 공통 단위 사각형으로 샘플링한다.
- 두 샘플 격자의 해상도를 보존하도록 합성 격자를 정한다. 특히 18쪽의 2×2 색상 이미지와
  1,891×165 글자 모양 마스크를 색상 이미지 해상도로 축소하지 않는다. codec의 선언 크기 검사는
  합성 전에 수행하고, 합성 후 픽셀 수·PNG·crop은 같은 결과를 사용한다. CTM·가시 범위는 유지한다.
- 기존 예산·취소·시간 확인과 명시적 Mask의 반대 극성을 보존하며, 중첩·잘못된 길이·참조와
  지원하지 않는 필터를 무시하지 않는다. 이미지 XObject의 SMask와 graphics-state soft mask는 구분한다.
- 새 샘플 19쪽의 Native/fresh Docker WASM TSV·실제 Studio 열기와 이미지 경계를 확인한다.
  첫 샘플에 승인된 시각 예외를 새 샘플로 자동 확장하지 않는다.
- 통합 실행에서 반복 header가 9쪽째 누적 해제량 한도를 넘겼다. 실제 phase 계측으로
  이미 해제된 Raw RGB/RGBA의 PNG 출력에 수행하지 않는 decode/색 변환 3회분이 추가 예약됨을
  확인했다. 같은 borrowed-sample 결과를 예산과 encoder가 함께 소비하고 실제 zlib scratch·
  출력은 계속 예약한다. Mask 합성의 RGB 복사도 빌려 쓰는 경로로 제거한다. 256MiB 누적 한도,
  버퍼별 한도·sticky 실패는 유지하며 실제 예약량 경계 검사로 검증한다.
- 사용자 추가 승인으로 반복 해석 개선도 포함한다. 원본 페이지에서 생산한 동일 연산열을
  텍스트·선·그림 소비자가 재사용하며, 소비자별 owned 복사는 계속 실제 요청 예산에 계상한다.
  캐시는 한 페이지로 제한한다. 외부 TextExtractor의 임의 스트림과 중첩 Form은 이 캐시로
  페이지 출처를 추정하지 않으며 기존 별도 해석·순환/깊이 한도를 유지한다.
- 전쪽 실행을 가로막은 형식 경계도 보정한다. page offset은 signed HWP 비트 필드로
  저장하며, 이미지의 변환은 HWP 좌표 단위에서 판정한다. 표 추론의 선분 한도 도달은
  추론만 보류하고 실제 원본 개체의 positional 경로를 유지한다. invalid geometry는 거절한다.

## 3. #7949 기본 빌드 계약

- `Cargo.toml`의 default에 `pdf-import`를 포함하고 선택 feature 자체는 유지한다.
- `src/parser/pdf/mod.rs`, `src/parser/mod.rs`, `src/wasm_api.rs`, CLI help·metadata·
  feature-off 오류와 관련 안내의 “기본 비활성/opt-in” 표현을 실제 빌드 계약에 맞춘다.
  미지원 빌드의 메시지에서만 필요한 재빌드 방법을 안내한다.
- `scripts/tests/pdf-import-wasm.test.mjs`의 `PDF_IMPORT_DEFAULT_PKG`는 기존에 off bundle을
  뜻했다. 새 기본 on과 명시적 off를 명확히 구분하도록 입력·검사·안내를 갱신한다.
  `tests/cases/issue_7911_pdf_cli.rs`·detection·frontend·resource 검사를 실제 feature별로 실행한다.
- Docker `wasm` 서비스·locked wrapper, CI와 release-binary·웹/Node 패키징의 Cargo 옵션을
  조사한다. 기본 feature를 끄는 경로가 있으면 목적에 맞춰 반영하고 실제 지원 범위를 기록한다.
- 개발 가이드·CLI·에이전트 명령 안내를 갱신한다. 전체 과거 보고서를 새 상태로 다시 쓰지 않는다.
- 새 제품 배포·스토어 등록, 전 제품 PDF UI 추가는 이 변경의 필수 범위로 확대하지 않는다.

## 4. 검증과 제출 순서

1. 이미 보존한 현재 devel의 실제 PDF-on WASM 실패를 수정 전 증거로 재사용한다.
   마스크 변경 뒤 작은 경계 실행과 Native 동일 영역 비교로 방향을 확인한다.
2. 최종 공동 소스에서 Native 기본/명시적 off와 Docker WASM 기본/명시적 off를 검증한다.
   Native는 기본 작업공간의 `/home/edward/mygithub/rhwp/target/pr-review`를 재사용한다.
   Docker는 표준 `wasm` 서비스의 named target volume과 locked wrapper를 재사용한다.
   기본 성공 증거는 추가 `--features pdf-import` 없이 생성한 결과여야 한다.
3. 실제 브라우저에서 capability·fetch WASM SHA·pkg/public SHA를 확인하고 정식 열기/드롭으로
   재현 PDF 1쪽과 공개 PDF 12쪽을 확인한다. 사용 중인 7700 대신 전용 검증 서버/탭을 먼저 쓴다.
   오류 뒤 문서 상태와 다음 정상 입력, HWP/HWPX 대조군을 확인한다.
4. 원본 PDF 자체를 독립 기준으로 전쪽 Native/fresh Docker WASM TSV를 먼저 산출한다.
   양쪽 최저 90% 이상·쪽수 일치와 대표 그림/투명 영역·본문·crop을 확대 대조한다.
   source·입력 SHA·WASM SHA·명령·직접 판독과 남은 차이를 같은 증거에 연결한다.
5. 선행 시각 조건을 충족한 뒤 정식 `tests/cases/`에 의미 있는 마스크 회귀를 추가한다.
   수정 전 해당 원인으로 FAIL/수정 후 PASS와 정상 대조군을 구분하고 절대 px/전체 SVG 해시로
   임시 배치를 고정하지 않는다. 코드가 바뀌면 영향 경계와 시각 확인을 다시 수행한다.
6. [변경 범위별 검증](../manual/pr_review/local_validation.md#43-변경-범위별-기본-검증)에
   따라 fmt·Native/WASM/workspace lint·빌드·정식 회귀와 Studio·제품 WASM/Worker 계약을 실행한다.
   문서만 갱신한 현 준비 단계에 Rust/WASM 재빌드·전체 회귀를 반복하지 않는다.
7. 단계별 결과·commit, 최신 devel 병합 확인·필요한 재검증 뒤 한국어 통합 PR을 준비한다.
   각 이슈를 해결한 변경·한계·검증을 구분하고 원격 반영·PR·CI·merge는 해당 단계 승인을 따른다.

로그·TSV·JSON·개인 입력은 ignored `output/pr-review/issue7949/`에 둔다.
공개 가능한 회귀 입력과 대표 시각 증거의 실제 Git 포함 여부는 권한·시각 gate 확인 뒤 정한다.
계획 준비 중에는 제품 소스·WASM을 변경하지 않았다. 사용자의 “승인합니다” 이후 승인된 공동 범위의 구현·검증을 진행한다. 기존 사용자 파일은 보존한다.
