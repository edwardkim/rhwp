# PR #2155 검토 실행 기록

## 대상

- PR: `edwardkim/rhwp#2155`
- 제목: `Task #2150: 21761835 잔여 팽창 분해 - 한글 NO_LS fresh 줄높이 공식 확정 + 오라클 도구 v2`
- 원 commit: `9c99b08c4de61f3eb49cd5e5b0edb75b183feef9`
- 로컬 체리픽 commit: `f4519e25f`

## 실행

```bash
gh pr edit 2155 --repo edwardkim/rhwp --add-reviewer jangster77
git fetch upstream pull/2155/head:local/pr2155
git cherry-pick -x 9c99b08c4de61f3eb49cd5e5b0edb75b183feef9
python3 -m py_compile tools/hangul_row_heights2.py tools/make_ls_ladder.py tools/probe_ls_ladder.py
```

## 후속 메모

런타임 코드 변경이 없는 분석/도구 PR로 처리했다.
