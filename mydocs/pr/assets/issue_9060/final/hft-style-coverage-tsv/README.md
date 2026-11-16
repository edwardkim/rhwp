# 새 base와 후보의 동일 환경 전쪽 대조

기록: 2026-11-15T14:03:30.072748+09:00.

base `f8234f6f90cb25d9601c9d5d762e0a565aea19e4`와 후보 제품 source `c95f9ade4e8acf9d901c01136ef2b9bacd60c497`를 같은 Linux·실제 글꼴·독립 한컴 Print·compat 2022·96 DPI에서 대조했다. 원래 Mac 측정값을 Linux의 대조군으로 혼용하지 않았다.

Native/fresh WASM 각각 389쪽, 총 778행을 대응했다. 2%p 초과 하락은 0행이다. 특별 보호 대상 3,542행의 하락 금지 검사는 별도로 유지하며 일반 대조의 허용폭으로 면제하지 않는다.

| 입력 | 경로 | 쪽수 | base TSV | 후보 TSV | 하락 행 | 최대 하락 %p |
| --- | --- | ---: | --- | --- | ---: | ---: |
| `recommendation` | native | 22 | [원본](base/native/recommendation.tsv) | [원본](candidate/native/recommendation.tsv) | 4 | 0.04 |
| `exam` | native | 8 | [원본](base/native/exam.tsv) | [원본](candidate/native/exam.tsv) | 1 | 0.01 |
| `api` | native | 105 | [원본](base/native/api.tsv) | [원본](candidate/native/api.tsv) | 0 | 0 |
| `cbta` | native | 204 | [원본](base/native/cbta.tsv) | [원본](candidate/native/cbta.tsv) | 16 | 0.10 |
| `footnote` | native | 50 | [원본](base/native/footnote.tsv) | [원본](candidate/native/footnote.tsv) | 5 | 0.08 |
| `recommendation` | wasm | 22 | [원본](base/wasm/recommendation.tsv) | [원본](candidate/wasm/recommendation.tsv) | 0 | 0 |
| `exam` | wasm | 8 | [원본](base/wasm/exam.tsv) | [원본](candidate/wasm/exam.tsv) | 0 | 0 |
| `cbta` | wasm | 204 | [원본](base/wasm/cbta.tsv) | [원본](candidate/wasm/cbta.tsv) | 3 | 0.04 |
| `api` | wasm | 105 | [원본](base/wasm/api.tsv) | [원본](candidate/wasm/api.tsv) | 1 | 0.01 |
| `footnote` | wasm | 50 | [원본](base/wasm/footnote.tsv) | [원본](candidate/wasm/footnote.tsv) | 1 | 0.02 |

CBTA 118·162·166쪽은 양쪽 raw 98% 이상을 별도로 확인했다. [주요 쪽의 실제 review·overlay와 직접 판독](../rebase-f823-reading/README.md)을 함께 본다. 공식 raw가 같아도 RGB 동일성이나 알려진 모든 그래픽 차이의 해결을 뜻하지 않는다.
