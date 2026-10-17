# Task #7957 4단계 — F3 실제 작업·중복 비용과 요청 한도

- Issue: #7958·#7960·#7963, 공동 #7957·#7959·#7961·#7962
- 상태: Native focused 130 PASS. 실제 새 Docker WASM·전쪽 시각 검증은 F5에서 수행한다.

`extract_painted_spans` → 공통 후처리 → 문서 전체 머리말 분류의 연결을 확인했다. 위치 기반
가져오기 lowering은 이 artifact 분류를 소비하지 않으며 머리말도 표시해야 한다. 공통 후처리의
내용/좌표 정리는 유지하고 이 분류만 제외했다. 일반 텍스트 추출의 분류는 유지한다. 선택하지 않은
쪽을 이 분류 때문에 해석하던 동작도 제거했다. 같은 페이지의 lexical parse/cache를 ink·image
소비자도 재사용하고, decoded content는 기존 Arc를 전달해 실제 바이트 복사를 줄인다.

JPEG의 `decode_jpeg_with_budget`의 실제 RGB/CMYK 예약 → `into_rgb_with_budget` 변환 예약 →
bounded PNG scratch·output 예약은 유지한다. 이 경로 위에 중복 계상하던 가상 3×RGBA allowance만
제거했다. raw/CCITT의 기존 allowance는 유지한다. 픽셀 변환·PNG 압축 방법은 바꾸지 않았다.
기존 JPEG baseline/progressive/subsampling/CMYK 데이터 계약 검사도 포함해 통과했다.

작업량은 heap/RSS가 아니다. 선택 쪽수에 따라 operators max(250,000, 100,000×선택 쪽),
decoded max(256 MiB, 64 MiB×선택 쪽), path max(80,000, 20,000×선택 쪽)를 적용하며
각각 4,000,000 / 2 GiB / 1,000,000의 절대 요청 상한으로 제한한다. 개별 stream/codec 32 MiB,
path 20,000 등은 유지한다. source page tree 뒤 같은 요청 예산을 낮추며 누적 사용량이나 최초
실패를 초기화하지 않는다. 신뢰 호스트가 준 더 낮은 값도 올리지 않는다.

| Native 입력 | 결과 | operators | decoded 누적 bytes |
| --- | --- | --- | --- |
| content operators | 12쪽 성공 | 274,718 (이전 549,436) | 57,688,761 (이전 99,265,975) |
| path budget | 107쪽 성공 | 421,337 | 1,115,599,341 (이전 1,163,489,907) |
| decoded bytes | 14쪽 성공 | 56 | 179,986,971 (이전 799,328,649) |

이전 성공 수치는 진단용 넓은 신뢰 정책으로 측정한 값이고 수정 후는 제품 기본 정책이다.
실행 시간은 이 probe의 관측이며 보편적인 성능 보장으로 표현하지 않는다. 입력·소스·실행 파일
hash와 counters는 ignored `output/pr-review/issue7957/f3/`에 있다.

선택 1쪽 뒤의 미선택 대조군에 260,000개 연산을 넣으면 기존 Docker WASM은 선택한 쪽과
무관하게 거절한다. 수정 Native는 선택 쪽 하나·parse 1회·operator 5회로 성공한다.
8 연산 정책의 작은 정식 API 검사도 PASS다. 새 work restriction 검사는 사용량 보존·감소만 허용·
소진의 sticky 상태를 검사한다. 합성 데이터/자원 계약이며 배치 golden을 추가하지 않았다.

162쪽 원본은 기존 page/operator 거절을 넘은 뒤 39쪽 image_handles에서
`invalid or truncated bounded Flate stream`이 드러났다. 이를 성공으로 표시하지 않으며 F4에서
원본 stream·독립 decoder와 대조한다. 기존 #7959 Interpolate·#7961 간접 DecodeParms도 F4 대상이다.
