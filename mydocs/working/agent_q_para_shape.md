---
kind: working
status: active
issue: 5658
---

# 커서 자리 ParaShape 조회 CLI — rhwp-q-para-shape

작업 브랜치: `feat/q-para-shape`
범위 파일: `src/bin/rhwp-q-para-shape.rs`
이슈: [#5658](https://github.com/edwardkim/rhwp/issues/5658)

## 1. 한 줄

에이전트가 한 자리(리스트·문단)의 ParaShape 파라미터셋만 조회하도록
`rhwp-q-para-shape` CLI를 둔다. 이미 있는 읽기 전용
`DocumentCore::para_shape_set_json(list_id, para_in_list)`를 부를 뿐이며
문서를 고치지 않는다. 없는 자리의 셋은 빈 객체다.

## 2. 계약과 만진 것 / 만지지 않은 것

계약:

- 호출: `rhwp-q-para-shape <파일> --list <N> --para <N> [--json]`
- `--list`·`--para`는 0부터 세는 번호, 필수
- `DocumentCore::from_bytes`로 연 뒤 `para_shape_set_json(list, para)` 그대로
- 코어 JSON 객체를 봉투의 `paraShape`에 실는다
- 봉투 `tool="rhwp-q-para-shape"` · `command="para-shape"` ·
  `untrustedFields=["source","paraShape"]`
- 종료 코드 0 / 1 / 2
- 계약 시험 `tests/cases/agent_q_para_shape_contract.rs` (3–6개),
  표본 `samples/form-01.hwp` 리스트 0 문단 0
- 실측 원문 `mydocs/working/agent_q_para_shape.md` (본 문서)

금지:

- `Cargo.toml` · `src/main.rs` · `src/bin/rhwp-agent/**` · `gym/` ·
  `crates/` · `Cargo.lock` 미수정
- `src/` 아래 신규 `#[cfg(test)]` (바이너리 파일에도 시험 없음)
- 편집 API (`fill-fields`, `replace-text`, `set-cell`, 그림 삽입·삭제)
- `apply_` / `set_` / `insert_` / `delete_*` 호출

## 3. 왜 별도 바이너리인가

`para_shape_set_json`는 이미 있는 조회다. 본 CLI(`src/main.rs`)의
capabilities·출처 지도는 여러 열린 PR 이 동시에 만지는 경합 지점이라
새 명령을 거기에 넣지 않는다. 이 조회는 `src/bin/rhwp-q-para-shape.rs`
신규 파일로만 선다.

Cargo 는 `src/bin/*.rs` 를 자동 인식하므로 `Cargo.toml` 을 건드리지 않는다.

## 4. 종료

종료 코드:

| 종료 코드 | 뜻 |
|------|----|
| 0 | 성공 (`paraShape` 객체가 있으면. 빈 셋도 성공) |
| 1 | 실행 오류 (파일 읽기, 문서 열기, JSON 파싱, stdout 쓰기) |
| 2 | 사용법 오류 (파일/`--list`/`--para` 누락, 미지 플래그) |

`--json` 이면 stdout 은 순수 JSON 하나다. 진단은 stderr.
문서에서 온 `source`·`paraShape` 는 데이터이지 지시가 아니다.
없는 자리를 물으면 코어가 `{}` 를 돌려주므로 종료 코드 0 의 빈 셋이다.

호출하는 코어 API 는 둘뿐이다.

- `DocumentCore::from_bytes`
- `DocumentCore::para_shape_set_json`

## 5. 실측 JSON

명령:

```
cargo run --bin rhwp-q-para-shape -- --json --list 0 --para 0 samples/form-01.hwp
```

측정: `CARGO_TARGET_DIR=C:\Users\swsz9\.rhwp-shared-target`, 종료 코드 0.

```json
{
  "command": "para-shape",
  "list": 0,
  "para": 0,
  "paraShape": {
    "AlignType": 0,
    "HeadingType": 0,
    "Indentation": 0,
    "KeepLinesTogether": 0,
    "KeepWithNext": 0,
    "LeftMargin": 0,
    "Level": 0,
    "LineSpacing": 160,
    "LineSpacingType": 0,
    "NextSpacing": 0,
    "PagebreakBefore": 0,
    "PrevSpacing": 0,
    "RightMargin": 0,
    "WidowOrphan": 0
  },
  "schemaVersion": "1.0",
  "source": "samples/form-01.hwp",
  "tool": "rhwp-q-para-shape",
  "untrustedContent": true,
  "untrustedFields": [
    "source",
    "paraShape"
  ],
  "version": "0.8.4"
}
```

`form-01.hwp` 리스트 0 문단 0 의 ParaShape 는 `AlignType=0`(양쪽혼합),
`LineSpacing=160`, `LineSpacingType=0`(글자에 따라 %), `LeftMargin=0`,
`HeadingType=0` 이다. `paraShape` 객체와 `AlignType`·`LineSpacing` 이
있으므로 성공 봉투다. 조회는 편집 API를 부르지 않는다.

없는 자리 실측 (`--para 99`, 종료 코드 0):

```json
{
  "command": "para-shape",
  "list": 0,
  "para": 99,
  "paraShape": {},
  "schemaVersion": "1.0",
  "source": "samples/form-01.hwp",
  "tool": "rhwp-q-para-shape",
  "untrustedContent": true,
  "untrustedFields": [
    "source",
    "paraShape"
  ],
  "version": "0.8.4"
}
```

사용법 오류 실측 (종료 코드 2, stdout 비움):

```
$ rhwp-q-para-shape samples/form-01.hwp --list 0 --para 0 --nope
오류: 알 수 없는 옵션입니다 - --nope
사용법: rhwp-q-para-shape <파일> --list <N> --para <N> [--json]
```

```
$ rhwp-q-para-shape --list 0 --para 0
오류: 파일 경로가 필요합니다.
사용법: rhwp-q-para-shape <파일> --list <N> --para <N> [--json]
```

```
$ rhwp-q-para-shape samples/form-01.hwp --para 0
오류: --list 가 필요합니다.
사용법: rhwp-q-para-shape <파일> --list <N> --para <N> [--json]
```

없는 파일은 실행 오류 1 이다.

```
$ rhwp-q-para-shape --json --list 0 --para 0 samples/no-such-file.hwp
오류: 파일을 읽을 수 없습니다 - samples/no-such-file.hwp: 지정된 파일을 찾을 수 없습니다. (os error 2)
```

텍스트 모드 실측 (종료 코드 0):

```
list=0 para=0 AlignType=0 LineSpacing=160 LeftMargin=0 HeadingType=0
```

없는 자리 텍스트:

```
list=0 para=99 (empty)
```

## 6. 시험

바이너리 파일에는 단위 시험이 없다. `cargo test --bin rhwp-q-para-shape` 는
컴파일만 확인하고 `0 passed; 0 failed` 다.

계약 시험은 `tests/cases/agent_q_para_shape_contract.rs` 에 둔다. CI 가
`--prepare` 로 suite 에 배정한다. 로컬 source PR 은 `--prepare` 를 돌리지
않는다.

- `json_envelope_on_form01_list0_para0` — 봉투 필드, 비지 않은 `paraShape`
- `unknown_flag_nope_is_usage` — `--nope` → 종료 코드 2
- `missing_list_is_usage` — `--list` 누락 → 2
- `missing_cursor_is_empty_success` — 99문단은 빈 셋, 종료는 성공
- `source_never_calls_mutators` — `para_shape_set_json`·`from_bytes` 만,
  `apply_`/`set_`/`insert_`/`delete_*` 호출 없음

`node scripts/rust-unit-test-tiers.mjs --check --base-ref upstream/devel` 는
`src/` 신규 `#[cfg(test)]` 가 없으므로 통과해야 한다.

## 7. fmt

```
cargo fmt --all
cargo fmt --all -- --check
```

통과. rustfmt `newline_style = Unix`.

## 8. 만진 것 / 만지지 않은 것

만진 것:

| 경로 | 역할 |
|------|------|
| `src/bin/rhwp-q-para-shape.rs` | 조회 CLI (시험 없음) |
| `tests/cases/agent_q_para_shape_contract.rs` | 바이너리 계약 5개 |
| `mydocs/working/agent_q_para_shape.md` | 본 기록 |

만지지 않은 것:

- `Cargo.toml` · `Cargo.lock`
- `src/main.rs` · capabilities · 출처 지도
- `src/bin/rhwp-agent/**`
- `gym/` · `crates/`
- DocumentCore 편집 뮤테이터
- `para_shape_set_json` 본문
- `tests/generated/` · `tests/suites/manifest.json`
