# layout-anomaly M02-f 판정 픽스처

이 폴더는 `rhwp layout-anomaly` 의 판정·성적표를 합성 렌더 트리로 고정한다.
레이아웃 엔진·canvaskit_policy·serializer·pdf·equation 은 바꾸지 않는다.

## 가족

- `overflow_simple`: 80건
- `overflow_nested`: 13건
- `overlap`: 48건
- `text_overlap`: 20건
- `off_canvas`: 42건
- `empty_page`: 31건
- `visibility`: 4건
- `combined`: 3건
- `tolerance`: 24건

## 읽는 법

- `trees/*.json` — 페이지/본문 상자 + 노드 트리 + `expect` 판정.
- `matrices/*.tsv` — 같은 건의 한 줄 행렬. 배치 리포트·회귀 표용.
- `transcripts/*` — 사람 성적표·JSON 봉투·배치 NDJSON 표본.

테스트는 `tests/cases/layout_anomaly_m02f_fatten.rs` 가 트리를 재조립해
`scan_page` 실측과 `expect` 를 대조한다.
