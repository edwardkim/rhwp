# 예외: 부모 해시 불일치

## 신호

자식 캡슐의 `parent.sha256` 이, `parent.capsule` 을 자식 파일 기준으로 푼
실파일 바이트 SHA-256 과 다르다. 또는 자식 `receipt.inputSha256` 이 부모
`receipt.outputSha256` 과 다르다.

전자는 부모 파일이 발급 이후 바뀌었거나 자식이 위조된 것이다.
후자는 연대기가 끊긴 것이다 (`lineageOk` 의 정의 — work-receipt 단어).

## 하지 않는 것

- 부모 파일을 자식이 기억하는 해시로 **다시 써** 맞춘다
- 자식 `parent.sha256` 을 실파일에 맞게 에디터로 고친다 (불변 파괴)
- 불일치를 무시하고 `--parent` 를 한 번 더 붙인다

## 하는 것

1. 후속 `--parent` 를 붙이지 않는다
2. `--verify-journal` 과 `result.json` 으로 오케스트레이터 쪽은 따로 본다
   (캡슐 체인과 위임 봉투는 다른 축)
3. 단건 재현이 필요하면 work-receipt `rhwp lineage` 로 보낸다
4. 새 작업은 새 뿌리 캡슐로만. working doc 에
   `parent_hash_mismatch` 와 두 해시를 적는다
5. 표본 exit 3 (판정) —
   `fixtures/exceptions/parent_hash_mismatch.json`,
   `fixtures/envelopes/parent_hash_mismatch.json`

## 워크스루

[`../examples/09_parent_hash_mismatch.md`](../examples/09_parent_hash_mismatch.md).
레이아웃: `fixtures/layouts/parent-mismatch/`.
