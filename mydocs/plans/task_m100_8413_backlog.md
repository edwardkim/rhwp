# #8413–#8416 PDF 가져오기 후속 백로그

2026-11-01 메인테이너 지시: 남은 큰 문제는 백로그로 기록하고 이번 PR을 Open·self-review한다.
곡선 clipping·회전 텍스트·마스크, 암호 인증, 자원 정책과 쪽별 진행률은 이번 제출 범위다.
아래 세 항목은 후속 대기이며 이번 PR에서 해결했다고 주장하지 않는다. 이 분리는 현재 검사의
실패나 신규 회귀를 면제하지 않는다. 후속 구현과 원격 신규 이슈 등록은 별도 진행한다.

근거는 [구현·검증 기록](../working/task_m100_8413_implementation.md)과
[전쪽 TSV·해시·측정값](../pr/assets/issue8413/curve-mask-validation-20261101/manifest.json)이다.
관측 제품 소스는 `f17ca45e14`이며 리베이스 후 `5ef817b434`와 제품 파일 내용이 같다.
입력은 [공개 재현 자료](../../samples/issue8413/README.md)에 연결한다. 담당은 메인테이너
`edwardkim`, 구현 담당·일정은 미정이다. 우선순위는 착수 제안이며 납기 약속이 아니다.

## B01 · P1 — 복잡한 PDF의 Studio 초기 표시 지연

- 상태: 대기, 실제 지연 관측. 원인별 비용 분해·개선량은 미확정.
- 입력: `samples/issue8413/f87pro.pdf` 1쪽. 표준 프로필, 대화창 제한 시간 300초로 가져온다.
- 관측: 실제 Studio의 가져오기부터 표시 완료까지 139.95초. Native 가져오기·HWPX 저장은
  10.19초, WASM 가져오기·HWPX/HWP 저장 및 재열기 묶음은 53.56초다. 서로 다른 범위의
  단일 관측이며 단순 차감으로 조판 비용을 계산하거나 보편적인 성능 수치로 쓰지 않는다.
- 조사 진입점: `rhwp-shared/pdf/import-session.js`의 응답·재열기, Studio `src/core/pdf-open.ts`,
  `src/wasm_api/pdf_request.rs`의 editor reopen, 최종 문서 조판·paint·초기 화면 표시.
  Worker 완료와 UI 완료를 구분하고 실제 long task·객체 수·복사량을 계측한다.
- 처리: 변환, 직렬화/전달, 편집기 재열기, 조판, 첫 화면 표시를 같은 입력·환경에서 각각
  측정한다. 원문 도형이나 편집 가능성을 버리는 전체 페이지 bitmap 변환으로 우회하지 않는다.
- 완료 조건: 고정 장비·글꼴·WASM에서 반복한 전후 단계별 시간과 UI 응답성을 제시하고,
  취소·시간 초과·오류 뒤 정상 요청, 1쪽 내용·저장 재열기를 유지한다. 같은 전쪽 Native/WASM
  비교에서 90% 이상·devel 대비 2%p 초과 하락 없음·직접 판독을 확인한다.
- 현재 안내: 큰 입력은 제한 시간을 명시적으로 늘릴 수 있으나 초기 표시 성능 해결은 아니다.
  기본 30초가 모든 대형 입력에 충분하다고 안내하지 않는다.

## B02 · P2 — PDF 본문의 글꼴 굵기와 영문 간격 차이

- 상태: 대기, 실제 시각 차이 관측. 회전 방향·char ratio 수정과 별개다.
- 입력: `samples/issue8413/PDF_ENCRYPTION_UNSUPPORTED.pdf` 57쪽, 특히 55쪽 본문과
  알려진 차이 영역. 전쪽 raw 최저는 Native 91.72% / WASM 91.64%다.
- 관측: 올바른 원본 글꼴 프로그램을 로컬 공급해도 일부 굵기·영문 간격이 다르다.
  전체 쪽수·내용 보존이나 90% 점수만으로 글꼴 표현 완료로 판정하지 않는다.
- 조사 진입점: backend `extractors/text.rs`의 원본 glyph advance·text matrix,
  `src/parser/pdf/lowering.rs`의 글꼴 식별·문자 폭·공통 IR, 최종 renderer의 글꼴 소비.
  이름 alias, 임베디드 subset, fallback, 실제 advance·장평·문자 간격을 구분한다.
- 처리: 먼저 원본 프로그램과 metrics·글꼴 선택을 재현 가능한 조건으로 고정하고,
  생산 → IR → 측정 → 실제 paint의 차이가 생기는 계층을 찾는다. 샘플별 px 이동이나
  시스템에 우연히 설치된 글꼴을 요구하는 회귀를 추가하지 않는다.
- 완료 조건: 같은 본문을 동일 배율로 직접 비교하고 굵기·문자 간격·기준선·내용 순서를
  확인한다. 전쪽 Native/WASM TSV와 정상 fallback 대조군을 유지하며, 배포할 글꼴은
  재배포 허가를 별도로 확인한다. 공개된 PDF의 글꼴 추출은 글꼴 재배포 허가를 뜻하지 않는다.

## B03 · P2 — 객체별 래스터화한 가는 벡터 선의 농도 차이

- 상태: 대기, 출력 해상도·coverage 차이 관측. 선 누락으로 단정하지 않음.
- 입력: `samples/issue8413/f87pro.pdf`의 키보드 세선 영역. 전쪽 Native/WASM raw 99.82%.
- 관측: 원본의 0.135pt 선은 PDFium 96dpi 직접 출력과 216dpi 객체별 PNG를 축소하는
  현재 경로에서 농도가 다르다. 독립 PDFium 216dpi 확대에서는 같은 선의 존재·형상을 확인했다.
- 조사 진입점: `src/parser/pdf/path_paint.rs`의 객체별 raster, PNG 보간 출처와
  Native/Canvas의 최종 sampling. clip·alpha 보존과 선 coverage를 각각 확인한다.
- 처리: 출력 배율·DPI별 stroke coverage와 resampling을 분리해 측정하고 가능한 공통
  표현 개선을 검토한다. 특정 원본의 선 폭이나 회색값을 임의로 보정하지 않는다.
- 완료 조건: 96dpi와 확대 출력에서 교차선·곡선·투명 마스크·그림 경계를 직접 비교하고
  Native/WASM 모두 정상 대조군의 색·alpha·성능을 유지한다. 후속 보정 뒤 같은 입력으로
  전쪽 TSV와 관계형 회귀를 갱신한다.

## 인계 범위

원본 PDF는 2026-11-01 메인테이너의 공개 커밋 승인에 따라 원래 바이트로 제공한다.
검증용 추출 글꼴·개인 글꼴 프로그램은 포함하지 않는다. Native peak RSS, WASM linear memory
capacity와 누적 decoded 예산은 서로 다른 값이며 총 메모리 상한의 증거로 혼용하지 않는다.
이 백로그는 기록이며 세 항목의 구현·성능 개선·CI·머지 완료를 뜻하지 않는다.
