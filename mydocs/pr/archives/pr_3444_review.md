# PR #3444 검토 기록 — wasm-pack 다운로드 일시 실패 재시도

| 항목 | 내용 |
| --- | --- |
| 원 PR | [#3444](https://github.com/edwardkim/rhwp/pull/3444) — `Task #3431: [CI] wasm-pack 다운로드 curl 재시도` |
| 작성자·검토자 | `@lpaiu-cs` (external contributor) · `@jangster77` (collaborator) |
| base / source head | `devel` / `5831dce7b3bdbc1e950d28e27ec5181baa0b25c7` (`ci/3431-wasm-pack-curl-retry`) |
| 원 변경 규모 | 1 file, +3 / -1, 2 commits(action 1 + devel merge 1) |
| 통합 검토 | `review/lpaiu-cs-20260727`; `upstream/devel` `7779e737ac5c5df3428d1a06f1099be16375be49` 기준 |
| 원 변경 적용 | `fbc254af89df874d130b500945afeaff785c49cf`→`d43fdc5574e512deff13b395b5d099cb7b996ea4`; devel merge 제외 |
| collaborator 보정 | `0b58a0d4497d2154b37e797ce49b8eca79357fd2` 중 완전 archive 다운로드·cleanup 범위 |
| 관련 이슈 | [#3431](https://github.com/edwardkim/rhwp/issues/3431); 통합 PR에서 `Closes #3431` |
| 작성 시점 source 상태 | `MERGEABLE` / `CLEAN`, draft 아님; source Build & Test [성공](https://github.com/edwardkim/rhwp/actions/runs/30229744163/job/89867607608) |
| 라우팅 | base: `collaborator_external_pr`; modifiers: `intake_and_review`, `local_validation`, `multi_pr_update_branch` |

Loaded documents: `pr_review_workflow.md`, `pr_review/README.md`,
`collaborator_external_pr.md`, `intake_and_review.md`, `local_validation.md`,
`multi_pr_update_branch.md`.

## 원 변경과 발견한 위험

원 PR은 GitHub Releases의 일시 408/429/5xx와 connection refused를 `curl --retry 5`로 흡수해,
wasm-pack 다운로드 한 번의 504 때문에 전체 matrix를 재실행하는 비용을 줄인다. 재시도 자체는 이슈의 직접
원인에 맞는다.

하지만 `curl --retry ... | tar -xzf -`를 유지하면 전송이 중간에 끊긴 뒤 retry가 같은 stdout stream으로
이어지는 상황에서 tar가 앞선 불완전 gzip 조각을 이미 소비한다. curl의 재시도가 곧 완전한 archive 단위
복구라는 보장이 없어 원 변경 그대로는 충분하지 않았다.

## Collaborator 보정

`0b58a0d44`에서 다음처럼 좁혔다.

- `curl -o "$tmpdir/wasm-pack.tar.gz"`로 완전 파일을 먼저 받은 뒤 별도 `tar -xzf`를 실행한다.
- `--retry-all-errors`를 추가해 전송 중단도 archive 전체 재다운로드 대상으로 둔다.
- `trap 'rm -rf "$tmpdir"' EXIT`로 download·extract·install 어느 단계에서 실패해도 정확한 임시 directory를
  정리하고, 정상 종료 때 trap을 해제한다.
- 기존 원자적 `wasm-pack.tmp.$$`→최종 경로 rename과 self-hosted skip 계약은 유지한다.

## 검증

- Ruby YAML parse: composite action의 `runs.using == composite`와 run block 존재 확인.
- 추출한 embedded Bash: `bash -n` 통과.
- `git diff --check`: 통과.
- fresh `wasm-pack build --target web --out-dir pkg`: 통과.
- source head Build & Test는 contributor 원 commit에서 성공했다. 보정된 action의 실제 6개 CI 호출 지점은
  최신 통합 PR full CI에서 다시 확인해야 한다.
- action 변경이며 Cargo source·renderer·fixture가 아니므로 별도 Cargo focused test, visual sweep,
  IR baseline 갱신은 이 PR 단독 trigger가 아니다. 통합 후보 공통 Cargo suite는 별도로 모두 통과했다.

## Risk와 최종 권고

네트워크 재시도는 영구 404나 손상 archive를 성공으로 숨기면 안 된다. 보정은 curl 실패를 유지하고 완전 파일
수신 뒤 tar가 독립 실패하도록 해 이 경계를 지킨다. **보정 후 기술적으로 수용 가능**하다. 통합 PR 본문에는
`Closes #3431`을 사용한다. 최종 merge 조건은 최신 통합 head full CI에서 action의 모든 실제 호출과
aggregate가 성공하고, mergeable 상태와 작업지시자 승인이 확인되는 것이다.
