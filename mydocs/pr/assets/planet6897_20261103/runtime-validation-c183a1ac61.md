# c183 최적화 WASM의 실제 API·브라우저 검증

제품 source `c183a1ac61c2aa4188989fd93032cae68d3538c1`.
JS SHA-256 `bb9be9d806756697eddee5d185e80e9256bd0d97947d78a94777222100f12393`, WASM SHA-256 `f79bffaabc76cd4917791dc3dbececa2560cba8001fcb17578ba03076a5ce77e`.

실제 browser assertion 271개, worker 17개, 렌더·저장된 페이지 12쪽을 확인했다. 정상 worker 11개 외에 메모리 실패 3개·engine trap 3개 대조를 포함한다. 공개 API 계약 2개는 기능 호출 검증이며 원 결함의 수정 전 검출 증거를 대신하지 않는다.

PDF 편집 가능 가져오기 3종과 실제 편집 1종을 최신 WASM API로 실행해, 기존 Native 생성 입력과 결과 바이트가 같음을 대조했다. 이번에 Native PDF 가져오기까지 새로 실행했다고 표현하지 않는다.

| 실행 | 실제 명령 | 시작 → 종료 UTC | exit · 로그 SHA-256 |
| --- | --- | --- | --- |
| public-api | `node --test /home/tsjang/rhwp/output/pr-review/planet6897-20261103/rebase-22947f6a90-public-api.test.mjs` | 2026-11-04T21:30:04.312825+00:00 → 2026-11-04T21:30:06.310607+00:00 | 0 · `25adae3a078806f25a330f79cf2df9adb0075c5b65346e4eae16f72770f356d3` |
| browser-runtime | `node --test scripts/tests/pdf-import-wasm.test.mjs` | 2026-11-04T21:30:06.311340+00:00 → 2026-11-04T21:30:33.621608+00:00 | 0 · `1cf2c4e8f6f87f76e035d602a6712a4f7f0ab5ba3dd0706d7773df5954180635` |
| producer-parity | `node /home/tsjang/rhwp/output/pr-review/planet6897-20261103/check-wasm-producer-parity.mjs` | 2026-11-04T21:30:33.622652+00:00 → 2026-11-04T21:30:51.010116+00:00 | 0 · `e802e869f3a79908f4325538132d9110d7b2cea11ca759bea7c83ce6f2e30519` |
| wasm-newbase3 | `/home/tsjang/rhwp/venv/bin/python /home/tsjang/rhwp/output/pr-review/planet6897-20261103/run_visual_pairs.py wasm --pairs rebase-22947f6a90-focused-pairs.tsv --code-label marker-slot-c183a1ac61 --rhwp-bin /home/tsjang/rhwp/output/pr-review/planet6897-20261103/rhwp-marker-slot-c183a1ac61 --wasm-pkg /home/tsjang/rhwp/output/pr-review/planet6897-20261103/producer-packages/marker-slot-c183a1ac61 --page-workers 1` | 2026-11-04T21:30:51.010832+00:00 → 2026-11-04T21:36:40.195222+00:00 | 0 · `bd6ea29bff6c9916465bbe3bc58a25ce14eeccc7a1dea899f135315c57ba8f9c` |

후속 37e 보정 source의 완료로 승격하지 않는다. 이 실제 API 실행과 전쪽 시각 측정·전체 Rust·최종 PR 판정은 각각의 증거다.
