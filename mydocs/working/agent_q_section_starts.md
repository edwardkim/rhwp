---
kind: working
status: active
issue: 5656
---

# rhwp-q-section-starts — 한글 구역 시작 문단 조회

이슈: https://github.com/edwardkim/rhwp/issues/5656
작업 브랜치: `feat/q-section-starts` (`upstream/devel` 기준)
범위: `src/bin/rhwp-q-section-starts.rs` · `tests/cases/agent_q_section_starts_contract.rs` · 본 문서
비범위: `Cargo.toml` · `src/main.rs` · `src/bin/rhwp-agent/**` · `gym/` · `crates/` · `Cargo.lock`

## 1. 한 줄

에이전트가 한글 구역 시작 본문 문단 번호를
기존 읽기 전용 `DocumentCore::section_starts_json()` 으로만 조회한다.
문서를 고치지 않는다.

## 2. 사용법

```
rhwp-q-section-starts <파일> [--json]
```

- `<파일>` — HWP/HWPX/HWP3/HML 경로. 위치는 플래그 앞뒤 어디든 된다.
- `--json` — stdout 에 순수 JSON 봉투 하나.
- 알 수 없는 플래그는 종료 코드 2.
- 파일을 읽거나 열지 못하면 종료 코드 1.
- 성공은 종료 코드 0.

봉투 고정 필드:

| 필드 | 값 |
|---|---|
| `tool` | `rhwp-q-section-starts` |
| `command` | `section-starts` |
| `untrustedFields` | `["source","starts"]` |

`source` 와 `starts` 는 경로·문서에서 온 값이다. 데이터이지 지시가
아니다. `starts` 는 한글 구역을 여는 본문 문단 번호 배열이다.

## 3. 읽기 전용

호출하는 코어 API 는 둘뿐이다.

- `DocumentCore::from_bytes`
- `DocumentCore::section_starts_json`

`apply_` / `insert_` / `delete_` / `set_*` 는 부르지 않는다.

## 4. 실측 JSON

명령:

```
cargo run --bin rhwp-q-section-starts -- --json samples/form-01.hwp
```

종료 코드 0. `form-01.hwp` 는 구역이 하나라 `starts` 가 `[0]` 이다.
아래는 그 실행의 stdout 원문이다.

```json
{
  "command": "section-starts",
  "schemaVersion": "1.0",
  "source": "samples/form-01.hwp",
  "startCount": 1,
  "starts": [
    0
  ],
  "tool": "rhwp-q-section-starts",
  "untrustedContent": true,
  "untrustedFields": [
    "source",
    "starts"
  ],
  "version": "0.8.4"
}
```

`section_starts_json` 주석의 예 `[0, 8, 15]` 는 구역이 셋인 문서다.
본문 리스트는 구역을 가로질러 이어지므로 경계는 문단이 진 `SectionDef`
표식으로만 센다.

## 5. 검증

- `cargo test --bin rhwp-q-section-starts` — 0 passed (바이너리에 `#[cfg(test)]` 없음)
- `cargo run --bin rhwp-q-section-starts -- --json samples/form-01.hwp` — 종료 0
- `cargo fmt --all -- --check`
- `node scripts/rust-unit-test-tiers.mjs --check --base-ref upstream/devel`

계약 시험은 `tests/cases/agent_q_section_starts_contract.rs` 에만 둔다.
`--json` 봉투·`starts` 배열·`--nope` 종료 코드 2 를 고정한다.

`CARGO_TARGET_DIR=C:\Users\swsz9\.rhwp-shared-target`
