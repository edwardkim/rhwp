# #8946 호스트 글꼴 조판 참여 구현 계획

수정된 #8946의 명시적 옵션과 메트릭 정책을 기준으로 진행한다. 기준 소스는
`99b9d7ec5ca36884ea8fd981ac815c1edc34c653`이다.

## 구현 계약

- `setProvider(provider, { layout: true })`로 Canvas2D 조판 참여를 활성화한다.
  옵션 생략과 false는 기존 계약이다. 설정은 연결 단위이고 문서 교체에도 유지하며,
  공급자 재연결마다 새 옵션으로 정하고 null 연결 해제 시 초기화한다.
- 실제 프로그램의 선택은 기존 Rust 선택 계획과 환경 생성 API를 소비한다.
  자동 `preserveSourceMetrics`와 명시적 `replaceSourceMetricsFor`를 유지한다.
  저장 LineSeg와 원본 메트릭 정책의 일괄 변경을 도입하지 않는다.
- 준비한 동일 파일을 환경 선택·측정·paint에 공급하고 연결 세대, revision,
  face ID, 파일 SHA와 faceIndex를 보존한다. 늦은 요청은 문서·공급자 세대로 폐기한다.
- 최초 열기와 갱신·해제에서 같은 자동 환경 경로를 사용한다. 명시적 사용자
  환경은 소유권 검사로 보호하고 자동 환경과 자원만 정리한다.

## 원인과 소비 경로

원본 요청은 getDocumentFontSelectionPlan에 보존되나
prepareDeclaredFontEnvironment의 활성 호스트 검사에서 중단된다.
prepareCanvasMetrics는 이미 styleSubstitution을 거친 layer tree를 조회하므로
원본 딸기 요청이 누락된다. 로컬 경로의 실제 프로그램 snapshot →
buildFontEnvironmentFromAvailablePrograms → setFontEnvironment →
파생 상태 재구성과 Canvas 메트릭·paint 경로에 승인된 호스트 프로그램을 연결한다.

## 검증

기존 v0.9.0 재현은 원인 근거로 보존한다. 최종 판정에는 고정 base와 후보의
동일 입력·브라우저·글꼴 조건 대조를 사용한다.

1. 옵션 기본값·연결 수명·지연 응답, 프로그램 SHA와 TTC face, 메트릭 정책 검증.
2. TypeScript와 Studio 전체 Node 검사 및 Canvas2D/CanvasKit 기존 수용 검사.
3. 최초 열기·열린 문서 옵션 변경·A/B 교체·제거/재공급·해제·문서 교체.
4. 저장 조판/편집 재조판, 명시 환경·내장 글꼴·실패 대조, 저장/dirty/undo/redo.
5. 영향 문서 Native/fresh WASM 전쪽 TSV, 독립 Print 쪽수와 내용 대조,
   변경 영역 직접 확대 판독 및 최종 head의 적용 검증 게이트.

실제 실행 결과는 작업 기록에 추가한다. 이 계획의 검증 항목은 실행 완료 표시가 아니다.
