# #7998 기존 출력 차이 보정과 전체 검증

- 앞 단계: [1차 구현·검증](task_m100_7998_stage1.md), 코드 `1d54f8a09`, 검사/기록 `876597051`.
- 2026-10-18 작업지시자가 기존 출력 차이의 이번 범위 포함 및 전체 검증·로컬 PR 준비를 승인했다.
- 1차 전쪽 PNG·TSV·빌드와 실제 Worker 증거를 보존하며, 같은 제품 코드는 다시 빌드하지 않는다.
- 증거 루트: `/home/edward/mydev/rhwp-review-evidence/pdf-table-feedback-20261017/output-fixes/`.
- 최종 상태: 승인된 전체 로컬 검증·제출 자료 준비를 완료했다. 메인테이너 직접 시각 판정은
  요청 후 대기 중이며 원격 반영 승인은 아직 받지 않았다. 구조 추론의 후속 범위는 #7998에 남는다.
- 원격 source push·PR 등록은 승인 전이며 수행하지 않는다.

## 독립 원인 확인과 수정 경계

1. Native 3쪽은 PDF `INPILL+Batang` → subset tag 제거 → IR `Batang` → Native font chain
   → 실제 Typeface의 연결을 확인한다. 앞선 디렉터리에는 큰 HCR font가 먼저 정렬돼 64MiB
   입력/retained-copy 제한이 소진되었고 Batang이 적재되지 않았다. `batang.ttc` 하나를 우선
   공급한 기존 빌드에서는 89.65203→91.04017%다. 원본 글꼴 공급 제한을 늘리거나 요청 family를
   다른 글꼴로 바꾸지 않고 실제 필요한 face를 공급하고 trace와 전쪽 영향을 확인한다.
2. 견적서의 가는 fill/괘선은 원본 stream → 공통 IR 원래 두께 → Native/Canvas paint를 따른다.
   동일한 현재 Native PNG는 Cairo 24.02.0과 99.85878%, MuPDF 1.23.10과 99.75785%이며
   기존 Poppler `thinlinemode=shape`만 72.76381%다. 원본에는 stroke-adjust 지정도 없다.
   추가 thin-fill 대조군의 독립 면적·AA 관측으로 reference의 pixel-boundary 보정 문제와
   제품 paint 오류를 구분한다. 원본 도형을 두껍게 하거나 이진화/2px/90% 기준을 바꾸지 않는다.
   [Poppler 옵션 정의](https://skia.googlesource.com/third_party/poppler/+/master/utils/pdftoppm.1)는
   shape가 원래 소수 위치를 픽셀 경계로 이동시키는 향상 모드임을 명시한다.

실제 제품 결함을 발견하면 구현을 고치고 영향 캡처를 갱신한다. 검증 도구의 reference 생성이
원래 위치/coverage를 바꾼 결함이면 독립 입력·복수 PDF renderer·작은 경계의 실행 근거를
먼저 남겨 도구를 수정한다. 점수 상승만으로 기존 기준을 교체하지 않는다.

## Reference 보정의 독립 검사

80×100pt PDF에 같은 검정 60×0.12pt 사각형을 선언하고, 선을 그리지 않은 채 `w`만
기본값/0.14로 바꾼 입력을 비교했다. `f`/`f*` 두 fill 연산을 검사한다. shape에서는 농도가
바뀌거나 검정 fill이 완전히 사라져 정식 `PdfRasterFillStateTests`가 수정 전 실패했다.
none에서는 fill이 보존되고 무관한 선폭 상태가 결과를 바꾸지 않아 같은 검사가 통과했다.
별도 면적 계측은 PDF renderer 사이의 AA 차이도 보였으므로 이상적 면적과 완전히 같다는
주장을 하지 않는다. 판정 근거는 독립 fill 보존·stroke-state 불변성과 실제 복수 renderer 대조다.

`visual_sweep.py`의 reference 명령과 provenance 및 canonical 가이드를 none으로 함께
보정했다. 원래 shape 결과는 삭제/덮어쓰기하지 않는다. source PDF·제품 PNG·점수 함수·
이진화/공차/90% 조건은 변경하지 않았다. 원래 실패는 `raster-contract-before.log`,
통과는 `raster-contract-after.log`에 보존했다.

## 최신 devel 통합

전체 검증 전 fetch에서 devel이 `7d01ef610f`에서 `8766fb47d8`로 16개 커밋 전진했다.
Unicode 검색/편집 보정이며 이번 PDF 추론·renderer 및 reference 파일과 겹치지 않는다.
제품 소스가 바뀌므로 이 base를 통합하고 전체 검증을 한 번 수행한다. 기준/후보 producer는
기존 산출물을 덮어쓰기 전에 별도 보존하고 새 base의 Native/fresh Docker WASM을 준비한다.

## 실제 face 확인과 lint 보정

새 base의 실제 Skia renderer snapshot에 같은 공급 경로를 전달한 trace를 실행했다.
기본/혼잡 디렉터리에서는 본문 Batang 677문자가 system Noto Sans KR로 선택됐다.
Batang TTC만 든 디렉터리를 공급하면 같은 문자가 custom Batang으로 선택됐다. 별도 Gulim
5문자와 DotumChe 69문자의 fallback은 구분해 기록했다(`native-face-summary.json`).
전체 검증도 양쪽 producer에 동일한 Batang 디렉터리를 공급하며 원본 파일 hash를 고정한다.

root Clippy가 `table_detection.rs`의 bool-then filter_map 한 곳을 검출했다. 같은 순서·조건의
filter/map으로 바꾸며 경계/배치 계약이나 직렬화 값은 바꾸지 않는다. 실패 로그는 보존하고,
새 고정 head에서 root 및 후속 필수 lint를 통과한 뒤 후보 build·시각·전체 회귀를 진행한다.

## Docker source freshness

기준을 같은 named volume의 /app에서 빌드한 뒤 후보의 기존 backend source mtime이 더
오래되어 기준 artifact를 재사용했다. root는 새 API를 참조하지만 dependency는 그 API가
없는 상태로 컴파일이 실패했다. 컨테이너 안/밖 backend source SHA가 일치하는 것을 확인한
뒤 바뀐 네 backend 파일만 touch했다. 바이트는 그대로이며 package/target을 clean하지 않았다.
`candidate-docker-wasm-stale-backend.*`와 `docker-source-freshness.json`에 실패·조치를
보존하고 재컴파일 실행과 최종 실제 WASM을 확인한다. 실패한 산출물은 수용 근거로 쓰지 않는다.

## 최종 로컬 검증

- source: 기준 `8766fb47d8`, 통합 후보 `3c6fddb0b`. framework는 후보의 보정된 reference 명령이다.
- 기준 Native/fresh Docker WASM 13쪽: 모두 90% 이상이며 독립 PDF와 12·1쪽이 일치했다.
  최저 operators는 Native 90.53448% / WASM 90.17084%, 견적서는 99.85062% / 99.74444%다.
- Python Visual Sweep 157건, Studio 타입 검사 및 2,318건 통과(2건 skip), manifest 사전 검사 통과.
- root·WASM32·workspace all-target Clippy, workspace build, 고정 base manifest 검사 모두 통과했다.
- release lib는 workspace 합계 3,986건 통과(root 3,804건, ignore 12건)했다.
  전체 nextest는 10,825건 전부 통과했고 45건 skip이다. Native Skia lib는 4,043건
  통과(root 3,861건, ignore 12건), 지정 회귀는 누락 그림 2건과 직접 PDF export 14건
  모두 통과했다. doc test는 8건 통과·3건 ignore다. 서로 겹치는 실행의 건수를 합산해
  고유 검사 수로 보고하지 않는다.
- 긴 검증은 승인 뒤 실행했고 CARGO_BUILD_JOBS=4, nextest test-threads=6으로 공유 target을
  재사용했다. nextest 0.9.137의 권장 버전·선택 설정 경고는 남지만 실행 실패는 없었다.
- 최종 Python 검사에는 Windows에서 쓸 수 없는 `f*` 파일명을 `evenodd`로 바꾼 보정까지
  포함한다. PDF의 실제 `f*` 연산은 유지했다. 이 변경이나 타입 선언 순서·기록/이미지 추가는
  제품 Rust 바이트를 바꾸지 않으므로 이미 확인한 Native/Docker WASM 빌드·시각 캡처를 재사용했다.

## 후보의 전쪽·공개 API 검증

| 입력 | 전체 쪽수 | 후보 Native 최저 | 후보 fresh WASM 최저 | 같은 backend의 기준 대비 |
|---|---:|---:|---:|---|
| 원래 operators | 12 | 91.04017% | 90.62487% | 하락 없음. 3쪽 +0.50569 / +0.45403%p |
| Excel 견적서 | 1 | 99.85062% | 99.74444% | 동일 |
| 정상 2×2 격자 | 1 | 100.00000% | 99.53052% | 동일 |

모든 쪽의 raw 값이 90% 이상이고, 같은 조건의 base/후보 쪽수가 일치한다.
3쪽 외 실물 12쪽은 backend별 PNG가 기준과 같다. 이는 의미 표 1→0개 보류의 편집 범위를
면제하거나 지역 가설의 실제 표 생성을 완료했다는 판정이 아니다.

Native/Fresh WASM의 3·5쪽·견적서·정상 격자에 compare·standalone overlay·review를 만들고
직접 열었다. 3쪽 본문의 실제 Batang과 글자/간격을 2배로 대조했다. 견적서의 가는 선 위치는
동일하며 농도는 Native/MuPDF가 가깝고 Poppler none/Cairo와 AA 차이가 남는다. 이 차이를
숨기거나 원본 두께를 변경하지 않았으며 별도 확대 그림에 함께 두고 메인테이너 판정을 요청했다.
정상 격자는 네 셀의 내용/경계가 보존됐고 Native/WASM의 외곽 픽셀도 직접 확인했다.

최종 Docker WASM SHA-256은 `3c4754baf18f12b1b7284473b0960e427c09b77e62a8585b837130724a15b7fc`다.
실제 Studio 7793의 초기화 요청에서 이 응답의 바이트/SHA, 12·1쪽 Canvas, page error 0건을
확인했다. pkg/public의 네 파일을 맞췄다. d.ts는 같은 interface 안의 version/init 함수 선언
순서만 바뀌어 새 산출물로 동기화했으며 signature 변경은 없다. 타입·패키지 검사도 통과했다.

실제 Worker 7개·123 assertion 및 12쪽 render/save·오류 복구 검사가 새 WASM으로 통과했다.
CLI에서도 상세 옵션의 JSON 필수/중복 거절·기존 출력 보존, 기본 요약/선택 상세 4건을 확인했다.
새 구조 진단과 HWPX 저장본을 생성했고 원본 두 문서의 구조 레이어 검토 화면을 갱신했다.

- 비교/구조 검토: `E:\tmp\issue7998-table-review\current\index.html`
- 전쪽 비교: `E:\tmp\issue7998-table-review\current\visual\index.html`
- Studio: `http://localhost:7793/?review=issue7998-integrated-3c6fddb0&lang=ko`
- 실행 로그·producer·원본 TSV: 기존 증거 루트의 `integration-20261018/`
- 공개 가능한 TSV/대표 PNG: `mydocs/pr/assets/issue_7998/` (실제 견적서 원본/캡처 제외)

## 입력의 저장소 대응

`content_operators-002.pdf`의 바이트는 기존 추적 파일 `pdf/pr7704/6778-square-2020.pdf`와
완전히 같다(SHA-256 `d4c1fe7a8a3c025b2be3648ba6e2ad1123eff3dde008983edadcef7e8f7b6656`).
같은 PDF를 새 이름으로 중복 커밋하지 않고 해당 파일을 재사용한다. 그 PDF를 정답지로 쓰는
기존 HWP 회귀의 소스는 이번 PDF import 입력 경로와 다르며 HWP 조판 구현은 바꾸지 않았다.
새 정상 2×2 입력은 `pdf/issue7998/solid-control.pdf`에 포함돼 있다.
실제 견적서와 상세 문자 장부·연락처가 있는 캡처, 원본 글꼴은 승인 계획 1절에 따라 로컬로
보존한다. 공개 제출 자료는 SHA·집계 TSV 및 연락처 없는 대표 입력/계약으로 범위를 구분한다.

## 로컬 제출 준비와 남은 승인

`integration-20261018/full-results.json`에 고정 source의 전체 명령·종료값·실행 시간을,
각 step의 `.log`에 원문을 보존했다. lint·Studio·package·CLI·source audit 결과도 같은
디렉터리에서 연결한다. `submission-audit.json`은 제품 소스 hash 불변, pkg/public 네 파일
일치, primary 사용자 피드백 불변과 공개 자산에 견적서 원본/캡처·글꼴이 없음을 확인한다.
공개 자산의 12 TSV는 원본 바이트이며 12 PNG는 이번 고정 source의 대표 review/overlay다.

로컬 한국어 PR 본문·등록 명령은 같은 증거 디렉터리의 `pr-body-ko.md`·`remote-plan-ko.md`에
준비한다. 정확한 최종 commit URL로 대표 이미지를 표시하고 원본 TSV를 연결한다. 이 단계에서
GitHub PR·review/오늘할일을 만들지 않으며 CI 성공이나 이슈 종료를 선기록하지 않는다.
현재 절편은 `Refs #7998`로 제출하며 새 지역 가설의 실제 Builder/IR 소비와 나머지 B01–B06은
후속이다. 직접 시각 판정 결과와 원격 push·PR 생성 승인을 받은 뒤 해당 절차로 진행한다.
