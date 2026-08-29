# 피드백 — Task M100 #3789 Hyper-Waterfall 절차 보정

- **일자**: 2026-08-27 KST
- **대상**: [#3789](https://github.com/edwardkim/rhwp/issues/3789)
- **source commit**: `17fa14198`
- **CI commit**: `514ff74bc`
- **보고 commit**: `3c509c7d1`
- **지적자**: 작업지시자

## 피드백

> 하이퍼 워터폴 규칙을 준수해서 작업했는지 검토해줘.

검토 결과 기술 구현과 검증은 계획대로 완료됐지만, Stage 2·3 종료 보고와 중간 작업지시자 승인 없이 다음
단계로 진행했다. 작업지시자는 이력을 재작성해 완전 준수처럼 보이게 하지 않고 실제 계보를 문서에
명시하는 보정을 승인했다.

## 준수한 게이트

- #3789 담당자 지정·착수 comment와 중복 open PR 확인을 구현 전에 수행했다.
- 착수 당시 최신 `upstream/devel@1b91c2025`에서 접두사 없는 승인된 branch를 만들었다.
- 수행·구현 계획을 제품 변경 전 `fcaff2afd`로 고정하고 작업지시자의 착수 승인을 받았다.
- source 책임 이동과 CI 경계 보정을 `17fa14198`, `514ff74bc`로 분리했다.
- focused·release-test·clippy·CI 정책·format·문서 링크 검증을 통과했다.
- 별도 승인 전 remote push와 PR 생성을 수행하지 않았다.

## 누락된 게이트

| 절차 | 실제 상태 |
| --- | --- |
| Stage 1 종료 보고·승인 뒤 Stage 2 진입 | 계획 승인은 있었지만 Stage 1 working 보고는 사후 작성 |
| Stage 2 종료 보고·승인 뒤 Stage 3 진입 | source commit 뒤 별도 보고·승인 없이 CI 변경 진행 |
| Stage 3 종료 보고·승인 뒤 Stage 4 진입 | CI commit 뒤 별도 보고·승인 없이 전체 검증 진행 |
| 단계별 contemporaneous 기록 | Stage 1~4와 최종 보고를 `3c509c7d1`에서 함께 작성 |

## 원인과 영향

작업지시자의 `진행해줘`를 계획된 전체 로컬 구현·검증의 포괄 승인으로 해석했다. 이 해석은 제품·CI
commit을 단계별로 나누는 데에는 충분했지만, 각 단계 보고를 먼저 공유하고 다음 단계 승인을 받는 별도
품질 게이트를 대체하지 못한다.

기술 diff와 테스트 증적은 commit별로 보존됐지만, 작업지시자가 Stage 2 결과를 보고 CI 경계 변경 방향을
중간에 교정하거나 Stage 3 결과를 보고 전체 검증 범위를 승인할 기회가 사라졌다. 사후 작성된 Stage 문서가
동시대 기록처럼 읽힐 수 있다는 감사상 모호성도 생겼다.

## 보정

1. Stage 1~4 문서 제목과 metadata에 사후 감사 보고임을 명시한다.
2. 각 Stage 전환에서 실제로 생략된 보고·승인을 해당 문서의 종료 판단에 기록한다.
3. 최종 보고서에 기술 게이트 준수와 단계 승인 부분 미준수를 분리해 판정한다.
4. 원 구현 commit을 rebase·amend해 과거 계보를 바꾸지 않는다.
5. 현재 보정 승인은 감사 가능성을 회복하지만 원래 이탈을 소급 승인하지 않는다고 명시한다.

## 다음 유효 게이트

감사 시점에는 `upstream/devel`이 착수 기준보다 진전했다. 문서 보정 commit 뒤 최신 `devel`로
재기준화하고 충돌·관련 검증을 다시 확인한다. remote push와 PR 생성은 그 결과를 공유한 뒤 별도 승인을
받는다. PR 번호가 확정된 뒤에만 collaborator self-review와 필요한 오늘할일을 trailing 기록으로 추가한다.

## 보정 후 실행

작업지시자는 절차 보정 결과를 확인하고 다음 Stage 진행을 승인했다. 최신 `upstream/devel@2166f4065`의
자동 merge tree가 충돌 없음을 확인한 뒤 `39d6aa1dd`로 current-base merge했다. #3789 focused Rust
113개, CI policy Node 67개, workflow Python 70개와 format·manifest·문서 게이트가 통과했다. 이 결과를
[Stage 5 보고](../working/task_m100_3789_stage5.md)로 먼저 공유하고, 전체 release-test·clippy는 다음
승인 게이트로 분리한다.

## 2차 재최신화

Stage 5 보고 뒤 upstream이 다시 52커밋 진전했다. 작업지시자가 재최신화를 승인해
`upstream/devel@5645e1f5b`를 `3db893274`로 current-base merge하고, focused Rust 113개, Node 67개,
Python 71개와 정적 게이트를 재검증했다. 이 결과는 [Stage 6 보고](../working/task_m100_3789_stage6.md)에
동시점 기록한다. 전체 release-test·clippy를 자동으로 이어 실행하지 않고 Stage 7 승인 게이트로 남겨,
보정 이후에는 단계 종료 보고와 다음 단계 승인을 실제 순서대로 분리한다.

## Stage 7 승인과 실행

작업지시자가 Stage 6 보고를 확인한 뒤 Stage 7 진행을 별도로 승인했다. 전체 nextest 8,473개와 현재
권위 문서의 필수 clippy가 통과한 뒤 [Stage 7 보고](../working/task_m100_3789_stage7.md)를 작성했다. 제거된
과거 wrapper와 필수 범위 밖 `--all-features` 진단 실패도 생략하지 않고 별도 관찰로 기록했다. remote
push와 PR 생성은 자동으로 이어가지 않고 다음 승인 게이트로 남긴다.

## Stage 8 제출 직전 재최신화

작업지시자가 remote push와 PR 생성을 승인한 뒤 fetch에서 최신 `devel`의 15커밋 진전을 확인했다. 제출
승인을 오래된 base push의 근거로 쓰지 않고 `upstream/devel@1a43a507c`를 current-base merge한 뒤 focused와
전체 회귀를 다시 실행했다. [Stage 8 보고](../working/task_m100_3789_stage8.md)를 최초 원격 candidate에
포함하고, PR 번호가 생긴 뒤에만 self-review와 오늘할일을 trailing commit으로 작성한다.

## Stage 9 리뷰 보정과 안전 중지 복구

PR 리뷰에서 최초 CI 경계 전제의 실제 workflow consumer 불일치를 확인했다. 작업지시자가 보정을 승인해
source 보정을 `eeffb3e8f`에 먼저 고정했고, 이동 전에 작업지시자의 안전 중지 요청에 따라 clean checkpoint로
남겼다. 재개 승인 뒤에만 `upstream/devel@f6a6bee8f`를 `16ea38cd2`로 병합하고 focused·전체 회귀를
수행했다.

이 단계는 과거 Stage 1~4 기록을 고쳐 완전 준수로 만들지 않는다. 리뷰 판단, 보정 commit, 안전 중지,
재개, 최신화와 검증의 실제 순서를 [Stage 9 보고](../working/task_m100_3789_stage9.md)에 동시점 기록한다.
원격 push와 보정 완료 comment는 다시 별도 승인 게이트로 남긴다.
