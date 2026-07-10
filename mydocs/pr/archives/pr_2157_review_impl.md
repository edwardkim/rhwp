# PR #2157 검토 실행 기록

## 대상

- PR: `edwardkim/rhwp#2157`
- 제목: `Issue #2151: HWP3 그림 pgy=0 페이지 시작 후 거짓 쪽 경계 — prev_last_pgy 리셋`
- 원 commit: `a36dd82cb1858872dc2bdac43e3c96799d0743f8`
- 로컬 체리픽 commit: `86a7ffaa5`

## 실행

```bash
gh pr edit 2157 --repo edwardkim/rhwp --add-reviewer jangster77
git fetch upstream pull/2157/head:local/pr2157
git cherry-pick -x a36dd82cb1858872dc2bdac43e3c96799d0743f8
target/release-test/rhwp dump-pages samples/hwp3-sample14.hwp --page 0
target/release-test/rhwp dump-pages samples/hwp3-sample11.hwp --page 0
/opt/homebrew/bin/npx -y --package=file:/Users/tsjang/rhwp/tools/hwp-convert-mcp-client-20260709-231800.tar.gz -- \
  hwp2020-mcp-convert --env-file /Users/tsjang/Cloud/Devel/hwp-convert/.env.local \
  --input /Users/tsjang/rhwp/samples/hwp3-sample14.hwp \
  --target pdf --output-dir /Users/tsjang/rhwp/pdf/hwp3 \
  --output-filename hwp3-sample14-2020.pdf --timeout-seconds 900
/opt/homebrew/bin/npx -y --package=file:/Users/tsjang/rhwp/tools/hwp-convert-mcp-client-20260709-231800.tar.gz -- \
  hwp2020-mcp-convert --env-file /Users/tsjang/Cloud/Devel/hwp-convert/.env.local \
  --input /Users/tsjang/rhwp/samples/hwp3-sample11.hwp \
  --target pdf --output-dir /Users/tsjang/rhwp/pdf/hwp3 \
  --output-filename hwp3-sample11-2020.pdf --timeout-seconds 900
```

## 후속 메모

HWP3 sample14/sample11 모두 MCP PDF와 rhwp page count가 일치했다.
