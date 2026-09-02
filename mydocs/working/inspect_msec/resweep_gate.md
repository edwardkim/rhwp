# 재스윕 게이트 (#5476)

송신 경로의 닫는 술어는 눈이 아니라 봉투다.

```
edit redact --dry-run --no-raw   findingCount == 0
inspect hidden-text --json       clean == true
inspect injection --json         clean == true
inspect unicode --json           clean == true
```

어느 하나라도 거짓이면 배포하지 않고 처리 단계로 돌아간다.
탐지가 있어도 inspect 3축의 exit 는 0 이다. 실패와 발견을 종료 코드로 섞지 않는다.

평문 PII 는 3축 어디에도 안 걸린다. 그래서 redact dry-run 이 네 번째 질문이다.
3축이 모두 clean 이어도 dry-run 이 0 이 아니면 내보내지 않는다.

| 축 | 게이트 필드 | 통과 | 실패여도 exit |
|---|---|---|---|
| hidden-text | clean | true | 0 |
| injection | clean (+ highestConfidence) | true | 0 |
| unicode | clean | true | 0 |
| redact --dry-run | findingCount | 0 | 0 |

훑지 않은 영역(`scanScopes` 밖, `--include-offpage` 꺼진 off_page,
`--include-fields` 꺼진 누름틀)은 깨끗함이 아니라 검사 안 함이다.
