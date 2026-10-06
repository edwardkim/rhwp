# PR #6798 검토 기록

## 판정: 메인터너 보정 후 수용 가능

**보정 및 로컬 검증 완료.** 저장 좌표를 무조건 신뢰하던 흐름 오염을 실제 충돌·좌표 유효성·전체 표 수용 조건으로 제한했고, 음성 계약을 포함한 집중 5개가 통과했다.

이 판정은 아래 로컬 후보의 PR 변경 범위에 한정한다. GitHub 승인 이벤트·원 PR 직접 merge·전체 이슈 해결 또는 전체 문서 시각 동일을 뜻하지 않는다.

## 검토 기준과 적용 이력

- 검토일: 2026-09-07.
- 원 PR: [#6798](https://github.com/edwardkim/rhwp/pull/6798), 관련 이슈 [#6797](https://github.com/edwardkim/rhwp/issues/6797).
- 제목: 수정(renderer): 표 항목도 앞 문단의 자리차지 밴드를 비켜 간다 (#6797)
- 누적 브랜치: `review/planet6897-ci-green-20260907`.
- 기준 devel: `07bc5e5490f75118f08370de19aeee73ce1667cb`.
- **실제 로컬 시험 대상: `81ec9b869daa27d8546bbbbcb29587df93144f93` 시점의 검증 작업 트리.** 메인터너 체크포인트 `f2047e1f4`와 #6796 후속 체리픽 2개, 전체 원본/축소 입력의 좌표 대조 회귀를 포함한다.
- 이번에 조회한 원격 head: `deb4396ebe9b2ddd39f46a9e0dbd0a685c735e6b`, OPEN, MERGEABLE / CLEAN.
- 로컬에 반영한 원 PR 범위: `deb4396ebe9b2ddd39f46a9e0dbd0a685c735e6b`까지. 해당 원격 head를 포함한다.
- 검토 경로: collaborator 매개 외부 PR의 출처 보존 체리픽 통합. 10개 원 PR을 함께 시험했으며 단독 before/after를 새로 실행한 것처럼 기록하지 않는다.
- 메인터너 체크포인트 `daf5744bfd4cb62d6c3e6d9b6cd104816da758ec`의 #6784/#6798/#6801 보정과 후속 작업 중 보정을 구분한다.
- 기존 reviewer 요청은 유지했다. 이번 테스트·문서 갱신에서 reviewer나 통합 PR owner review를 추가 요청하지 않았다.

| 원 커밋 | 로컬 적용 커밋 |
| --- | --- |
| `915e1143eac51e088847ce5af7b62ea39e7232ec` | `29194a603` |
| `42060363c8b98aed15517bf881438f202de44937` | `3aac5c327` |
| `deb4396ebe9b2ddd39f46a9e0dbd0a685c735e6b` | `c547a52b2` |

## 검토 결과와 보정 상태

- 메인터너 보정은 현재 표와 이전 exclusion의 실제 충돌·순서, 유효한 저장 anchor, 현재 단/쪽에 전체 표가 들어갈 수 있는지를 확인한다. 임의 픽셀 문턱이나 마지막 bbox clamp에만 의존하지 않는다.
- 이미 회피된 offset 표, 범위 밖 저장 좌표, 합성·누락 저장 anchor가 추가 점프를 유발하지 않는 공개 API 시험을 통과했다. 원본 양성 및 host 텍스트 비이동 보호를 합쳐 집중 5개다.
- 이전의 vpos=1,000,000 변형에서 관측한 흐름 y=13,412.7px는 보정 전 진단 수치다. 당시에도 최종 bbox와 흐름 좌표는 달랐으며 이를 현재의 최종 출력 실패로 남기지 않는다.
- 원본 양성의 두 표 분리와 본문 흐름 보존은 유지했다. 기존 KTX/IR/코퍼스 회귀를 포함한 전체 시험도 통과했다.

## 기존 코멘트 상세 대조

| 기존 댓글 | 요청 또는 주장 | 현재 후보에서의 판단 |
| --- | --- | --- |
| [5557695450](https://github.com/edwardkim/rhwp/pull/6798#issuecomment-5557695450) | 64px 경험 문턱, 잘못된 control 선택, 대상 식별, engine 2020 요청 | 경험 문턱 제거·대상 식별·engine 2020 증적에 더해 현재 유효성/충돌 음성 회귀를 보완했다. |
| [5558038773](https://github.com/edwardkim/rhwp/pull/6798#issuecomment-5558038773) | 저장 사다리 기반 회피로 교체 | 원본 양성 효과를 유지하면서 저장 사다리만으로 추가 점프를 허용하지 않도록 메인터너가 범위를 좁혔다. |
| [5558770629](https://github.com/edwardkim/rhwp/pull/6798#issuecomment-5558770629) | 이미 회피된 표의 추가 이동 및 범위 밖 vpos | 이미 회피된 표와 범위 밖·합성·누락 좌표 변형이 현재 집중 회귀에서 통과했다. 과거 흐름 오염 재현은 보정 전 이력으로 구분한다. |

댓글에 적힌 기여자 환경의 과거 실측과 이번 로컬 시험을 구분한다. 이번에 새 댓글이나 review thread를 게시하지 않았다.

## 실제 검증 결과

### 원 PR 최신 head CI

원격 `deb4396ebe9b2ddd39f46a9e0dbd0a685c735e6b`의 조회 시 종합 상태는 **SUCCESS**다. 아래 결과는 원 PR의 CI이며 작업 중 보정을 포함한 로컬 통합 후보의 원격 CI가 아니다.

| 원 PR 최신 head 검사 | 조회 결과 |
| --- | --- |
| [Canvas visual diff](https://github.com/edwardkim/rhwp/actions/runs/34094366561/job/101654585691) | SUCCESS |
| [adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/34094366699/job/101654632982) | SUCCESS |
| [Analyze (javascript-typescript)](https://github.com/edwardkim/rhwp/actions/runs/34094366737/job/101654605755) | SUCCESS |
| [prop roundtrip](https://github.com/edwardkim/rhwp/actions/runs/34094366780/job/101654583715) | SUCCESS |
| [Analyze (python)](https://github.com/edwardkim/rhwp/actions/runs/34094366737/job/101654605802) | SUCCESS |
| [Analyze (rust)](https://github.com/edwardkim/rhwp/actions/runs/34094366737/job/101654605882) | SUCCESS |
| [Build & Test](https://github.com/edwardkim/rhwp/actions/runs/34094366797/job/101657566963) | SUCCESS |
| [CodeQL](https://github.com/edwardkim/rhwp/runs/101655294107) | SUCCESS |

조회한 preflight 및 실행 worker는 완료 상태다. `cancel-stale-runs`, `WASM Build`, `Frontend unit gates`, `Workflow promotion preflight`, `Refresh nextest target duration data`의 SKIPPED는 실행 통과 건수에 넣지 않는다.

### 누적 후보의 로컬 시험

- 최신 전체 integration 회귀: **9,153 통과 / 0 실패 / 46 skip**, `--no-fail-fast`, nextest summary 276.450초, exit 0.
- 정식 fixture 10개(기존 9개와 #6796 축소본)를 `RHWP_SECURITY_SWEEP_SAMPLES_JSON`에 명시해 최신 전체 회귀의 보안 코퍼스 검사를 통과했다. 자동 변경 파일 탐지에만 의존하지 않았다.
- 이전 제품 보정 단계의 관련 집중 회귀: **50 통과 / 0 실패**, 0.245초, exit 0. 이번에 동일 필터를 재실행했다고 쓰지 않는다.
- 이번 #6796 후속 집중 회귀: **7 통과 / 0 실패**, 0.235초, exit 0. 원본 2개, 축소본 4개, #5734 보호 1개이며 필터 비선택 9,192개는 전체 회귀의 skip 46개와 구분한다.
- 이전 집중 실행에서 이 PR의 [집중 시험 원본](../../../tests/cases/issue_6797_table_item_clears_float_band.rs): **5개 통과**.
- 최종 Rust fmt, workspace all-targets clippy(`-D warnings`), suite prepare/check, `git diff --check` 통과. 이 검사는 본 문서 갱신 전 코드 후보에 대해 실행했다.
- 이전 제품 보정 단계에서 Native Skia 그림 placeholder **2개**, direct PDF export **4개**가 통과했다. 이번 #6796 후속 후보에서 native feature 실행을 다시 수행하지 않았으며, 과거 선택 0개 및 manifest 불일치를 통과로 세지 않는다.
- 앞선 동일 제품 보정 단계의 workspace 빌드·WASM clippy·Native Skia lib·Docker 없는 WASM 빌드도 exit 0이었다. 전부 이번 최종 실행에서 다시 수행했다고 쓰지 않는다.
- 정확한 실행 명령, 기준선 독립 비교, 후속 원 커밋의 통합 내역는 [통합 검증 및 시각 기록](planet6897_ci_green_20260907_visual_sweep.md)을 따른다.

이전 관련 집중 실행의 실제 통과 항목(위 최신 전체 회귀에도 포함):

- `maintainer_an_already_clear_offset_table_is_not_snapped_again`
- `table_item_clears_the_previous_float_band`
- `maintainer_synthetic_and_missing_stored_anchors_do_not_supply_a_jump`
- `maintainer_out_of_column_stored_coordinates_do_not_force_a_bottom_clamp`
- `a_host_with_text_is_not_moved_by_the_band`

## 시각 증적과 판정 한계

물리 7쪽에서 pi70/71 대상 표가 분리된 배치를 대조했다. 기준 PDF의 일부 그래프 그림 부재와 rhwp 대체 글리프는 남아 있다. 1쪽의 다른 Square 그림/TAC 축을 이 PR의 해결 범위나 신규 회귀로 단정하지 않는다.

- 원본: [samples/issue6797/156160455-social-pig-farm-income.hwp](../../../samples/issue6797/156160455-social-pig-farm-income.hwp), 458,752 bytes.
- 원본 SHA-256: `1b99b763aac36a14a9f463e35ee894a23eb1083780040eab5e0f02a481c694b8`.
- [기준 PDF](../../../pdf/156160455-social-pig-farm-income-2020.pdf): engine 2020, 11쪽, 393,142 bytes.
- 기준 PDF SHA-256: `0b6d2573b68c4e4a59db108766380c0d9e9fe41237be8346d61757b28a485c65`.
- 기존에 준비된 PDF를 재사용했다. 해시·크기는 이번에 확인한 파일 값이며 옛 재변환 파일의 메타데이터를 재사용하지 않는다.
- 이 대표 PNG는 이전 메인터너 보정 단계의 `maintainer-all` sweep 산출물이며 이번에 다시 생성했다고 쓰지 않는다. 이후 #6796 후속은 음수 오프셋 조건식의 설명·변수명 정리와 별도 입력/회귀 추가로, 기존 원본의 렌더 조건을 확대하지 않았다. 새 축소본 대조는 #6796 기록에 별도로 보관한다.
- 자동 flag나 전체 배경을 포함한 pixel match를 시각 수용률로 사용하지 않는다. 실제 Studio UI 클릭 검증은 수행하지 않았다.

### 대표 증적: 물리 7쪽

![PR 6798 물리 7쪽 rhwp·기준 PDF·overlay 비교](../assets/planet6897_ci_green_20260907/pr_6798_p7.png)

## 남은 사항과 다음 단계

현재 로컬 후보에서 저장 좌표 흐름 오염의 보류 사유는 해소됐다. 실제 통합 PR의 최종 head CI와 merge 후 gate는 별도로 필요하다.

#6796의 후속 `9da47cb2`/`097c499f`는 전체 원본 보정을 보존한 채 통합했고, 집중 7개·전체 9,153개 회귀와 축소본 대조를 완료했다. #6804의 `15a881d0`는 동일 기준선 내용을 `f2047e1f4`에 보존한 경우로, 원 SHA의 체리픽 이력과 내용 반영을 구분한다. 시험 중 10개 원 PR을 다시 조회했으며 마지막 확인 이후 추가 head 변경은 없었다.

## PR·이슈 코멘트 게시 계획

[시각 증적 댓글 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)과 [post_merge.md](../../manual/pr_review/post_merge.md)를 따른다. 아래는 게시 계획이며 이미 게시하거나 승인받은 GitHub 조치가 아니다.

1. 최종 통합 head를 확정하고 실제 PR/merge SHA 및 devel CI가 성공한 뒤 수용 결과를 기록한다. 원 PR 직접 merge가 아니라 출처 보존 체리픽 통합 수용임을 명시한다.
2. #6797의 표 회피 목표와 남은 그래프/글꼴 등 다른 축을 구분하여 실제 post-merge 단계에서 종료 범위를 확인한다.
3. 같은 작업의 메인터너 후속 댓글이 이미 있으면 그 댓글을 수정하고 중복 등록하지 않는다. 기여자의 기존 댓글을 덮어쓰지 않는다.
4. 아래 대표 PNG를 코멘트 본문에 직접 표시하고 기준 PDF 링크, 물리 페이지, 실제 보정 범위와 잔여 차이를 함께 적는다. `<MERGE_SHA>`는 증적이 실제 포함된 SHA로 바꾼다.
5. UTF-8 body file로 게시·수정한 뒤 API로 body를 재조회한다. closing reference와 실제 issue 상태를 확인하며 PR close와 issue close를 별도로 처리한다.

```markdown
![PR 6798 물리 7쪽 비교](https://raw.githubusercontent.com/edwardkim/rhwp/<MERGE_SHA>/mydocs/pr/assets/planet6897_ci_green_20260907/pr_6798_p7.png)
[기준 PDF](https://github.com/edwardkim/rhwp/blob/<MERGE_SHA>/pdf/156160455-social-pig-farm-income-2020.pdf)
```

## 이번 작업 상태

이번 후속 작업에서는 기존 보정 체크포인트 `f2047e1f4` 및 출처 보존 체리픽 `44830b2c1`, `81ec9b869`를 로컬에 만들었다. 이후 작업지시자의 PR 생성 승인에 따라 검토 문서·대표 PNG·오늘할일을 같은 통합 branch에 포함한다. 검토 판정 시점에는 통합 PR의 원격 CI·merge·후속 처리가 완료되지 않았다. reviewer를 자동 지정하지 않으며 로컬 시험 성공을 통합 PR의 원격 CI 성공으로 쓰지 않는다.

PR 준비 단계에서 9월 7일 오늘할일에 이번 검토 기록을 추가한다. 로그, 중간 PNG/SVG/JSON, 임시 진단 프로그램, generated suite 산출물, 임시 WASM pkg는 증적 커밋 대상이 아니다. 사용자 `pkg/`, 공유 target, 다른 작업의 branch/worktree/stash는 보존한다.
