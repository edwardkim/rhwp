# task_m100_4435 Stage 1 — 원본 스트림 경로 상수의 의존 방향 정정

- **이슈**: [#4435](https://github.com/edwardkim/rhwp/issues/4435)
- **브랜치**: `fix/issue-4435-origin-path-constants`
- **분기 기준**: `upstream/devel` `9f5911e86` (0 behind)
- **상태**: 게이트 통과, PR 게시
- **기록일**: 2026-08-10 KST

## 1. 결함

`HWPX_ORIGIN_STREAM_PATH` 와 `HWP3_ORIGIN_STREAM_PATH` 가
`src/document_core/converters/hwpx_to_hwp.rs` 에 정의돼 있는데 소비자가 파서였다
(`src/parser/mod.rs:362`, `:547`). 즉 `parser` 가 `document_core::converters` 에 의존했다.

파서는 바이트를 공통 IR 로 바꾸는 가장 아래 층이고 `document_core::converters` 는 그 위에서
IR 을 정렬하는 층이다. 방향이 거꾸로다.

## 2. 조사 — 상수는 순수한 이름이었다

옮기기 전에 전 소비처를 뽑았다. 세 상수 모두 `hwpx_aux_entry` / `raw_streams` 의 경로 문자열
비교에만 쓰이고, 정의 모듈의 로직을 전제하지 않는다. **상수만 옮기면 되는 경우가 맞았다.**

형제 상수 `HWP5_ORIGIN_HWPX_MARKER_PATH` 는 이미 `src/model/document.rs:16` 에 있었다.
셋을 같은 자리로 모았다 — `:16`, `:26`, `:42`.

## 3. gestell — 잔여 결합이 남았는가

`#4400` 에서 같은 함정이 있었다(옮긴 함수의 형제를 남겨 두어 헬퍼 가시성을 넓혀야 했다).
여기서는 재수출·별칭·가시성 확대 없이 정의 위치만 바뀌었다. 옛 경로에 남은 이름이 없다.

이관 후 `src/parser/` 에 남은 `document_core` 참조는 **하나뿐이고 `#[cfg(test)]` 안**이다 —
`src/parser/hwp3/mod.rs:5678` 의 `test_hwp3_save_as_hwp5_roundtrip` 이
`DocumentCore` 를 써서 왕복을 검증한다. 통합 성격의 테스트가 상위 층을 부르는 것은 의존
방향 문제가 아니라 테스트 배치 문제라 이번 범위에서 손대지 않았다.

**프로덕션 파서 코드에는 `document_core` 참조가 남아 있지 않다.**

## 4. 검증

동작이 바뀌지 않으므로 red→green 쌍이 없다. 비회귀와 구조적 증명으로 답한다.

- `cargo fmt --all -- --check` exit 0
- `cargo clippy --all-targets -- -D warnings` exit 0
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests` exit 0 —
  `test result: ok` 블록 **502개, FAILED 0건**
- 구조적 증명: 위 3절의 grep 결과.

## 5. 미처리

GitHub Actions, 작업지시자 승인, merge.
