---
kind: report
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-05
---

# PR #7509 리뷰 — 인라인 개체 뒤 삽입·논리 범위 복사

## 최종 판정

**승인.** 검토한 범위에서 새 결함이나 통합 차단 사유를 발견하지 않았다. 각주·미주·인라인 도형 한 개가 있는 문단의 삽입·복사 경계를 직접 확인했다.

이 판정은 로컬 review 결과다. 작업지시자는 이 기록을 원 PR head에 직접 trailing commit으로 push하도록 승인했다. GitHub approve/comment/merge는 별도 수행하지 않았다. Merge 전에는 작업지시자의 승인과 최신 head·CI·mergeability 재확인이 필요하다. #7444 전체 해결로 표현하거나 이슈를 닫지 않는다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| PR / 작성자 / base | [#7509](https://github.com/edwardkim/rhwp/pull/7509) / semanticist21 / devel |
| 원 head = 검토 code candidate | `2487de3bdbdd897cc7f612e8be78ae917f91e48b` |
| 원 커밋 | `435f04507ba1cc5b43a12c39d73158ea9f01a872`, `2487de3bdbdd897cc7f612e8be78ae917f91e48b` |
| 로컬 branch | `review/pr7509-publish-20261005` — 원 head 뒤 승인된 current-base bridge + 문서 trailing |
| 최신 devel | `ea5ef5f6a8e405f5109407e46acb4e9a0455c24a` |
| current-base merge simulation | `git merge-tree --write-tree upstream/devel upstream/pr7509-head` 성공; tree `f59ab64d988c5271098375e88318533198aed4ab` |
| 규모 | 6개 파일, 삽입·복사 구현과 회귀/무효화 보호 검사 |
| Reviewer | jangster77 지정 확인 |
| 작성 시점 상태 | OPEN / draft false / MERGEABLE / CLEAN; 실행 대기·실패 check 없음 |
| 관련 이슈 | [#7444](https://github.com/edwardkim/rhwp/issues/7444), 부분 해결이므로 Refs 유지 |
| 선택 절차 | collaborator_external_pr + intake_and_review + local_validation + review_only_fast_pass; 기존 contributor, 단일 PR |

## 변경과 실제 호출 경로

- `insertTextLogical → logical_to_text_offset → insert_text_at_caret_native → Paragraph::insert_text_at_caret`: 글자 위치와 개체 뒤 여부를 함께 전달한다. 앞 입력은 기존 규칙을 유지하고, 뒤 입력은 해당 제어문자 슬롯 다음에 새 글자 오프셋을 둔다. 기존 native 삽입 래퍼는 `false`로 위임하며, 후속 무효화·reflow·recompose 경로를 재사용한다.
- `copySelectionLogical → copy_selection_logical_native → copy_selection_range → clip_paragraph_caret_range_for_clipboard → Paragraph::split_at`: 캐럿 축을 글자 위치/개체 앞뒤 경계로 바꾸고, 실제 split은 movable-control 축으로 변환한다. 시작이 개체 뒤면 그 개체를 제외하고, 끝이 개체 뒤면 포함한다.
- 문단 논리 끝에서는 후행 non-TAC 도형도 기존 전체 복사 계약대로 포함한다. 기존 `copySelection` 및 셀/머리말 복사 래퍼는 기존 글자 축 경계를 전달한다.
- `copy_selection_logical_native`는 문서 본문을 바꾸지 않고 클립보드만 갱신한다. `insert_text_native`의 면제는 새 삽입 함수로 위임하기 위한 것으로, 본문 무효화 보호를 제거하지 않는다.

## 검증 입력과 결과

HWP/HWPX/PDF 파일은 사용하지 않았다. `createEmpty`와 공개 native 개체 삽입 API로 만든 합성 문단이며, 독립 기대값은 문자·개체를 각각 한 칸으로 보는 토큰 순서와 반열린 선택 범위에서 정했다. 한컴 출력 일치 증거로 해석하지 않는다.

모든 로컬 Cargo 실행은 Linux Ubuntu 서버의 공유 `target/pr-review`에서 순차 수행했다.

| 검사 | 결과 / 증거 |
| --- | --- |
| 원 PR focused 회귀 | `node scripts/run-rust-test.mjs issue_7444_caret_axis_after_inline_control -- --cargo-profile release-test --target-dir target/pr-review`: **3 PASS** |
| Reviewer 추가 삽입 경계 | 각주·미주·TAC 도형 × `ABCD` / `가😀나` / 빈 본문 × 개체의 모든 글자 위치 × 앞/뒤 입력: **60개 경계 PASS**. `😀X` 삽입 후 실제 본문과 개체의 글자 위치 검사 |
| Reviewer 추가 복사 경계 | 같은 입력에서 `0 ≤ start ≤ end ≤ logical length` 모든 범위: **504개 범위 PASS**. JSON의 복사 텍스트, 실제 붙여넣은 텍스트, 개체 포함/제외 수량 검사. 개체만 선택·빈 선택·문단 시작/끝·UTF-16 surrogate 입력 포함 |
| 추가 probe 포함 실행 | 원 테스트 3개 + reviewer 테스트 2개: **5 PASS**, 최종 exit 0. Probe를 제거한 뒤 원 test source와 원 head의 일치 확인 |
| 무효화 보호 검사 | `node scripts/run-rust-test.mjs issue_2724_passthrough_invalidation_guard -- --cargo-profile release-test --target-dir target/pr-review`: **5 PASS**, wrapper 위임·stale 면제·분류 drift·ledger 검사 포함 |
| `git diff --check` | PASS |
| 문서 링크 | `python3 scripts/check_markdown_links.py mydocs/pr/archives/pr_7509_review.md`: **PASS**, 문서 1개 내부 링크 이상 없음 |

증거는 로컬 `output/pr-review/pr7509/`에 보존한다: `logs/focused.log`, `logs/probes.log`, `logs/guard.log`, `reviewer-probes.rs`. Probe는 검토용 임시 검사이며 PR source에 추가하지 않았다. 최초 probe 실행의 suite 재준비 누락과 인자 타입 오류는 검토자 실행 오류였고, 수정·재준비 뒤 5개 테스트가 실제 실행되어 통과했다. PR 결함 재현으로 세지 않는다.

### 정확한 code head의 CI 재사용

[Full CI 36962561741](https://github.com/edwardkim/rhwp/actions/runs/36962561741)의 `headSha`가 위 code candidate와 정확히 일치하며 conclusion은 success다. Lint의 native/WASM/workspace 검사, Native Skia, archive A–D 빌드·회귀 및 Build & Test 성공을 확인했다. [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/36962561788), [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/36962561559), [adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/36962561775), [Proptest](https://github.com/edwardkim/rhwp/actions/runs/36962561761)도 성공이다.

정본 3.2.2의 네 조건을 확인했다: 정확한 녹색 head, source 보정 없음(임시 probe 원복), 최신 base merge simulation 성공 및 diff 검사, renderer/layout 수정 비해당. 따라서 같은 head의 로컬 전체 nextest·Native Skia·lint 묶음을 반복하지 않았다. WASM 배포 산출물/브라우저를 reviewer가 새로 실행하지 않았으며, contributor의 Node fresh WASM 결과는 PR 본문 제공 증거로만 구분한다.

## 조판·시각 증적 판정

**비해당:** renderer/layout/pagination/paint backend와 저장 LineSeg 수용 조건 변경이 없다. 변경 대상은 편집 API가 원래 수행하던 삽입에서 제어문자 앞뒤를 보존하는 동작과 내부 클립보드 범위다. 기존 reflow·recompose 호출을 유지한다. 이번 판정은 개체/문자의 모델 순서와 범위 계약에 한정되며 페이지 좌표·한컴 외관 개선을 주장하지 않는다. Visual Sweep과 PDF/PNG asset은 새로 만들지 않았다.

## 발견 사항과 남은 범위

- 검토한 원 head의 범위 안에서 실행으로 확인한 새 회귀나 코드상 blocker는 없다.
- **범위 밖 / 미검증:** Studio 연결층, 선택 삭제·음영·글자 모양, 셀 문단의 새 논리 API, 같은 글자 위치의 여러 개체, 그림·표·수식 및 field 혼재의 직접 추가 검증. CI 전체 회귀의 성공을 이 경계들의 직접 확인으로 바꾸지 않는다.
- PR 본문이 밝힌 기존 문제: 개체 바로 앞에서 복사해 붙일 때 각주가 뒤로 이동하는 split 오프셋 문제. 추가 검사는 포함/제외 수량을 검증했고 복사된 개체의 정확한 붙여넣기 위치를 보장하지 않는다. 해당 기존 문제의 전후 재현은 reviewer가 별도로 실행하지 않았다.
- 역방향 `copySelection(0,0,2,0,0)`의 기존 후행 각주 포함 동작 변화는 작성자가 공개했다. Studio는 정렬된 범위를 전달하며, 이번 직접 검사는 정렬된 범위만 대상으로 했다.
- #7444는 위 남은 기능을 위해 OPEN으로 유지한다. 원 PR을 해결 범위 이상으로 확대해 설명하지 않는다.

## Merge 후 contributor PR comment 계획

이번 검토 기록은 원 PR head에 직접 후행 문서 commit으로 반영한다. 향후 merge 승인 시 최신 exact head CI를 재확인하고 merge SHA를 고정한다. 후속 comment에는 삽입·논리 범위 복사에 한정한 수용 범위, 검증 결과, #7444의 잔여 범위를 한국어 존댓말로 안내한다. 시각 판정이 없으므로 이미지 asset 게시 계획은 비해당이다. Comment 게시에는 UTF-8 실제 개행의 `--body-file`과 API 재조회를 사용한다.

## 후행 기록 — 2026-10-05

작업지시자의 요청에 따라 검토 기록을 archive로 옮기고 오늘할일과 함께 single-parent trailing 문서 commit으로 기록했다. Archive 이동은 PR merge 또는 이슈 종료를 뜻하지 않는다. 검증 code head와 원 contributor source를 보존하고, 이 archive review와 오늘할일만 원 PR head 위에 직접 후행 commit으로 push한다. GitHub approve·merge는 이번 지시에 포함하지 않는다.

## 원 PR 직접 반영 — 2026-10-05

원 code head에 오늘할일을 새로 만들면 현재 base의 같은 파일과 add/add 충돌이 발생했다. 작업지시자가 current-base bridge 뒤 문서 trailing을 원 PR에 직접 push하도록 승인했다. 현재 base `1f25503cc45d13c1b3cd44514e373af8888f419c`를 두 번째 parent로 갖는 bridge `a167009bbb219ad5b5381343fce39b24735433be`는 자동 병합으로 충돌 없이 생성했고 수동 source/test/workflow 보정은 없다. 앞 절의 로컬 검증은 원 code head의 결과이며 새 base 통합 자체의 전체 회귀 결과로 승격하지 않는다. 새 PR head CI/fast-pass 및 mergeability를 별도로 확인해야 한다. 이 뒤 trailing commit은 이 archive review와 오늘할일만 포함한다.

Push 전 최종 merge simulation은 충돌 없이 통과했고, 실제 merge tree의 두 변경 문서 상대 링크와 최신 base의 오늘할일 기존 기록 보존을 확인했다. 원 head부터의 변경 파일에서 LFS 추적 대상은 0개이며 `git lfs status`도 신규 push object를 표시하지 않았다. 일반 dry-run의 LFS lock 권한 오류는 Git ref 권한과 분리해 판독했고, `GIT_LFS_SKIP_PUSH=1` dry-run으로 원 contributor branch의 fast-forward 갱신 가능성을 확인했다. 다른 pre-push hook은 유지한다. 실제 push 뒤의 원격 SHA·CI 상태는 별도 확인한다.

## Merge 후 확정 기록 — 2026-10-05

- 원 PR #7509는 final head `e4af1bd2c2ba3201b22ed05cd59a6e9164576285`를 고정하여 정상 merge 방식으로 병합했다. Merge SHA: `fc36d71452a1aedae5f1f2b05737efe60a00f37f`. 최신 upstream/devel 포함을 확인했다.
- [후행 CI 37273295997](https://github.com/edwardkim/rhwp/actions/runs/37273295997)는 success다. preflight는 candidate `2487de3bdbdd897cc7f612e8be78ae917f91e48b`의 green Build & Test와 `current-base-merge-tree-match`를 확인하고 fast-pass를 수용했다. Heavy worker skip과 final Build & Test success를 확인했다. CodeQL·Render Diff·Adapter·Proptest 최신 head workflow도 success다. 이것을 새 전체 회귀 실행으로 해석하지 않는다.
- 문서 처리: archive review와 오늘할일은 원 PR에 이미 포함됐다. 이번 확정 SHA/CI 결과만 별도 문서 후속 기록 PR으로 반영한다. source/test/workflow/asset 수정은 없다.
- #7444는 부분 해결이므로 OPEN 유지 대상이다. 원 PR 및 이슈에는 실제 merge SHA·검증 요약·잔여 범위 안내를 남긴다. Contributor fork branch와 공유 `target/pr-review`는 보존하고 이번 작업의 local branch/ref/log만 정리한다.
