# planet6897 19개 PR 통합 검토 증적

기준은 `5ea1227505978156082121f1601cb03044a0583f`, 검증 제품 코드는
`21d62361fa670858e63fdfc475c236f112e20a3d`다. #8854는 이미 devel에서 해결되어 통합에서 제외했다.

최신 Native 44조건 2,109쪽 검증이 완료됐다. 90% 미만·쪽수 불일치·보호 기준 하락은 없다.
원 PR 입력 27조건 721쪽과 정본 보호 18조건 1,771쪽은 편람 HWPX 383쪽을 공유하며,
같은 producer의 결과를 한 번 측정해 두 역할에 연결했다. 조건별 전체 값은
`native-final-validation.json`, 원 PR별 TSV와 같은 쪽의 raw·PDF·review·overlay는 각 `cases/`에 있다.

직접 판독에서 발견한 #8752의 2쪽 그림 뒤 줄 가림과 #8734의 HWP5/HWPX 27쪽 명령문 줄 합침은
`0f2ebe265e`에서, 고용서비스 보고서 79쪽의 본문·그림·캡션 겹침은 `21d62361fa`에서 공통 프레임
소유 규칙으로 보정했다. 이 네 조건의 같은 쪽 Native/fresh WASM은 각각 100.00%이며,
`pr8752/maintainer/`와 `pr8734/maintainer/`에 전후 증적과 실제 FAIL→PASS·반례를 보존한다.
현재 코드의 관련 집중 회귀는 19개가 통과했다.

최소 글꼴 환경 전체 nextest는 13,227개 통과·실패 0개·45개 건너뜀이다. Native Skia 루트 3,852개와
관련 라이브러리 182개·추가 그림/PDF 검사, 세 Clippy·workspace build·fmt·manifest·unit tiers·doc tests도
최종 단계에서 통과했다. 상세 결과와 파생 suite 준비 오류의 해소 이력은 `rust-final-validation.json`에 있다.
원 PR 입력 27조건의 base 대비 2%p 초과 하락이 없음은 `native-base-comparison.json`에서 확인한다.

**제출 전 검증 진행 중:** fresh WASM 전쪽·실제 Studio/SVG 소비 경로의 최종 검증과
원 PR별 리뷰 문서는 아직 완료 전이다. Native 결과를 이들 경로의 통과로 대신하지 않는다.
100.00%는 2px 내용 실루엣 지표이며 RGB 전체 픽셀 동일을 뜻하지 않는다.

- `native-claim-index.json`: 원 PR·입력별 전쪽·주장·최저 페이지 목록.
- `protection/native/`: 현재 제품 코드의 정본 보호 18조건 전쪽 TSV와 판정.
- `code-source-association.json`: 고정 binary/package와 커밋된 실제 제품 코드의 해시 대응.
- `native-direct-reading-notes.json`, `native-direct-reading-reuse-audit.json`: 현재 Native 증적 65쪽의 직접 판독과 동일 raw/PDF 재사용 근거.
- 각 `provenance.json`: 실행 producer·입력/PDF·PNG SHA-256.
- 각 원 PR의 `maintainer/`: 단계별 실제 실패 검출·보정·전후 증적.

고용서비스 보고서 51쪽의 기존 표 하단 경계와 원 HWP3 27쪽 캡션의 기존 차이는 해당 판독 기록에서
이번 수정과 구분했다. #7282의 원 PR이 제외한 KGS 서식까지 해결했다고 표시하지 않는다.
글꼴 바이너리·로그·임시 스크립트는 이 자산에 포함하지 않는다.
