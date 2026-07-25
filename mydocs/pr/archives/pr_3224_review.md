# PR #3224 검토 기록 — r22 10k 한글 오라클 서베이 보고서

## 메타와 통합 판단

| 항목 | 내용 |
| --- | --- |
| 원 PR | [#3224](https://github.com/edwardkim/rhwp/pull/3224) |
| 작성자 | `planet6897` |
| 원 PR 기준 / head | `devel@50803cc1e987bea87722ec30c1ca794314e171ac` / `a83c33dce6688148774ae9247a9f649ecd9e8d79` |
| 문서 작성 시점 참고값 | OPEN, non-draft, MERGEABLE/BEHIND, maintainerCanModify=true, reviewer `jangster77`, maintainer 보류·리뷰 코멘트 없음. CI preflight 및 Build & Test 성공, 문서-only 조건으로 heavy job은 skipped (2026-07-24 조회). |
| 검토 브랜치 / 통합 기준 | `integrate/planet6897-20260724` / `upstream/devel@973de548faedc6709ef862a1a12aa7146c225ac5` |
| 누적 순서 | 1/2 — #3224 → #3241 |
| 체리픽 | `650dbbe1` → `4761738`, `e65ca4a` → `f7057ac`, `a83c33d` → `6bc39f7`; 충돌 없음 |
| 최종 merge 조건 | 통합 PR 최신 head의 required CI 성공 및 작업지시자 승인 |

검토는 기본 작업트리에서 진행 상태를 확인할 수 있도록 최신 `devel` 위
`integrate/planet6897-20260724`에서 수행했다. 원 PR은 오래된 base를 가리키므로 직접 merge하지 않고,
작성자 커밋을 `git cherry-pick -x`로 최신 통합 후보에 보존했다.

## 변경 범위와 근거의 성격

- `mydocs/report/survey_10k_r22_20260723.md`만 추가하는 문서 PR이다. r22는 이전 planet6897 렌더 4건
  (#3019, #3084, #3086, #3129)을 `devel@886bda08b`에 적용한 당시 tree의 10,000건 oracle 측정 기록이다.
- 보고서는 쪽수 회귀 0, 공통 비교 가능 표본의 픽셀 평균 93.88→93.90, +2pp 이상 개선 62건/하락 2건을
  기록한다. r19 대비 복귀 7건은 해당 4 PR 효과가 아니라 base devel의 #2808 효과라고 분리한 점이
  적절하다.
- 후속 makeup 결과를 같은 보고서에 반영해 ERR 51건을 오염 31·보호 15·기타 미지원 5로 재분류하고,
  STALL 102건은 미측정으로 남겼다. 이전의 표본 편향 서술을 철회한 것도 보고서 내부와 일관된다.
- 원시 10k corpus와 장시간 실행 로그는 저장소에 포함되지 않는다. 따라서 이 문서는 현재 tree의 독립
  회귀 게이트가 아니라 당시 실행의 조사·추적 기록으로 취급한다. 코드 merge 안전성의 단독 근거로
  과장하지 않는 현 서술을 유지해야 한다.

렌더러·샘플·golden·실행 코드는 바꾸지 않으므로 #3224 단독으로는 visual sweep 및 cargo 검증 대상이 아니다.
보고서 파일명은 일반 task 보고서 명명 규칙의 대상이 아닌 조사 보고서라서 유지한다.

## 검증

| 항목 | 결과 |
| --- | --- |
| 체리픽 / 변경 범위 | 최신 `upstream/devel` 위 3개 문서 커밋 적용, 충돌 없음 |
| Markdown 링크·경로 | `scripts/check_markdown_links.py`로 review·계획서·오늘할일 5개 문서 검사, 내부 상대 링크 이상 없음 |
| 통합 tree 정적 검증 | `git diff --check`, `cargo fmt --check` 공통 최종 게이트 통과 |

이 PR의 문서만으로 cargo를 별도 재실행하지 않았다. 다만 같은 통합 tree의 #3241 renderer 변경 때문에
전체 renderer gate를 한 번 수행했고, 결과는 [#3241 검토 기록](pr_3241_review.md)에 남겼다.

## 리스크와 권고

- 10k 수치의 재현에는 저장소 밖 oracle·COM 환경과 장시간 실행이 필요하다. 후속 보고서가 수치를 갱신할
  때도 표본 seed, 기준 tree, STALL/ERR 분류 규칙을 함께 보존해야 비교가 가능하다.
- 현재 원 PR은 BEHIND다. 통합 PR로 최신 devel에 재배치한 결과의 CI만 merge 판단에 사용한다.

**권고: 수용.** #3241과 함께 통합 PR 후보에 포함하고, 원 PR은 통합 PR merge 후에만 supersede close 여부를
별도 승인으로 처리한다.

## Merge 완료 및 후속 처리

- [#3256](https://github.com/edwardkim/rhwp/pull/3256)이 2026-07-24
  `e05a5a8bdcf636844e80744d91fff706c1c28ce7`로 `devel`에 merge 됐다.
- 통합 PR의 CI(Build & Test·Native Skia·기본 기능 테스트 8 shard), CodeQL, Render Diff가 모두 성공했다.
- [#3224](https://github.com/edwardkim/rhwp/pull/3224)는 통합 완료 안내와 감사 코멘트 후 supersede close했다.
  이 보고서에는 별도 close 대상 이슈가 없다.
