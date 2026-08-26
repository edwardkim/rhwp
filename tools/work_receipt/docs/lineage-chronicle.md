# lineage — 연대기 무결

```
rhwp lineage <머리캡슐> [--deep] [--keyring <키링>] [--anchor-log <로그>] [--json]
```

머리에서 뿌리로 거슬러 오른다. `parent.capsule` 이 상대 경로면
**현재 캡슐의 디렉터리**에 붙인다.

## 3축

| 축 | 언제 | 참인 조건 |
| --- | --- | --- |
| `parentOk` | 자식이 부모를 지목할 때 | 기록 해시 == 부모 파일 바이트 |
| `lineageOk` | 같은 순간 | 부모 `outputSha256` == 자식 `inputSha256` |
| `reproduced` | `--deep` | 임시 재실행 해시·step 수·입력 해시 일치 |

뿌리 링크는 세 축이 모두 null 이다. 그래도 `valid=true`, `depth=1`.

하나라도 false 면 `brokenAt` 이 그 캡슐을 가리키고 exit 3.
머리 파일 없음은 exit 1 (IO). 중간 부모 없음은 체인 깨짐(exit 3).

## fail-closed

다음을 root 로 오인하지 않는다.

- `parent` 키 없음
- `parent.sha256` 없음 / 비 hex
- `parent.capsule` 없음
- `planSha256` 없음
- `plan` ≠ `planText`
- `receipt.steps` ≠ `plan.steps.len`

`--keyring` 이 없으면 `signerOk` 축 자체가 없다.
`--anchor-log` 의 `anchoredOk=false` 는 체인을 깨지 않는다
(등재 강제는 게이트의 일).

순환은 가드 1000. 픽스처는 바늘만 고정하고 1000링크를 만들지 않는다.
