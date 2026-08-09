# PR #4094 검토

## 결론

**메인터너 보정 후 수용 가능으로 판단한다.** 저장 HWPX의 Square 어울림 표 보정 뒤 남은 tail 쪽
경계를 fixture와 회귀 test로 고정해, HWP 2020 MCP 기준 PDF와 같은 17쪽을 반환하도록 보완했다.

| 항목 | 내용 |
| --- | --- |
| PR | [#4094](https://github.com/edwardkim/rhwp/pull/4094) |
| 작성자·대상 | `planet6897` → `devel` |
| contributor 원 head | `e689ef0412344e683bf65163e4f23033d3b1b390` |
| code candidate | `a9c29700698a0c916462fedbe5025cf3cbd16e76` |
| 최신 source head | `afc83ff72186c49894081b5786ad4ef7aa60d06f` (`devel` 병합) |
| 관련 이슈 | #4090, #3820 |
| 작성 시점 PR 상태 | `mergeable=MERGEABLE`, `mergeStateStatus=CLEAN` |

## 변경 범위

contributor 원 변경은 Square 어울림 표가 세로 흐름에 표 전체 높이를 더하지 않도록 배제 밴드를
도입한 것이다. 최소 fixture `samples/issue4090/156492236_규제샌드박스_min.hwpx`를 추가해
한컴 기준의 17쪽과 차이를 재현했다.

메인터너 보정은 저장 HWPX의 마지막 tail 줄이 문서 끝에 도달할 때만 명시적 page break를 적용하도록
한정했다. `pi=59`, `pi=74`, `pi=183`의 물리 줄은 각각 5→6, 7→8, 15→16쪽으로 분리되며, 일반 stored
line·다단·다음 logical line이 있는 경우에는 page break를 열지 않는다.

## 기준 PDF와 시각 판정

- 입력 fixture SHA-256:
  `d6f4d431b9a4d934b3b4e4330546ef61768c953c2e1328010d2f75440fefa070`
- 기준 PDF:
  `pdf/issue4090/156492236_규제샌드박스_min-hancom2020-production-verify.pdf`
- 기준 PDF SHA-256:
  `60b4d14e7305d148a913f281c6629b531a3fedcc7e6f76042c0994169001ccfc`
- 산출 방식: HWP 2020 MCP `PrintToPDFEx`, `PrintMethod=0`
- PDF 정보: A4, 17쪽, 2,583,139 bytes
- rhwp `dump-pages --json`: 17쪽

이번 판단은 페이지 수와 tail 줄 소유권 정합까지다. **17쪽 전체에 대해 rhwp 렌더와 기준 PDF의
raster/pixel 1:1 비교는 수행하지 않았다.** 글꼴·표·개체·머리말을 포함한 fidelity는 #3820의
후속 PDF 직접 대조 범위로 유지한다.

## 검증

- `cargo fmt --check`를 실행해 통과했다.
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --lib hwpx_explicit_page_break_tail -- --nocapture`를
  실행해 2건이 통과했다.
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_4090_hwpx_tail_page_break -- --nocapture`를
  실행해 1건이 통과했다.
- `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`를 실행해 통과했다.
- `CARGO_INCREMENTAL=0 wasm-pack build --target web --out-dir pkg`를 실행해 통과했다.
- 최초 review 기록 push 뒤 source branch에 최신 `devel` 병합 `afc83ff…`가 추가됐다. 이 head에서
  [CI](https://github.com/edwardkim/rhwp/actions/runs/31148171463),
  [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/31148171307),
  [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/31148171363), Native Skia, 모든
  default-feature test shard와 Build & Test aggregate가 성공했다.

## 최종 조건

최신 `devel` 병합 뒤 이 갱신 review·오늘할일 commit을 source branch에 추가한 뒤, 현재 녹색인
`afc83ff…`를 후보로 하는 review-only fast-pass aggregate와 최신 mergeability를 확인한다. 이후 merge와
기여자 안내 코멘트는 작업지시자의 별도 승인을 받은 뒤에만 수행한다.

```text
base route: collaborator_external_pr.md
modifiers: intake_and_review.md, local_validation.md, visual_fixture_evidence.md,
           review_only_fast_pass.md, post_merge.md
loaded documents: pr_review_workflow.md, pr_review/README.md,
                  collaborator_external_pr.md, intake_and_review.md,
                  local_validation.md, visual_fixture_evidence.md,
                  review_only_fast_pass.md
code candidate: a9c29700698a0c916462fedbe5025cf3cbd16e76
current head: afc83ff72186c49894081b5786ad4ef7aa60d06f
```
