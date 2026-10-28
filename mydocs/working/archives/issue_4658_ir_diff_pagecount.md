---
kind: working
status: active
issue: 4658
---

# ir-diff 쪽수 계약 (#4658)

작업 브랜치: `fix/4658-ir-diff-pagecount`
대상: `rhwp ir-diff --json`

한 줄 요약: IR 필드가 같아도 `info.pageCount` 가 다르면 `identical:true` 를 내지 않는다.

## 이슈가 요구한 것

`samples/2026_oss_rst.hwp`(6쪽) 와 `samples/hwpx/2026_oss_rst.hwpx`(7쪽) 를
`ir-diff --json` 하면 종전엔 `identical:true`·`diffCount:0` 이었다. 조판이
다른데 IR 동등으로 읽히면 변환 게이트가 거짓 통과한다.

하지 말 것: 두 파일의 쪽수를 같게 맞추기, 두 번째 IR 발명.

## 원인 (읽기 B)

IR 비교는 이미 `line_segs`·표 `page_break`·셀 재귀를 본다. 6↔7 은 빠진 IR
필드가 아니라 **출처 프로파일**이다. 커밋된 hwpx 에는 `META-INF/rhwp-hwp5-origin`
이 없고 native HWPX 조판(분할 tolerance 등)을 타고, hwp 는 HWP5 시멘틱을 탄다.
이슈 댓글(planet6897)과 같다. `identical` 은 조판 동등을 함의하지 않는다.

## 계약

- `--json` 봉투는 항상 `pageCountA`·`pageCountB` 를 싣는다. 값은 `info --json` 과
  같은 조판 쪽수다.
- 쪽수가 다르면 `identical:false`, `diffCount>=1`, `categories.pageCount` 가 1,
  종료 코드 3.
- IR 비교 목록을 넓히지 않는다. 출처 마커를 IR 필드로 승격하지 않는다.

## 만진 경로

- `src/cli/queries/ir_comparison.rs` — `info` 와 같은 `load_document` 로 쪽수를 읽어 비교
- `src/cli/metadata/capabilities/extended.rs` — `recordFields` 에 `pageCountA`/`pageCountB`
- `tests/cases/issue_4658_ir_diff_pagecount.rs`
- `mydocs/manual/ir_diff_command.md` · `cli_commands.md` · `agent_knowledge_map.md`

만지지 않은 경로: 페이지네이션 엔진, HWPX 직렬화, 스킬 본문, `tests/generated`.

## 시험

```
cargo fmt --all -- --check
cargo test --test issue_4658_ir_diff_pagecount -- --nocapture
rhwp info samples/2026_oss_rst.hwp --json
rhwp info samples/hwpx/2026_oss_rst.hwpx --json
rhwp ir-diff samples/2026_oss_rst.hwp samples/hwpx/2026_oss_rst.hwpx --json
```

## PR 메모

`closes #4658`. `gh pr create --repo edwardkim/rhwp --base devel --body-file`.
첫 체크박스 = `cargo fmt --all -- --check`.
