# 03 Phase A — 코퍼스 지도 (실패한 문서는 실패로 남긴다)

정본: playbook §3 단계 A, engagement.py `map_corpus`.

## 하는 일

corpus 를 재귀 순회해 확장자가 `.hwp`/`.hwpx`(대소문자 무시)인 파일을
정렬한 뒤, 문서마다 `rhwp info --json` 을 친다. 광고되면 `explain --json`.

결과는 `corpus_map.json`:

```json
{
  "schemaVersion": "1",
  "generatedBy": "tools/strategist/engagement.py",
  "corpus": "/abs/path",
  "documentCount": 4,
  "mappedCount": 2,
  "documents": [
    {"file": "공개_공고.hwpx", "sizeBytes": 88210, "status": "ok", "info": {}},
    {"file": "암호_내부.hwp", "sizeBytes": 12004, "status": "failed", "infoExit": 2}
  ]
}
```

## 실패는 데이터다

`info` 가 0이 아니거나 시간 초과면:

- `status` = `"failed"`
- `infoExit` = 정수 또는 시간 초과 시 `null`
- 그 행을 **배열에서 제거하지 않는다**
- `documentCount` 는 파일 수, `mappedCount` 는 `ok` 수

에이전트가 "실패한 파일을 빼고 다시 돌리면 깔끔하다"고 판단해도
**빼지 않는다.** 전수성의 정의가 "선언한 코퍼스 전부가 지도에 남는 것"
이기 때문이다. SWS L2 가 이 기록을 다시 본다.

## 하지 않는 것

- 실패 문서를 휴지통/별도 폴더로 이동해 재실행
- `mappedCount` 만 보고 `documentCount` 를 숨김
- info 실패를 search 단계에서도 침묵 스킵(search 실패는 대장 `failures`)
- 암호 문서를 추측 비밀번호로 여는 새 명령 발명

## 읽기 순서

1. `documentCount` 와 `documents.length` 가 같은지 본다.
2. `mappedCount` < `documentCount` 이면 failed 행의 `file`·`infoExit` 를
   회신 1부에 적는다.
3. 고객에게 암호·손상 사본을 요청할지는 에이전트 판단이다. 다만 요청
   전에 실패를 지워 성공처럼 보이게 만들지 않는다.

## 픽스처

- `fixtures/corpus_maps/all_ok.json`
- `fixtures/corpus_maps/mixed_failed.json`
- `fixtures/corpus_maps/all_failed.json` (search 전 실패 — 엔진은 search
  가 전부 실패하면 RuntimeError)

예제: [examples/03_failed_doc_stays.md](../examples/03_failed_doc_stays.md),
[examples/15_unreadable_corpus.md](../examples/15_unreadable_corpus.md).

다음: [04_evidence_ledger.md](04_evidence_ledger.md).
