# CodeQL 조판 진단 출력 보정

## exact head와 선행 해결 이력

제출 head `ab1015668ec888bfbb96ec4679f7c648d0ab576d`의 Actions Full CI·CodeQL 네 언어·Render Diff·Adapter·Proptest와 최종 정책 감사는 성공했다. 별도의 GHAS `CodeQL` check `126910511432`는 High 2건 때문에 실패했다. 제품 CI 성공을 보안 체크 성공으로 대신하지 않고 merge를 보류했다.

실제 PR merge ref는 `323280a80c16bb1eb5bc14d5eabbef0e61a72abc`, Rust analysis ID는 `2099065996`, CodeQL 2.27.4이며 결과는 126개다. PR diff가 건드린 두 sink는 기존 alert #481·#482이고, PR merge-ref open은 12개다. 기본 branch open 목록이 빈 것과 이 PR 경고가 없는 것은 다르다.

앞선 [#8548 보정](../semanticist21-20261103/ci-head223-followup.md)과 실제 제품 commit `3af2224e4e`·`5e374d01f641745d20bc9424ce233021a327ed16`을 추적했다. 독립 Rust analysis `2093550961`에서 해당 세 sink가 사라져 결과 129→126·open 15→12가 된 근거와 현재 base에 포함된 코드를 대조했다. 이번 두 sink는 그 보정 범위에 포함되지 않았으며 base에도 그대로 있다. 이전 분류 승인을 재사용하지 않는다.

## 실제 데이터 흐름과 보정 범위

| alert | 제출 head sink | flow / location 합 | 내용 |
| --- | --- | ---: | --- |
| #481 | height_measurer.rs:4654–4665 | 4 / 314 | DIAG_TRUST: 셀 줄 높이·겹침·저장 extent 등 |
| #482 | height_measurer.rs:4925–4951 | 4 / 289 | DIAG_ROWH_LINE: 저장 줄 위치·높이·간격·글꼴 크기 |

8개 flow·603개 location을 [정규화한 실제 SARIF 경로](codeql-blocking-flows.json)에 보존했다. 모두 NativeSession의 `from_bytes_with_password` 반환 Core에서 DPI 또는 LayoutEngine.dpi로 전파되어 수치 변환과 stderr에 도달한다. Core의 DPI와 LayoutEngine은 `commands/document.rs:426/430`에서 DEFAULT_DPI=96으로 초기화되며 API set_dpi도 수치 입력이다. 원 암호 바이트를 그대로 출력하는 경로로 기록하지 않는다. 다만 앞선 보안 정책대로 보호 문서의 파생 조판 정보를 진단 stderr로 내보내는 선택적 채널을 제거한다.

`RHWP_DIAG_ROWH`의 두 기존 진단 그룹과 같은 채널의 Native span-height 진단을 제거한다. 실제 required_height·combined·행 높이 solver·줄 소유·분할·배치·본문 점유·저장 계산은 유지한다. 진단용 재구성/로그 전용 변수는 함께 제거하되 일반 렌더 계산은 바꾸지 않는다. query·언어·경로·보안 설정·alert 분류·workflow·검증 기준은 바꾸지 않는다. [제거 블록의 줄 수와 해시](rowh-diagnostic-removal.json).

## 실제 음성 대조와 검증 계획

보정 전 실제 Native binary(기존 시각 producer `2f44d7d0ab`)로 공개 실제 암호 HWPX fixture를 stdin 인증해 SVG로 내보냈다. 최소 글꼴 환경에서 RHWP_DIAG_ROWH·RHWP_DIAG_ROWH_LINES·RHWP_LAYOUT_DEBUG를 켰을 때 DIAG_TRUST 24회·DIAG_ROWH 24회를 출력했고 SVG 24쪽을 정상 생성했다. 새로운 로그 부재 계약은 이 binary에서 FAIL이다. 줄별 DIAG_ROWH_LINE·SPANH는 이 fixture에서 실행되지 않았으므로 그 경로의 runtime before-FAIL로 주장하지 않는다. [실측](security-rowh-before.json).

기존 `hwpx_password_fixture`의 실제 암호 CLI 계약에 두 환경변수와 새 진단 prefix 부재 검사를 추가한다. 테스트 함수·수량과 인증·복호화·정상 SVG 생성 보호는 유지한다. 수정 후 실제 binary의 로그 부재와 24쪽 SVG 동일성을 대조하고, 전체 최소 글꼴 회귀·Rust lint·Native CI의 실제 22개 명령 및 Native 전체 라이브러리를 검증한다.

## 조판 증거의 재사용

이번 제품 diff는 opt-in 진단 출력 블록과 그 안에서만 수행하던 진단 재구성 제거다. 해당 환경변수가 없는 일반 경로의 계산·상태 갱신·최종 출력은 같다. 8건의 기능 문서 Native/fresh WASM 75쪽씩, 성능 문서 39쪽씩과 편람 383쪽의 실제 producer SHA·전쪽 TSV·직접 판독을 그대로 연결한다. 새 raster를 측정했다고 기록하지 않는다. 실제 암호 fixture의 SVG 보존은 별도의 출력 채널 계약이며 독립 Print 피델리티 주장으로 쓰지 않는다.

새 source의 로컬 검증과 독립 CodeQL 결과를 확인하기 전에는 이 보정을 완료로 판정하지 않는다. 나머지 기존 open 10건을 이 PR에서 모두 해결했다고 주장하지 않는다.
