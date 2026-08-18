# 자동화 파이프라인 게이트

권위는 `mydocs/manual/cli_commands.md` 의 해당 절과 레시피 3·4·10 이다.
이 장은 기존 CLI 를 설명하고 소비 규칙을 고정한다. 새 명령·새 탐지 규칙을 발명하지 않는다.
## 권장 스크립트 골격

```bash

set -euo pipefail

IN=$1

OUT=$2

tmp=$(mktemp -d)

rhwp inspect hidden-text "$IN" --json > "$tmp/h.json"

rhwp inspect injection   "$IN" --json > "$tmp/i.json"

rhwp inspect unicode     "$IN" --json > "$tmp/u.json"

rhwp edit redact "$IN" --dry-run --no-raw --json > "$tmp/p.json"

jq -e '.clean==true' "$tmp/h.json"

jq -e '.clean==true' "$tmp/i.json"

jq -e '.clean==true' "$tmp/u.json"

# PII 가 있으면 적용

if jq -e '.findingCount>0' "$tmp/p.json" >/dev/null; then

  rhwp edit redact "$IN" -o "$tmp/r.hwp" --no-raw --verify --json

  rhwp edit sanitize "$tmp/r.hwp" -o "$OUT" --json

else

  rhwp edit sanitize "$IN" -o "$OUT" --json

fi

rhwp edit redact "$OUT" --dry-run --no-raw --json | jq -e '.findingCount==0'

rhwp inspect hidden-text "$OUT" --json | jq -e '.clean==true'

```

새 CLI 가 없다. 기존 명령을 조합한다.

## 로그

`$tmp/p.json` 을 아티팩트로 올릴 때 `noRaw==true` 와 raw 키 부재를 검사한다.

실패하면 업로드하지 않는다.

## 종료 코드

inspect 의 exit 0 을 `set -e` 성공으로만 읽지 않는다. jq -e 가 판정한다.

redact 사용법 오류(exit 2)는 파이프라인 실패가 맞다 — 산출 경로를 빼먹은 것.

## 병렬

3축 inspect 는 읽기 전용이라 병렬로 돌릴 수 있다. redact/sanitize 는 순차.

중간 파일을 공유 디렉터리에 남기지 않는다.
## 실패 주입

파이프라인 시험은 다음을 각각 한 번씩 넣어 본다.

1. findingCount>0 인 dry-run 봉투 → 적용 분기로 가는지
2. clean:false 인 hidden 봉투 → 공유가 막히는지
3. --no-raw 없는 봉투 → 아티팩트 업로드가 막히는지
4. redact 산출 경로 생략 → exit 2 로 실패하는지
5. sanitize 생략 → 절차 게이트가 거부하는지

이 다섯은 새 명령 없이 기존 봉투로 재현한다.
