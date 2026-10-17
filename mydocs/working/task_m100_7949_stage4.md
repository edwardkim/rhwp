# Task #7949·#7950 — SMask·Matte와 실제 예약량 보정

- Issue: #7949, #7950
- 작성일: 2026-10-16 (Asia/Seoul)
- 제품 source: `1af44c874` 이전까지의 구현과 동일 source의 Native 검증
- 상태: 마스크·PNG 자원 계약 통과. 새 문서 전체 열기는 후속 통합 보정 중

SMask의 1·2·4·8비트 회색 alpha, Decode, RGB·회색 Matte를 구현했다.
고해상도 mask와 저해상도 색상을 같은 단위 사각형에 합성하되 두 해상도를 보존한다.
원래 bbox·CTM은 유지하고 crop은 최종 bitmap 크기를 소비한다. codec 선언 크기 검사는
합성 전에, 실제 합성 픽셀 수 검사는 이후에도 수행한다.

- 수정 전 CLI: SMask 재현 9건 모두 `PDF_IMAGE_UNSUPPORTED`.
- 수정 후 실제 import→HWPX→PNG: 9건의 alpha·색상·해상도 통과. JPEG Matte 포함.
- 정식 PDF resource 검사: **111/111 통과**(`resource-budget-final.log`).
- RGBA를 포함한 해제된 샘플은 PNG에 빌려 준다. 실제 PNG scratch·출력은 계속 예약하며,
  존재하지 않는 이미지 크기만큼의 decode/변환 예약과 Mask RGB 복사를 제거했다.
  256MiB 누적 한도·32MiB 버퍼 한도·sticky 오류는 유지했다.

새 PDF를 전쪽 실행하니 마스크 외의 기존 경계가 드러났다. 쪽별 실행은 17/19쪽 성공이며,
13쪽은 아주 작은 CTM off-diagonal·음수 원점, 19쪽은 표 추론의 선분 한도가 원인이었다.
별도 중간 계측에서는 15쪽까지 반복되는 해석/그림 작업이 연산 한도에도 도달했다.
중간 계측과 실제 제품의 첫 오류는 구분한다. 현재 제품 전쪽은 `PDF_LOWERING_FAILED`다.

사용자가 중복 처리 개선까지 이번 범위에 포함하도록 추가 승인했다. 다음 단계에서 동일 페이지의
선·그림 소비자가 공유할 파싱 결과와 실제 예약량을 보정하고, 좌표 형식 및 선택적 표 추론의
경계도 확인한다. 아직 19쪽 열기·시각 gate 완료나 PR 준비 완료로 보고하지 않는다.
입력·중간 producer·로그는 ignored `output/pr-review/issue7949/type002/`에 보존했다.
