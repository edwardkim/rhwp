---
kind: working
status: active
last_verified: 2026-10-11
---

# #7820 3단계 — 전체 회귀와 Rust lint

메인테이너가 원격 CLI 직접 출력 성공을 확인한 뒤 전체 회귀·Native Skia 검사·Rust lint
실행을 승인했다. [2단계](task_m100_7820_stage2.md)의 Docker WASM·전쪽 TSV·터미널 판정을
이어받는다. 최신 `upstream/devel`을 fetch했으며 비교 base는
`790a6f8f388445e7bff2bcb165aa7b384f9ece80`, 구현 source는
`dc72a324923c404fb38a46667fc841304f3669e9`이다.

기본 작업공간의 공용 `/home/edward/mygithub/rhwp/target/pr-review`를 사용하며
같은 target의 Cargo 작업을 순차 실행한다. 실행 원문·정책 base·종료 코드는 ignored
`output/pr-review/issue7820-20261011/full-validation/`에 보존한다.

## 실행 범위

파생 integration suite 준비, fmt, 기본·WASM·workspace all-target Clippy와 workspace build,
고정 base의 manifest 검사, Native Skia feature Clippy, release-test 전체 nextest,
Native Skia lib·missing-picture·direct-PDF 검사와 새 Sixel Native 계약 검사를 실행한다.
단계별 결과는 실행 완료 뒤 기록한다. generated suite·manifest는 stage하지 않는다.

## 최초 lint 결과와 보정

fmt·기본 Clippy·WASM Clippy·workspace build·workspace all-target Clippy·manifest 검사는
통과했다. 추가 Native Skia all-target Clippy에서 `items_after_test_module`을 검출했다.
`renderer/skia/renderer.rs`의 `draw_arrow_heads`가 테스트 모듈 뒤에 있는 기존 배치이며
고정 base에도 함수 내용과 배치가 같다. allow 속성을 추가하지 않고 함수 전체를 테스트
모듈 앞에 옮긴다. `helper-move.json`으로 base와 함수 본문 동일성을 확인했으며
조판·그리기 동작이나 테스트 기대값은 바꾸지 않았다. 최종 source에서 lint·회귀를 다시
실행하며 최초 실패 원문은 덮어쓰지 않는다.

helper 이동 뒤 필수 lint와 추가 Native Skia lint는 모두 통과했다. 전체 nextest 최초 결과는
10,570건 중 10,569건 통과·1건 실패·50건 skipped다. 실패한
`agent_codex_contract::every_capability_command_has_a_codex_chapter`는 새 명령의
대전 장 누락을 정확히 검출했다. 생성기의 변환·렌더 가족에 `export-sixel`을 등록하고
같은 `command_chapter`·`write_if_changed`로 해당 장만 바이너리 자기서술·명령별
상세 도움말에서 생성했다. 기존 장은 그대로 보존했다. 생성 절차는
`agent-codex-regeneration.json`, 최초 실패는 `helper-move/nextest-all.log`에 있다.
Python 생성기 계약 14건과 보완 후 agent Codex 계약 2건이 통과했다.
테스트 조건을 완화하지 않고 누락 문서를 보완했으며 전체 회귀를 다시 실행한다.

이번 승인은 로컬 검증 범위다. 원격 push·PR 생성·merge·이슈 close는 수행하지 않는다.
