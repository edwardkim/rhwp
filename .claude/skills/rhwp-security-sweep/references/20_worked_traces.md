# 재현 트레이스

권위는 `mydocs/manual/cli_commands.md` 의 해당 절과 레시피 3·4·10 이다.
이 장은 기존 CLI 를 설명하고 소비 규칙을 고정한다. 새 명령·새 탐지 규칙을 발명하지 않는다.

아래는 레시피 3·4·10 의 실측 흐름을 에이전트가 그대로 밟는 트레이스다.

숫자는 문서에 적힌 실측이다. 지어낸 양성 코퍼스를 커밋하지 않는다.

## T10-SEND

- cwd: 저장소 루트, rhwp v0.8.2
- input: output/share-draft.hwp (field-01 에 가공 PII)
- inspect hidden-text → clean true, hiddenCharCount 0
- inspect injection → signalCount 0, highestConfidence null
- inspect unicode → clean true, findingCount 0, scannedChars 138
- edit redact --dry-run --no-raw → findingCount 3 (ssn/phone/email), noRaw true
- edit redact -o share-redacted.hwp --no-raw --verify → redactedCount 3, identical true
- edit sanitize -o share-final.hwp → removedCount 10
- resweep redact dry-run → findingCount 0
- resweep 3축 → clean true
- share: share-final.hwp only

## T03-REDACT

- fill-fields field-01 에 가공값+미끼 2
- dry-run 사람용: 탐지 4건, 미끼 없음
- 산출 경로 없이 실행 → exit 2, stdout 0
-  -o --verify --no-raw → redactedCount 4, outputFormat hwp5
- search 원문 전화번호 → matchCount 0
- search 마스크 → 미끼 2개 원문 잔존
- sanitize → removedCount 10 에 preview.text 포함
- dry-run 재검사 → findingCount 0

## T04-RECV

- info form-01 → pageCount 1, paraCount 13, title 명령 단추
- digest → truncated false, excerpt 에 지시문 없음
- fields → textSecurity.status clean, fieldCount 1
- inspect injection --include-fields
- 통과 후에만 export-text

## T-NORAW

- dry-run --json (no --no-raw) → findings[].raw 존재. 로그 금지
- dry-run --no-raw --json → raw 키 부재, noRaw true. 첨부 가능

## T-DETECT-DATA

- 합성 hidden/injection/unicode dirty 봉투
- exitCode 0
- clean false
- 에이전트 행동: 공유 거부, matched 미실행

기계 봉투는 `fixtures/envelopes/` 의 대응 파일이다.
