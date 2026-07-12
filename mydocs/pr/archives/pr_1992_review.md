# PR #1992 리뷰 - README 로드맵 링크 경로 수정

## 메타

| 항목 | 내용 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/1992 |
| 제목 | Issue #1975: Readme 로드맵 문서 링크 경로 수정 |
| 작성자 | planet6897 |
| base | `devel` |
| head | `fix/1975-readme-roadmap-link` |
| 문서 작성 시점 head SHA | `4e26b78c36ed27f8f240b02dff6d3deb6b251ac8` |
| 실제 변경 commit | `e3cff6a6319977b7648e9d0ee5843e3d2d34dbee` |
| 체리픽 commit | `a0c88d5ba` |
| 규모 | 1 file, +1 / -1 |
| 변경 파일 | `README.md` |
| 처리 방식 | 옵션 2 문서 통합 PR에 실제 변경과 review 문서를 함께 반영 |

## 관련 이슈

- https://github.com/edwardkim/rhwp/issues/1975
- README의 로드맵 링크가 존재하지 않는 `mydocs/report/rhwp-milestone.md`를 가리키는 문제다.
- 실제 파일은 `mydocs/report/archives/rhwp-milestone.md`에 있다.

## 변경 범위

- README 링크를 `mydocs/report/archives/rhwp-milestone.md`로 정정한다.
- 코드, 테스트, 렌더 출력 경로 변경은 없다.
- PR head에는 `devel` merge commit이 2개 포함되어 있으므로, 옵션 2 문서 통합 브랜치에는 실제 변경 commit만 체리픽했다.

## 시각 검증

문서 링크 수정 PR이므로 visual sweep 대상이 아니다.

## 로컬 검증

```bash
git cherry-pick e3cff6a6319977b7648e9d0ee5843e3d2d34dbee
test -f mydocs/report/archives/rhwp-milestone.md
git diff --check
```

결과:

- 실제 target 파일 존재 확인: 통과
- `git diff --check`: 통과

## 검토 결과

README 링크 정정만 포함한 문서성 변경이며 차단 이슈는 없다. 옵션 2 문서 통합 PR로 반영하고, merge 후 원 PR #1992에는 체리픽 반영 사실을 코멘트한 뒤 close 처리한다.

