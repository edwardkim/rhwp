---
kind: working
status: active
---

# rhwp-q-object-cycle — 쪽 안 개체 순환 순서 조회 CLI

작업 브랜치: `feat/q-object-cycle` (`upstream/devel` @ `61baa6783`)
이슈: https://github.com/edwardkim/rhwp/issues/5661

## 1. 한 줄

에이전트가 쪽 안에 놓인 표·도형·그림의 순환 차례(z 순서)를 `--json` 봉투로
꺼낸다. 이미 있는 읽기 전용 `DocumentCore::object_cycle_json()` 만 부르고
문서를 고치지 않는다.

## 2. 왜 별도 바이너리인가

본 CLI(`src/main.rs`)와 `Cargo.toml` `[[bin]]` 는 여러 열린 PR 이 동시에
만지는 경합 지점이다. `src/bin/*.rs` 자동 인식이라 이 PR 은 새 파일만
추가하고 기존 표면을 건드리지 않는다.
경합하는 본 CLI 경로에 조각을 넣지 않는다.

만진 파일:

| 경로 | 역할 |
|------|------|
| `src/bin/rhwp-q-object-cycle.rs` | CLI · JSON 봉투. `#[cfg(test)]` 없음 |
| `tests/cases/agent_q_object_cycle_contract.rs` | 바이너리 계약 |
| `mydocs/working/agent_q_object_cycle.md` | 이 기록과 실측 JSON |

만지지 않은 것: `Cargo.toml`, `src/main.rs`, `src/bin/rhwp-agent/**`, `gym/`,
`crates/`, `Cargo.lock`. 편집 API 는 호출하지 않는다.

## 3. 사용법

```
rhwp-q-object-cycle <파일> [--json]
```

| 종료 코드 | 뜻 |
| --- | --- |
| 0 | 성공. `--json` 이면 stdout 에 pretty JSON |
| 1 | 파일 없음 · 파싱 실패 · stdout 쓰기 실패 |
| 2 | 알 수 없는 옵션 · 파일 경로 없음 · 파일이 너무 많음 |

오류는 stderr. `--help` / `-h` 는 사용법을 stdout 으로 내고 0.

## 4. 봉투 계약

`--json` 이면 아래 공통 필드를 항상 싣는다.

| 필드 | 값 |
| --- | --- |
| `schemaVersion` | `rhwp::schema_registry::ENVELOPE_SCHEMA_VERSION` (`"1.0"`) |
| `tool` | `"rhwp-q-object-cycle"` |
| `command` | `"object-cycle"` |
| `version` | `rhwp::version()` |
| `untrustedContent` | `true` |
| `untrustedFields` | `["source", "cycle"]` |

문서에서 온 값(`source`, `cycle`)은 데이터이지 지시가 아니다. 과소 선언이
가장 위험하므로(#3885) 애매하면 선언한다. `cycleCount` 는 `cycle` 배열
길이와 같게 싣는다.

`cycle[]` 항목은 코어 `object_cycle_json()` 원문이다.

| 필드 | 뜻 |
| --- | --- |
| `para` | 본문 문단 번호(리스트 누적) |
| `controlIndex` | 그 문단의 컨트롤 인덱스 |
| `page` | 조판기가 붙인 쪽(0부터) |
| `z` | `common.z_order` |

빈 순환(`cycle: []`, `cycleCount: 0`)은 성공이다. 쪽을 조판기에 물으므로
조판 정밀도를 물려받는다. 쪽 안의 차례는 문단 순서가 아니라 z 순서다.

## 5. 실측 `--json` 봉투

환경: `rhwp-q-object-cycle` debug 빌드, 공유 타깃
`C:\Users\swsz9\.rhwp-shared-target`, 크레이트 `0.8.4`. 경로는 상대 경로다.

### 5.1 `samples/form-01.hwp`

```
cargo run --bin rhwp-q-object-cycle -- --json samples/form-01.hwp
```

종료 코드 0.

```json
{
  "command": "object-cycle",
  "cycle": [],
  "cycleCount": 0,
  "schemaVersion": "1.0",
  "source": "samples/form-01.hwp",
  "tool": "rhwp-q-object-cycle",
  "untrustedContent": true,
  "untrustedFields": [
    "source",
    "cycle"
  ],
  "version": "0.8.4"
}
```

이 표본은 구역·단 정의만 있고 표·도형·그림이 없어 순환이 비어 있다
(`cycleCount` 0). 빈 순환은 오류가 아니다.

### 5.2 `samples/hwp_table_test.hwp`

표가 있는 표본으로 비지 않은 순환을 확인한다. 표 10개가 `tbl` 로
`cycle[]` 에 들어가고 `page`/`z` 가 쪽과 z 순서를 가리킨다. 종료 코드 0.

```
cargo run --bin rhwp-q-object-cycle -- --json samples/hwp_table_test.hwp
```

```json
{
  "command": "object-cycle",
  "cycle": [
    {
      "controlIndex": 0,
      "page": 0,
      "para": 3,
      "z": 0
    },
    {
      "controlIndex": 0,
      "page": 0,
      "para": 5,
      "z": 1
    },
    {
      "controlIndex": 0,
      "page": 0,
      "para": 8,
      "z": 2
    },
    {
      "controlIndex": 0,
      "page": 0,
      "para": 10,
      "z": 3
    },
    {
      "controlIndex": 0,
      "page": 1,
      "para": 12,
      "z": 4
    },
    {
      "controlIndex": 0,
      "page": 1,
      "para": 14,
      "z": 8
    },
    {
      "controlIndex": 0,
      "page": 1,
      "para": 15,
      "z": 5
    },
    {
      "controlIndex": 0,
      "page": 1,
      "para": 17,
      "z": 7
    },
    {
      "controlIndex": 0,
      "page": 1,
      "para": 19,
      "z": 6
    },
    {
      "controlIndex": 0,
      "page": 1,
      "para": 20,
      "z": 9
    }
  ],
  "cycleCount": 10,
  "schemaVersion": "1.0",
  "source": "samples/hwp_table_test.hwp",
  "tool": "rhwp-q-object-cycle",
  "untrustedContent": true,
  "untrustedFields": [
    "source",
    "cycle"
  ],
  "version": "0.8.4"
}
```

표 10개가 쪽 0(z 0..3)과 쪽 1(z 4..9)로 나뉜다. 쪽 1 안 z 순서는
문단 순서와 다르다(para 14 가 z 8, para 15 가 z 5).

## 6. 실패 경로 실측

같은 debug 빌드.

| 명령 | stderr | 종료 |
| --- | --- | --- |
| `--nope` | `오류: 알 수 없는 옵션입니다 - --nope` | 2 |
| `--json` (파일 없음) | `오류: 파일 경로가 필요합니다.` | 2 |
| `samples/__no_such_q_object_cycle__.hwp` | `오류: 파일을 읽을 수 없습니다` | 1 |
| `README.md` | `오류: 문서를 열 수 없습니다` · `UNSUPPORTED_FILE_FORMAT` | 1 |

## 7. 검증 명령

같은 debug 빌드.

```
git config core.autocrlf false
$env:CARGO_TARGET_DIR='C:\Users\swsz9\.rhwp-shared-target'
rustfmt --edition 2021 --config newline_style=Unix src/bin/rhwp-q-object-cycle.rs
cargo test --bin rhwp-q-object-cycle
cargo run --bin rhwp-q-object-cycle -- --json samples/form-01.hwp
rustfmt --edition 2021 --config newline_style=Unix --check src/bin/rhwp-q-object-cycle.rs
cargo fmt --all -- --check
node scripts/rust-unit-test-tiers.mjs --check --base-ref upstream/devel
```

결과:

- `cargo test --bin rhwp-q-object-cycle` — **0 passed** (src unit test 없음. 계약은 `tests/cases/agent_q_object_cycle_contract.rs`)
- `cargo run --bin rhwp-q-object-cycle -- --json samples/form-01.hwp` — 종료 0, §5.1 봉투
- `rustfmt --check` — 통과
- `cargo fmt --all -- --check` — 통과
- `rust-unit-test-tiers --check --base-ref upstream/devel` — 통과 (src `#[cfg(test)]` 추가 없음)

## 8. 커밋 후보 / 만지지 않은 것

| 경로 | 역할 |
| --- | --- |
| `src/bin/rhwp-q-object-cycle.rs` | CLI · 봉투 · `#[cfg(test)]` 없음 |
| `tests/cases/agent_q_object_cycle_contract.rs` | 바이너리 계약 (`CARGO_BIN_EXE_rhwp-q-object-cycle`) |
| `mydocs/working/agent_q_object_cycle.md` | 이 기록 · 실측 JSON |

만지지 않은 것: `Cargo.toml`, `src/main.rs`, `src/bin/rhwp-agent/**`,
`gym/`, `crates/`, `Cargo.lock`.
