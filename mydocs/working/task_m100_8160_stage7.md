# #8160 단계 7 — #8190 반영 후 리베이스와 재검증

## 최신 base 전진과 보존

PR #8188의 code head `fbb42fbe649b8a1b592edca5107624d68d8ffb92`는 d9 기준 Full CI·CodeQL·
Render Diff·Adapter·Proptest·CI Impact Policy를 모두 통과했다. 그 뒤 작성한 문서 trailing
`d7beeee89f7d79f190667a3f0ebace9c8fadfccd`는 push하지 않았다. push 직전 live base 확인에서
#8190이 병합된 `e7a0dec55c6c6c6365e4ff50babe7583e1cf1c70`을 발견했고,
Cargo workspace exclude와 오늘할일의 실제 충돌을 확인해 진행을 보류했다.

사용자의 기존 rebase·전체 검사·자동 승인·후속 처리 지시에 따라 같은 작업을 계속한다.
추가 확인을 기다리지 않으며, 기존 검증 결과를 새 base의 결과로 바꾸지 않는다.

- 현재 branch: `task/8160-sample16-raw100`.
- 새 base: `e7a0dec55c6c6c6365e4ff50babe7583e1cf1c70`, PDF JBIG2/JPEG 2000 opt-in PR #8190.
- rebase 전 코드 후보: `fbb42fbe649b8a1b592edca5107624d68d8ffb92`.
- rebase 후 첫 후보: `a41ae037ab5413036bae3544326ce9beddf693df`.
- 미게시 trailing·기존 코드 보존 branch: `task/8160-sample16-before-e7-rebase`.
  이전 최초 rebase 보존 branch도 유지한다.
- 검증한 bundle: `/home/tsjang/rhwp-local-verification/issue8160/issue8160-before-e7-rebase.bundle`.
  prerequisite 605이며 d7·이전 67 head와 source0c·803 이력을 복구할 수 있다.

미게시 검토·오늘할일은 보존 branch에 남기고 실제 green code head부터 rebase했다.
현재 source에는 #8190의 오늘할일을 그대로 상속했으며, #8188의 새 review·오늘할일은 새 code CI
성공 뒤에 작성한다. PR 본문에도 새 base 충돌과 재검증 중인 상태를 표시했다.

## 실제 충돌 해소와 기능 보존 계획

28개 commit을 같은 순서로 rebase했다. range-diff에서 27개 patch는 같고 한 개의 변경은
Cargo exclude 문맥에 새 base의 네 codec vendor와 이번 JPEG encoder를 함께 보존한 부분이다.
source `renderer`·`paint`·`wmf`·Studio 및 전쪽 검사·fixture·세 원본은 이전 green 후보와 같다.
이 확인은 실행 검증을 대체하지 않는다.

1. locked root/독립 Skia metadata와 source 후보 graph를 확인한다. 새 base가 추가한 의존성과
   기능을 보존하며 실제 lock·graph 누락은 Cargo가 생성한 최소 변경으로 수정한다.
2. #8190의 개별 feature·지원 정보·예산·실패 후 복구·저장 보존을 기존 정식 검사와 실제 CLI/WASM
   진입으로 확인한다. default·JBIG2·JPX·양쪽 구성을 구분하고 base의 독립 PDF·bitmap을 사용한다.
3. 새 source에서 필수 Rust lint·workspace·정책·전체 nextest·Skia·release/doc·패키지·Frontend를
   정본 순서로 완료한다. 기존 `target/pr-review`를 순차 사용한다.
4. 새 Native와 fresh 표준 최적화 WASM을 실제 빌드·동결하고 세 문서 전쪽 384개 PNG·TSV를
   재산출한다. threshold 232·2px·양방향 정수 0을 그대로 요구하고 중요 영역을 직접 판독한다.
5. 25개 집중 회귀·192쪽 자동 보호·정상 대조군과 실제 browser 소비를 확인한다.
6. 모두 통과한 뒤 이전 remote head fbb에 대한 명시적 lease로 같은 PR head를 갱신한다.
   새 정확한 code head CI 성공 → review·양쪽 오늘할일 보존 → trailing CI → 일반 merge·후속 처리다.

원본·기준 Print PDF는 새 MCP로 이미 생성한 같은 바이트를 사용한다. e7의 PDF opt-in 검증은
다른 원본이며 이번 세 HWP/HWPX 384쪽의 100%와 섞지 않는다. 기존의 광범위 검증·0c 산출물과
실패한 도구 로그는 역사적 사실로 보존한다. 진행 중인 새 검증을 완료로 표시하지 않는다.
