# 11 — 실패 신호 → 처방

SKILL.md 정지 표의 본문이다.

| 신호 | 원인 | 처방 | 정지 | 전형 exit |
| --- | --- | --- | --- | --- |
| fields/fill exit 1, stdout 빈 | 파일 없음·쓰기 실패 | 경로 확인. 원본 불변 | F01 | 1 |
| `fieldCount: 0` | 누름틀 없음 | table-exchange | F02 | 0 |
| `textSecurity.status` ≠ `"clean"` | 은닉/주입 신호 | security-sweep, 채우지 않음 | F03 | 0 |
| 질문이 목록뿐 | — | names/guide/memo 보고 정지 | F04 | 0 |
| `ambiguous` 비어 있지 않음 | 같은 이름 여러 곳 | `이름[N]` | F05 | 0 |
| `notFound` 에 필드명 | 오타·없는 이름 | fields 의 name 복사 | F06 | 0 |
| `notFound` 가 name-field 뿐 | 파일명 컬럼 | 게이트에서 제외 | F11 | 0 |
| `verify.identical: false` | 재파싱 차이 | svg / render-diff. 산출은 남음 | F12 | 3 |
| `오류: --data 에 데이터 행이 없습니다` | 헤더만 | 상류 명단 | F09 | 2 |
| 깨진 JSON / `--data` 없음 | 사용법 | UTF-8 JSON | — | 2 |
| `batch fill` 무반응 | stdin 축 혼동 | `--form --data` | F10 | 0 |
| `overflow` 비어 있지 않음 | 그림이 쪽 밖 | x/y/width/height | F08 | 0 |
| `removedCount: 0` | 이미 정리 | 멱등 | F08 | 0 |
| `stream did not contain valid UTF-8` | CP949 | UTF-8 재저장 | F01 | 1 |
| 알 수 없는 하위명령 | 발명 | cli_commands 만 | F14 | 2 |
| 머리말 칸이 목록에 없음 | 사각지대 | 재귀 확장 금지, 사람 보고 | F14 | 0 |

## 성공처럼 보이는 미완료

다음 네 개는 exit 0 이다. 그래도 제출 불가.

1. `ambiguous` 잔류
2. `notFound` 잔류 (name-field 제외 후)
3. `filledCount` 가 의도보다 작음
4. `textSecurity` 미확인

게이트는 exit 와 봉투를 **둘 다** 본다.
