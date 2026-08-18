# 08 종료 코드와 판정 봉투

정본: playbook §6.

## 코드

| exit | 언제 | 에이전트 |
| --- | --- | --- |
| 0 | 엔게이지먼트 완료 또는 validate pass | 산출물 사용 가능 |
| 1 | 실행 실패 (capabilities 실패, 필수 명령 미광고, search 전패, scaffold 실패) | 바이너리·광고·타임아웃을 점검. 재시도는 같은 engagement |
| 2 | 입력 오류 (파일 없음, JSON 깨짐, 필수 필드, 빈 corpus, 빈 questions) | engagement.json 을 고친다 |
| 3 | `--validate` 전용. 근거 없는 주장 | 위반 목록을 고치고 재검증. 납품 금지 |

exit 3 은 도구 고장이 아니다. 판정이 데이터로 나온 것이다.

## 엔게이지먼트 stdout

```json
{
  "schemaVersion": "1",
  "generatedBy": "tools/strategist/engagement.py",
  "mode": "engagement",
  "objective": "…",
  "corpusDocuments": 5,
  "mappedDocuments": 4,
  "evidenceCount": 18,
  "searchFailures": 2,
  "questionCount": 3,
  "claimCount": 2,
  "noEvidenceQuestions": ["Q3"],
  "scaffoldAdvertised": false,
  "scaffold": null,
  "artifacts": ["corpus_map.json", "evidence.json", "spec.json"]
}
```

`mappedDocuments` < `corpusDocuments` 이면 회신에 실패 문서를 적는다.

## validate stdout

```json
{
  "mode": "validate",
  "claimCount": 2,
  "ledgerEntryCount": 18,
  "violationCount": 1,
  "violations": [{"claim": "CLAIM-2", "kind": "unlinked", "detail": "…"}],
  "verdict": "fail"
}
```

`swsAudit` 는 부가. 감사 예외는 `swsAudit.error` 로만 남고 exit 를
바꾸지 않는다.

## 실패 stdout

입력 오류·실행 실패는 설명을 stderr 에 남긴다. 에이전트는 stderr 를
버리고 "성공"으로 읽지 않는다.

예제: [examples/17_malformed_engagement.md](../examples/17_malformed_engagement.md).

다음: [09_out_of_scope.md](09_out_of_scope.md).
