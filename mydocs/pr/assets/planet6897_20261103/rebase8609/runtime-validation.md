# 최신 리베이스 fresh WASM의 실제 API·브라우저 실행

제품 source `ba2ebfd34a4ebf402bce8e002254b672a09ef764`, base `f529dc92d4d172bf32390d7749743a45d9e41bfc`.
실제 WASM 빌드 head는 `dc703ef8180e0c410aa754011c34660e33e68480`이며 snapshot source와의 제품 입력 해시가 같다.
최적화 wrapper의 실제 종료 후 별도 고정한 package만 사용했다.
JS SHA-256 `bb9be9d806756697eddee5d185e80e9256bd0d97947d78a94777222100f12393`.
WASM SHA-256 `3a9125c0a3fa65e1ebbe45abfd0e2070248fbb96d1f5e083d6e3108baa62ed61`.

- 실제 브라우저: **271 assertion·17 Worker·12쪽 렌더/저장**, exit 0.
- Worker는 정상 11개·메모리 실패 대조 3개·engine trap 대조 3개다. 의도한 실패 대조를 새 결함으로 집계하지 않는다.
- 공개 API 계약 2개: 번호 자리표 분할 후 HWP/HWPX 재개방, 실제 TAC 사진 2열·3줄의 소유·정렬·비겹침 보존.
- 최신 WASM의 PDF 가져오기 3종과 실제 편집 1종을 실행하고 기존 Native 생성 입력과 바이트가 같음을 대조했다. 이번에 Native importer까지 새로 실행했다고 표시하지 않는다.
- 실제 API·브라우저 결과는 전쪽 시각 측정이나 원 결함의 수정 전 FAIL 검출 증거를 대신하지 않는다.

| 실행 | 실제 명령·환경 | 시작 → 종료 UTC | exit·로그 SHA-256 |
| --- | --- | --- | --- |
| public-api | `RHWP_REBASE_API_PKG=/home/tsjang/rhwp/output/pr-review/planet6897-20261103/producer-packages/rebase8609-dda217dba0 node --test /home/tsjang/rhwp/output/pr-review/planet6897-20261103/rebase-22947f6a90-public-api.test.mjs` | 2026-11-05T01:47:43.913787+00:00 → 2026-11-05T01:47:45.887085+00:00 | 0 · `d01c72ca75ee4341ad112b20214d4a4547518c4d87c891b38e0bfff2a46bab3d` |
| browser-runtime | `PDF_IMPORT_PKG=/home/tsjang/rhwp/output/pr-review/planet6897-20261103/producer-packages/rebase8609-dda217dba0 PDF_IMPORT_EVIDENCE=/home/tsjang/rhwp/output/pr-review/planet6897-20261103/wasm-runtime-rebase8609-dda217dba0 node --test scripts/tests/pdf-import-wasm.test.mjs` | 2026-11-05T01:47:45.904871+00:00 → 2026-11-05T01:48:10.054377+00:00 | 0 · `40a48966d0da971a8fad26e7dbc8b3a9dad80540e0e9cbb6a87f445e3b0a1990` |
| producer-parity | `RHWP_REVIEW_WASM_PKG=/home/tsjang/rhwp/output/pr-review/planet6897-20261103/producer-packages/rebase8609-dda217dba0 RHWP_REVIEW_PARITY_OUTPUT=wasm-producer-parity-rebase8609-dda217dba0 node /home/tsjang/rhwp/output/pr-review/planet6897-20261103/check-wasm-producer-parity.mjs` | 2026-11-05T01:48:10.065770+00:00 → 2026-11-05T01:48:27.249377+00:00 | 0 · `00b51604fb46f7c54c289e0481eb541f2a1d9c614f029e32d09b6fcc64f15690` |

## 가져오기·실제 편집의 산출물 대조

| 실행 | 기존 Native 생성 입력 SHA-256 | 최신 WASM 산출물 SHA-256 | 동일 |
| --- | --- | --- | --- |
| PDF 가져오기 · 2010 | `54df47aa980966679a67f579916972d26d6147a08d327368398abc2552d84412` | `54df47aa980966679a67f579916972d26d6147a08d327368398abc2552d84412` | 예 |
| PDF 가져오기 · 2020 | `f1929a65e3901b90087fe9ccb4b0e446ceed32926b1479d4307e22474d23f49e` | `f1929a65e3901b90087fe9ccb4b0e446ceed32926b1479d4307e22474d23f49e` | 예 |
| PDF 가져오기 · plain | `5a13ddd9b7fe2a51c95009e34c8e0a27337d35f145c637a3193025e9c455b1c6` | `5a13ddd9b7fe2a51c95009e34c8e0a27337d35f145c637a3193025e9c455b1c6` | 예 |
| 실제 편집 · samples/issue6123/3112461_railway_emc_criteria.hwpx | `8009a8f11dd1f53545ed457fdee47326310caf26c56f2eda9bde7d226d7ab9e9` | `8009a8f11dd1f53545ed457fdee47326310caf26c56f2eda9bde7d226d7ab9e9` | 예 |

## 정식 브라우저 검사 재실행

검증할 source에서 최적화 WASM을 먼저 빌드한 뒤 다음 저장소 내 명령을 실행한다.

```bash
PDF_IMPORT_PKG=pkg \
PDF_IMPORT_EVIDENCE=output/pr-review/검증명/wasm-runtime \
  node --test scripts/tests/pdf-import-wasm.test.mjs
```

임시 공개 API·생성물 대조 helper와 원시 실행 로그는 로컬 output에 보존한다.
그 실행을 정식 Rust 회귀 추가나 독립 Print 시각 통과로 승격하지 않는다.
[전체 진행 상태](../README.md), [최신 Native 영향 경계](native-own-validation.md)와 함께 확인한다.
