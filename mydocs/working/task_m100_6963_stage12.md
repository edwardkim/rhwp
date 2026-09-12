# #6963 단계 12 — PR #6984 경계 결함 보정

- 사용자 승인: 두 결함 보정·로컬 검증 서버 준비. 원격 push·코멘트는 직접 확인 후 별도 승인 대기.
- 기준 head: `065bda2307db18eafa8394a360289c8558ffcc7b`
- 로컬 브랜치: `codex/pr6984-hyperlink-corrections`

## 원인과 구현 계획

1. 링크 끝 글자 삭제의 undo는 일반 삽입으로 링크 범위와 글자 모양을 복구할 수 없다.
   링크 글자를 삭제할 때만 기존 삭제 조각 저장소로 상위 본문 문단을 보존한다.
   본문·표·중첩 표·글상자가 같은 경로를 사용한다. 조각 복원은 redo에서 재캡처하고
   히스토리 폐기와 실패 시 해제한다. 조각이 있는 삭제는 문자 병합을 막아 복원 시점이
   다른 삭제와 섞이지 않게 한다. 일반 문자 삭제와 링크 뒤 이어 쓰기는 기존 경로를 유지한다.
2. 링크 시작에 삽입된 글자는 [start, end) 링크에 포함되므로 시작 글자의 서식을 물려받는다.
   raw UTF-16 필드 마커 뒤에 있는 시작 char-shape 경계도 고정한다. 링크 끝의 일반 서식,
   인접 링크의 별도 범위, HWP/HWPX 저장 왕복을 회귀 검증한다.

## 검증 계획

- native hyperlink 편집 회귀, 실제 WASM + Studio DeleteTextCommand/CommandHistory undo/redo.
- 한 글자 전체 삭제, 연속 삭제, 일반 텍스트와 경계, 중첩 셀·글상자 및 저장 재열기.
- Rust 필수 lint 묶음, Studio TypeScript/build·Node 회귀, fresh WASM.
- 실제 로컬 브라우저에서 두 재현 여정을 확인하고 서버를 사용자에게 전달한다.
- 원격 게시·merge는 실행하지 않고 사용자 승인을 기다린다.

## 로컬 preview 검증 결과 (2026-09-13)

- `cargo fmt --all -- --check`, suite manifest `--check`: 통과.
- `node scripts/run-rust-test.mjs issue_6963_hyperlink_edit -- --cargo-profile release-test --target-dir target/pr-review --no-fail-fast`:
  16/16 통과. 문단 시작·일반 글자 뒤·인접 링크 경계를 포함한다.
- fresh WASM: `CARGO_TARGET_DIR=target/pr-review scripts/wasm-pack-locked.sh --target web --out-dir pkg --no-opt` 성공.
  WASM SHA-256: `7971e239e0d038c3110f3dac078ee822a7f432c9cf20e212a97423654b17eeff`.
- `node --experimental-transform-types --no-warnings tests/support/hyperlink-wasm.runner.mjs`:
  실제 bridge/command/history 15개 검증 그룹 통과. 한 글자·전체·astral·연속 삭제,
  실제 한컴 중첩 셀·글상자, 3회 undo/redo와 각 undo의 HWP/HWPX 저장 왕복 포함.
- Studio `npm test`: 1,502 pass / 2 skip / 0 fail. `npm run build` (TypeScript 포함): 성공.
- 실제 브라우저 동일 여정: `링크` 삽입 → Home → X → End → Backspace → Cmd+Z.
  보정 전 `X`는 검정/밑줄 없음, 고치기의 표시 문자열은 `X링`.
  보정 후 `X`는 파랑/밑줄, 고치기의 표시 문자열은 `X링크`.
- 합성 입력의 편집 불변식 검증이며 새 한컴 동작 실측으로 표시하지 않는다.
  조판 줄 나눔·측정·PDF backend 변경 및 baseline 완화는 없다.

로컬 로그: `/private/tmp/pr6984-fix-native.log`, `pr6984-fix-wasm.log`,
`pr6984-fix-node.log`, `pr6984-fix-studio-build.log`, `pr6984-fix-wasm-build.log`.
화면: `/private/tmp/pr6984-fixed-start.jpg`, `/private/tmp/pr6984-fixed-undo.jpg`.

## 사용자 확인과 이후 단계

- 수정 전 재현: http://127.0.0.1:7795/ (기준 PR head).
- 보정 preview: http://127.0.0.1:7794/ (이번 로컬 보정).
- 사용자가 직접 확인한 뒤 승인할 때까지 원격 push·코멘트 게시를 보류한다.
- 이번 결과는 로컬 preview 검증이다. 새 코드의 Rust 전체 lint 묶음과 release-test 전체,
  최신 devel의 문서 충돌 정리 및 원격 CI 확인은 후속 PR 준비 단계에 남아 있다.
  기존 head CI 결과를 이번 보정의 CI 통과로 재사용하지 않는다.
- 링크 글자 삭제는 현재 한 번의 삭제 명령마다 Undo 항목을 유지한다.
  일반 문자 연속 삭제의 300ms 병합은 유지한다. 캡션은 기존 미지원 범위를 유지한다.
