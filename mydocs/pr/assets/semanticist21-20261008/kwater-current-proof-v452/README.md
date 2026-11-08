# KWATER 최신 fresh WASM 전쪽 증거

생산 코드: `b4ff8c81b5d986baf47bad9a3eee83cbd9d8367b`.
WASM SHA-256: `94fe70f13609c5a006b8e9044463fac7ae96fe5fe03ae33bc0c0153b19f7867c`.

fresh 전27쪽 exit0, 최저3쪽 96.87092%, 미달·누락0,
원본/독립 Print 쪽수 27/27. 현재 캡처의 대표 쪽을 직접 열었다.
26쪽 RGBA 동일, 10쪽 한 픽셀의 회색 1단계 차이. 27쪽 전체 render tree 동일.
글자 획·자간·래스터 차이는 남으며 완전 일치를 주장하지 않는다.
현재 Native 전쪽 재검증 및 actual Canvas는 대기 중이다. 양쪽 완료나 PR 승인으로 확대하지 않는다.

## 입력과 독립 한컴 Print

- `samples/hwpx/k-water-rfp.hwpx` — SHA-256 `036ae73c4982696b9aad50c43a1c47f50871d1447aef3209e058f1b30fa2fc14`; 실제 파일과 HEAD blob 일치.
- `pdf/hwpx/k-water-rfp-hwpx-2020.pdf` — SHA-256 `ce99496a432ed24e795d2050c28f208fb086556d833a790d9ed7beddcc5f3196`; 실제 파일과 HEAD blob 일치.

## 재현과 판독

원문 명령·글꼴·소스 해시·TSV·로그는 ignored
`output/pr-review/semanticist21-20261008/full-width-visible-alignment-v452/whole-fresh-execution-plan.json`, `whole-fresh-results.json`,
`kwater-fresh/kwater/silhouette_manifest.json`, `representative-review-provenance.json`,
`kwater-fresh-direct-reading.json`, `kwater-fresh-prior-pixel-comparison.json`에 보존한다.
전체 페이지 `--compat 2022 --silhouette-only --wasm-pkg pkg`를 실행했고
검증한 실제 공급 글꼴과 `--embed-fonts full`을 사용한다.

3쪽 문단·목록, 24쪽 이어받기/중첩 표·뒤 내용, 10쪽 본문/예시 표를 대조했다.
24쪽 표 확대는 원본 페이지 좌표 `(190,430)-(705,830)` 2배이며 왼쪽 fresh, 오른쪽 Print다.

## PNG 해시

- `fresh-p003-review.png` — SHA-256 `6056cf846125e90e5f4989fb8d2b4ea4350b17e219797f2a27399a86e0330754`
- `fresh-p003-overlay.png` — SHA-256 `b503d218d421c20aa26ae7438f2a63be39f8fdd755ce4f53fa18639821b7fc70`
- `fresh-p024-review.png` — SHA-256 `967e9849e6987ec1c0ea5a739c54e15885f6561716c1ff8c9ae3e97754ab7366`
- `fresh-p024-overlay.png` — SHA-256 `57511a63d94a358b98ccf95093e968866608a25a37c555b4f83e348888207b90`
- `fresh-p010-review.png` — SHA-256 `1f7b72a702d97ad2691dae4a78409d181c40d2f84e562dd1b4167c1339ed2095`
- `fresh-p010-overlay.png` — SHA-256 `aa360f92a66fc458c682cd0db8b59ccc15305be1490a4731c692dbc4b1c6a33b`
- `fresh-p024-table-zoom.png` — SHA-256 `bc51ceab9caca3979e9f5717dfbb556cb02720b8b5934b5312c87f029cd0ed11`
