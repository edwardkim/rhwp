# dump

권위: `mydocs/manual/cli_commands.md` 의 `dump` 절, `src/main.rs` 디스패치.
새 플래그를 발명하지 않는다. 여기 없는 옵션은 `--help` 와 매뉴얼을 본다.

## 한 줄

`rhwp dump <파일> [-s N] [-p M]`

## 요청 매핑

- "조판부호" → `dump`
- "문단 속성" → `dump`
- "LINE_SEG" → `dump`
- "표 속성" → `dump`

## 페이지

이 명령의 `-p`/`-s` 는 페이지가 아니라 문단/구역 인덱스(0부터)일 수 있다. dump-pages 의 `-p`(페이지)와 섞지 말 것.

## 계약 메모

- -s/-p 는 구역/문단 인덱스(0부터). dump-pages 의 -p 는 페이지다. 혼동 금지.
- ParaShape / LINE_SEG / 표·도형 속성. 상세는 dump_command.md.
- 문단 헤더 0.3 은 구역 0 문단 3.
- 용지 크기는 mm 와 HWPUNIT 을 병기한다 (예: 59528×84188 HU).

## 예외

공통 네 봉투:

1. 파일 없음 — `오류: 파일을 읽을 수 없습니다 - {path}: {os}` , exit 1
2. 페이지 범위 — `오류: 페이지 번호가 범위를 벗어났습니다 (0~{max})` , exit 2
3. native-skia 부재 — export-png 스텁 exit 2
4. 로드 실패 — `오류: 문서 파싱 실패 - {msg}` , exit 1

실패 경로의 `--json` stdout 은 비운다. 부분 JSON 을 파싱하지 말 것.

## 실측 레시피

```bash
rhwp dump samples/basic/KTX.hwp
```

산출은 `output/poc/agent-cli/` 아래로 분리한다. 원본은 읽기 전용 명령에서 불변이다.
convert 만 출력을 쓰며, 입력과 같은 경로를 거부하는 명령은 그 계약을 따른다.

## 하지 않는 것

- 이 명령의 새 별칭을 만들지 않는다.
- gym 과제로 이 명령을 대체하지 않는다.
- DocumentCore 를 열어 렌더를 고치지 않는다. 분석만 한다.

관련: [00_tree.md](00_tree.md) · [01_request_command_map.md](01_request_command_map.md) · 장 08

## 호출 카드

| 상황 | 명령 | 읽는 것 |
|---|---|---|
| 없는 경로 | `rhwp dump 없는파일.hwp` | exit 1, missing-file |
| 인자 없음 | `rhwp dump` | exit 2, no-input (인자가 필수인 명령) |
| 한컴 4쪽 | `rhwp dump doc.hwp -s 0 -p 3` | 0 기준 |
| JSON 실패 | `(json 없음)` | stdout 0바이트 |

## 소비 규칙

사람용 텍스트. 자동화 게이트가 필요하면 형제 명령의 `--json` 을 쓴다.
이 명령에 `--json` 을 발명해서 붙이지 않는다.

## 인계

- 긴 문서 파악만 → `rhwp-doc-triage` (info/digest/search). 이 스킬은 분석·디버그.
- 시각 회귀 숫자 판정 → `rhwp-visual-regression` (render-diff). 여기선 overlay 와 tree.
- 편집 → `rhwp-safe-edit`. 이 스킬은 원본을 고치지 않는다 (convert 제외, 출력 분리).

## 표본 경로

- `samples/basic/KTX.hwp` — 기본 표·본문
- `samples/basic/treatise sample.hwp` — info 표 1개 vs export-tables 3개
- `공문.hwp` — 사용자가 준 경로. 상대 경로 함정
- `편람.hwp` — 대형. --max-chars 없이 export-text 금지 기본
- `oracle.hwp` — 한컴 저장본
- `generated.hwp` — rhwp 저장본
- `source.hwpx` — HWPX 원본

## 대화 예

- 사용자: 이 쪽 뭐가 있나
  - 명령: `dump-pages -p N`
  - 메모: 페이지
- 사용자: 그 문단 속성
  - 명령: `dump -s 0 -p M`
  - 메모: 문단
- 사용자: 바이너리 레코드
  - 명령: `dump-records`
  - 메모: HWP5

## 재시도

같은 실패 봉투가 나오면 플래그를 발명하지 말고 입력을 고친다.
