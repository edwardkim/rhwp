# PR #2141 검토 실행 기록

## 대상

- PR: `edwardkim/rhwp#2141`
- 제목: `Issue #2136: near-top 저장 리셋 상한 2000→2500HU — sb=2500HU 리셋 배제 과적 해소`
- 원 commit: `0734589bf635316ac401d374eff55ba0e517009d`
- 로컬 체리픽 commit: `443642354`

## 실행

```bash
gh pr edit 2141 --repo edwardkim/rhwp --add-reviewer jangster77
git fetch upstream pull/2141/head:local/pr2141
git cherry-pick -x 0734589bf635316ac401d374eff55ba0e517009d
target/release-test/rhwp dump-pages samples/task2136/neartop_reset_sb2500.hwpx --page 0
/opt/homebrew/bin/npx -y --package=file:/Users/tsjang/rhwp/tools/hwp-convert-mcp-client-20260709-231800.tar.gz -- \
  hwp2020-mcp-convert --env-file /Users/tsjang/Cloud/Devel/hwp-convert/.env.local \
  --input /Users/tsjang/rhwp/samples/task2136/neartop_reset_sb2500.hwpx \
  --target pdf --output-dir /Users/tsjang/rhwp/pdf/task2136 \
  --output-filename neartop_reset_sb2500-2020.pdf --timeout-seconds 240
```

## 후속 메모

합성 fixture는 rhwp 테스트 기준으로 2쪽이지만 MCP 기준 PDF는 1쪽이다. 원 PR의 실제
실문서 근거와 합성 fixture를 분리해서 설명해야 한다.
