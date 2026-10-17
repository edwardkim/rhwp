# #7998 기존 출력 차이 보정과 전체 검증

- 앞 단계: [1차 구현·검증](task_m100_7998_stage1.md), 코드 `1d54f8a09`, 검사/기록 `876597051`.
- 2026-10-18 작업지시자가 기존 출력 차이의 이번 범위 포함 및 전체 검증·로컬 PR 준비를 승인했다.
- 1차 전쪽 PNG·TSV·빌드와 실제 Worker 증거를 보존하며, 같은 제품 코드는 다시 빌드하지 않는다.
- 증거 루트: `/home/edward/mydev/rhwp-review-evidence/pdf-table-feedback-20261017/output-fixes/`.
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
