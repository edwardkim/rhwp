# PR #9289 리뷰 — 자리차지 표와 같은 문단의 그림 원점

## 최신 통합 상태

#9365와 문서 변경 #9369를 포함한 `devel` 위의 적용 커밋을 기록했다. 집중 회귀 16건이 통과했으며, 최종 회귀와 시각 검증은 진행 중이다.

## 리베이스 전 검토 판정

원 PR의 조판·저장 주장을 통합 소스에서 확인했다. 전체 회귀와 정본 보호 결과, 실제 화면 검증 상태는 [통합 검증 기록](../assets/planet6897_stored_layout_20261123/README.md)에 연결한다.

## 접수와 검토 범위

[원 PR #9289](https://github.com/edwardkim/rhwp/pull/9289) · [이슈 #8495](https://github.com/edwardkim/rhwp/issues/8495) · 작성자 planet6897

- 원 PR head `cc8d6702e44f0133b38d73ebd50ce7ef2721d44f`
- 통합 기준 `321ab8f97791ecf58b56f178f49b7795a836fb79`
- 리베이스 전 시각·회귀 검증 코드 `330af8608ac741b14041d3e372284a6111346b9c`
- 최신 통합 기준 `e7b45a7a71e9d038bd137a9dc2ac3ef37e0ac2e5`
- 최신 제품 검증 후보 `8b54abbd12eaff27b004571721ae81885b828201`

그림 두 개와 치수선이 표와 같은 저장 줄 원점을 사용한다. 문단 앞 간격을 그림 원점 산정에서 뺀다.

### 체리픽 적용

| 적용 순서 | 원 커밋 | 최신 리베이스 후 적용 커밋 |
| ---: | --- | --- |
| 1 | `d62a3f46a08611c1b492c41ec59c7bae2279bd18` | `49116637e1b3377eab18d82b95ef649d6c8a82e5` |
| 2 | `7db2426080cd78e7a92be98946280ebe9183c731` | `3f03810810abd0cb71997ce1024cc75ab9d42159` |
| 3 | `cc8d6702e44f0133b38d73ebd50ce7ef2721d44f` | `d9f44b0d1bbb9835651c36e440e862bb00da505f` |

## 입력과 독립 인쇄 기준

- badge [원본](../../../samples/issue8495/council-badge-para-float-sibling-origin.hwp) 및 [데스크톱 한컴 Print PDF](../../../pdf/issue8495/council-badge-para-float-sibling-origin-2024.pdf).

## Native와 fresh WASM 검증

Visual Sweep의 96 DPI·2px raw 실루엣을 사용했다. macOS의 locked WASM 빌드는 `--no-opt`로 수행했다.

| 원본·쪽 | Native 기준 → 후보 | fresh WASM 기준 → 후보 |
| --- | ---: | ---: |
| badge 1쪽 | 88.49078% → 99.97577% | 88.57824% → 99.97597% |

- badge native 전쪽 1/1쪽, 최저 99.97577%(1쪽). [TSV](../assets/planet6897_stored_layout_20261123/pr-9289/badge/native/silhouette.tsv)
- badge native [동일 입력의 기준 TSV](../assets/planet6897_stored_layout_20261123/pr-9289/badge/native/base-silhouette.tsv).

- badge wasm 전쪽 1/1쪽, 최저 99.97597%(1쪽). [TSV](../assets/planet6897_stored_layout_20261123/pr-9289/badge/wasm/silhouette.tsv)
- badge wasm [동일 입력의 기준 TSV](../assets/planet6897_stored_layout_20261123/pr-9289/badge/wasm/base-silhouette.tsv).

## 주장 페이지의 증적

- badge 1쪽 native [raw PNG](../assets/planet6897_stored_layout_20261123/pr-9289/badge/native/raw-001.png) · [한컴 Print PNG](../assets/planet6897_stored_layout_20261123/pr-9289/badge/native/pdf-001.png) · [review](../assets/planet6897_stored_layout_20261123/pr-9289/badge/native/review-001.png) · [overlay](../assets/planet6897_stored_layout_20261123/pr-9289/badge/native/overlay-001.png)
- badge 1쪽 wasm [raw PNG](../assets/planet6897_stored_layout_20261123/pr-9289/badge/wasm/raw-001.png) · [한컴 Print PNG](../assets/planet6897_stored_layout_20261123/pr-9289/badge/wasm/pdf-001.png) · [review](../assets/planet6897_stored_layout_20261123/pr-9289/badge/wasm/review-001.png) · [overlay](../assets/planet6897_stored_layout_20261123/pr-9289/badge/wasm/overlay-001.png)

## Merge 후 contributor PR comment 계획

실제 merge와 원 PR head를 확인한 뒤 [자리차지 표와 같은 문단의 그림 원점](https://github.com/edwardkim/rhwp/pull/9289)에 기여 내용과 검증을 한국어 존댓말로 남긴다. 원 PR의 문서와 주장 페이지에 대응하는 실제 review와 overlay 이미지를 merge SHA의 raw URL로 표시한다.
