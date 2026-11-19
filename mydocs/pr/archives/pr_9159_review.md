# PR #9159 검토 · Native 여러 문단 선택과 캐럿 추적

**검토 완료.** [통합 PR #9189](https://github.com/edwardkim/rhwp/pull/9189) 코드 head `5b3e013dfb49e82f24fc769f81170e2310adf612`의 Full CI와 적용 대상 검사가 모두 통과했다. [CI](https://github.com/edwardkim/rhwp/actions/runs/44470122043) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/44470122101) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/44470121749) · [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/44470122018) · [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/44470122039) · [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/44470121845)

이번 검토 기록을 반영한 head의 필수 검사를 확인한 뒤 병합한다.

[원 PR](https://github.com/edwardkim/rhwp/pull/9159) head `cd2212dce18b70b78479d52a18f9e12af224f40d`를
`c526dd402bcf55d09cb8d0993537698a25d4740a` 기준 누적 브랜치 `review/integration-8701-seo-rii-desktop`에 반영했습니다.
로컬 검증 소스는 `64ba35858d4f511bdd993f24eae9337880a7f2b2`다. 위 원격 코드 head에는 검증 증적만 추가했다.

## 원 PR 주장

현재 선택의 활성 캐럿을 따라가며, 역방향 선택과 IME가 없는 경우도 처리합니다.

## 체리픽 추적

| 원 커밋 | 적용 커밋 |
| --- | --- |
| `610119910f03f4e028e1cc894b6e7347fd6892cc` | `20717c5ddcd259df505f36ff511be12462548dc4` |

## 실제 검증 범위

최신 Flutter 패키지의 native_caret_follow_test와 정상 선택·스크롤 대조

Mac의 Flutter 3.47.9·Dart 3.13.7에서 strict analyze를 통과했고, 순차 전체 Flutter 테스트는 388건 통과·8건 건너뜀이다.

[공통 보호문서 검토](../assets/issue8701-desktop-integration-20261119/protected-review.md)에서 Native·fresh WASM 전쪽 결과와 정본 TSV를 확인할 수 있다.

## Merge 후 contributor PR comment 계획

통합 PR의 병합 SHA와 최신 devel 반영을 확인한 뒤 원 PR에 링크를 남긴다. 여러 문단 선택의 이동 캐럿, 역방향 선택 및 정상 스크롤을 확인한 Flutter 검사 범위를 적는다.
