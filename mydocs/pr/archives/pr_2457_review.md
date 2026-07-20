# PR #2457 리뷰 - HWPX BETWEEN_LINES 줄간격 왕복 보존

## 메타데이터

| 항목 | 값 |
|---|---|
| 원 PR | [#2457](https://github.com/edwardkim/rhwp/pull/2457) |
| 작성자 | `kevin9327` |
| base | `devel` |
| 리뷰 경로 | collaborator-mediated 외부 PR, G5 체리픽 통합 |
| 원 커밋 / 통합 적용 커밋 | `f338d6357c21471b7d5068580882f9a1f2a166c` / `23382b987` |
| 적용 순서 | G5 3/8, #2456 뒤 체리픽, 충돌 없음 |
| 작성 시점 참고값 | 원 PR은 `MERGEABLE` / `BEHIND`, 원 head CI는 성공이다. 통합 PR의 최신 head CI와 mergeable 상태를 merge 전 다시 확인한다. |

## 변경 검토

serializer가 `SpaceOnly` 줄간격을 `BETWEEN_LINES`로 방출하지만 paraPr parser 두 곳은
`SPACEONLY`와 `SPACE_ONLY`만 읽었다. 따라서 rhwp가 저장한 줄 사이 여백이 재로드 시 Percent로
바뀌었다. PR은 두 parser 위치를 모두 보완해 방출 문자열을 다시 `SpaceOnly`로 복원한다.

## 검증

| 게이트 | 결과 |
|---|---|
| `para_shape_linespacing_between_lines_parses_as_space_only` | PASS |
| G5 공통 full regression / fmt / clippy / diff / WASM | PASS |

## 시각 검증 판단

줄간격 값이 저장·재로드 뒤 동일 model 값으로 복원되는 parser 계약 변경이다. renderer나 조판 알고리즘은
바꾸지 않으며 시각 fidelity 개선을 주장하지 않으므로 PDF visual sweep 대상이 아니다.

## 최종 의견

G5 통합 PR에 기여자 원 커밋을 보존해 수용한다. 공통 검증과 최종 조건은
[G5 통합 실행 계획](pr_2455_review_impl.md)을 따른다.
