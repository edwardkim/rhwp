# Task #7957 5단계 — F4 stream·마스크 호환과 실물 Native 가져오기

- Issue: #7957–#7963 공동 구현
- 상태: Native 실물 여섯 PDF 전체 313쪽 가져오기와 집중 계약 133 PASS.
  새 Docker WASM/실제 Worker·전쪽 Native/WASM 시각 gate·전체 회귀는 F5에서 확인한다.

## 원인과 적용 경로

- #7959: `/Interpolate true`인 마스크를 Unsupported로 거절했다. `Mask::read`에서 boolean을
  검증하고 `apply`의 공통 bitmap 좌표에서 sample-centre bilinear alpha를 계산한다.
  같은 해상도는 항등이며 기존 false는 nearest를 유지한다. RGB·Decode/Matte·CTM·clip·
  소유·RGBA 사전 예약은 유지하며 별도 image-sized interpolation buffer를 만들지 않는다.
- #7961: `/DecodeParms` 간접 참조의 PNG predictor를 누락해 RGB에 행 filter byte가 남았다.
  `decode_stream_with_encryption` → 제한 resolver → filter별 decoder/predictor → 마스크 합성
  순서로 해석한다. Filter/DecodeParms 배열은 순서를 보존하며 각 filter 직후 자신의 predictor를
  적용한다. scalar 참조·깊이/배열/할당 한도와 sticky 실패를 보존한다. 무필터 그림의 쓰이지 않는
  DecodeParms 때문에 거절하지 않는다. RGB 길이 검사를 완화하거나 행을 추측해 제거하지 않는다.
- #7957 후속: 39쪽 compressed stream의 마지막 checksum이 CR이고 `/Length`는 간접 참조다.
  기존 endstream 탐색이 그 CR과 구분 LF를 CRLF로 보아 데이터 1바이트를 잘랐다.
  bounded 객체 로드에서 Length 참조를 먼저 읽고 선언 길이대로 binary를 자른다. 독립 qpdf와
  Python zlib는 원본 stream을 정상 해제하므로 원본 손상으로 판정하지 않았다.

독립 근거: [ISO 32000-1:2008](https://opensource.adobe.com/dc-acrobat-sdk-docs/standards/pdfstandards/pdf/PDF32000_2008.pdf)
7.3.8.2·7.3.10·7.4.1(Table 5), 8.9.5.3. Interpolate 알고리즘은 reader 선택 사항이며 이번 구현은
bilinear다. 원본 PDF의 Poppler raster를 paint 기준으로 사용한다.

162쪽을 더 진행하면 기존 40,000 emitted spans 한도도 드러났다. 독립 text-showing 연산은 76,887회다.
product 최대 spans 120,000/collection entries 400,000 안에서 선택 쪽에 각각
max(40,000, 1,000×선택 쪽), max(100,000, 2,000×선택 쪽)을 적용한다. 같은 요청의 소비량은
지우지 않고 한도를 낮추기만 한다. frontend도 backend의 유효 span 한도를 소비한다.
Character shape 참조는 `CharShapeRef.char_shape_id`의 u32이며 여기에 적용하던 u16 사전 cap은
수정했다. u16 paragraph/font/binary 자원 cap과 u32 overflow 검사는 유지한다.

## 실행 결과와 증거

- Native: page 162, operators 12, image 4, path 107, extraction 14, decoded 14쪽 성공.
  원본 page count와 IR section count가 일치한다. layout page count·시각 통과로 대체하지 않는다.
- 생성 정상 PDF 세 개는 기존 Native에서 predictor/Interpolate/Flate 오류로 각각 실패하고
  새 Native에서 모두 성공했다. qpdf 검사·입력/producer hash·전후 결과는 ignored
  `output/pr-review/issue7957/f4/`에 보존했다.
- 정식 API/데이터 검사: 간접 Length 뒤 CR/LF/CRLF/space 보존, 간접 predictor와 filter별
  파라미터 순서, 참조 cycle 거절, true/false/reverse Decode의 alpha와 RGB 보존·예약 거절을
  포함한 frontend/resource 133 PASS. 배치 fixture·golden·절대 좌표는 추가하지 않았다.
- 원본 313쪽 Poppler raster 생성도 성공했으며 `f5/reference-raster-results.json`에 hash를 기록했다.
  Native/fresh WASM 출력과 비교하기 전이므로 이 사실을 시각 통과로 표시하지 않는다.

기본 Studio 7700의 기존 bundle·사용자 타입 2개를 보존했다. 새 runtime 검증은 별도 서버에서
진행한다. `parserAllocationBoundsProven=false`와 기존 codec/font/cache의 미입증 범위를 유지한다.
