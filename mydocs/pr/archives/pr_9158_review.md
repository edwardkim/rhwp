# PR #9158 검토 · 데스크톱 GPU backend 미지원 fallback

**검토 완료.** [통합 PR #9189](https://github.com/edwardkim/rhwp/pull/9189) 코드 head `5b3e013dfb49e82f24fc769f81170e2310adf612`의 Full CI와 적용 대상 검사가 모두 통과했다. [CI](https://github.com/edwardkim/rhwp/actions/runs/44470122043) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/44470122101) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/44470121749) · [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/44470122018) · [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/44470122039) · [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/44470121845)

이번 검토 기록을 반영한 head의 필수 검사를 확인한 뒤 병합한다.

[원 PR](https://github.com/edwardkim/rhwp/pull/9158) head `1f5548e4a4ec11471c4999bbaa0f7a261545a21e`를
`c526dd402bcf55d09cb8d0993537698a25d4740a` 기준 누적 브랜치 `review/integration-8701-seo-rii-desktop`에 반영했습니다.
로컬 검증 소스는 `64ba35858d4f511bdd993f24eae9337880a7f2b2`다. 위 원격 코드 head에는 검증 증적만 추가했다.

## 원 PR 주장

명시 GPU 요청에서 선택한 backend나 Native export가 없으면 CPU 경로를 제공합니다. Metal loader 진단은 export 누락과 실제 로드 오류를 구분합니다.

## 체리픽 추적

| 원 커밋 | 적용 커밋 |
| --- | --- |
| `b4efa7a19a592582e166bb111b9fda17d2c18bba` | `508ed4cff6d8ec0569b1fad0ca8b9b605d4a941c` |
| `1f5548e4a4ec11471c4999bbaa0f7a261545a21e` | `3c189f0d82ea99a4a773fccc52ed4ad97b9770e2` |

## 실제 검증 범위

Flutter GPU fallback 위젯 테스트, macOS Metal loader, Windows provider의 실제 실패·정상 경로

Mac의 Flutter 3.47.9·Dart 3.13.7에서 strict analyze를 통과했고, 순차 전체 Flutter 테스트는 388건 통과·8건 건너뜀이다.

win10-ted에서 MSVC 2022 x64로 통합 NativeApi.cpp를 빌드했다. 실제 DLL 로더에서 선택적 D3D export 누락, 필수 core export 누락, 손상 DLL 세 경우를 구분했다.

[공통 보호문서 검토](../assets/issue8701-desktop-integration-20261119/protected-review.md)에서 Native·fresh WASM 전쪽 결과와 정본 TSV를 확인할 수 있다.

## Merge 후 contributor PR comment 계획

통합 PR의 병합 SHA와 최신 devel 반영을 확인한 뒤 원 PR에 링크를 남긴다. 검토된 GPU 선택·CPU fallback 범위와 Windows 선택적/필수 export·손상 DLL 3건 결과를 적는다.
