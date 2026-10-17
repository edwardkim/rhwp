# Task #7957 12단계 — 추가 스캔 PDF의 image extraction buffer 진단

## 현재 판정

사용자가 추가한 `/mnt/e/tmp/extraction_buffer.pdf`는 현재 PR #7970의 code head
`145446111a25fa5e2f3072ab04f68fa31218a00e`와 동일 runtime `9c6c48c091`에서도
`PDF_RESOURCE_LIMIT: image extraction buffer`로 실패한다. 앞선 359쪽 통과와 별개의
신규 입력이며, 해당 결과를 이 문서의 성공 증거로 재사용하지 않는다.

입력은 1,963,228 bytes, 5쪽이다. 원본·SHA·그림·전쪽 비교 화면은 실제 개인정보가
포함되어 로컬 ignored 증거에만 보존하며 Git·공개 이슈·PR 첨부에 넣지 않는다. 공개 재현은
개인정보가 없는 동일 geometry·압축 방식의 생성 대조군을 사용한다.
QPDF 검사는 exit 0이다. Poppler로 추출한 5개 그림은 모두 8-bit RGB baseline JPEG이며
Pillow의 실제 JPEG decode가 성공했다. 이 독립 확인은 제품 decoder의 성공을 뜻하지 않는다.

## 원인과 소비 경로

제품의 page image handle → 선언 픽셀 16M/문서 64M 검사 → handle.decode →
`extract_image_from_xobject_at`의 `width × height × 8` 사전 추정 → 개별 stream 32 MiB
비교에서 실제 JPEG decode 전에 거절한다. 첫 그림은 1,928×2,725이며 추정값은
42,030,400 bytes다. 실제 RGB 출력은 15,761,400 bytes, RGBA는 21,015,200 bytes로
각각 32 MiB 이하다. 4·5쪽도 같은 추정 거절 범위다. PDF 전체 압축 크기와 개별 decoded
buffer 크기는 구분한다.

현재 JPEG 경로는 compressed bytes를 보관하고, PNG 저장 때 `decode_jpeg_with_budget` →
`jpeg_limited::decode` 또는 CMYK decoder → `into_rgb_with_budget` → bounded PNG writer로
연결된다. JPEG header/선언 일치, 실제 출력·private scratch의 codec 예약, stream/누적 decoded
예산은 별도 적용된다. 이 실제 예약과 가상의 8 bytes/pixel 선행 거절을 구분해야 한다.
그림의 지오메트리·해상도를 낮추거나 페이지를 raster 치환하는 보정은 제안하지 않는다.

## 다음 구현·검증 범위

- 코드 변경 전 JPEG 실제 예약 경계와 raw/고 bit-depth 등 비적용 경로를 대조한다.
  추정 거절만 제거하면서 실제 codec/출력/누적 예산과 잘못된 선언의 거절을 유지한다.
- 이 원본 5쪽과 4 bytes/pixel은 fit하지만 가상 8배만 초과하는 생성 경계를 확인한다.
  실제 buffer 초과·codec 예약 초과·JPEG header 불일치·실패 뒤 정상 요청을 함께 검사한다.
- 영향 5쪽 Native/fresh Docker WASM과 독립 PDF의 전쪽 TSV·그림 경계 확대 판독을 먼저
  수행한 뒤 관계형 회귀를 추가한다. 이전 16 MiB 글꼴 예외를 이 문서에 확대하지 않는다.
- 필수 전체 검증은 최종 source에서 한 번 수행하고, 변경하지 않은 원본·scope·evidence는
  producer 동일성을 확인해 재사용한다. 이전 CI를 새 source의 성공으로 표시하지 않는다.

같은 오류 문자열·파일명으로 기존 이슈를 검색했으며 일치 항목은 없었다. 작업지시자는
2026-10-17 “현재 PR에 포함”으로 PR #7970의 추가 구현·검증·반영 범위를 승인했다.
실제 디코더 예약이 적용되는 JPEG만 추정 거절을 제외하고 raw·고 bit-depth 등의 기존
선행 거절을 유지한다. 이 조건은 문서 ID·특정 크기가 아닌 압축 표현과 실제 예약 경로의
계약이다. 구현 전의 기존 source·CI·시각 결과는 변경 전 증거로 보존한다.

실제 명령·오류·5개 이미지 geometry·독립 추출은 ignored
`output/pr-review/issue7957/extraction-buffer-20261017/`에 보존한다.
