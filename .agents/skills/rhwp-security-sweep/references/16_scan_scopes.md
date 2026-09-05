# 검사 범위 — scanScopes

권위는 `mydocs/manual/cli_commands.md` 의 해당 절과 레시피 3·4·10 이다.
이 장은 기존 CLI 를 설명하고 소비 규칙을 고정한다. 새 명령·새 탐지 규칙을 발명하지 않는다.
## 규칙

훑지 않은 영역은 '깨끗함'이 아니라 '검사 안 함'이다.

봉투의 `scanScopes` 가 범위를 밝힌다. 추측하지 않는다.

## injection 기본 범위

body, tableCell, textBox, equation, footnote, endnote, header, footer, caption.

깊이 상한 8. 표 안 표 안 글상자로 스택을 태우지 않는다.

## --include-fields 추가

fieldName, fieldGuide, fieldCommand, hiddenComment, fieldMemo.

누름틀 메타데이터는 본문 텍스트가 아니라 별도 축이다.

기본이 본문 위주인 이유: 오탐 예산. 서식은 한 번 더 돈다.

## hidden-text 범위

색·크기 판정은 조판 정보가 있는 글자.

쪽 밖은 `--include-offpage` 없이 제외. 제외를 clean 으로 쓰지 않는다.

## unicode 범위

본문 + 표 셀 + 글상자 + 수식. 1패스 코드포인트 스캔.

`kindFilter` 가 all 이 아니면 다른 축은 검사 안 함.

## redact 범위

본문 치환 경로(`replace_all_native`). 그림 속 글자는 밖.

fields 머리말/각주 사각지대와 겹친다. 눈으로 잡을 수밖에 없는 자리가 있다.

## sanitize 범위

메타·미리보기. 본문 아님. export-text 전후 동일이 계약.

## 범위 고지 문장

에이전트가 사용자에게 말할 때 쓰는 문장.

- `includeFields:false`: "누름틀 안내문은 이번 검사 범위가 아닙니다. 서식이면 --include-fields 로 한 번 더 돕니다."
- `includeOffPage:false`: "쪽 밖 배치는 기본 제외입니다. 필요하면 --include-offpage."
- `kindFilter != all`: "유니코드는 {filter} 축만 봤습니다."
- `minConfidence: high`: "low/medium 신호는 필터로 빠졌습니다. 깨끗함과 다릅니다."

## 표 — 누가 어디를 보나

| 명령 | 본문 | 표셀 | 필드 안내 | 쪽 밖 | 메타/미리보기 |
|---|---|---|---|---|---|
| hidden-text | ○ | ○(조판) | × | 옵션 | × |
| injection | ○ | ○ | 옵션 | × | × |
| unicode | ○ | ○ | ×(본문 패스) | × | × |
| redact | ○(치환 경로) | ○ | 사각 가능 | × | × |
| sanitize | × | × | × | × | ○ |

이 표 밖을 검사하는 새 명령을 만들지 않는다.
