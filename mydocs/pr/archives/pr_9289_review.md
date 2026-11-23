# PR #9289 리뷰 — 자리차지 표와 같은 문단의 그림 원점

## 최종 판정

**승인 — 원 PR의 주장 범위와 통합 후보의 로컬 검증 완료.** 통합 제품 검증 소스는 `e70dc6231a22ec0bd569537c4e5cb0ff930fd606`다. 병합은 제출한 head에서 GitHub CI와 충돌 여부를 확인한 뒤 진행한다.

## 접수와 검토 범위

[원 PR #9289](https://github.com/edwardkim/rhwp/pull/9289) · [이슈 #8495](https://github.com/edwardkim/rhwp/issues/8495) · 작성자 planet6897

- 원 PR head `cc8d6702e44f0133b38d73ebd50ce7ef2721d44f`
- 통합 기준 `321ab8f97791ecf58b56f178f49b7795a836fb79`
- 리베이스 전 시각·회귀 검증 코드 `330af8608ac741b14041d3e372284a6111346b9c`
- 최신 통합 기준 `626a5b30ab773a48aec004a22ad789e508ae9c49`
- 최신 제품 후보 `e70dc6231a22ec0bd569537c4e5cb0ff930fd606`
- 통합 시각 검증 소스 `b8baf0bc29bfbcff1aed9eb6c44982c71242d4d1`

그림 두 개와 치수선이 표와 같은 저장 줄 원점을 사용한다. 문단 앞 간격을 그림 원점 산정에서 뺀다.

### 체리픽 적용

| 적용 순서 | 원 커밋 | 최신 리베이스 후 적용 커밋 |
| ---: | --- | --- |
| 1 | `d62a3f46a08611c1b492c41ec59c7bae2279bd18` | `974f529e8976f20fa46941bb19f9f8422e3e95ff` |
| 2 | `7db2426080cd78e7a92be98946280ebe9183c731` | `3fc09ba9d8dd415a6410ea625992b2b62af79701` |
| 3 | `cc8d6702e44f0133b38d73ebd50ce7ef2721d44f` | `e5bb666769a89e21a44fe2446b970e5981fcf926` |

## 입력과 독립 인쇄 기준

- badge [원본](../../../samples/issue8495/council-badge-para-float-sibling-origin.hwp) 및 [데스크톱 한컴 Print PDF](../../../pdf/issue8495/council-badge-para-float-sibling-origin-2024.pdf).

## Native와 fresh WASM 검증

표와 그림은 앞선 검증 SHA의 Visual Sweep 96 DPI·2px raw 실루엣을 기록한다. macOS의 locked WASM 빌드는 `--no-opt`로 수행했다.

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

## 최신 리베이스와 증적 재사용

기준 `626a5b30ab773a48aec004a22ad789e508ae9c49`, 리베이스 결과 `b6e1c2c2d1a429c7912e3114eb988a5f0839fa1b`. 적용 커밋 32개의 패치가 일치했고 제품·테스트·원본·PDF·시각 검증 스크립트의 트리는 리베이스 전후에 동일했다. 후속 메인터너 보정은 명시적으로 글꼴 슬롯과 자연 전진을 바꾼 경우에만 활성화한다. [통합 증적](../assets/planet6897_stored_layout_20261123/README.md)

## 통합 제출과 CI

[통합 PR #9372](https://github.com/edwardkim/rhwp/pull/9372)의 코드 후보와 통과한 GitHub 검사는 [통합 검토 기록](pr_9372_review.md)에 정리했다.
