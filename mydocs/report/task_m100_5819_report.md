# #5819 표 생성 CLI/MCP 옵션 개선 결과

- Issue: [#5819](https://github.com/edwardkim/rhwp/issues/5819)
- 기준 `devel`: `f537df5ea328ba7d8c0c8e85c0199897c0047049`
- 최종 검증 head: `be3581023549ff7a1db81c600f5c81f71f9af38f` (제품 코드 `4a9b47726`, 위임 분류 원장 보정 포함)
- branch: `fix/5819-insert-table-options`
- 수행 환경: macOS, `target/pr-review`, Rust 1.93.1
- 계획: [구현 범위와 검증 계획](../plans/task_m100_5819.md)

## 현존 여부와 변경

기본 `insert-table` CLI/MCP는 이미 있었지만, 최신 기준 head로 빌드한 CLI에서 `--widths`,
`--alignments`, `--at-field`, `--repeat-header` 각각이 `알 수 없는 옵션`과 exit 2를 반환했다.
추가 요구는 남아 있었다.

공통 표 생성 경로에 열 너비·문단 정렬·반복 머리행 옵션을 추가하고 CLI/MCP에 연결했다.
기존 코어 `create_table_native` 호출의 기본 동작은 보존했다. 새 CLI의 기본 반복 머리행은 true이며
첫 행을 머리행으로 지정한다. 절대 너비는 입력한 합계로, 비율은 기존 기본 표 폭에 대한 100%로
계산한다. 열별 서식은 생성 시 한 번에 적용하며 셀별로 전체 문서를 반복 재계산하지 않는다.

`--at-field`는 유일한 본문 필드 문단 바로 뒤에 독립 표를 삽입한다. 기존 필드의 ID·이름·안내문·
내용을 보존한다. `--dry-run`도 같은 생성·검증 경로를 실행하고 저장만 생략한다. 잘못된 옵션은
입력 파일과 기존 출력 파일을 바꾸지 않는다. MCP는 값을 받는 bool 옵션의 false를 전달하며,
`dryRun:false` 같은 presence flag는 계속 생략한다.

전체 옵션은 [CLI 계약](../manual/cli_commands.md#edit-insert-table)에 기록했다.

## 한컴 OLE 실측과의 대조

이슈와 작업지시자가 제공한 실측은 다음 두 동작이다.

1. OLE `WidthType=2`에서만 `ColWidth`의 열별 너비가 반영된다.
2. TableCreate의 HWPX 기본값은 `repeatHeader="1"`, `pageBreak="CELL"`이다.

rhwp는 OLE를 호출하지 않고 생성 셀의 HWPUNIT 너비와 표 전체 폭을 직접 기록한다. 실제
CLI 생성 HWPX의 XML에서 2행의 셀 너비가 각각 `3000,6000,9000`이며 등분되지 않는 것을
검사했다. 기본 XML은 `repeatHeader="1" pageBreak="CELL"`, 명시적 false는
`repeatHeader="0" pageBreak="CELL"`로 저장되었다. 양쪽 `--verify`는 `diffCount:0`이었다.

내부 enum 이름과 XML 어휘는 다르다. 현행 serializer/parser에서 HWPX `CELL`은
`TablePageBreak::RowBreak`/HWP5 값 2에 대응한다. 초기 후보 `15e38649c`의 `CellBreak` 선택은
자기 왕복 검사만으로는 검출되지 않았으나 XML 대조에서 `TABLE` 출력으로 드러났다.
`4a9b47726`에서 생성 IR과 원시 HWP5 TABLE 속성을 함께 고치고 XML 속성 자체를 검사하는
독립 회귀를 추가했다. serializer의 공통 매핑이나 기존 golden·허용치는 바꾸지 않았다.

## 검증

첫 전체 회귀는 9,537건이 통과하고 `classification_drift_is_blocked` 1건이 실패했다.
새 공통 생성 함수로 위임하는 기존 래퍼가 미분류였으므로 `DelegatesTo("create_table_with_options_native")`와
근거를 등록했다. 분류/위임 대상 도달/면제 정합/무효화 밀도/스캐너 자체 검사 5건이 통과했으며,
래칫 수치나 판정 보류 상한을 변경하지 않았다. 이 보정을 포함한 최종 head에서 검증 묶음을 다시 실행했다.

검증 완료일은 2026-09-13(KST)이다.

| 범위 | 실행 결과 |
| --- | --- |
| #5819 집중 CLI/MCP/XML 회귀 | 5 PASS |
| 패스스루 분류·위임 회귀 | 5 PASS |
| `cargo fmt --all` 및 `-- --check` | PASS |
| native Clippy `--locked --target-dir target/pr-review -- -D warnings` | PASS |
| WASM Clippy `--locked -p rhwp --lib --target wasm32-unknown-unknown --target-dir target/pr-review -- -D warnings` | PASS |
| workspace build `--locked --workspace --target-dir target/pr-review` | PASS |
| all-target Clippy `--locked --workspace --all-targets --target-dir target/pr-review -- -D warnings` | PASS |
| manifest `--prepare`/`--check`, unit tiers `--check` | 모두 PASS |
| 최종 전체 release-test 회귀 | **9,538 PASS, 46 SKIP, 실패 0**; 실행 334.750초, exit 0 |
| `git diff --check`, 변경 Markdown 상대 링크 검사 | PASS |


집중 회귀는 `tests/cases/issue_5819_insert_table_options.rs`의 실제 CLI/MCP 호출 5개다.
HWP/HWPX 저장·재파싱, 절대/비율 너비, 열 정렬, 반복 머리행 on/off, 필드 보존,
16종 잘못된 입력의 실제/preview 요청, 정상 dry-run, MCP false 전달, HWPX XML 외부 계약을 검사했다.

```bash
node scripts/run-rust-test.mjs issue_5819_insert_table_options -- \
  --cargo-profile release-test --target-dir target/pr-review
cargo nextest run --locked --cargo-profile release-test \
  --target-dir target/pr-review --tests --no-fail-fast
```

## 검증 입력 커밋 확인

이번 기능의 직접 재현·회귀 입력은 아래 기존 tracked 파일을 재사용했다. 기준 코드 commit의
`git show <commit>:<path>` blob과 실제 실행 파일을 바이트 단위로 비교했고 SHA-256이 일치했다.
개인 Downloads 경로를 수용 근거로 사용하지 않았다. CLI/MCP 테스트가 생성한 임시 저장본과
로그·JSON은 파생 출력으로 남기고 저장소에는 테스트 원본과 입력 fixture를 보존한다.

| 입력 | 출처·역할 | SHA-256 |
| --- | --- | --- |
| `samples/field-01.hwp` | 기존 HWP 본문·중복 필드 fixture, 절대/비율 너비와 필드 보존·MCP 회귀 | `518cb939079e6e0640a5f813597f744e2528a17ca52ee418929f1c8f4b5380c0` |
| `samples/issue5162_field_wraps_table.hwpx` | 기존 HWPX 필드/표 fixture, HWPX 재파싱과 XML 계약 | `c8f07209f7d60d60c32c7918800545c1b3079ef4e0178eaf44ca2bfe23cfc0c8` |

## 적용 범위와 남은 제한

- 조판 알고리즘·측정/배치·baseline/golden 변경은 없다. 지정한 생성 속성과 포맷 계약의 보존을
  검증했으며 PDF/화면 일치 개선으로 해석하지 않는다. visual sweep은 실행하지 않았다.
- 표 셀·글상자 내부 필드는 기존 본문 표 생성 CLI의 범위 밖이며 명시적 오류로 거부한다.
  필드 문단 내부의 정확한 문자 위치 삽입도 이번 계약에 포함하지 않는다.
- 이슈의 235쪽 실제 템플릿은 첨부되지 않아 그 문서에서의 성능·이미지 보존·198/199/204쪽
  `LAYOUT_OVERFLOW`를 검증하거나 해결했다고 주장하지 않는다.
- 기존 관련 이슈 #3608, #4994, #4995의 전체 범위를 이 변경으로 완료 처리하지 않는다.
- #5819 assignee는 `jangster77`로 지정했다. [PR #7068](https://github.com/edwardkim/rhwp/pull/7068)을
  devel 대상 Open PR로 생성하고 `Closes #5819`로 연결했다. 요청 옵션은 구현·검증했으며,
  이슈 작성자가 별도 관찰 대상으로 명시한 overflow를 추가 종료 조건으로 두지 않는다.
  이슈 상태는 병합 전 OPEN이며 devel 병합 후 종료 여부를 확인한다. 실제 사용자 템플릿 검증은 별도 범위다.

## 완료 시점의 devel 전진 확인

검증 중 `upstream/devel`은 #6965/#6966 병합으로 `b116c11d0736fb1c105ed64bfc694b43dcd48b7d`까지
전진했다. 기준 이후 변경은 Chrome/Firefox 확장 다운로드 처리와 그 문서·테스트이며 이 작업의
Rust/CLI/MCP 변경 경로와 겹치지 않았다. `git merge-tree --write-tree HEAD upstream/devel`은
충돌 없이 성공했다. 작업 branch에 불필요한 merge를 추가하지 않았으며, 위 전체 회귀 결과는
명시한 검증 head에 대한 결과다.

## PR 제출 전 별도 worktree 검증

제출 후보 `728452ca673d02dadc3be831d8e26eeeab0ab2f2`는 위 전체 회귀 head 이후 계획·결과 문서만
변경했다. `/Users/tsjang/rhwp-5819-rust-review` detached worktree에서 같은 후보의 prepare,
fmt 적용·검사, native/WASM32/all-target Clippy, workspace build, manifest·unit-tier check
9단계를 다시 실행해 모두 exit 0을 확인했다. tracked diff는 없고 파생 suite는 커밋하지 않았다.
전체 회귀는 원 작업 checkout의 결과이며, 별도 worktree에서는 제출 전 lint·정책 검사를 수행했다.

[Self-review](../pr/archives/pr_7068_review.md)와 [오늘할일](../orders/20260913.md)을 같은 PR에 포함한다.
최신 head CI와 병합 승인은 남은 게이트다.
