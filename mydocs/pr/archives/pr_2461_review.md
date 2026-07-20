# PR #2461 리뷰 - HWPX DOUBLE_SLIM 탭 리더 파싱 보존

## 메타데이터

| 항목 | 값 |
|---|---|
| 원 PR | [#2461](https://github.com/edwardkim/rhwp/pull/2461) |
| 작성자 | `kevin9327` |
| base | `devel` |
| 리뷰 경로 | collaborator-mediated 외부 PR, G5 체리픽 통합 |
| 원 커밋 / 통합 적용 커밋 | `bc41a7cc6d5fcd0b77462bd8e4bc89dddb66fa36` / `e6b22d4c9` |
| 적용 순서 | G5 6/8, #2460 뒤 체리픽. #2459가 추가한 인접 테스트와 충돌해 두 회귀를 모두 유지하도록 해소 |
| 작성 시점 참고값 | 원 PR은 `MERGEABLE` / `BEHIND`, 원 head CI는 성공이다. 통합 PR의 최신 head CI와 mergeable 상태를 merge 전 다시 확인한다. |

## 변경 검토

`DOUBLE_SLIM` 탭 리더 문자열이 parser에서 `NONE`으로 떨어져 이중 실선 탭 리더의 값 8이 저장 후
사라졌다. PR은 문자열을 값 8로 복원한다. 체리픽 충돌은 같은 test module의 삽입 위치만 겹친 것으로,
3D 테두리 회귀와 DOUBLE_SLIM 회귀를 함께 남겼다.

## 검증

| 게이트 | 결과 |
|---|---|
| `parse_tab_item_leader_double_slim_maps_to_8` | PASS |
| G5 공통 full regression / fmt / clippy / diff / WASM | PASS |

## 시각 검증 판단

탭 리더의 HWPX enum 복원만 바꾸고 typeset 또는 renderer를 바꾸지 않는다. 구조 회귀로 검증했으며
visual sweep은 적용하지 않는다.

## 최종 의견

G5 통합 PR에 기여자 원 커밋을 보존해 수용한다. 공통 검증과 최종 조건은
[G5 통합 실행 계획](pr_2455_review_impl.md)을 따른다.
