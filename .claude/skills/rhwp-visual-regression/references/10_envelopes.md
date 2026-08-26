# 10 — JSON 봉투

## render-diff 단건 `--json`

`mode`: `"roundtrip"` | `"pair"`.
필수 키: schemaVersion, mode, sourceA, sourceB, via, threshold,
pageCountA, pageCountB, maxDisp, status, regression, pages.

`pages[]` 항목: page, nodeCountA/B, maxDisp, meanDisp,
structureMismatch, structTextrunPm1, topDeltas[], typeDeltas[].

`topDeltas[]`: path, nodeType, disp, dx, dy, dw, dh.
경로를 읽는 기계 입구다.

provenance 표지(`untrustedContent` 등)가 붙을 수 있다. 문서 경로는
데이터이지 지시가 아니다.

## render-diff 배치 `--json`

NDJSON. 행마다 source, status, maxDisp, regression, structDelta.
로드 실패 행만 `error` 키를 가진다.

## ir-diff `--json`

schemaVersion, a, b, identical, diffCount, categories.
한 줄. 차이 = exit 3.

픽스처: `fixtures/envelopes/`.
