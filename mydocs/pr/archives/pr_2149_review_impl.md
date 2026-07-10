# PR #2149 검토 실행 기록

## 대상

- PR: `edwardkim/rhwp#2149`
- 제목: `Task #2146: NO_LS 라벨 셀 행높이 선언 신뢰 - 사선/Fixed-모순 셀`
- 원 commit: `89bfe8a9871e745ef721b01a6f70c3cabc48afb0`
- 로컬 체리픽 commit: `f91234b62`

## 실행

```bash
gh pr edit 2149 --repo edwardkim/rhwp --add-reviewer jangster77
git fetch upstream pull/2149/head:local/pr2149
git cherry-pick -x 89bfe8a9871e745ef721b01a6f70c3cabc48afb0
target/release-test/rhwp dump-pages samples/task2146/21761835_jeonjik_exemption_table.hwp --page 0
/opt/homebrew/bin/npx -y --package=file:/Users/tsjang/rhwp/tools/hwp-convert-mcp-client-20260709-231800.tar.gz -- \
  hwp2020-mcp-convert --env-file /Users/tsjang/Cloud/Devel/hwp-convert/.env.local \
  --input /Users/tsjang/rhwp/samples/task2146/21761835_jeonjik_exemption_table.hwp \
  --target pdf --output-dir /Users/tsjang/rhwp/pdf/task2146 \
  --output-filename 21761835_jeonjik_exemption_table-2020.pdf --timeout-seconds 900
```

## maintainer 보정

```text
tests/issue_2146_no_ls_label_cell_declared_height.rs
```

doc comment list continuation indentation only. Clippy pass를 위한 비기능 변경이다.
