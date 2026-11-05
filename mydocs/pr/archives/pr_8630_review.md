# PR #8630 리뷰 — 묶음 글상자 클릭·Esc 복귀와 CI 디스크 관측

## 최종 판정

**승인 — #8578의 편집 오류를 보정하고 최종 head의 필수 CI를 확인한 뒤 병합했다.**
검토 head는 `1be2ece76965d52de6666aa7b99bc7cb1969f225`, 병합 커밋은
`5f277edb31e217d0b9f320ee74c7838997ba5442`다. 2026-11-05 16:42:44 KST에
정상 squash merge했으며 관리자 우회를 사용하지 않았다. 병합 직전 `MERGEABLE / CLEAN`,
완료 성공 check 35개·정상 skipped 6개를 확인했다.

원 PR에는 CI 변경이 포함되므로 [옵션 2](../../manual/pr_review/post_merge.md#711-ci-변경-pr은-옵션-2로-후속-기록)에
따라 이 review와 [오늘할일](../../orders/20261105_pr8630.md)을 병합 후 별도 기록 PR로 반영한다.
원 PR의 문서-only trailing commit으로 전체 CI를 반복하지 않는다.

## 접수 정보와 검토 범위

| 항목 | 확인한 값 |
| --- | --- |
| 원 PR·작성자 | [#8630](https://github.com/edwardkim/rhwp/pull/8630), `jangster77` |
| 관련 이슈 | [#8578](https://github.com/edwardkim/rhwp/issues/8578), 편람 3·4쪽의 클릭·Esc 편집 전환 |
| 기준 devel | `6e1de406209ba9a58032f6ca6de45b18aa687d1c` |
| 실제 컴파일·로컬 검증 producer | `2c0432cc0646cf2ad0b277b50e4c685ab7092a80` |
| 최종 PR head | `1be2ece76965d52de6666aa7b99bc7cb1969f225` |
| 병합 SHA | `5f277edb31e217d0b9f320ee74c7838997ba5442` |
| 리뷰어 | 별도 지정 없음. collaborator self PR의 검토·사용자 승인 경로 적용 |

원본 Group의 깊이 우선 자식 순서에서 실제 글상자만 독립적인 편집 주소를 소비하도록 했다.
조회·커서·입력·clipboard·중첩 표·inline control·hyperlink가 같은 indexed helper를 사용한다.
장식 배경은 본문 클릭 후보에서 제외하고, 실제 빈 글상자는 주소를 유지한다.
Esc의 원 소유 본문 복귀와 삭제·붙여넣기의 인접 문단 착지를 분리했다.
빈 소유 본문의 캐럿은 확정된 본문 흐름 원점을 소비하며 실제 공백은 보존한다.

문서 ID·쪽 번호별 예외, 묶음 자동 해제, 경고 숨김으로 우회하지 않았다.
기존 #919의 바깥 글상자 소유와 #1892의 공백 보존 계약은 원 해결·후속 보정을 추적한 뒤
기대값을 유지하고 구현에서 복구했다. [원인·소비 경로와 조판 원칙 검토](../../report/task_m100_8578_report.md)를
참조한다. 이번 범위는 편집 소유와 기존 출력 보존이며 다른 문서의 조판 보정으로 확대하지 않는다.

## 검증 입력과 결과

- 원본: `samples/2025 행정업무운영 편람(최종).hwpx`, SHA-256
  `c6dd7e847a99f219681afc5a29c80a9665c04df9cda4d820a3350d739664fdf6`.
- 독립 기준: `pdf/2025 행정업무운영 편람(최종)-hwpx-kopub-2024.pdf`, SHA-256
  `299cb2d994bba8b63f78b681bd74580286e298f25689b4ff4f40c66c54bfa85e`.
  기존 #8389의 한컴 Print 383쪽 자료를 사용했으며 새 MCP 출력·원본·글꼴·문턱 변경은 없다.
- 두 입력과 최종 증적은 병합 커밋에 포함됐다. [producer](../assets/issue_8578/20261105/producer.json)의
  소스 23개 해시가 최종 head와 일치함을 확인했다.
- 재리베이스 뒤 새 source에서 전체 회귀·Skia·실제 편집·해당 편람 출력을 다시 검증했다.
  이후 두 CI 보완 커밋은 편집 구현·입력·출력을 바꾸지 않는다.

| 실행 묶음 | 실제 결과 |
| --- | --- |
| 최소 글꼴 Linux 전체 nextest | 12,591 PASS / 0 FAIL / 45 skip, 936.659초, exit 0 |
| 로컬 Native Skia lib | 본체 3,858 PASS·12 ignore, 다른 libs 182 PASS |
| 로컬 Skia PNG·PDF | 2 PASS / 22 PASS |
| 집중 관계 검사 | 새 #8578 8건 + 기존 #1892·#919 각 5건 = 18 PASS, Mac·Linux 모두 성공 |
| Studio 단위 검사 | 2,721 PASS / 0 FAIL / 2 skip |
| 로컬 게이트 | fmt·Native/WASM/all-target Clippy·workspace build·manifest·unit tiers·E2E manifest·TypeScript 성공 |
| 실제 Canvas2D | 383쪽 열기, 3쪽 입력·선택·드래그·삭제·Enter·undo/redo·저장/reopen, 4쪽 본문 입력, 개체 선택→Esc→원 본문 입력 성공 |
| 사용자 확인 | 사용자가 정상 동작을 검증했다고 확인. 사용자 프로그램 해시는 별도로 단정하지 않음 |
| CI 기본 기능 회귀 | A 3,801 / B 2,301 / C 2,056 / D 1,804 / E 2,435, 합계 12,397 PASS / 0 FAIL / 45 skip |
| CI 나머지 게이트 | Lint·Native Skia·Frontend package·Build & Test·CodeQL·Render Diff·Adapter·Proptest·CI Impact Policy 성공 |

로컬과 CI의 실행 묶음을 합산하거나 같은 테스트 수로 표시하지 않는다.
최종 [CI](https://github.com/edwardkim/rhwp/actions/runs/41985496879),
[CodeQL](https://github.com/edwardkim/rhwp/actions/runs/41985496858),
[Render Diff](https://github.com/edwardkim/rhwp/actions/runs/41985496736),
[Adapter](https://github.com/edwardkim/rhwp/actions/runs/41985496885),
[Proptest](https://github.com/edwardkim/rhwp/actions/runs/41985496820)를 확인했다.

## 시각 증적과 실제 소비 경로

변경 쪽인 3·4쪽은 Native와 fresh WASM 모두 각각 100.00%이며 base/후보 PNG 해시와 TSV가 동일하다.
별도 Native 편람 gate 383쪽의 최저는 130쪽 99.96%로 미달·누락·기준 대비 하락이 없고
383쪽 PNG 해시도 동일하다. 최종 fresh WASM 전383쪽을 실행했다고 확대하지 않는다.
같은 배율에서 제목·안내문·목차·장식·줄 배치의 review/overlay를 직접 판독했고 새 차이는 없었다.

| 대상 | raw | review | overlay |
| --- | --- | --- | --- |
| Native 3쪽 | [PNG](../assets/issue_8578/20261105/native-p003-raw.png) | [review](../assets/issue_8578/20261105/native-p003-review.png) | [overlay](../assets/issue_8578/20261105/native-p003-overlay.png) |
| Native 4쪽 | [PNG](../assets/issue_8578/20261105/native-p004-raw.png) | [review](../assets/issue_8578/20261105/native-p004-review.png) | [overlay](../assets/issue_8578/20261105/native-p004-overlay.png) |
| fresh WASM 3쪽 | [PNG](../assets/issue_8578/20261105/wasm-p003-raw.png) | [review](../assets/issue_8578/20261105/wasm-p003-review.png) | [overlay](../assets/issue_8578/20261105/wasm-p003-overlay.png) |
| fresh WASM 4쪽 | [PNG](../assets/issue_8578/20261105/wasm-p004-raw.png) | [review](../assets/issue_8578/20261105/wasm-p004-review.png) | [overlay](../assets/issue_8578/20261105/wasm-p004-overlay.png) |

[Native base 383쪽](../assets/issue_8578/20261105/native-base.tsv) /
[후보](../assets/issue_8578/20261105/native-head.tsv),
[fresh WASM base 3·4쪽](../assets/issue_8578/20261105/wasm-base.tsv) /
[후보](../assets/issue_8578/20261105/wasm-head.tsv),
[Studio 3쪽 실제 편집](../assets/issue_8578/20261105/studio-page003-edit.png) /
[4쪽 본문](../assets/issue_8578/20261105/studio-page004-body.png)를 연결한다.

Mac WASM은 locked wrapper `--target web --out-dir pkg --no-opt` 로컬 대체 빌드다.
초기 공유 캐시에서 기준 프로그램을 재사용한 잘못된 후보는 제외하고 재컴파일한 프로그램으로
실제 글상자 API·브라우저를 확인했다. 최종 WASM SHA-256은
`d98be3e9a846e33ebc43a792070afe47b0ad742774c17652fdc56abaaaa48cde`이며
root pkg·Studio public·실제 초기화 응답을 대조했다. Docker 최적화 통과로 표시하지 않는다.

## CI 디스크 관측과 계약 검사 보정

사용자 요청으로 Lint의 단계 전후·실패 종료에 `df -h`를 추가했다.
기존 24개 단계의 명령·순서·캐시·워커 구성과 실패 종료 코드는 유지했다.
workspace build와 all-target Clippy 사이도 기록하며 산출물 삭제·워커 분리는 추가하지 않았다.
셸 종료 코드 보존 8개 경우, actionlint·YAML·명령 보존 대조를 확인했다.

| 최신 성공 Lint의 관측 지점 | `df -h` 여유 공간 표시값 |
| --- | --- |
| checkout 전 / 후 | 88G / 73G |
| Rust 설치 후 / 캐시 복원 후 | 72G / 72G |
| Native Clippy 전 / 후 | 71G / 71G |
| WASM Clippy 후 | 70G |
| workspace build 후 / all-target Clippy 후 | 65G / 65G |
| 최종 WASM check 후 | 65G |

[최신 Lint](https://github.com/edwardkim/rhwp/actions/runs/41985496879/job/127105350942)는
47개 관측을 남기고 성공했다. 표시값은 반올림된 용량이며 디렉터리별 정밀 사용량이 아니다.
처음 실패한 runner는 초기 용량 기록이 없어 이번 성공과의 차이를 특정 원인으로 단정하지 않는다.

`69bec30d40`에서 `run:`을 여러 줄 형식으로 바꾼 뒤 기존 #8392 계약 검사의 한 줄 문자열
assertion이 실패했다. 최종 `1be2ece769`은 기존 `cargo_commands()`로 Native/WASM의 정확한
인자를 대조하도록 보정했다. 워크플로 계약 896건이 성공했고 명령 삭제·`-D warnings` 완화·
WASM 타깃 제거는 계속 실패로 검출했다. 제품 검사·선택적 verifier 제외 범위는 유지한다.

## 병합 후 처리와 contributor PR comment 계획

병합 후 검증 CI를 재실행하지 않고 자동 duration 갱신의 결과만 확인한다.
duration 결과: [자동 갱신 run 41989153494](https://github.com/edwardkim/rhwp/actions/runs/41989153494)가
성공했고 `refreshed_targets=46`을 확인했다. 병합 후 CI·CodeQL 등 제품 검증은 재실행하지 않았다.

이 기록 PR을 완료한 뒤 최종 devel을 동기화하고 #8578의 close 상태를 확인한다.
같은 병합 SHA·증적의 댓글이 없으면 #8578과 원 PR #8630에 한국어 존댓말로 완료 사실과
CI URL, 위 로컬 결과·실제 편집 범위·남은 차이를 게시한다.
[Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)을 연결하고
병합 SHA `5f277edb31e217d0b9f320ee74c7838997ba5442`로 고정한 해당 3·4쪽 raw·review·overlay
이미지를 포함한다. 먼저 이미지 응답의 형식·해시를 확인하고 `--body-file` 게시 후 API 본문과
표시를 재확인한다. 이슈에 부모가 없음을 확인했으며 다른 이슈를 함께 닫지 않는다.

마지막에 이번 작업의 Mac 기준 worktree·Linux 검증 worktree·전용 branch를 소유와 사용 종료를
확인해 정리한다. 기본 `~/rhwp`, 무관한 audit/다른 이슈 worktree, 원본·PDF·글꼴과 공유 target의
다른 profile은 보존한다. root `output/` 전체와 `target/pr-review/debug`는 현재
[종료 정리](../../manual/pr_review/post_merge.md#77-branch-worktree-검토-전용-target-정리)에 따라
실제 사용 종료 후 삭제하고 결과는 output 밖 정리 로그에 기록한다.
