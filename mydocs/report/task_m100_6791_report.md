# 최종 결과 — Task M100 #6791 공개 기여 검증 안내

- Issue: [#6791](https://github.com/edwardkim/rhwp/issues/6791)
- 브랜치: `codex/6791-contributing-validation`
- 담당자: `postmelee`, 마일스톤: `v1.0.0`
- 문서 검증 SHA: `b5a33bfe3b85b9b0f4ebe119a1d1addeb8c1e43b`
- PR: [#6810](https://github.com/edwardkim/rhwp/pull/6810) — Open, base `devel`
- 상태: 사용자 승인 후 원격 push·PR 생성 완료. 최신 CI·병합 승인 대기

## 결과와 출처

첫 외부 기여자 baba9811의 [PR #6786](https://github.com/edwardkim/rhwp/pull/6786) 본문
「외부 기여자 검증 절차 확인」이 제기한 문제를 별도 이슈로 처리했다. 공개 문서에서 모든 PR의 Rust lint를
요구하는 표현과 source checkout의 suite 생성 금지 안내가 연결되지 않아, Studio만 바꾼 기여자가
generated suite 누락 fmt 실패를 어떻게 처리해야 하는지 판단하기 어려웠다.

두 공개 파일을 수정했다.

- [CONTRIBUTING.md](../../CONTRIBUTING.md): Rust·Studio 단독·혼합 등의 검증 범위, fresh WASM 준비,
  원본 commit → 별도 review worktree → prepare·fmt·lint·해당 회귀 → manifest → 동일 SHA 제출 순서를 명시했다.
  suite 누락과 실제 포맷 위반, 생성과 커밋을 구분하고 포맷 보정 후 원본 새 commit을 재검증하도록 했다.
- [PR 템플릿](../../.github/pull_request_template.md): 변경 범위·검증 SHA·실행 결과·해당 없음 사유를 기록하고
  공개 절차의 검증을 선택하도록 정렬했다. 실패·미실행을 PASS로 쓰지 않는 원칙을 유지했다.

Rust lint·회귀 정책과 CI·생성기·Cargo·제품 코드는 바꾸지 않았다. 회귀 안내에 남아 있던 직접
`cargo test --test svg_snapshot` 예제는 현재 generated suite 구성을 해석하는 기존 wrapper로 정정했다.
추가 공개 파일 변경은 없으며, 내부 운영 문서를 외부 기여자가 해석해야 하는 의무도 만들지 않았다.

## 하이퍼 워터폴 진행 기록

중복 검색 후 #6791을 실제 등록하고 assignee·마일스톤을 설정했다. 최신 upstream/devel에서 이 세션의
작업 branch를 준비한 뒤 수행계획 → 구현계획 → Stage 1 → Stage 2 → Stage 3의 승인과 commit을 순서대로
남겼다. 승인 요청 전에 각 단계의 로컬 산출물과 해당 검증을 완료했다.

| 단계 | 구현·검증 기록 |
| --- | --- |
| 수행계획 | [수행계획](../plans/task_m100_6791.md), 최초 commit `b7f931acf`, 승인 기록 `a2c00f16d` |
| 구현계획 | [구현계획](../plans/task_m100_6791_impl.md), 최초 commit `0f76777df`, 승인 기록 `16e2ea171` |
| Stage 1 | [범위·frontend 정렬](../working/task_m100_6791_stage1.md), `eb046527a` |
| Stage 2 | [worktree·제출·템플릿 정렬](../working/task_m100_6791_stage2.md), 승인 `c210cb7e7`, 구현 `b5a33bfe3` |
| Stage 3 | [공개 명령 실검증](../working/task_m100_6791_stage3.md), 승인 `16c41bc87`, 결과는 이 보고서와 함께 commit |

PR 채번 전에는 review·오늘할일을 만들지 않았고, 사용자 원격 승인 후 #6810을 생성해
[self-review](../pr/archives/pr_6810_review.md)와 [오늘할일](../orders/20260906.md)을 후속 기록했다.
#6786의 기능 리뷰·head·본문·comment·merge는 별도 작업으로 유지했다.

## 검증과 제한

새 clean source worktree에서 공개 bash 블록을 변경 없이 실행해 review worktree를 생성했다.
prepare → `cargo fmt --all -- --check` → manifest `--check`와 SHA·clean 상태 확인이 모두 통과했다.
28 suites + 20 exceptions, 48/48 integration targets를 확인했다. tracked Rust 원본·root Cargo.toml·Cargo.lock
2,206개 파일의 hash는 source와 review 모두 변하지 않았고, harness 28개·manifest는 review의 ignored
파일로만 남았다. 원시 로그 위치와 재현 방법은 Stage 3 보고서에 있다.

문서 링크·공백 검사도 통과했다. 내부·교차 anchor 21개와 Rust 절 bash 구문 10개는 Stage 2에서 확인했고
이후 공개 내용은 바뀌지 않았다. 최종 보고 기록은 commit 전에 링크 검사한다.

현재 upstream/devel `016fe3ceed904633e74e70127a4cceaa1f18a756`과의 merge-tree 검사는 충돌 없이 통과했다.
통합 결과의 공개 두 파일은 검증 SHA와 동일했다. 이후 base·head가 바뀌면 원격 조치 전에 다시 확인한다.

문서만 바꾸므로 전체 Rust build·Clippy·nextest·Native Skia·Studio build·브라우저 테스트를 반복하지 않았다.
문서에서 그 검사를 안내하는 것과 실제 실행한 검증을 구분한다. PR 생성으로 GitHub CI가 시작됐으며,
현재 classifier v7은 `.github/pull_request_template.md` 때문에 이 PR을 전체 CI로 분류한다.

## 게시한 PR 본문

제목: `docs: 기여 검증 범위와 Rust worktree 준비 순서 명확화 (#6791)`

아래 본문을 UTF-8 파일 `/private/tmp/rhwp-6791-pr-body.md`로 준비해 게시했다. 이 임시 파일이 없어지면 아래
본문을 다시 저장해 `--body-file`로 전달한다.

```markdown
## 변경 요약

Studio만 변경한 기여자가 Rust 전체 검증 요구와 source checkout의 suite 생성 금지 안내를 함께 해석해야 했던 공백을 보완합니다. CONTRIBUTING과 PR 템플릿에 변경 범위별 검증을 명시하고, Rust 검증은 원본 commit → 별도 worktree → suite 준비·fmt·lint·해당 회귀 → manifest 확인 → 같은 SHA 제출 순서로 연결합니다.

기여자도 자신의 검증 worktree에서 suite를 생성할 수 있으며, generated harness·manifest는 계속 PR에 포함하지 않습니다. 포맷 보정은 source branch의 새 commit에 반영해 다시 검증하도록 했습니다. CI·생성기·Cargo·제품 코드의 변경은 없습니다.

## 관련 이슈와 출처

Refs #6791. 문제 제기의 출처는 baba9811의 PR #6786 본문 「외부 기여자 검증 절차 확인」입니다. 기존 #5571/#5682 및 PR #6393의 제출·생성·Rust lint 계약을 실행 가능한 안내로 연결합니다. #6786의 기능 변경이나 리뷰는 이 PR 범위에 포함하지 않습니다.

## 검증

- 검증 commit: b5a33bfe3b85b9b0f4ebe119a1d1addeb8c1e43b. 후속 작업은 계획·결과 기록과 최신 devel 동기화이며, 공개 문서 두 파일은 검증한 내용과 동일합니다.
- 새 clean source에서 공개 명령을 그대로 실행: 별도 review worktree 생성, suite prepare, cargo fmt --all -- --check, manifest --check, SHA·clean 상태 확인 통과.
- Rust 원본·Cargo 파일 2,206개 hash 불변. harness 28개·manifest는 review worktree의 ignored 파일이며 제출 diff에 포함하지 않았습니다.
- 변경 문서 상대 링크·공백 검사, 내부·교차 anchor 21개, Rust 절 bash 구문 10개 확인.
- 현재 devel과 merge-tree 충돌 없음. 통합 결과의 공개 두 파일은 검증한 내용과 동일합니다.

문서 절차 검증으로 전체 Rust build·Clippy·nextest·Native Skia·Studio build는 반복하지 않았습니다. 현행 classifier는 PR 템플릿 변경을 전체 CI로 분류하며, 최신 GitHub required checks는 병합 전에 확인합니다.

## 성능 영향

문서만 변경하므로 제품 실행 성능 영향은 없습니다. 성능 측정은 하지 않았습니다.
```

## 승인 후 실행한 원격 명령

현재 remote는 `upstream=edwardkim/rhwp`, `origin=postmelee/rhwp`다. collaborator self 절차에 따라 upstream의
작업 branch로 제출한다. 중복 PR·원격 branch가 없음을 확인한 뒤 아래 명령을 실행했다. 먼저 최신 devel을 충돌 없이
병합한 `15f5a5574f1e9f989fdead277ccf2345e604e331`을 제출했으며, 공개 두 파일은 검증 SHA와 동일했다.

```bash
git status --short
git push upstream HEAD:refs/heads/task_m100_6791
gh pr create --repo edwardkim/rhwp --base devel --head task_m100_6791 \
  --title 'docs: 기여 검증 범위와 Rust worktree 준비 순서 명확화 (#6791)' \
  --body-file /private/tmp/rhwp-6791-pr-body.md --milestone v1.0.0 --assignee @me
```

2026-09-06 사용자 “진행해줘”로 원격 push·PR 생성을 승인받아 Open PR #6810을 생성했다.
API 재조회로 게시 본문 일치·한글 보존·담당자 `postmelee`·마일스톤 `v1.0.0`을 확인했다.
archive self-review와 기존 내용을 보존한 오늘할일을 같은 branch의 후속 commit으로 기록한다. 최신 CI·mergeability를 확인하고 merge·이슈 close는 후속 승인으로 처리한다.
이슈를 자동으로 닫는 문구 대신 `Refs #6791`을 써 이슈 close 승인도 분리했다.

## 남은 상태

로컬 작업·검증·Open PR 생성은 완료했다. 남은 단계는 기록 commit을 포함한 최신 GitHub CI 확인과
별도 merge·이슈 close 승인이다. 계획·검증용 worktree와 로그는 승인·통합 후 정리할 수
있도록 보존하며 다른 작업의 경로·target에는 변경을 가하지 않았다.
