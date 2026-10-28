# #8174 T07 완료 범위와 종료 조건

2026-10-28. 기준은 T06 완료 `c918b3f9b60e12e332ee1d38f66ea524d975a740`다.
작업지시자의 「T07 역시 나누지 않고 한 단위로 작업 범위를 잡고 진행해주세요」를
계획·구현·실제 포장·독립 소비·보고·로컬 커밋까지의 승인으로 기록한다.
F01–F05를 모두 완료하며 중간 승인을 반복하지 않는다.

| ID | 범위 | 완료 조건 |
| --- | --- | --- |
| F01 | 확정된 MD01–MD09를 게시 계약·API·지원표에 반영 | 14개 이름·버전·공개 registry 상태를 조회하고 본체 0.9.0, Rust 1.93.1, 핵심 parser/IR/core/service/저장 계약과 0.9.x 호환 경계를 명시. 아직 게시하지 않은 이름의 소유권을 확보했다고 표시하지 않음 |
| F02 | 게시 manifest·파일 선별·고지·내장 파일 정비 | 승인된 패키지의 publish 제한을 해제하고 버전·지원 metadata·동기화 계약을 반영. 실제 Cargo archive 전 파일을 고정 Git source와 대조하며 사유 corpus·비밀·중간 산출물 제외 확인 |
| F03 | 초회 수동 게시와 부분 실패 복구 준비 | exact SHA/version/digest, 자식→부모 순서, 이미 게시된 동일 버전의 checksum 대조, owner/권한·실제 parent dry-run의 실행 시점을 명시. 읽기 전용 preflight·실패 경계를 검증하고 기존 네 채널 자동 배포와 분리 |
| F04 | 현재 14개 실제 포장과 독립 Rust/CLI/WASM 소비 | repository path·patch 없는 독립 workspace에서 다운로드 checksum·선택 graph를 검사. parser/IR/service/core/저장·기본/최소 feature·오류/예산/복구·기본/Native CLI·WASM facade on/off·실제 브라우저·README 예제·rustdoc 실행. 최종 구현과 기존 T06 보정의 보존을 연결 |
| F05 | 가이드·결정 대장·TODO·완료 보고·로컬 커밋 | 현재 source/archive/명령/실제 결과를 고정. 과거 S05 결과를 이번 결과로 승격하지 않으며 T07 필수 미완료 0건 확인 |

선행 구현은 v1 PR #8181의 통합 `d9fd85ed5`, #8183/PR #8190의 통합 `e7a0dec55`,
[S05 실제 포장](../working/task_m100_8174_v2_stage5.md), T04–T06 완료를 따른다.
배포 가이드의 MINOR 동기화 규칙을 지키며 버전 변경을 실제 릴리즈·게시로 설명하지 않는다.
사용자용 보안 가이드는 실제 요청 경계와 host 책임, `parserAllocationBoundsProven=false`,
전체 heap/RSS와 동기 main-thread 선점의 비보장을 유지한다.

모든 Cargo 실행은 공용 `target/pr-review`에서 순차 수행한다. 검증 registry는 loopback에서
index/download만 제공하며 사용자 전역 설정·cache를 수동 수정하지 않는다. 실행 로그·archive·
consumer 산출물은 ignored `output/pr-review/issue8174/t07-completion-20261028/`에 둔다.
최종 공개용 자료는 비공개 입력을 제외한다. 구현이 바뀌면 영향 검증도 다시 수행한다.

T08의 누적 source-unit 이관·전체 통합 성능과 T09의 최신 devel 기능 보존·최종 PR 제출은
기존 다음 항목이다. 이를 이유로 T07에 필요한 실제 포장·소비 검증을 넘기지 않는다.
실제 crates.io parent dry-run은 자식이 공개된 뒤 실행해야 하므로 이번 로컬 registry 소비와
구분해 인계한다. 원격 push·PR/comment·registry 업로드·owner 변경·토큰 설정·태그는
이번 범위에 포함하지 않는다.


## 완료 판정

2026-10-28. F01–F05를 한 단위로 완료했고 필수 잔여는 0건이다.
[T07 완료 보고](../working/task_m100_8174_t07_completion.md)에 확정한 0.9.0 계약·14개 실제
archive/독립 소비·MSRV/지원표·수동 게시 preflight·전쪽 시각/직접 판독과 source 대조를 기록했다.
실제 게시와 T08·T09는 별도이며, 토큰·owner·태그·원격 저장소를 변경하지 않았다.
