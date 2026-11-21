# planet6897 원 PR 5건 통합 검토

검증 기준은 `devel`의 `d7b5286cbf6401678a70eed3d66236a68dd2a72c`이며 출력 생산 소스는 `65b6fe383c3bd8cbcf83de7909c7a6c24eb32c35`다. 원 PR의 주장 문서는 데스크톱 한컴 2020·2024의 Hancom PDF Print에 대조했다.

| 원 PR 검토 | Native·fresh WASM 각각 검증한 쪽수 | Native 최저 | fresh WASM 최저 |
| --- | ---: | ---: | ---: |
| [#9240](../../archives/pr_9240_review.md) | 64 | 98.12% | 98.15% |
| [#9269](../../archives/pr_9269_review.md) | 62 | 98.87% | 98.86% |
| [#9278](../../archives/pr_9278_review.md) | 7 | 99.98% | 99.98% |
| [#9282](../../archives/pr_9282_review.md) | 3 | 100.00% | 100.00% |
| [#9287](../../archives/pr_9287_review.md) | 9 | 99.78% | 99.66% |

두 경로 모두 주 재현 문서 145쪽이 98% 이상이다. #9240 충돌은 #9277의 표 조각 그리드에서 이어받은 셀 배경을 적용하도록 해소했다. 문서별 수정 이유, 주장 페이지의 review·overlay, 실제 출력 경로는 개별 검토서에 있다. [최신 전쪽 결과](validation/claims-v5.json) · [메인터너 보정 근거](maintainer-repair.md)

전체 Rust 회귀 **13,877건**과 Native Skia 검증 세 범위가 통과했다. 신규 원본 보안 검사, 독립 Print 쪽수 기준 검사, 기여자 공통 보호 8조건도 통과했다. 실제 Studio 화면·인쇄 60건의 브라우저 초기화 WASM SHA를 확인했다. 선택된 경로는 Canvas2D 56건과 CanvasKit 직접 출력 4건이다. [Rust 검증](validation/rust-v5.json) · [공통 보호 검증](validation/contributor-v5.json) · [Studio 검증](validation/actual-studio-v5.json)

보호 문서 **46조건·4,046쪽**의 후보 TSV를 정본과 비교해 하락·누락 0쪽을 확인했다. 같은 공급 조건의 고정 `devel`과 후보 대조는 진행 중이다. `sample10`의 Native는 763쪽 전부에서 고정 기준·후보·정본 TSV가 바이트 단위로 일치했다. 같은 조건의 WASM 대조에서는 474쪽의 HNC 이중선이 **99.96% → 100.00%**로 복원됐으며 나머지 762쪽의 표시값은 같다. [보호 문서 후보 TSV](protected-candidate-v5/README.md) · [sample10 Native 대조](protected-sample10-native/README.md) · [sample10 WASM 대조](protected-sample10-wasm/README.md)
