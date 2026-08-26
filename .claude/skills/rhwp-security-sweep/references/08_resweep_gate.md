# 재스윕 게이트 — findingCount==0 AND clean==true

권위는 `mydocs/manual/cli_commands.md` 의 해당 절과 레시피 3·4·10 이다.
이 장은 기존 CLI 를 설명하고 소비 규칙을 고정한다. 새 명령·새 탐지 규칙을 발명하지 않는다.
## 술어

처리가 끝났다고 믿는 것과 기계가 0 이라고 말하는 것은 다르다.

최종본에 1·2단계를 다시 돌린다.

```text

SHARE := (redact.findingCount == 0)

      AND (hidden.clean == true)

      AND (injection.clean == true)

      AND (unicode.clean == true)

```

하나라도 거짓이면 공유하지 않고 처리로 돌아간다.

## 명령

```bash

rhwp edit redact 배포본.hwp --dry-run --no-raw --json

rhwp inspect hidden-text 배포본.hwp --json

rhwp inspect injection   배포본.hwp --json

rhwp inspect unicode     배포본.hwp --json

```

jq 게이트 예:

```bash

rhwp edit redact 배포본.hwp --dry-run --no-raw --json | jq -e '.findingCount == 0'

rhwp inspect hidden-text 배포본.hwp --json | jq -e '.clean == true'

```

## 레시피 10 실측 통과

```json

{"dryRun":true,"findingCount":0,"findings":[],"redactedCount":0}

{"clean":true,"hiddenCharCount":0}

{"clean":true,"signalCount":0,"highestConfidence":null}

{"clean":true,"findingCount":0}

```

## 실패 사례

| id | 남는 것 | share |

|---|---|---|

| G02 | findingCount 3, 3축 clean | 아니오 |

| G03 | hidden clean false | 아니오 |

| G04 | injection clean false | 아니오 |

| G05 | unicode clean false | 아니오 |

| G08 | 전부 dirty 인데 exit 0 | 아니오 |

exit 0 은 탐지 성공이다. 게이트를 통과한 것이 아니다.

## sanitize 짝은 숫자에 안 보인다

G07: 게이트 숫자는 통과처럼 보여도 sanitize 를 안 돌렸으면 미리보기가 남을 수 있다.

절차상 짝을 강제한다. 숫자만 보지 않는다.

## 픽스처

`fixtures/gate_cases.json` 과 `fixtures/envelopes/resweep_*.json`.

테스트는 share 기대와 술어가 같은지 본다.
