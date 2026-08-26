# replay verify — 제3자 재현

상대가 준 것은 계획과 산출 해시 두 장이다. 파일을 믿으라는 말이 아니다.

```
rhwp replay --plan-json '<같은 계획>' --expect-output-sha256 <64hex> --json
```

## 읽는 법

| 봉투 | 의미 |
| --- | --- |
| `mode=verify` | 기대를 줬다 |
| `reproduced=true` | 임시 재실행 산출 해시 = 주장 |
| `reproduced=false` | 주장 기각. exit 3 |
| `outputSha256` | 방금 재실행한 실측 |
| `expectedOutputSha256` | 상대 주장 에코 |
| `toolVersion` | 재현 불일치 때 먼저 대조할 힌트 |

`toolVersion` 이 다르다고 audit/lineage 가 자동 실패하지는 않는다.
스킬 pitfalls 가 선대조를 요구하는 이유다. 픽스처
`audit-layouts/toolversion-mismatch` 가 그 축을 고정한다.

## 함정

- 기대 해시 형식 오류는 판단이 아니라 사용법이다.
- 사용자 `output` 경로는 verify 중에도 만들어지지 않는다.
- 계획 파일이 없으면 exit 1 (IO). 깨진 JSON 은 exit 2.
