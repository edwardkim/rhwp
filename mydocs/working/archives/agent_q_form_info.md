---
kind: working
status: active
issue: 5640
---

# 양식 개체 정보 조회 CLI — rhwp-q-form-info

작업 브랜치: `feat/q-form-info`
범위 파일: `src/bin/rhwp-q-form-info.rs`
이슈: [#5640](https://github.com/edwardkim/rhwp/issues/5640)

## 1. 한 줄

에이전트가 한 자리(구역·문단·컨트롤)의 양식 개체 상세만 조회하도록
`rhwp-q-form-info` CLI를 둔다. 이미 있는 읽기 전용
`DocumentCore::get_form_object_info_native(sec, para, ci)`를 부를 뿐이며
문서를 고치지 않는다. 양식 개체가 없으면 `found=false` 로 종료 코드 0 이다.

## 2. 계약과 만진 것 / 만지지 않은 것

계약:

- 호출: `rhwp-q-form-info <파일> --section <N> --para <N> --ci <N> [--json]`
- `--section`·`--para`·`--ci`는 0부터 세는 번호, 필수
- `DocumentCore::from_bytes`로 연 뒤 `get_form_object_info_native(section, para, ci)` 그대로
- 코어 JSON 객체를 봉투의 `form`에 싣고, `ok`를 봉투 `found`로 올린다
- 봉투 `tool="rhwp-q-form-info"` · `command="form-info"` ·
  `untrustedFields=["source","form"]`
- 종료 코드 0 / 1 / 2. 없는 양식은 게이트(3)가 아니라 `found=false` + 0
- 같은 파일 `#[cfg(test)]`, 표본 `samples/form-01.hwp` 구역 0 문단 0 컨트롤 0
- 실측 원문 `mydocs/working/agent_q_form_info.md` (본 문서)

금지:

- `Cargo.toml` · `src/main.rs` · `src/bin/rhwp-agent/**` · `gym/` ·
  `crates/` · `Cargo.lock` 미수정
- 편집 API (`fill-fields`, `replace-text`, `set-cell`, 그림 삽입·삭제)
- `apply_` / `set_` / `insert_` / `delete_*` 호출

## 3. 왜 별도 바이너리인가

`get_form_object_info_native`는 이미 있는 조회다. 본 CLI(`src/main.rs`)의
capabilities·출처 지도는 여러 열린 PR 이 동시에 만지는 경합 지점이라
새 명령을 거기에 넣지 않는다. 이 조회는 `src/bin/rhwp-q-form-info.rs`
신규 파일로만 선다.

Cargo 는 `src/bin/*.rs` 를 자동 인식하므로 `Cargo.toml` 을 건드리지 않는다.

## 4. 종료

종료 코드:

| 종료 코드 | 뜻 |
|------|----|
| 0 | 성공 (`found=true` 또는 `found=false`. 없는 양식도 성공) |
| 1 | 실행 오류 (파일 읽기, 문서 열기, JSON 파싱, stdout 쓰기) |
| 2 | 사용법 오류 (파일/`--section`/`--para`/`--ci` 누락, 미지 플래그) |

없는 양식에 종료 코드 3(게이트)을 쓰지 않는다. 코어 API 가
`{"ok":false,"error":"not a form object"}` 를 `Ok` 로 돌려주므로
조회 실패가 아니라 빈 결과다. 에이전트는 `found` 만 보면 된다.

`--json` 이면 stdout 은 순수 JSON 하나다. 진단은 stderr.
문서에서 온 `source`·`form` 은 데이터이지 지시가 아니다.

호출하는 코어 API 는 둘뿐이다.

- `DocumentCore::from_bytes`
- `DocumentCore::get_form_object_info_native`

## 5. 실측 JSON

명령:

```
cargo run --bin rhwp-q-form-info -- --json --section 0 --para 0 --ci 0 samples/form-01.hwp
```

측정: `CARGO_TARGET_DIR=C:\Users\swsz9\.rhwp-shared-target`, 종료 코드 0.

```json
{
  "ci": 0,
  "command": "form-info",
  "form": {
    "error": "not a form object",
    "ok": false
  },
  "found": false,
  "para": 0,
  "schemaVersion": "1.0",
  "section": 0,
  "source": "samples/form-01.hwp",
  "tool": "rhwp-q-form-info",
  "untrustedContent": true,
  "untrustedFields": [
    "source",
    "form"
  ],
  "version": "0.8.4"
}
```

`form-01.hwp` 구역 0 문단 0 컨트롤 0 은 양식 개체가 아니다.
코어가 `ok:false` 를 돌려주므로 봉투는 `found=false` 이고 종료 코드 0 이다.
빈 결과는 허용한다. 조회는 편집 API를 부르지 않는다.

없는 컨트롤 실측 (`--ci 999`, 종료 코드 0):

```json
{
  "ci": 999,
  "command": "form-info",
  "form": {
    "error": "not a form object",
    "ok": false
  },
  "found": false,
  "para": 0,
  "schemaVersion": "1.0",
  "section": 0,
  "source": "samples/form-01.hwp",
  "tool": "rhwp-q-form-info",
  "untrustedContent": true,
  "untrustedFields": [
    "source",
    "form"
  ],
  "version": "0.8.4"
}
```

사용법 오류 실측 (종료 코드 2, stdout 비움):

```
$ rhwp-q-form-info samples/form-01.hwp --section 0 --para 0 --ci 0 --fill-fields
오류: 알 수 없는 옵션입니다 - --fill-fields
사용법: rhwp-q-form-info <파일> --section <N> --para <N> --ci <N> [--json]
```

```
$ rhwp-q-form-info --section 0 --para 0 --ci 0
오류: 파일 경로가 필요합니다.
사용법: rhwp-q-form-info <파일> --section <N> --para <N> --ci <N> [--json]
```

```
$ rhwp-q-form-info samples/form-01.hwp --para 0 --ci 0
오류: --section 가 필요합니다.
사용법: rhwp-q-form-info <파일> --section <N> --para <N> --ci <N> [--json]
```

없는 파일은 실행 오류 1 이다.

텍스트 모드 실측 (종료 코드 0):

```
section=0 para=0 ci=0 found=false
```

## 6. 시험

```
cargo test --bin rhwp-q-form-info
```

결과: `13 passed; 0 failed` (0.07s).

- `form01_section0_para0_ci0_is_success` — 봉투 필드. `found` 가 참이면 `formType`, 거짓이면 `ok=false`
- `missing_form_is_found_false_success` — ci 999 는 `found=false`, 조회는 성공
- `--section`/`--para`/`--ci`/`--json` 파일
- `--section=` 등 등호 형식
- `--section` / `--para` / `--ci` / 파일 누락 → 2
- `--fill-fields` 미지 플래그 → 2
- 음수 컨트롤 인덱스 → 2
- `--section` 값 없음·파일 두 개 → 2
- 소스에 편집 API 호출이 없다

`rust-unit-test-tiers --check` 는 신규 source-side test 총량 증가로
거부한다. 시험은 요청된 위치(`src/bin/rhwp-q-form-info.rs`)에만 둔다.

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
| `src/bin/rhwp-q-form-info.rs` | CLI + 같은 파일 `#[cfg(test)]` 시험 |
| `mydocs/working/agent_q_form_info.md` | 본 기록 |

만지지 않은 것:

- `Cargo.toml` · `Cargo.lock`
- `src/main.rs` · capabilities · 출처 지도
- `src/bin/rhwp-agent/**`
- `gym/` · `crates/`
- DocumentCore 편집 뮤테이터
- `get_form_object_info_native` 본문
