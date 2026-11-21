# planet6897 원 PR 9건 통합 검토

기준 `ca231d64f43f19da1d1d1aec4cf26a05b817aa22` · 후보 `84fd1cb0b77861b082cf33cf84de9e7bbe06832c`. 데스크톱 한컴 2020·2024 Hancom PDF Print와 96 DPI 2px raw 실루엣으로 비교했다.

| 원 PR 리뷰 | 출력별 쪽수 | Native 최저 | fresh WASM 최저 |
| --- | ---: | ---: | ---: |
| [#9208](../../archives/pr_9208_review.md) | 22 | 99.68% | 99.68% |
| [#9209](../../archives/pr_9209_review.md) | 46 | 98.37% | 98.36% |
| [#9214](../../archives/pr_9214_review.md) | 32 | 99.20% | 99.18% |
| [#9218](../../archives/pr_9218_review.md) | 45 | 99.55% | 99.59% |
| [#9219](../../archives/pr_9219_review.md) | 14 | 99.87% | 99.87% |
| [#9221](../../archives/pr_9221_review.md) | 10 | 99.79% | 99.62% |
| [#9227](../../archives/pr_9227_review.md) | 3 | 100.00% | 100.00% |
| [#9232](../../archives/pr_9232_review.md) | 21 | 99.91% | 99.90% |
| [#9236](../../archives/pr_9236_review.md) | 22 | 98.56% | 98.51% |

각 경로의 215쪽이 모두 98% 이상이다. 새 후보의 전체 Rust 회귀는 13,864건 통과, 실패 0건, 기존 제외 45건이다.
제품별 TSV와 대표 review·overlay는 개별 기록에 있다. [리베이스 시각 비교](rebase-claims-validation.json).
정본 보호 46조건·4,046쪽에서 하락·누락 0쪽을 확인했다. [최종 보호 검증](final-protection/validation.json).

<!-- current-validation -->

현재 후보에서 전체 Rust 회귀 13,864건과 Native Skia·Clippy·workspace build를 통과했다.
Studio·공용 JS 테스트 2,820건이 통과했고 2건은 제외됐다.
리베이스로 반영된 #9277의 5개 독립 Print 조건은 Native와 fresh WASM 각각 566쪽이 98% 이상이다.
규제영향분석서 2024의 WASM은 Print에 쓰인 글꼴을 공급한 뒤 전 31쪽을 재출력했다. 최저 99.27%다.
정본 보호는 Native와 fresh WASM 각각 23조건을 통과했다. 화학 문서의 표시값은 Native 17쪽과 fresh WASM 13쪽에서 상승했다.
기여자 공통 보호 8조건과 편람 전용 Native 383쪽 게이트를 통과했다.
[제품 검증 영수증](current-validation.json) · [정본 보호 전쪽 검증](final-protection/validation.json).

[검증 입력·Print PDF·TSV의 커밋 바이트 대조](submission-inputs.json).

중첩 표는 `H2GTRM.TTF` 기호 공급 조건에서 Native와 fresh WASM 각 17쪽의 TSV가 기존 보호 TSV와 일치했다. sample10 Native 763쪽은 Print에서 확인한 Haansoft Batang의 기호 공급 조건을 적용했다. 중첩 표와 sample10의 고정 기준·후보 TSV 및 PNG를 전쪽 대조해 차이 0쪽을 확인했다. [중첩 표 결과](nested-hy-gothic-validation.json) · [최종 보호 검증](final-protection/validation.json).
