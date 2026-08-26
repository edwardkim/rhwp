# 종료 코드 — 탐지 ≠ 실패

권위는 `mydocs/manual/cli_commands.md` 의 해당 절과 레시피 3·4·10 이다.
이 장은 기존 CLI 를 설명하고 소비 규칙을 고정한다. 새 명령·새 탐지 규칙을 발명하지 않는다.
## 규칙 3 — 판정은 데이터다

`inspect` 3축은 신호가 있어도 exit 0 이다 (#2707).

1은 런타임 실패 전용이다. '위험 문서 발견'은 정상 판정 결과다.

소비자는 봉투의 `clean` / `findingCount` / `highestConfidence` 로 분기한다.

exit 0 + `clean:false` 는 DATA 다. 도구가 고장난 것이 아니다.

## 표

| 명령 | 0 | 1 | 2 | 3 |

|---|---|---|---|---|

| inspect 3축 | 판정 성공(신호 포함) | 열기/런타임 | 인자(threshold 범위, kind 오타) | — |

| redact dry-run | findings 보고 | 런타임 | 사용법 | — |

| redact -o | 적용(0건이면 파일 없음) | 런타임 | -o 없음, mask 불법, 원본 자기경로 | verify 불일치 |

| sanitize | 적용 | 런타임 | 사용법 | — |

## 스크립트 안티패턴

```bash

# 잘못 — 신호가 있으면 set -e 가 죽지 않으므로, 반대로 clean 을 안 읽으면 통과로 착각

rhwp inspect injection 첨부.hwp --json

echo 깨끗함   # clean 을 안 봤다

```

```bash

# 옳음

rhwp inspect injection 첨부.hwp --json | jq -e '.clean == true'

```

exit 를 재해석하는 래퍼를 만들지 않는다. 봉투를 읽는다.

## redact 사용법 오류

산출 경로 없이 실행하면 stderr 안내 + stdout 0바이트 + exit 2.

실측 문구: '마스킹은 되돌릴 수 없습니다. 산출 경로를 -o <출력> 으로…'

`--mask` 두 글자 이상은 조용히 자르지 않고 exit 2.

## verify

`--verify` 는 저장 직후 IR 자기검증. 차이 시 exit 3.

배포하지 말고 `ir-diff` 로 원인을 좁힌다.
