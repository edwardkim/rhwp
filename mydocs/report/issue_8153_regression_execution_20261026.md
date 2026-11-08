# #8153: Native·브라우저 회귀 실행 연결

[계획](../plans/issue_8153_regression_execution_20261026.md)의 실제 누락 3개 Native case와
브라우저 검사 1건을 기존 실행 경로에 연결했다. 새 job·step·Cargo target을 만들지 않았고
일반 회귀에서 이미 실행하는 case를 추가 호출하지 않는다. 기존 Native job의 release-test와
release 분기는 상호 배타적이며 실행당 신규 검사는 2+3+1=6건이다.

검증 source `be1b4f2ddb`의 Native selector plan은 mixed-axis → suite 026,
Print glyph → suite 021, symbol fallback → suite 019다. 각 파일의 모듈 이름 필터와 `native-skia` feature를
같이 전달한다. 기본 feature에서 수집된 것을 실행으로 세거나 0건 실행을 통과로 세지 않는다.
공용 브라우저 진입점은 기존 단독 검사를 import해 같은 실제 Canvas 계약을 실행한다.
sample16 관련 기존 삭제나 다른 문서의 검사 범위는 바꾸지 않는다.

최초 workflow 계약 검사는 세 추가 target이 CI 영향 분류기에 없어서 3건 실패했다.
호출을 추가하는 것만으로 해당 파일 변경 시 job이 실행되는 것은 아니었다. 분류기 소유
목록에도 세 원본 경로를 등록하고 기존 동작 기반 검사에서 파일별 Rust/Native 실행,
Canvas 비실행 및 분류 이유를 확인했다. 이 오류와 최초 로그를 보존했다.

- actionlint: exit 0.
- Rust selector 잠금·Cargo 격리 Node 검사: 4 PASS/0 FAIL.
- CI 영향 분류·정책·trusted reuse Node 검사: 333 PASS/0 FAIL.
- workflow 계약 Python 검사: 보완 뒤 257 PASS/0 FAIL.
- 세 실제 Native case 명령: 검토 worktree의 같은 source에서 순차 실행해 각각 2·3·1건 PASS,
  실패·skip 0, 각 명령의 최종 exit 0을 확인했다. 첫 명령 580.47초는 source 경로가 다른
  검토 worktree의 빌드와 실행을 포함하며, 다음 두 명령은 32.18초·21.42초다.
- 새 표준 최적화 WASM을 지정한 기존 공용 브라우저 진입점: 실제 8건 PASS, 실패·skip 0.

수정한 workflow·분류기·브라우저 진입점은 기존 필수 검사나 feature를 제거하지 않는다.
정책 검사와 실제 Native/브라우저 결과를 각각 기록했다. 새 source 원본이나 검사 본문이
바뀌면 최종 제출 source의 파생 suite를 다시 준비하고 논리 selector로 실행한다.
suite 번호는 생성 결과이며 고정 계약이 아니다. 일례로 기존 #6311 검사 정정 중의
파생 계획에서는 symbol fallback을 suite 024에 배정하는 것도 확인했다.
임시 selector plan과 실행 로그는 ignored `handbook-cell-none-candidate/execution-wiring/` 및
작업 루트 `logs/handbook-regression-execution-*`에 보존한다.

고정 base는 `9418ed0c7cf289c5c4ac2b3bb8cdc18a1b637a9a`다. 사용자 지시로 upstream
fetch·동기화를 하지 않았다. 로컬 연결 보정 단계이며 원격 CI·게시·PR 제출 완료가 아니다.
전쪽 100% 및 나머지 필수 검증을 완료한 뒤 PR 준비까지만 진행한다.
