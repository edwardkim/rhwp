# PR #2460 리뷰 - HWPX Pattern8x8 그림 효과 파싱 보존

## 메타데이터

| 항목 | 값 |
|---|---|
| 원 PR | [#2460](https://github.com/edwardkim/rhwp/pull/2460) |
| 작성자 | `kevin9327` |
| base | `devel` |
| 리뷰 경로 | collaborator-mediated 외부 PR, G5 체리픽 통합 + 회귀 보강 |
| 원 커밋 / 통합 적용 커밋 | `7e6efca0fba50c1e3d7b3b4d484ca978be1d1697` / `8dcd5d12e` |
| 추가 보강 커밋 | `685c7943c` - 실제 `hp:pic` section에서 `PATTERN_8_8` 보존 회귀 추가 |
| 적용 순서 | G5 5/8, #2459 뒤 체리픽, 충돌 없음 |
| 작성 시점 참고값 | 원 PR은 `MERGEABLE` / `BEHIND`, 원 head CI는 성공이다. 통합 PR의 최신 head CI와 mergeable 상태를 merge 전 다시 확인한다. |

## 변경 검토

그림 효과 parser가 `PATTERN_8_8`을 인식하지 못해 기본 `RealPic`으로 바꿨다. PR은 해당 효과를
`ImageEffect::Pattern8x8`로 복원한다. 통합 검토에서는 문자열 helper만 확인하는 빈틈을 피하기 위해 실제
HWPX section의 `hp:pic`을 파싱하는 회귀를 추가했다.

## 검증

| 게이트 | 결과 |
|---|---|
| `picture_pattern_8_8_effect_is_preserved` | PASS |
| G5 공통 full regression / fmt / clippy / diff / WASM | PASS |

## 시각 검증 판단

효과 enum을 잃지 않게 하는 parser 변경이며 현재 renderer의 pattern paint 구현을 바꾸지 않는다. 따라서
PDF visual sweep으로 paint fidelity를 단정하지 않고, 실제 HWPX 구조를 통과하는 회귀를 merge 근거로 삼는다.

## 최종 의견

기여자 변경에 통합 회귀 보강을 더해 G5 통합 PR로 수용한다. 공통 검증과 최종 조건은
[G5 통합 실행 계획](pr_2455_review_impl.md)을 따른다.
