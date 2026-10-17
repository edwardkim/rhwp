# 수동 Print 기준 원본 글꼴 전진과 탭 리더 소유 보정

- 입력: `samples/hwp3-sample16-hwp5.hwp`, 독립 기준: `pdf/hwp3-sample16-hwp5-lossless-2024.pdf`.
- 사용자 제공 Windows 한컴2024 Print64쪽이다. 입력/PDF SHA와 소스 hash는 [validation.json](validation.json)에 기록했다.
- 실제 Native PNG/fresh WASM Canvas PNG,192DPI/compat2024,canonical Visual Sweep 2px raw90%를 유지했다.
- Native 전64쪽 최저93.07399%, WASM95.87967%, 각90% 미만0쪽. 동일 raw 문턱이며 전체48조건 통과를 뜻하지 않는다.
- 중요한19쪽은 Native99.43242/WASM99.61477%이며 같은 HFT 굵은 제목 중 양쪽 최저가 가장 높은 페이지를 선택했다. 본문 소유para412를 확인하고 독립 PDF의 '터' x299.676676/y636.159993 CSSpx를 ±2CSSpx로 검사한다. 목차의 유사 제목은 제외했다.
- 원본 HFT/실제 TrueType/미확인 style 대조, 정상 탭과 TAC 앞뒤 리더의 내용·소유·중복 검사를 포함해 수정 전16건12PASS4FAIL → 수정 후16PASS다. 최초 목차 오선택 실패는 재현 증거에서 제외하고 올바른 본문으로 다시 실행했다.
- 실제 소비 stroke 폭을 snapping 판단과 paint에 함께 전달한다. 공개 LineOp의 소수3자리 직렬화와 실제 Canvas 인수 차이는 최대0.0005CSSpx로 대조했으며 시각 허용치를 바꾼 것이 아니다.
- 거대 셀80쪽 default WASM의 고정 devel 대비0.03797%p 하락,18쪽 WMF 패턴 색상, 새 source 전체 회귀/실제 Studio 검증은 남아 있다. PR 준비 완료가 아니다.

| 쪽 | Native | fresh WASM |
|---|---|---|
| 2 | [review](native-p002-review.png) · [overlay](native-p002-overlay.png) | [review](wasm-p002-review.png) · [overlay](wasm-p002-overlay.png) |
| 19 | [review](native-p019-review.png) · [overlay](native-p019-overlay.png) | [review](wasm-p019-review.png) · [overlay](wasm-p019-overlay.png) |
| 41 | [review](native-p041-review.png) · [overlay](native-p041-overlay.png) | [review](wasm-p041-review.png) · [overlay](wasm-p041-overlay.png) |

[Native 전쪽 TSV](native-all64.tsv) · [WASM 전쪽 TSV](wasm-all64.tsv)
