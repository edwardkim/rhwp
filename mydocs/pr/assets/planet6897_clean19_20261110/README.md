# planet6897 19개 PR 통합 검토 증적

기준 `5ea1227505978156082121f1601cb03044a0583f`, 현재 검증 코드
`009b3cf42bf1d4105ff3279c1211613baf088359`의 중간 검토 기록이다.
#8854는 이미 devel에서 해결되어 통합에서 제외했다.

Native 44조건 2,109쪽의 TSV는 누락·중복·쪽수 불일치·90% 미만이 없고,
정본 보호 18문서의 공식 표시값 하락도 없다. 각 원 PR의 `cases/`에는 해당 주장 쪽의
raw·PDF·review·overlay를 함께 보존한다. 최소값 쪽은 주장 쪽과 역할을 구분했다.

**제출 보류:** 직접 판독에서 #8752의 2쪽 저장 줄이 그림 뒤에 가려지는 것을 확인했다.
96.18%라는 점수로 완료를 대신하지 않으며 이 결함의 공통 경로 보정·회귀 검사·재검증을 진행한다.
fresh WASM 전체·실제 소비 경로 검증과 최종 개별 리뷰 문서는 아직 완료 전이다.
이 Native 기록을 이후 코드의 통과로 재인용하지 않는다.

- `native-claim-index.json`: 원 PR·입력별 전쪽/주장/최저 페이지 목록.
- `protection/native/`: 정본 보호 기준과 비교한 현재 Native 전쪽 TSV.
- 각 `provenance.json`: 코드·실행 producer·입력/PDF·PNG SHA-256.
- `native-direct-reading-notes.json`: 직접 판독 기록과 확인한 미완료 항목.
- `native-direct-reading-reuse-audit.json`: 이전 직접 판독과 현재 raw/PDF의 바이트 동일성 감사.
- `pr8734/maintainer/`, `pr8839/maintainer/`: 단계별 실제 FAIL→PASS·수정 전후 증적.

글꼴 바이너리·로그·임시 스크립트는 이 자산에 포함하지 않는다.
