# PR #6794 검토 기록

## 판정: 승인

**로컬 검증 완료.** 표지 하단으로 잘못 귀속되던 차례를 2쪽으로 배치하는 조판 계약이 집중 회귀와 기준 PDF 대조를 통과했다.

이 판정은 아래 로컬 후보의 PR 변경 범위에 한정한다. GitHub 승인 이벤트·원 PR 직접 merge·전체 이슈 해결 또는 전체 문서 시각 동일을 뜻하지 않는다.

## 검토 기준과 적용 이력

- 검토일: 2026-09-07.
- 원 PR: [#6794](https://github.com/edwardkim/rhwp/pull/6794), 관련 이슈 [#6793](https://github.com/edwardkim/rhwp/issues/6793).
- 제목: 수정(layout): 다음 문단이 새 쪽 상단이면 TAC host 꼬리 간격을 흐름에 태우지 않는다 (#6793)
- 누적 브랜치: `review/planet6897-ci-green-20260907`.
- 기준 devel: `07bc5e5490f75118f08370de19aeee73ce1667cb`.
- **실제 로컬 시험 대상: `81ec9b869daa27d8546bbbbcb29587df93144f93` 시점의 검증 작업 트리.** 메인터너 체크포인트 `f2047e1f4`와 #6796 후속 체리픽 2개, 전체 원본/축소 입력의 좌표 대조 회귀를 포함한다.
- 이번에 조회한 원격 head: `2bed19a6cf60757508df16c719fe24d593b382b3`, OPEN, MERGEABLE / CLEAN.
- 로컬에 반영한 원 PR 범위: `2bed19a6cf60757508df16c719fe24d593b382b3`까지. 해당 원격 head를 포함한다.
- 검토 경로: collaborator 매개 외부 PR의 출처 보존 체리픽 통합. 10개 원 PR을 함께 시험했으며 단독 before/after를 새로 실행한 것처럼 기록하지 않는다.
- 메인터너 체크포인트 `daf5744bfd4cb62d6c3e6d9b6cd104816da758ec`의 #6784/#6798/#6801 보정과 후속 작업 중 보정을 구분한다.
- 기존 reviewer 요청은 유지했다. 이번 테스트·문서 갱신에서 reviewer나 통합 PR owner review를 추가 요청하지 않았다.

| 원 커밋 | 로컬 적용 커밋 |
| --- | --- |
| `c840e3ca671b83123a6506f4c48b94d1b3bd93cf` | `d58c3e913` |
| `056eda73ea49fb860ade88bb76372905c4fd06da` | `9d52c07f3` |
| `2bed19a6cf60757508df16c719fe24d593b382b3` | `a32574090` |

## 검토 결과와 보정 상태

- 기존 vpos==0 및 단 높이 1/4 경험 분기 대신 저장 사다리와 페이지 수용 관계를 사용하는 변경을 유지했다.
- 1쪽에 차례가 없고 2쪽 상단에 차례가 있으며 표지의 텍스트가 용지 안에 있는 집중 3개가 통과했다.
- 누적 전체 회귀도 통과했다. 승인 범위는 표지/차례의 페이지 귀속이며, #6793의 다른 겹침이나 모든 sentinel 입력까지 해결했다고 주장하지 않는다.

## 기존 코멘트 상세 대조

| 기존 댓글 | 요청 또는 주장 | 현재 후보에서의 판단 |
| --- | --- | --- |
| [5557491197](https://github.com/edwardkim/rhwp/pull/6794#issuecomment-5557491197) | 목차의 잘못된 1쪽 귀속, 잘못된 시험, vpos 0/1/4 경험 조건 | renderer 경험 분기 제거와 조판의 2쪽 귀속을 현재 집중 3개 및 1/2쪽 시각 대조에서 확인했다. |
| [5558432170](https://github.com/edwardkim/rhwp/pull/6794#issuecomment-5558432170) | 저장 사다리 기반 조판 보완 및 기존 핀 보호 | 전체 12쪽 기준을 유지하면서 1쪽 표지/2쪽 차례를 확인했다. 최종 전체 회귀도 실패 없이 완료됐다. |

댓글에 적힌 기여자 환경의 과거 실측과 이번 로컬 시험을 구분한다. 이번에 새 댓글이나 review thread를 게시하지 않았다.

## 실제 검증 결과

### 원 PR 최신 head CI

원격 `2bed19a6cf60757508df16c719fe24d593b382b3`의 조회 시 종합 상태는 **SUCCESS**다. 아래 결과는 원 PR의 CI이며 작업 중 보정을 포함한 로컬 통합 후보의 원격 CI가 아니다.

| 원 PR 최신 head 검사 | 조회 결과 |
| --- | --- |
| [Canvas visual diff](https://github.com/edwardkim/rhwp/actions/runs/34092837192/job/101650386073) | SUCCESS |
| [adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/34092837392/job/101650433568) | SUCCESS |
| [Analyze (javascript-typescript)](https://github.com/edwardkim/rhwp/actions/runs/34092837406/job/101650435852) | SUCCESS |
| [prop roundtrip](https://github.com/edwardkim/rhwp/actions/runs/34092837293/job/101650396048) | SUCCESS |
| [Analyze (python)](https://github.com/edwardkim/rhwp/actions/runs/34092837406/job/101650435924) | SUCCESS |
| [Analyze (rust)](https://github.com/edwardkim/rhwp/actions/runs/34092837406/job/101650435918) | SUCCESS |
| [Build & Test](https://github.com/edwardkim/rhwp/actions/runs/34092837408/job/101653637451) | SUCCESS |
| [CodeQL](https://github.com/edwardkim/rhwp/runs/101651604080) | SUCCESS |

조회한 preflight 및 실행 worker는 완료 상태다. `cancel-stale-runs`, `WASM Build`, `Frontend unit gates`, `Workflow promotion preflight`, `Refresh nextest target duration data`의 SKIPPED는 실행 통과 건수에 넣지 않는다.

### 누적 후보의 로컬 시험

- 최신 전체 integration 회귀: **9,153 통과 / 0 실패 / 46 skip**, `--no-fail-fast`, nextest summary 276.450초, exit 0.
- 정식 fixture 10개(기존 9개와 #6796 축소본)를 `RHWP_SECURITY_SWEEP_SAMPLES_JSON`에 명시해 최신 전체 회귀의 보안 코퍼스 검사를 통과했다. 자동 변경 파일 탐지에만 의존하지 않았다.
- 이전 제품 보정 단계의 관련 집중 회귀: **50 통과 / 0 실패**, 0.245초, exit 0. 이번에 동일 필터를 재실행했다고 쓰지 않는다.
- 이번 #6796 후속 집중 회귀: **7 통과 / 0 실패**, 0.235초, exit 0. 원본 2개, 축소본 4개, #5734 보호 1개이며 필터 비선택 9,192개는 전체 회귀의 skip 46개와 구분한다.
- 이전 집중 실행에서 이 PR의 [집중 시험 원본](../../../tests/cases/issue_6793_tac_host_page_tail_padding.rs): **3개 통과**.
- 최종 Rust fmt, workspace all-targets clippy(`-D warnings`), suite prepare/check, `git diff --check` 통과. 이 검사는 본 문서 갱신 전 코드 후보에 대해 실행했다.
- 이전 제품 보정 단계에서 Native Skia 그림 placeholder **2개**, direct PDF export **4개**가 통과했다. 이번 #6796 후속 후보에서 native feature 실행을 다시 수행하지 않았으며, 과거 선택 0개 및 manifest 불일치를 통과로 세지 않는다.
- 앞선 동일 제품 보정 단계의 workspace 빌드·WASM clippy·Native Skia lib·Docker 없는 WASM 빌드도 exit 0이었다. 전부 이번 최종 실행에서 다시 수행했다고 쓰지 않는다.
- 정확한 실행 명령, 기준선 독립 비교, 후속 원 커밋의 통합 내역는 [통합 검증 및 시각 기록](planet6897_ci_green_20260907_visual_sweep.md)을 따른다.

이전 관련 집중 실행의 실제 통과 항목(위 최신 전체 회귀에도 포함):

- `the_toc_heading_belongs_to_the_second_page`
- `no_text_leaves_the_paper_on_the_cover_page`
- `the_toc_heading_starts_at_the_top_of_the_second_page`

## 시각 증적과 판정 한계

기준 PDF와 rhwp는 12쪽이다. 물리 1쪽 표지 및 2쪽 차례의 페이지 소유를 직접 대조했다. 표지 세로 위치·글꼴·세부 간격 차이는 남아 있으며 전체 픽셀 일치를 뜻하지 않는다.

- 원본: [samples/issue6793/1611000-201000141-small-air-transport-study.hwp](../../../samples/issue6793/1611000-201000141-small-air-transport-study.hwp), 51,200 bytes.
- 원본 SHA-256: `25cc60d379f68246875e968978c13f3273fcc3f6b04fc1fe726434e4626e447d`.
- [기준 PDF](../../../pdf/1611000-201000141-small-air-transport-study-2020.pdf): engine 2020, 12쪽, 179,155 bytes.
- 기준 PDF SHA-256: `34bc3082d666d9112ed073de468586490dd57c93c920bfd6d681e5221e4dda85`.
- 기존에 준비된 PDF를 재사용했다. 해시·크기는 이번에 확인한 파일 값이며 옛 재변환 파일의 메타데이터를 재사용하지 않는다.
- 이 대표 PNG는 이전 메인터너 보정 단계의 `maintainer-all` sweep 산출물이며 이번에 다시 생성했다고 쓰지 않는다. 이후 #6796 후속은 음수 오프셋 조건식의 설명·변수명 정리와 별도 입력/회귀 추가로, 기존 원본의 렌더 조건을 확대하지 않았다. 새 축소본 대조는 #6796 기록에 별도로 보관한다.
- 자동 flag나 전체 배경을 포함한 pixel match를 시각 수용률로 사용하지 않는다. 실제 Studio UI 클릭 검증은 수행하지 않았다.

### 대표 증적: 물리 1쪽

![PR 6794 물리 1쪽 rhwp·기준 PDF·overlay 비교](../assets/planet6897_ci_green_20260907/pr_6794_p1.png)

### 대표 증적: 물리 2쪽

![PR 6794 물리 2쪽 rhwp·기준 PDF·overlay 비교](../assets/planet6897_ci_green_20260907/pr_6794_p2.png)

## 남은 사항과 다음 단계

이 PR 범위의 추가 제품 코드 보정은 요구하지 않는다. 현재의 승인은 로컬 판정이며 GitHub 승인 이벤트나 통합 PR merge 완료가 아니다.

#6796의 후속 `9da47cb2`/`097c499f`는 전체 원본 보정을 보존한 채 통합했고, 집중 7개·전체 9,153개 회귀와 축소본 대조를 완료했다. #6804의 `15a881d0`는 동일 기준선 내용을 `f2047e1f4`에 보존한 경우로, 원 SHA의 체리픽 이력과 내용 반영을 구분한다. 시험 중 10개 원 PR을 다시 조회했으며 마지막 확인 이후 추가 head 변경은 없었다.

## PR·이슈 코멘트 게시 계획

[시각 증적 댓글 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)과 [post_merge.md](../../manual/pr_review/post_merge.md)를 따른다. 아래는 게시 계획이며 이미 게시하거나 승인받은 GitHub 조치가 아니다.

1. 최종 통합 head를 확정하고 실제 PR/merge SHA 및 devel CI가 성공한 뒤 수용 결과를 기록한다. 원 PR 직접 merge가 아니라 출처 보존 체리픽 통합 수용임을 명시한다.
2. 원 PR의 페이지 귀속 수정 수용과 #6793 전체 해결을 구분한다. 잔여 축이 있는 이슈는 종료 전 범위를 다시 확인한다.
3. 같은 작업의 메인터너 후속 댓글이 이미 있으면 그 댓글을 수정하고 중복 등록하지 않는다. 기여자의 기존 댓글을 덮어쓰지 않는다.
4. 아래 대표 PNG를 코멘트 본문에 직접 표시하고 기준 PDF 링크, 물리 페이지, 실제 보정 범위와 잔여 차이를 함께 적는다. `<MERGE_SHA>`는 증적이 실제 포함된 SHA로 바꾼다.
5. UTF-8 body file로 게시·수정한 뒤 API로 body를 재조회한다. closing reference와 실제 issue 상태를 확인하며 PR close와 issue close를 별도로 처리한다.

```markdown
![PR 6794 물리 1쪽 비교](https://raw.githubusercontent.com/edwardkim/rhwp/<MERGE_SHA>/mydocs/pr/assets/planet6897_ci_green_20260907/pr_6794_p1.png)
![PR 6794 물리 2쪽 비교](https://raw.githubusercontent.com/edwardkim/rhwp/<MERGE_SHA>/mydocs/pr/assets/planet6897_ci_green_20260907/pr_6794_p2.png)
[기준 PDF](https://github.com/edwardkim/rhwp/blob/<MERGE_SHA>/pdf/1611000-201000141-small-air-transport-study-2020.pdf)
```

## 이번 작업 상태

이번 후속 작업에서는 기존 보정 체크포인트 `f2047e1f4` 및 출처 보존 체리픽 `44830b2c1`, `81ec9b869`를 로컬에 만들었다. 이후 작업지시자의 PR 생성 승인에 따라 검토 문서·대표 PNG·오늘할일을 같은 통합 branch에 포함한다. 검토 판정 시점에는 통합 PR의 원격 CI·merge·후속 처리가 완료되지 않았다. reviewer를 자동 지정하지 않으며 로컬 시험 성공을 통합 PR의 원격 CI 성공으로 쓰지 않는다.

PR 준비 단계에서 9월 7일 오늘할일에 이번 검토 기록을 추가한다. 로그, 중간 PNG/SVG/JSON, 임시 진단 프로그램, generated suite 산출물, 임시 WASM pkg는 증적 커밋 대상이 아니다. 사용자 `pkg/`, 공유 target, 다른 작업의 branch/worktree/stash는 보존한다.
