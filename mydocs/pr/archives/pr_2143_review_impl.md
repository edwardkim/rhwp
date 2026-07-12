# PR #2143 검토 실행 기록

## 대상

- PR: `edwardkim/rhwp#2143`
- 제목: `Issue #2098/#2138: 불확실 앵커 footer fit 62px 마진 — 10k r12 회귀 60건 대응`
- 원 commit: `8528810eb9a2da293250e73d99f9b8a21cc9cb4f`
- 로컬 체리픽 commit: `eed791673`

## 실행

```bash
gh pr edit 2143 --repo edwardkim/rhwp --add-reviewer jangster77
git fetch upstream pull/2143/head:local/pr2143
git cherry-pick -x 8528810eb9a2da293250e73d99f9b8a21cc9cb4f
target/release-test/rhwp dump-pages samples/task2098/page_bottom_fixed_anchor_margin_split.hwpx --page 0
/opt/homebrew/bin/npx -y --package=file:/Users/tsjang/rhwp/tools/hwp-convert-mcp-client-20260709-231800.tar.gz -- \
  hwp2020-mcp-convert --env-file /Users/tsjang/Cloud/Devel/hwp-convert/.env.local \
  --input /Users/tsjang/rhwp/samples/task2098/page_bottom_fixed_anchor_margin_split.hwpx \
  --target pdf --output-dir /Users/tsjang/rhwp/pdf/task2098 \
  --output-filename page_bottom_fixed_anchor_margin_split-2020.pdf --timeout-seconds 240
```

## 후속 메모

MCP PDF는 1쪽, rhwp/test는 2쪽이다. 이 불일치는 리뷰 문서와 PR 본문에 caveat로 남긴다.
