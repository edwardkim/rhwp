# PR #2463 리뷰 - HWPX symMark 강조점 파싱 보존

## 메타데이터

| 항목 | 값 |
|---|---|
| 원 PR | [#2463](https://github.com/edwardkim/rhwp/pull/2463) |
| 작성자 | `kevin9327` |
| base | `devel` |
| 리뷰 경로 | collaborator-mediated 외부 PR, G5 체리픽 통합 + 회귀 보강 |
| 원 커밋 / 통합 적용 커밋 | `9bca8907fea664fb7add7343b96ad43e2cea1b82` / `5adc69b78` |
| 추가 보강 커밋 | `685c7943c` - 7개 symMark 값 전체 parser 회귀 확장 |
| 적용 순서 | G5 8/8, #2462 뒤 체리픽, 충돌 없음 |
| 작성 시점 참고값 | 원 PR은 `MERGEABLE` / `BEHIND`, 원 head CI는 성공이다. 통합 PR의 최신 head CI와 mergeable 상태를 merge 전 다시 확인한다. |

## 변경 검토

기존 charPr parser는 `symMark` 속성을 no-op으로 무시해 강조점이 재로드 후 NONE이 됐다. PR은
`DOT_ABOVE`부터 `COLON`까지 serializer의 역방향 매핑을 추가한다. 통합 검토는 단일 DOT 사례에만 머물지
않도록 NONE을 포함한 0부터 6까지 모든 지원값을 확인하는 회귀로 보강했다.

## 검증

| 게이트 | 결과 |
|---|---|
| `parse_char_pr_captures_sym_mark` | PASS - 7개 지원값 전체 확인 |
| G5 공통 full regression / fmt / clippy / diff / WASM | PASS |

## 시각 검증 판단

강조점의 document model 저장값을 복원하는 parser 변경이며 glyph paint 구현을 바꾸지 않는다. 구조 회귀로
검증했으며 visual sweep은 적용하지 않는다.

## 최종 의견

기여자 변경에 전체 enum 회귀 보강을 더해 G5 통합 PR로 수용한다. 공통 검증과 최종 조건은
[G5 통합 실행 계획](pr_2455_review_impl.md)을 따른다.
