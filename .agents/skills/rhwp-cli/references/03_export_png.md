# export-png

권위: `mydocs/manual/cli_commands.md` 의 `export-png` 절, `src/main.rs` 디스패치.
새 플래그를 발명하지 않는다. 여기 없는 옵션은 `--help` 와 매뉴얼을 본다.

## 한 줄

`rhwp export-png <파일> [-p N] [--vlm-target claude] [--scale] [--dpi] [--profile]`

## 요청 매핑

- "PNG로" → `export-png`
- "VLM 입력" → `export-png`
- "스크린샷처럼" → `export-png`

## 페이지

이 명령의 `-p` 는 **0부터**다. 사용자가 한컴 쪽번호를 말하면 1을 뺀다.

## 계약 메모

- native-skia feature 없이 빌드되면 stderr '오류: export-png 명령은 native-skia feature 가 활성화되어야 합니다.' 후 exit 2.
- 기능 부재는 사용법(2)이다. 0으로 끝내면 스크립트가 성공으로 읽는다.
- --vlm-target: claude / gpt4v-low / gpt4v-high / gemini / qwen-vl / llava.
- 기본 프로필은 high-quality (인쇄 등가). 편집기식 표시는 --profile screen.

## feature 게이트

이 명령은 `native-skia` 가 필요하다.
없으면 stderr 에 기능 부재를 알리고 exit 2 (export-png) 또는 해당 백엔드는 exit 1 (pdf direct).
`capabilities` 의 `requiresFeature`/`available` 을 먼저 본다 (#3357).

## 예외

공통 네 봉투:

1. 파일 없음 — `오류: 파일을 읽을 수 없습니다 - {path}: {os}` , exit 1
2. 페이지 범위 — `오류: 페이지 번호가 범위를 벗어났습니다 (0~{max})` , exit 2
3. native-skia 부재 — export-png 스텁 exit 2
4. 로드 실패 — `오류: 문서 파싱 실패 - {msg}` , exit 1

실패 경로의 `--json` stdout 은 비운다. 부분 JSON 을 파싱하지 말 것.

## 실측 레시피

```bash
rhwp export-png samples/basic/KTX.hwp -p 0
```

산출은 `output/poc/agent-cli/` 아래로 분리한다. 원본은 읽기 전용 명령에서 불변이다.
convert 만 출력을 쓰며, 입력과 같은 경로를 거부하는 명령은 그 계약을 따른다.

## 하지 않는 것

- 이 명령의 새 별칭을 만들지 않는다.
- gym 과제로 이 명령을 대체하지 않는다.
- DocumentCore 를 열어 렌더를 고치지 않는다. 분석만 한다.

관련: [00_tree.md](00_tree.md) · [01_request_command_map.md](01_request_command_map.md) · 장 03

## 호출 카드

| 상황 | 명령 | 읽는 것 |
|---|---|---|
| 없는 경로 | `rhwp export-png 없는파일.hwp` | exit 1, missing-file |
| 인자 없음 | `rhwp export-png` | exit 2, no-input (인자가 필수인 명령) |
| 한컴 4쪽 | `rhwp export-png doc.hwp -p 3` | 0 기준 |
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

- 사용자: 3쪽 SVG 로 빼줘
  - 명령: `export-svg -p 2`
  - 메모: 한컴 3 = 인덱스 2
- 사용자: 인쇄 PDF
  - 명령: `export-pdf --profile print`
  - 메모: legacy 기본 금지
- 사용자: 텍스트만 조금
  - 명령: `export-text --json --max-chars 2000`
  - 메모: truncated

## 재시도

같은 실패 봉투가 나오면 플래그를 발명하지 말고 입력을 고친다.
