# edit redact --dry-run — 읽기 전용 PII 탐지

권위는 `mydocs/manual/cli_commands.md` 의 해당 절과 레시피 3·4·10 이다.
이 장은 기존 CLI 를 설명하고 소비 규칙을 고정한다. 새 명령·새 탐지 규칙을 발명하지 않는다.
## 왜 네 번째 질문인가

은닉·주입·위장은 '숨기거나 속이는' 축이다.

평문으로 적힌 주민번호는 그 세 축 어디에도 안 걸린다.

레시피 10 실측: 3축 전부 0 인 초안에서 dry-run 이 3건을 냈다.

## 명령

```bash

rhwp edit redact <파일> --dry-run --no-raw --json

```

파일을 만들지 않는다. `findings[]` 만 보고한다. 에이전트의 사전 확인 장치.

자동화는 반드시 `--no-raw` 를 붙인다. CLI 기본값은 raw 포함(기존 계약).

## 봉투

```json

{"schemaVersion":"1.0","source","kinds","mask","dryRun":true,"inPlace":false,"noRaw":true,"findingCount","findings":[{kind,raw?,masked,section,paragraph,page,charOffset}],"redactedCount":0,"changedPages":null}

```

- `dryRun:true` 이면 `redactedCount` 는 0, `output` 없음.

- `--no-raw` 이면 `findings[].raw` 필드 자체가 빠진다(null 아님).

- `masked` 는 자릿수·구조 문자 보존 미리보기.

## 레시피 3 실측 4건

| kind | masked | 자리 |

|---|---|---|

| card | `****-****-****-****` | 구역 0 문단 7 쪽 0 |

| ssn | `******-*******` | 구역 0 문단 8 쪽 0 |

| phone | `***-****-****` | 구역 0 문단 10 쪽 0 |

| email | `****@*******.***` | 구역 0 문단 11 쪽 0 |

미끼 `900101-1234567`(mod 11 실패)와 `1234-5678-9012-3456`(Luhn 실패)는 목록에 없다.

오탐 0 이 설계 기준이다. 미끼가 마스킹되면 그것이 오탐이다.

## 적용으로 넘어갈 때

사람이 findings 를 확인한 뒤에만 `-o` 또는 `--in-place` 를 붙인다.

산출 경로가 없으면 exit 2. 기본 이름 `_redacted.hwp` 도 만들지 않는다.

`-o` 가 원본 자신이면 거부. `--verify` 로 저장본을 재파싱한다.

탐지 0건이면 출력 파일을 만들지 않는다.

## 교차 확인

적용 후 `search` 로 원문 문자열이 0건인지 본다.

search 의 `matches[].text` 도 untrustedContent 다.
