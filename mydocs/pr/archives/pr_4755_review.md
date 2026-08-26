# PR #4755 검토 기록 - LayoutFrame 기반 LineSeg 재조판

## 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#4755](https://github.com/edwardkim/rhwp/pull/4755) |
| 제목 | `fix(layout): publish picture-band body edits atomically` |
| 작성자 | `humdrum00001010` (external contributor) |
| base | `devel@fbca0aa6c22db9a30e6c417190ae4ddfe924773e` |
| 검증한 code candidate | `d534f73851e8696e9bf7d2d80682c332e09d851a` |
| 후속 문서 head | `399e8065ddd36f1a39cac971dcd505d160fb87e0` |
| 규모 | code candidate 기준 24 files, +5,038/-321 |
| 검토 방식 | collaborator가 `maintainerCanModify=true`인 외부 PR source에 review-only 기록을 trailing commit으로 추가한다. |

## 라우팅과 source 고정

```text
base route: collaborator_external_pr.md
modifiers: intake_and_review.md, local_validation.md, visual_fixture_evidence.md,
           multi_pr_update_branch.md, review_only_fast_pass.md
current code candidate: d534f73851e8696e9bf7d2d80682c332e09d851a
source before reviewer record: 399e8065ddd36f1a39cac971dcd505d160fb87e0
visibility branch: review/humdrum00001010-20260814
```

`399e8065`는 `mydocs/plans/task_m100_3211_layout_frame_correctness.md`만 추가한 문서 전용
commit이다. 따라서 구현 검증은 그 직전 code candidate `d534f738`에 고정했고, 마지막 문서 추가는
코드 재검증 범위에서 제외했다. `upstream/devel`이 source의 조상임을 확인했고,
`git diff --check upstream/devel...HEAD`와 current-base `git merge-tree --write-tree`도 통과했다.

## 관련 이슈와 변경 범위

- [#3211](https://github.com/edwardkim/rhwp/issues/3211)의 그림 회피 영역, 표 셀, 일반 문단이 단일 폭
  LineSeg만으로 재조판되던 제약을 `LayoutFrame`의 물리 행·가로 구간 모델로 바꾼다.
- [#4279](https://github.com/edwardkim/rhwp/issues/4279)의 그림 band 본문 편집은 shadow 상태에서
  계산한 뒤 완성 band만 공개하도록 원자적으로 게시한다.
- [#4315](https://github.com/edwardkim/rhwp/pull/4315)의 선행 검증은 이 구현 PR로 대체한다.
- 글꼴 shaping·kerning에 따른 작은 줄 경계 차이는 [#4439](https://github.com/edwardkim/rhwp/issues/4439)의
  별도 범위로 남는다.

`layout_frame.rs`에 물리 행과 구간 carve/commit 계약을 두고, resumable line filling, 표 소유 폭,
그림 band 편집 게시을 연결했다. RenderTree는 완성된 LineSeg만 소비하며 미확정 Frame 상태를 소유하지
않는다.

## 완료한 로컬 검증

모든 Cargo 명령은 Windows PowerShell에서 `target/pr-review`를 순차 재사용했고 incremental 관련 환경
변수는 지정하지 않았다.

| 검증 | 결과 |
| --- | --- |
| `cargo test --profile release-test --target-dir target/pr-review --lib layout_frame` | 9/9 통과 |
| `cargo test --profile release-test --target-dir target/pr-review --lib real_p325_picture_band_matches_the_stored_seven_paragraph_geometry` | 1/1 통과 |
| `cargo test --profile release-test --target-dir target/pr-review --lib frame_reflow` | 10/10 통과 |
| `cargo test --profile release-test --target-dir target/pr-review --lib paragraph_frame_owner_width` | 2/2 통과 |
| `git diff --check upstream/devel...d534f738` | 통과 |
| 현재 base merge-tree | clean tree `c4ec08a707767465a072225d6aaac627dc5927c2` 생성 |

## 렌더 검증과 GitHub CI

이번 변경은 줄 재조판과 picture band 게시에 직접 영향을 주므로 renderer 검증을 적용했다. 실제
`samples/3-09월_교육_통합_2022.hwp`의 325쪽 picture band에서 저장된 일곱 문단 geometry를 대조하는
focused test가 통과했다. 새 golden 또는 PDF를 만들지 않았으므로 HWP 2020 변환은 수행하지 않았다.

동일 PR identity와 source repository에서 실행한 code candidate `d534f738`의
[CI](https://github.com/edwardkim/rhwp/actions/runs/31778690694),
[CodeQL](https://github.com/edwardkim/rhwp/actions/runs/31778690524),
[Render Diff](https://github.com/edwardkim/rhwp/actions/runs/31778690520)는 모두 성공했다.
`399e8065`의 문서 전용 후속 head에서도 CI·CodeQL·Render Diff의 aggregate와 preflight는 성공했고
heavy worker의 skipped는 review-only fast-pass의 정상 결과다.

`Cancel stale PR runs`의 `pull_request_target` 실행은 PR 조회 API가 404를 반환해 실패했다. 이는
이 PR의 코드나 review-only 경로와 무관한 workflow 정리 작업이며, 최신 Build & Test aggregate의
성공 여부와 별도로 관찰한다. 최신 reviewer 기록 head에서도 required aggregate가 실패하거나 pending이면
merge하지 않는다.

## 권고

로컬 검토에서 차단 결함은 발견하지 못했다. 이 review와 오늘할일만 추가한 trailing commit의
review-only fast-pass aggregate가 성공하고, 최신 head가 mergeable이며, 작업지시자 승인을 반영한 뒤
external contributor PR로 승인·squash merge한다. [#3211](https://github.com/edwardkim/rhwp/issues/3211)과
[#4279](https://github.com/edwardkim/rhwp/issues/4279)는 PR 본문이 `Refs`로만 연결하므로 merge 뒤에도
종료 상태를 자동으로 단정하지 않고 후속 범위를 확인한다.
