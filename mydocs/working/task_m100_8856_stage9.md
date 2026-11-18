# #8856 효과 그림의 2024 Print 색·마스크 분리 전달

사진 효과 보정과 실제 전쪽/대조군 증거를 `5454a1656c`로 커밋한 뒤 시작한다.
새 효과의 투명 PNG 실제 소비를 대조하면서 2024의 색 차이를 발견했다.
반경 0에서도 같은 중앙 색 차이가 있으므로 새 효과의 위치 회귀로 오인하지 않는다.
효과가 없는 기존 출력은 이번 보정에서 바꾸지 않는다.

독립 `alpha-half-r1000-2024.pdf`는 2500×1667의 1bit ImageMask와 별도의 RGB 이미지,
`/Mask 11 0 R`를 가진다. RGB는 마스크가 꺼진 위치에서 흰색이며, 켜진 위치의 RGB는
기존 alpha 128의 PArgb 전달색 (63,127,191)이다. PDF에 별도 opacity ExtGState는 없다.
색과 마스크를 각각 축소한 뒤 합성하면 중앙이 (206,222,238) 부근이며,
결합 RGBA를 premultiply하여 함께 축소한 Native의 (159,191,223)과 다르다.
raw 100%는 그 사각형의 실루엣만 같다는 뜻이다.

기존 공통 `print_image_sampling::rasterize_masked`는 Native/Canvas에서 별도의 색·마스크
재표본을 이미 지원한다. 그러나 준비된 효과 이미지가 일반 PNG 경로로 다시 들어가
이 전달 계약을 잃는다. 효과 이미지의 2024 Print payload가 장치 표본·기존 색과 새 효과
마스크를 소유하도록 연결한다. 반경·crop은 같은 공통 준비 결과를 사용하고, 원 alpha를
두 번 곱하거나 일반 Screen의 사진 RGB까지 바꾸지 않는다.

원본·MCP PDF·판정 문턱은 유지한다. 설치 글꼴 없는 공개 메모리 그림으로 실제 Print
PaintOp의 자원 종류·색/마스크 전달 누락을 먼저 실패로 검사하고, 수정 후 같은 대조군의
Native/fresh WASM 및 무효과·Screen 반례를 확인한다. 아직 전체 제출 검증은 완료가 아니다.

## 최종 집중 검증

Clippy의 `Option` 조기 반환 5건을 `?`로 정리한 최종 소스에서 다시 빌드했다.
최소 글꼴 집중 검사 11건과 Native/fresh WASM 각 9조건·10쪽이 실제 완료됐다.
기존 사진·대조군 6쪽씩은 stage8 PNG와 바이트가 같다. 2024 투명 효과 4조건의
raw는 양 backend 모두100.00000%이며, 중앙 RGB는 (207,223,239)로 PDF 대비
채널1단계 차이다. 효과 없는 반경0의 기존 색 차이는 이 보정에서 변경하지 않았다.
새 Chrome 초기 GET과 실제 캡처에서도 같은 JS/WASM·페이지 PNG를 확인했다.
Native Clippy·fmt·suite manifest·unit tier는 통과했다. 최종 PR 전체 검증은 아직 아니다.

[검증·전체 TSV·대표 이미지](../pr/assets/issue8856/stage9-soft-edge-print-planes/README.md) ·
[소스·실측 영수증](../pr/assets/issue8856/stage9-soft-edge-print-planes/validation.json)

임시 로그와 스크립트는 `output/issue8856/soft-edge-print-planes/lint-final/`에 보존한다.
그림 효과 밖의 글리프·글머리표 전진폭·간격 잔여는 다음 단계에서 계속 처리한다.
