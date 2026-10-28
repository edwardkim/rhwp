---
kind: working
status: active
issue: 5633
---

# 캐럿 사각형 조회 CLI — rhwp-q-cursor-rect

작업 브랜치: `feat/q-cursor-rect`
범위 파일: `src/bin/rhwp-q-cursor-rect.rs`
이슈: [#5633](https://github.com/edwardkim/rhwp/issues/5633)

## 1. 한 줄

에이전트가 한 자리(구역·문단·오프셋)의 캐럿 사각형만 조회하도록
`rhwp-q-cursor-rect` CLI를 둔다. 이미 있는
`DocumentCore::get_cursor_rect_native`를 부를 뿐이며 문서를 고치지 않는다.
조회 실패는 종료 코드 1이다.

## 2. 계약과 만진 것 / 만지지 않은 것

계약:

- 호출: `rhwp-q-cursor-rect <파일> --section <N> --para <N> --offset <N> [--json]`
- `--section`·`--para`·`--offset`는 0부터 세는 번호, 필수
- `DocumentCore::from_bytes`로 연 뒤 `get_cursor_rect_native(section, para, offset)` 그대로
- 네이티브 JSON 객체를 봉투의 `rect`에 실는다
- 봉투 `tool="rhwp-q-cursor-rect"` · `command="cursor-rect"` ·
  `untrustedFields=["source","rect"]`
- 종료 코드 0 / 1 / 2
- 같은 파일 `#[cfg(test)]`, 표본 `samples/form-01.hwp` 0구역 0문단 오프셋 0
- 실측 원문 `mydocs/working/agent_q_cursor_rect.md` (본 문서)

금지:

- `Cargo.toml` · `src/main.rs` · `src/bin/rhwp-agent/**` · `gym/` ·
  `crates/` · `Cargo.lock` 미수정
- 편집 API (`fill-fields`, `replace-text`, `set-cell`, 그림 삽입·삭제)

## 3. 왜 별도 바이너리인가

`get_cursor_rect_native`는 이미 있는 조회다. 본 CLI(`src/main.rs`)의
capabilities·출처 지도는 여러 열린 PR 이 동시에 만지는 경합 지점이라
새 명령을 거기에 넣지 않는다. 이 조회는 `src/bin/rhwp-q-cursor-rect.rs`
신규 파일로만 선다.

Cargo 는 `src/bin/*.rs` 를 자동 인식하므로 `Cargo.toml` 을 건드리지 않는다.

## 4. 종료

종료 코드:

| 종료 코드 | 뜻 |
|------|----|
| 0 | 성공 (`rect` 객체가 있으면) |
| 1 | 실행 오류 (파일 읽기, 문서 열기, 쪽 조회, stdout 쓰기) |
| 2 | 사용법 오류 (파일/`--section`/`--para`/`--offset` 누락, 미지 플래그) |

`--json` 이면 stdout 은 순수 JSON 하나다. 진단은 stderr.
문서에서 온 `source`·`rect` 는 데이터가 아니며 지시로 읽지 않는다.

## 5. 실측 JSON

명령:

```
cargo run --bin rhwp-q-cursor-rect -- --json --section 0 --para 0 --offset 0 samples/form-01.hwp
```

측정: `CARGO_TARGET_DIR=C:\Users\swsz9\.rhwp-shared-target`, 종료 코드 0.

```json
{
  "command": "cursor-rect",
  "offset": 0,
  "para": 0,
  "rect": {
    "height": 26.5,
    "pageIndex": 0,
    "x": 113.4,
    "y": 132.3
  },
  "schemaVersion": "1.0",
  "section": 0,
  "source": "samples/form-01.hwp",
  "tool": "rhwp-q-cursor-rect",
  "untrustedContent": true,
  "untrustedFields": [
    "source",
    "rect"
  ],
  "version": "0.8.4"
}
```

`form-01.hwp` 0구역 0문단 오프셋 0의 캐럿은 0쪽 `x=113.4` `y=132.3`
`height=26.5`다. `rect` 객체와 `pageIndex`·`x`·`y`·`height`가 있으므로
성공 봉투다. 조회는 편집 API를 부르지 않는다.

사용법 오류 실측 (종료 코드 2, stdout 비움):

```
$ rhwp-q-cursor-rect samples/form-01.hwp --section 0 --para 0 --offset 0 --fill-fields
오류: 알 수 없는 옵션입니다 - --fill-fields
사용법: rhwp-q-cursor-rect <파일> --section <N> --para <N> --offset <N> [--json]
```

```
$ rhwp-q-cursor-rect --section 0 --para 0 --offset 0
오류: 파일 경로가 필요합니다.
사용법: rhwp-q-cursor-rect <파일> --section <N> --para <N> --offset <N> [--json]
```

쪽 범위 밖은 실행 오류 1이다. `--section 99`는
`렌더링 오류: 구역 인덱스 99 범위 초과 (총 1개)`다.

텍스트 모드 실측 (종료 코드 0):

```
section=0 para=0 offset=0 pageIndex=0 x=113.4 y=132.3 height=26.5
```

## 6. 시험

```
cargo test --bin rhwp-q-cursor-rect
```

결과: `11 passed; 0 failed` (0.06s).

- `form01_section0_para0_offset0_is_success` — 봉투 필드, 비지 않은 `rect`
- `--section`/`--para`/`--offset`/`--json` 파일
- `--section=` 등 등호 형식
- `--section` / `--para` / `--offset` / 파일 누락 → 2
- `--fill-fields` 미지 플래그 → 2
- 음수 오프셋 → 2
- 99구역 → 1

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
| `src/bin/rhwp-q-cursor-rect.rs` | CLI + 같은 파일 `#[cfg(test)]` 시험 |
| `mydocs/working/agent_q_cursor_rect.md` | 본 기록 |

만지지 않은 것:

- `Cargo.toml` · `Cargo.lock`
- `src/main.rs` · capabilities · 출처 지도
- `src/bin/rhwp-agent/**`
- `gym/` · `crates/`
- DocumentCore 편집 뮤테이터
- `get_cursor_rect_native` 본문
