# PR #2462 리뷰 - HWPX 이중·삼중선 탭 리더 직렬화 보존

## 메타데이터

| 항목 | 값 |
|---|---|
| 원 PR | [#2462](https://github.com/edwardkim/rhwp/pull/2462) |
| 작성자 | `kevin9327` |
| base | `devel` |
| 리뷰 경로 | collaborator-mediated 외부 PR, G5 체리픽 통합 |
| 원 커밋 / 통합 적용 커밋 | `31e733036fdbdf976c56fe86826acf95f1bf992c` / `5d0ac30a9` |
| 적용 순서 | G5 7/8, #2461 뒤 체리픽, 충돌 없음 |
| 작성 시점 참고값 | 원 PR은 `MERGEABLE` / `BEHIND`, 원 head CI는 성공이다. 통합 PR의 최신 head CI와 mergeable 상태를 merge 전 다시 확인한다. |

## 변경 검토

`tab_leader_str`은 9/10/11을 기본 `NONE`으로 방출해 `THIN_THICK`, `THICK_THIN`, `TRIM` 탭
리더를 저장할 수 없었다. parser는 이미 같은 세 문자열을 수용하므로, PR은 serializer match arm을 보완해
parser-serializer 대칭을 완성한다.

## 검증

| 게이트 | 결과 |
|---|---|
| `tab_leader_str_emits_double_and_triple_line_types` | PASS |
| G5 공통 full regression / fmt / clippy / diff / WASM | PASS |

## 시각 검증 판단

직렬화 문자열의 보존 범위만 바꾸며 렌더링을 변경하지 않는다. visual sweep은 적용하지 않는다.

## 최종 의견

G5 통합 PR에 기여자 원 커밋을 보존해 수용한다. 공통 검증과 최종 조건은
[G5 통합 실행 계획](pr_2455_review_impl.md)을 따른다.
