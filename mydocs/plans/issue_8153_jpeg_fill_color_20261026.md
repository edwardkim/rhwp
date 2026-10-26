# #8153: 전체 JPEG 채움의 원본 RGB와 Print 압축

## 원 해결과 독립 근거

현재 261쪽의 두 배경 그림은 Native/fresh WASM에서 미대응 픽셀이 남는다.
두 원본의 실제 ImageNode는 JPEG BinData·전체 채움(Total)·효과 없음·불투명·
crop 없음·변환 없음이다. 현재 paint는 RGB555로 바뀐 PNG를 보낸다.
독립 한컴 2024 Print는 같은 966×322·966×768 자원을 JPEG로 전달한다.
두 원본 Print에서 대응 자원의 RGB가 각각 같음도 확인했다.

기존 압축 전달·색 변환은 `issue_8153_print_image_codec_20261023.md`의
31개 자원·독립 크기 대조를 따른다. 원본 팔레트 판정 보정 a56f96eceb,
공통 품질 판정 04437c268a, 영역 맞춤 표본 연결 b8ca19e50d도 추적했다.
새 조사는 기존 Picture의 RGB555 계약이 잘못됐다는 근거가 아니다.

8adc3a4010의 실제 Native paint에서 원본과 resolved 자원을 분리했다.
채움 모드만 진단 사본에서 Picture로 바꾸면 현재 공통 인코더는 JPEG를 만들지만
RGB555 변환이 남아 독립 RGB와 다르다. 기본 image decoder도 미세한 RGB 차이가 있다.
반면 이미 있는 IJG RGB 디코더와 정식 JPEG 인코더로 원본 RGB를 전달하면
독립 PDF의 두 자원 모두 RGB 전픽셀이 정확히 같다. 품질 38은 독립 양자화표와
기존 `폭(point)/원본 픽셀 폭`의 공통 품질 계산에 일치하며 새 수치로 넣지 않는다.
관측된 화소를 기대값으로 바꾸거나 기준 PDF를 다시 출력하지 않았다.

자료는 ignored `remaining-raw100-diagnostic/remaining-image-resource-audit/current8adc/`의
actual Native metadata, `existing-ijg-source-encode-result.json`,
`paired-independent-print-rgb-proof.json`과 원본/PDF 자원에 있다.
사양은 원본·전체 채움·효과·단위 계약의 근거이며 PDF 생성기의 압축 정책 자체를
HWP 사양의 명시 수치라고 주장하지 않는다. SO 분석 자료는 output에만 남긴다.

## 공통 생산·소비

`ImageNode의 원본 JPEG·전체 채움 → 기존 device_image::eligible → 최종 paint 폭 →
IJG 원본 RGB·공통 품질/인코더 → OutputDevice 자원 → 공통 표본 → Native/Canvas`를 연결한다.
기존 print_device_image_payload는 채움 모드를 거부하므로 그 공통 진입점에서 구분한다.
Picture의 RGB555 색 전달은 유지하고, 전체 JPEG 채움은 독립 출력이 증명한 원본 RGB를 쓴다.
같은 원본을 Picture와 채움으로 쓰는 캐시도 색 계약이 섞이지 않게 구분한다.
새 decode 알고리즘·페이지/문서 ID·색 맞춤 상수·좌표 이동을 추가하지 않는다.

현재 입증한 계약은 불투명·전체 원본의 JPEG FitToSize/Total이다.
PNG/BMP의 기존 전달, 반복·자연 크기·부분 crop·명시적 interpolation·회전·효과·
alpha 합성은 기존 경로를 유지한다. JPEG의 색 디코딩 계약 차이를 다른 형식으로 넓히지 않는다.
원본 바이트·배치·소유·폭/높이·clip을 바꾸지 않는다.

## 실제 검증

두 원본 261쪽의 원본 JPEG·채움 소유·순서·화소 크기와 독립 JPEG의 복호화 RGB를
정식 issue_8153 테스트에서 보호한다. Screen/Print 전환 중 원본 바이트 보존도 검사한다.
독립 자원은 기존 PDF에서 바이트 그대로 추출해 provenance와 함께 정식 fixture에 둔다.
새 등록은 현재 두 문서의 전쪽 Native/fresh WASM 90% 이상을 확인한 뒤 진행한다.
같은 검사 원문을 고정 8adc 라이브러리에서 실패시킨 뒤 새 후보에서 재실행한다.
기존 Picture·팔레트·crop·표본·문자/그림 소유 대조를 함께 실행한다.

먼저 영향 쪽의 실제 PNG/TSV·확대 판독으로 방향을 확인하고, 새 코드의 전쪽 비교와
최적화 fresh WASM을 수행한다. 목표는 계속 전쪽 1532조건 raw100·미대응 0과 필수 테스트다.
사용자 지시 전 upstream 동기화·원격 게시·push·PR 생성 없이 PR 준비까지만 한다.
