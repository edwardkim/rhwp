# PR #7315 리뷰 — Studio 대화상자·도구 상자 영문 표시 6단계

## 접수 정보

| 항목 | 값 |
| --- | --- |
| PR | [#7315](https://github.com/edwardkim/rhwp/pull/7315) |
| 작성자 | `rubidus-api` 외부 contributor |
| 관련 이슈 | [#5852](https://github.com/edwardkim/rhwp/issues/5852) 6단계 |
| 원 contributor head | `15358c30b50cb3890ce30c2488183d3611736704` |
| 통합 적용 commit | `6570f2935` (`git cherry-pick -x`) |
| 통합 base | `be33c935939c84d4fca9692ddd64a5d951876ded` (`upstream/devel`) |
| 코드·증적 후보 head | `4e054c2ae` |

라우팅: `collaborator_external_pr` + `intake_and_review` + `local_validation`.
원 contributor commit은 rewrite하지 않았고 최신 `upstream/devel`에서 만든 통합 검토 브랜치에
`-x`로 적용했다. 이 문서와 오늘할일은 그 후보 뒤 review-only trailing 기록이다.

## 변경 검토

25개 Studio 파일에서 셀 테두리·문단·글자 모양·수식·쪽 테두리·그림 속성 대화상자와 도구 상자에
남은 한국어 표시 문자열을 locale key로 전환하고, 영문·한국어 catalog에 대응 값을 추가한다.
변경된 `t()`/`i18nText()`/`nameKey` 호출 108개는 두 catalog에 모두 존재한다. catalog 키 수는
영문 1,947개, 한국어 1,947개이며 어느 방향에서도 누락 키가 없었다.

변경은 Studio UI 문자열과 관련 unit test에 한정된다. renderer·layout·pagination·paint·WASM API·
HWP/HWPX fixture·기준 PDF는 바뀌지 않아 PDF Visual Sweep은 비해당이다.

## 완료한 검증

| 검증 | 결과 |
| --- | --- |
| `npm --prefix rhwp-studio run test` | 1,761 passed, 0 failed, 2 skipped |
| `npm --prefix rhwp-studio run build` | 통과 (`tsc` 포함) |
| en/ko locale key 대조 | 양방향 누락 0개 |
| 변경 호출 108개 key 해석 | 영문·한국어 catalog 모두 존재 |
| 원 PR GitHub CI | 완료·성공 상태를 검토 시점에 확인; 통합 head CI와 구분 |

## 최종 판정

**승인.** 문자열 전환 범위와 두 locale catalog의 key 대응, Studio test/build를 확인했으며 추가
메인터너 코드 보정이 필요한 문제를 발견하지 못했다. 이 판정은 GitHub approve·push·merge를 뜻하지
않는다.

통합 PR을 생성한 뒤에는 이 review·오늘할일 trailing commit을 포함한 최신 head의 required CI,
`MERGEABLE`/`CLEAN`, 그리고 작업지시자의 merge 지시를 다시 확인한다. 병합 뒤에만 원 contributor
PR에 실제 통합 PR·merge SHA·CI와 수용 범위를 한국어로 알리고 후속 상태를 처리한다.
