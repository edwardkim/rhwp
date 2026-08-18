# 기존 CLI 표면만 — 발명 금지

권위는 `mydocs/manual/cli_commands.md` 의 해당 절과 레시피 3·4·10 이다.
이 장은 기존 CLI 를 설명하고 소비 규칙을 고정한다. 새 명령·새 탐지 규칙을 발명하지 않는다.
## 허용 명령

- `inspect hidden-text`

- `inspect injection`

- `inspect unicode`

- `edit redact` (`--dry-run`, `--no-raw`, `-o`, `--in-place`, `--kind`, `--mask`, `--verify`)

- `edit sanitize` (`--keep-preview`, `-o`)

- 수신 사다리 보조: `info`, `digest`, `fields`, `search`, `export-provenance-map`

## 허용 플래그 (이미 있는 것만)

`--json`, `--threshold-pt`, `--include-offpage`, `--min-confidence`, `--include-fields`,

`--kind`, `--mask`, `--dry-run`, `--no-raw`, `--verify`, `--keep-preview`, `-o`, `--in-place`.

## 만들지 않는 것

`inspect pii`, `edit sweep`, `security-gate`, `auto-redact-on-export`,

워터마크 제거, 계좌/여권 탐지 확장, DocumentCore 새 쿼리.

## 권위 우선순위

1. `mydocs/manual/cli_commands.md` 현재 절

2. 레시피 3·4·10 실측

3. 이 스킬의 픽스처 (1·2 와 모순되면 픽스처가 틀린 것)

`mydocs/tech/agent_security/*` 일부는 redact/inspect 구현 이전 설계 메모가 남아 있다.

그 문서의 `[설계] hiddenText 없음` 은 구형이다. CLI 매뉴얼이 이긴다.

## 시험 범위

계약 시험은 스킬 파일·픽스처 정합만 본다. 코어 구현을 바꾸지 않는다.

## 플래그  invent 금지 목록

에이전트가 만들고 싶어 하는 것 → 대신 쓸 것.

| 만들고 싶은 것 | 대신 |
|---|---|
| `--fail-on-dirty` | jq `.clean==true` |
| `--gate` | 재스윕 네 명령 + 술어 |
| `--redact-pii-kinds=account` | search + 사람, replace-text |
| `--strip-watermark` | 없음. 거부 |
| `--auto-no-raw-default-cli` | 스킬 자동화만 --no-raw |
| `--include-headers-fields` | 문서화된 사각지대. 발명하지 않음 |

## 파일 경계

이 PR 이 만지는 것: `.claude/skills/rhwp-security-sweep/**`,
`mydocs/working/agent_security_sweep.md`, `tests/cases/agent_security_sweep_*.rs`,
`tests/fixtures/agent_security_sweep/`, capability 카탈로그 한 행.

만지지 않는 것: `gym/`, 다른 스킬, DocumentCore, 새 [[bin]].
