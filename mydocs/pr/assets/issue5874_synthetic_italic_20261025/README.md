# #5874 합성 기울임 기대값 재검토

원 이슈의 [입력](../../../../samples/issue5874/italic-repro.hwpx)과
[독립 한컴 2020 Print](../../../../pdf/issue5874/italic-repro-hancom2020-print.pdf)를 재사용한다.
입력·PDF SHA, 생산 소스, 실제 명령과 원본 TSV 해시는 [증거 원장](evidence.json)에 있다.
Print PDF의 텍스트 행렬 `1 0 .333 -1 tx ty Tm`이 무차원 shear의 독립 근거다.
옛 테스트의 `.25`를 `.333`으로 고치며, 구현의 상수를 가져와 정답으로 쓰지 않는다.
기준선·문자 전진·실제 italic/oblique face의 비합성·혼합 face의 안전 경계 검사는 유지한다.

| 경로 | 전쪽 TSV | 대표 review | standalone overlay | raw 최저 |
| --- | --- | --- | --- | ---: |
| Native208 | [원문](native/silhouette.tsv) | [review](native/review.png) | [overlay](native/overlay.png) | 100% |
| fresh WASM055 | [원문](wasm/silhouette.tsv) | [review](wasm/review.png) | [overlay](wasm/overlay.png) | 100% |

양쪽 모두 독립 PDF와 전체 한 쪽이 일치한다. 같은 배율로 기울기·본문 위치를 판독했다.
작은 글자의 raster 농도 차이는 남아 있으며 raw100%를 엄격 RGB 일치로 해석하지 않는다.
현재 변경은 테스트 전용이고 이 생산물의 runtime을 바꾸지 않는다. Native와 WASM의 실제
소스 SHA가 다름을 원장에 구분했으며, 두 생산 소스 사이 제품 코드 동등성은 output의
`native208-wasm055-production-equivalence-3263.json`에 보존했다.

원 해결은 PR #6775, merge `1f861362ab372f9fa26e38f9a534a89286f641c4`다.
현재 검사와 공개 진단 계약 묶음은 16 PASS/0 FAIL이고, 전체 Rust 또는 통합 PR 준비 완료를 뜻하지 않는다.
