# PR #2142 검토 실행 기록

## 대상

- PR: `edwardkim/rhwp#2142`
- 제목: `Issue #2137: TopAndBottom float 앵커 saved-bounds 신뢰 — 소형 글상자 여백 스필`
- 원 commit: `ce6a7eced019412cd5cc1f7898b23fa6c64e181f`
- 로컬 체리픽 commit: `bb9f9734a`

## 실행

```bash
gh pr edit 2142 --repo edwardkim/rhwp --add-reviewer jangster77
git fetch upstream pull/2142/head:local/pr2142
git cherry-pick -x ce6a7eced019412cd5cc1f7898b23fa6c64e181f
target/release-test/rhwp dump-pages samples/task2137/156618554_petfood_press.hwp --page 0
/opt/homebrew/bin/npx -y --package=file:/Users/tsjang/rhwp/tools/hwp-convert-mcp-client-20260709-231800.tar.gz -- \
  hwp2020-mcp-convert --env-file /Users/tsjang/Cloud/Devel/hwp-convert/.env.local \
  --input /Users/tsjang/rhwp/samples/task2137/156618554_petfood_press.hwp \
  --target pdf --output-dir /Users/tsjang/rhwp/pdf/task2137 \
  --output-filename 156618554_petfood_press-2020.pdf --timeout-seconds 240
```

## 후속 메모

README, rhwp page count, MCP PDF page count가 모두 1쪽으로 일치한다.
