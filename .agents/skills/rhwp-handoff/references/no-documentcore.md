# DocumentCore 편집 로직을 발명하지 않는다

세션이 바뀌었다고 문서 모델 코어를 고치지 않는다. 인계는 운영이다.
편집이 필요하면 이미 있는 표면만 쓴다.

## 써도 되는 것 (기존)

- `rhwp edit …` / `rhwp run <plan.json>`
- `rhwp replay --capsule` / `--parent`
- `python tools/handoff/orchestrator.py`
- 다른 스킬이 가리키는 CLI (`fields`, `fill-fields`, `export-tables` …)

## 쓰면 안 되는 것

- `src/document_core/` 아래에 새 편집 연산 추가
- incoming 이 "이전 세션이 못 끝낸 버그"라며 코어 패치를 인계의 다음 명령으로 적기
- `DocumentCore::apply_*` 를 세션 픽스처에 호출 예로 넣기
- 새 `rhwp edit session-resume` 같은 명령

## 왜

인계 묶음이 깨진 것은 대부분 파일 부재·해시 불일치·dirty 트리·디스크다.
코어를 고쳐서 해결되는 문제가 아니다. 코어가 진짜 필요하면 **별 이슈**로
빼고, 이 스킬의 working doc 다음 명령에 넣지 않는다.

## 픽스처

`fixtures/envelopes/documentcore_invented.json` — `rejected: true`,
`_skillMeta.exit` 2 (사용법: 발명 호출).
`fixtures/scenarios` 의 `refuse-documentcore-*` 항목.
