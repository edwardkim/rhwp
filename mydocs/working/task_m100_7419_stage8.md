# #7419 Stage8 — 기준 쪽수 결함의 판정 정정과 PR 실행 계획

## 사용자 판정과 승인

2026-10-16 사용자가 "devel 이 잘못된거잖아 그건"이라고 지적했다. 작은 용지·partial 폭 부족·
wrapper의 기준2/1/1쪽은 정상 출력 기준이 아니라 이번 수정이 복구한 결함이다. 현재 출력과
독립 한컴 MCP Print는3/2/2쪽으로 일치한다. 이를 현재 출력의 회귀처럼 다뤄 PR을 계속
차단한 판단을 정정한다. 과거 Stage3–7의 보류 기록은 당시 판정으로 보존한다.

이 세 합성 반례만 기준 쪽수 결함 복구로 수용한다. 기준에 없는 쪽의 점수를 만들어내거나
TSV를 통과로 덮어쓰지 않는다. 다른 입력·페이지·출력 경로의0.01%p 하락 기준과 현재 출력의
독립 Print 쪽수·90/98%·내용 보존·직접 확대 판독 조건은 유지한다.

- 순수 기준 SHA: `0953598d8e08af5e9149b3e4cdbbbf2f5f2bae64`.
- 실제 검증 renderer source: `f5a4774049fa10cc661f7220de0127d61fa081ba`.
- 직전 기록 head: `b36175f09c18d9625007d6585cee03313dc26295`.
- Native/host fresh WASM13문서19쪽씩 최저100%·독립 Print 쪽수 일치·직접 판독 완료.
- 쪽수 복구3문서도 새 MCP Print7쪽 전부 기존 Print와96dpi RGB 픽셀이 동일하다.
- 같은 쪽이 있는10문서12쪽/경로와 추가 진단8공통 쪽 모두0.01%p 이상 하락0이다.
- 존재하지 않는 기준6행은 계속NA다. 전체 일대일 비교 성공으로 보고하지 않는다.

사용자가 PR 생성 후 code candidate CI 완료 → 오늘할일·PR review trailing commit →
그 head의 CI 확인 → merge → 후속 처리 순서를 승인했다. PR·push·CI·merge·후속 처리를
이 순서로 수행하며 아직 실행하지 않은 결과를 성공으로 기록하지 않는다.

## 첫 기여자 처리와 실행 순서

작성자imsebeom은 실제 GitHub FIRST_TIME_CONTRIBUTOR다. 원 PR7433 head와 fork ref는
`d895e51ac5fd1621abb99f0a20aa52034e01e083`로 같고 reviewer는jangster77이다.
사용자 지시의 devel 기반 체리픽 통합 경로를 유지한다. 원 기능/검사의 author와 출처를
보존하며 contributor fork를 rewrite·push·삭제하지 않는다.

1. 최신 원격 devel과 통합 head를 고정하고 merge simulation·공백·링크·LFS 판독을 마친다.
2. 원본 저장소의 임시 head branch에 push하고 devel 대상 통합 PR을 생성한다.
   원PR의 기여와 보정 범위·첫 기여 감사/환영·실제 로컬 결과·대표 review/overlay를 본문에 적는다.
3. 정확한 code candidate의 Full CI·CodeQL 및 필요한 별도 check의 실제 성공을 확인한다.
   실패는 같은 승인 범위에서 보정하고 관련 로컬 검증 뒤 새 head로 재확인한다.
4. 원PR별 archive review와 오늘할일의 최종 판정·실제CI URL을 같은 branch의 문서-only
   trailing commit으로 기록한다. 통합PR 채번용 별도review·문서PR은 만들지 않는다.
5. trailing head의 preflight/최종required CI·mergeability를 실제 확인한 뒤 정상 merge한다.
   admin bypass는 사용하지 않는다.
6. post_merge.md 순서로 devel 반영·이슈7419·원PR7433의 실제 상태와 comment를 처리한다.
   한국어 존댓말로 감사·rhwp 첫 기여 환영·보정 필요 사유·실제검증을 구분하고 merge SHA의
   실제 review/overlay 이미지를 포함한다.7416의 별도 문제는 닫지 않는다.
7. 필수 TSV·로그·재현 자료를 output 밖에 보존한 뒤 실행 중인 다른 작업의 소유를 확인하고
   output 루트 전체 및 이번 임시 branch/worktree를 정리한다. 공유 target/pr-review와 사용자
   Studio JS·다른 작업의 자료·contributor fork는 보존한다.

Docker 미검증 유지 선택은 그대로다. 기존 host dev/no-opt fresh WASM을 Docker 성공으로
보고하지 않는다. Rust 코드·검사·입력·생산자는 Stage6 이후 불변이므로 문서·동일 픽셀의
새 Print 반영만을 이유로 전체 로컬 Rust 회귀를 반복하지 않는다.
