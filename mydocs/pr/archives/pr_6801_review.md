# PR #6801 검토 기록

## 판정: 메인터너 보정 후 수용 가능

**최신 원 커밋 포함, 보정 및 로컬 검증 완료.** 추가 원 커밋 c7dade57을 포함했고, 탭의 bbox·charX·hit-test와 점선 리더 경계를 함께 보정해 집중 5개 및 KTX 스냅샷을 통과했다.

이 판정은 아래 로컬 후보의 PR 변경 범위에 한정한다. GitHub 승인 이벤트·원 PR 직접 merge·전체 이슈 해결 또는 전체 문서 시각 동일을 뜻하지 않는다.

## 검토 기준과 적용 이력

- 검토일: 2026-09-07.
- 원 PR: [#6801](https://github.com/edwardkim/rhwp/pull/6801), 관련 이슈 [#6800](https://github.com/edwardkim/rhwp/issues/6800).
- 제목: 수정(renderer): 탭이 든 TextRun 의 bbox 폭을 다음 런 시작에서 자른다 (#6800)
- 누적 브랜치: `review/planet6897-ci-green-20260907`.
- 기준 devel: `07bc5e5490f75118f08370de19aeee73ce1667cb`.
- **실제 로컬 시험 대상: `81ec9b869daa27d8546bbbbcb29587df93144f93` 시점의 검증 작업 트리.** 메인터너 체크포인트 `f2047e1f4`와 #6796 후속 체리픽 2개, 전체 원본/축소 입력의 좌표 대조 회귀를 포함한다.
- 이번에 조회한 원격 head: `c7dade57c5f3b9d4bf2fe663613afe60b953612e`, OPEN, MERGEABLE / CLEAN.
- 로컬에 반영한 원 PR 범위: `c7dade57c5f3b9d4bf2fe663613afe60b953612e`까지. 해당 원격 head를 포함한다.
- 검토 경로: collaborator 매개 외부 PR의 출처 보존 체리픽 통합. 10개 원 PR을 함께 시험했으며 단독 before/after를 새로 실행한 것처럼 기록하지 않는다.
- 메인터너 체크포인트 `daf5744bfd4cb62d6c3e6d9b6cd104816da758ec`의 #6784/#6798/#6801 보정과 후속 작업 중 보정을 구분한다.
- 기존 reviewer 요청은 유지했다. 이번 테스트·문서 갱신에서 reviewer나 통합 PR owner review를 추가 요청하지 않았다.

| 원 커밋 | 로컬 적용 커밋 |
| --- | --- |
| `d455198325222e31bc1c85e8fbfe792e12e6f9d2` | `847982088` |
| `194c4ae5867f87f365580279c383f1c2f251f5c6` | `dbef8d6bb` |
| `b01767d7` | `b5102fd16` |
| `c7dade57c5f3b9d4bf2fe663613afe60b953612e` | `c9945466f` |

## 검토 결과와 보정 상태

- 원격 c7dade57c5f3b9d4bf2fe663613afe60b953612e를 c9945466f로 반영했다. 전체 자식 순회는 기존 메인터너 보정에 이미 있었으므로 이를 유지했고, 이 후속 적용의 순변경은 관련 설명 2줄이다.
- 다음 TextRun 탐색은 중간의 비텍스트 자식을 건너뛰되 기존 폭/적용 범위를 보호한다. 공개 문자 위치는 재생성 때도 같은 resolved tab advance를 사용하도록 했다.
- 과거 bbox 끝 249.1px와 공개 문자 끝 718.1px의 469.0px 차이는 보정 전 수치다. 현재 공개 charX 끝과 bbox, hit-test 경계 및 중간 탭 비적용 회귀가 통과했다.
- resolve_trailing_tab_end의 논리 advance 경계와 leader_limit_width의 잉크 경계를 분리했다. 다음 보이는 글자 전의 공백이 KTX 점선 리더를 5.76px 짧게 만들던 실패를 해결했다.
- 기준 KTX SVG를 변경하지 않았다. 탭 집중 5개, svg_snapshot::issue_267_ktx_toc_page, 최신 전체 9,153개 회귀가 통과했다. 실제 Studio에서 마우스로 커서를 클릭하거나 선택하는 UI 검증은 실행하지 않았다.

## 기존 코멘트 상세 대조

| 기존 댓글 | 요청 또는 주장 | 현재 후보에서의 판단 |
| --- | --- | --- |
| [5557833781](https://github.com/edwardkim/rhwp/pull/6801#issuecomment-5557833781) | 무차별 bbox 절단, 글자/장식 불일치, 대상 식별, 기준 엔진 | 탭 재배치의 bbox뿐 아니라 공개 charX와 hit-test를 같은 실제 advance로 고정하고 음성 회귀를 통과했다. |
| [5558170377](https://github.com/edwardkim/rhwp/pull/6801#issuecomment-5558170377) | 재배치 지점 보완 및 compute_char_positions 미반영 고지 | compute_char_positions 미반영은 과거 상태다. 현재 재생성 경로와 점선 리더 보호까지 보정했고 KTX golden을 그대로 둔 채 통과했다. |

댓글에 적힌 기여자 환경의 과거 실측과 이번 로컬 시험을 구분한다. 이번에 새 댓글이나 review thread를 게시하지 않았다.

## 실제 검증 결과

### 원 PR 최신 head CI

원격 `c7dade57c5f3b9d4bf2fe663613afe60b953612e`의 조회 시 종합 상태는 **SUCCESS**다. 아래 결과는 원 PR의 CI이며 작업 중 보정을 포함한 로컬 통합 후보의 원격 CI가 아니다.

| 원 PR 최신 head 검사 | 조회 결과 |
| --- | --- |
| [Canvas visual diff](https://github.com/edwardkim/rhwp/actions/runs/34112919588/job/101713081772) | SUCCESS |
| [adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/34112919840/job/101713105240) | SUCCESS |
| [Analyze (javascript-typescript)](https://github.com/edwardkim/rhwp/actions/runs/34112919893/job/101713114658) | SUCCESS |
| [prop roundtrip](https://github.com/edwardkim/rhwp/actions/runs/34112919826/job/101713083665) | SUCCESS |
| [Analyze (python)](https://github.com/edwardkim/rhwp/actions/runs/34112919893/job/101713114632) | SUCCESS |
| [Analyze (rust)](https://github.com/edwardkim/rhwp/actions/runs/34112919893/job/101713114583) | SUCCESS |
| [Build & Test](https://github.com/edwardkim/rhwp/actions/runs/34112919853/job/101716191272) | SUCCESS |
| [CodeQL](https://github.com/edwardkim/rhwp/runs/101713921850) | SUCCESS |

조회한 preflight 및 실행 worker는 완료 상태다. `cancel-stale-runs`, `WASM Build`, `Frontend unit gates`, `Workflow promotion preflight`, `Refresh nextest target duration data`의 SKIPPED는 실행 통과 건수에 넣지 않는다.

### 누적 후보의 로컬 시험

- 최신 전체 integration 회귀: **9,153 통과 / 0 실패 / 46 skip**, `--no-fail-fast`, nextest summary 276.450초, exit 0.
- 정식 fixture 10개(기존 9개와 #6796 축소본)를 `RHWP_SECURITY_SWEEP_SAMPLES_JSON`에 명시해 최신 전체 회귀의 보안 코퍼스 검사를 통과했다. 자동 변경 파일 탐지에만 의존하지 않았다.
- 이전 제품 보정 단계의 관련 집중 회귀: **50 통과 / 0 실패**, 0.245초, exit 0. 이번에 동일 필터를 재실행했다고 쓰지 않는다.
- 이번 #6796 후속 집중 회귀: **7 통과 / 0 실패**, 0.235초, exit 0. 원본 2개, 축소본 4개, #5734 보호 1개이며 필터 비선택 9,192개는 전체 회귀의 skip 46개와 구분한다.
- 이전 집중 실행에서 이 PR의 [집중 시험 원본](../../../tests/cases/issue_6800_tab_run_bbox_width.rs): **5개 통과**.
- 최종 Rust fmt, workspace all-targets clippy(`-D warnings`), suite prepare/check, `git diff --check` 통과. 이 검사는 본 문서 갱신 전 코드 후보에 대해 실행했다.
- 이전 제품 보정 단계에서 Native Skia 그림 placeholder **2개**, direct PDF export **4개**가 통과했다. 이번 #6796 후속 후보에서 native feature 실행을 다시 수행하지 않았으며, 과거 선택 0개 및 manifest 불일치를 통과로 세지 않는다.
- 앞선 동일 제품 보정 단계의 workspace 빌드·WASM clippy·Native Skia lib·Docker 없는 WASM 빌드도 exit 0이었다. 전부 이번 최종 실행에서 다시 수행했다고 쓰지 않는다.
- 정확한 실행 명령, 기준선 독립 비교, 후속 원 커밋의 통합 내역는 [통합 검증 및 시각 기록](planet6897_ci_green_20260907_visual_sweep.md)을 따른다.

이전 관련 집중 실행의 실제 통과 항목(위 최신 전체 회귀에도 포함):

- `a_mid_run_tab_with_text_after_it_is_untouched`
- `maintainer_public_character_positions_share_the_resolved_tab_end`
- `tab_run_bbox_does_not_swallow_the_next_run`
- `maintainer_hit_test_uses_the_resolved_trailing_tab_positions`
- `the_trailing_tab_run_ends_exactly_where_the_next_run_starts`

## 시각 증적과 판정 한계

대상 문서는 실제 1쪽이다. sweep의 review_202100017.png는 문서 식별자에서 유래한 파일명이며 대표 PNG를 물리 1쪽으로 정규화했다. 일부 대체 글리프가 남아 있고 픽셀 대조는 charX/hit-test의 직접 검증이 아니므로 공개 API 회귀 결과를 별도로 사용한다.

- 원본: [samples/issue6800/1192000-202100017-policy-research-report.hwp](../../../samples/issue6800/1192000-202100017-policy-research-report.hwp), 90,112 bytes.
- 원본 SHA-256: `fd0b95cb4239b08e2ab9130b6b697af56dda379f1c9029dbf5f5a0b97af5ceee`.
- [기준 PDF](../../../pdf/1192000-202100017-policy-research-report-2020.pdf): engine 2020, 1쪽, 62,122 bytes.
- 기준 PDF SHA-256: `34c9173724fa5d2ef5e0b2a796b56c9b564d3f5e0c9224609dffd507921ada48`.
- 기존에 준비된 PDF를 재사용했다. 해시·크기는 이번에 확인한 파일 값이며 옛 재변환 파일의 메타데이터를 재사용하지 않는다.
- 이 대표 PNG는 이전 메인터너 보정 단계의 `maintainer-all` sweep 산출물이며 이번에 다시 생성했다고 쓰지 않는다. 이후 #6796 후속은 음수 오프셋 조건식의 설명·변수명 정리와 별도 입력/회귀 추가로, 기존 원본의 렌더 조건을 확대하지 않았다. 새 축소본 대조는 #6796 기록에 별도로 보관한다.
- 자동 flag나 전체 배경을 포함한 pixel match를 시각 수용률로 사용하지 않는다. 실제 Studio UI 클릭 검증은 수행하지 않았다.

### 대표 증적: 물리 1쪽

![PR 6801 물리 1쪽 rhwp·기준 PDF·overlay 비교](../assets/planet6897_ci_green_20260907/pr_6801_p1.png)

## 남은 사항과 다음 단계

최신 원격 c7dade57까지 포함한 현재 후보의 탭/점선 회귀 보류 사유는 해소됐다. 다른 원 PR의 후속 반영 상태와 이 PR의 CI 결과를 혼동하지 않는다.

#6796의 후속 `9da47cb2`/`097c499f`는 전체 원본 보정을 보존한 채 통합했고, 집중 7개·전체 9,153개 회귀와 축소본 대조를 완료했다. #6804의 `15a881d0`는 동일 기준선 내용을 `f2047e1f4`에 보존한 경우로, 원 SHA의 체리픽 이력과 내용 반영을 구분한다. 시험 중 10개 원 PR을 다시 조회했으며 마지막 확인 이후 추가 head 변경은 없었다.

## PR·이슈 코멘트 게시 계획

[시각 증적 댓글 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)과 [post_merge.md](../../manual/pr_review/post_merge.md)를 따른다. 아래는 게시 계획이며 이미 게시하거나 승인받은 GitHub 조치가 아니다.

1. 최종 통합 head를 확정하고 실제 PR/merge SHA 및 devel CI가 성공한 뒤 수용 결과를 기록한다. 원 PR 직접 merge가 아니라 출처 보존 체리픽 통합 수용임을 명시한다.
2. #6800 수용 근거에는 bbox 개선뿐 아니라 공개 charX·hit-test·KTX 회귀를 적는다. 미실행 Studio UI 클릭 검증은 성공이라고 쓰지 않는다.
3. 같은 작업의 메인터너 후속 댓글이 이미 있으면 그 댓글을 수정하고 중복 등록하지 않는다. 기여자의 기존 댓글을 덮어쓰지 않는다.
4. 아래 대표 PNG를 코멘트 본문에 직접 표시하고 기준 PDF 링크, 물리 페이지, 실제 보정 범위와 잔여 차이를 함께 적는다. `<MERGE_SHA>`는 증적이 실제 포함된 SHA로 바꾼다.
5. UTF-8 body file로 게시·수정한 뒤 API로 body를 재조회한다. closing reference와 실제 issue 상태를 확인하며 PR close와 issue close를 별도로 처리한다.

```markdown
![PR 6801 물리 1쪽 비교](https://raw.githubusercontent.com/edwardkim/rhwp/<MERGE_SHA>/mydocs/pr/assets/planet6897_ci_green_20260907/pr_6801_p1.png)
[기준 PDF](https://github.com/edwardkim/rhwp/blob/<MERGE_SHA>/pdf/1192000-202100017-policy-research-report-2020.pdf)
```

## 이번 작업 상태

이번 후속 작업에서는 기존 보정 체크포인트 `f2047e1f4` 및 출처 보존 체리픽 `44830b2c1`, `81ec9b869`를 로컬에 만들었다. 이후 작업지시자의 PR 생성 승인에 따라 검토 문서·대표 PNG·오늘할일을 같은 통합 branch에 포함한다. 검토 판정 시점에는 통합 PR의 원격 CI·merge·후속 처리가 완료되지 않았다. reviewer를 자동 지정하지 않으며 로컬 시험 성공을 통합 PR의 원격 CI 성공으로 쓰지 않는다.

PR 준비 단계에서 9월 7일 오늘할일에 이번 검토 기록을 추가한다. 로그, 중간 PNG/SVG/JSON, 임시 진단 프로그램, generated suite 산출물, 임시 WASM pkg는 증적 커밋 대상이 아니다. 사용자 `pkg/`, 공유 target, 다른 작업의 branch/worktree/stash는 보존한다.
