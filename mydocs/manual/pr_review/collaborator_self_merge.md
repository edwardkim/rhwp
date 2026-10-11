---
kind: guide
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-09
---

# Collaborator self-merge 후보

이 경로는 collaborator가 본인 PR을 merge 후보로 준비할 때만 쓴다. maintainer의 외부 contributor PR 일반
처리를 대체하지 않는다.

## 8.1 적용 조건

- PR 작성자 또는 준비자가 repository collaborator다.
- PR 번호가 이미 있어 review 문서명을 확정할 수 있다.
- merge 뒤 별도 문서 commit을 만들지 않기 위해 review 문서를 현재 PR diff에 포함한다.
- ready 전환, self-review 기록 확정, merge 판단은 작업지시자 승인 뒤에만 한다.

### 8.1.1 렌더링 회귀의 추가·변경

신규 회귀의 추가와 실패한 기존 회귀의 변경 모두 관련 모든 페이지의 Native/fresh WASM 일치율
90% 이상을 먼저 확인한다. 실패한 기존 회귀는 **원본 입력 출력 보정 → 버전에 맞는 독립 한컴
Print PDF로 90% 이상 확인 → 필요한 검사 변경** 순서로 처리한다. 실물 배치를 절대 px 좌표·고정
px 영역으로 선택하지 않고 문단·줄·개체의 소유·순서·보존·포함·겹침 관계로 검사한다.
미달·측정 불가이면 기대값·baseline 갱신과 준비 완료 판정을 보류한다. 상세 절차는
[시각 선행 조건](visual_fixture_evidence.md#렌더링-회귀-테스트-신규-추가의-시각-검증-선행-조건)과
[기존 기대값 재검토](visual_fixture_evidence.md#기존-회귀-테스트의-기대값-재검토)를 따른다.

**SHA를 고정한 `upstream/devel`에서 이미 90% 미만인 원본에 등록되어 있던 회귀 검사**가
부적합한 경우에만
[별도 이슈 이관](visual_fixture_evidence.md#90-미만-회귀-대상의-별도-이슈-이관)에 따라 새 이슈를 등록하고
해당 회귀 검사와 HWP/HWPX/PDF를 활성 검증 경로에서 제거·이관한다. 별도 이슈에서
보정·90% 이상·확대 판독 후 관계형 회귀를 복원한다.
같은 원본·독립 Print PDF의 base/head 비교로 기존 미달을 입증한다. 현재 수정으로 생긴 회귀는
구현을 고치며, base가 미검증이면 이관 판정을 보류한다. 이관 범위와 현재 PR의 완료 범위를 구분한다.

90% 이상이어도 [수식·간격 확대 판독](visual_fixture_evidence.md#90-통과-후-수식간격의-확대-판독)을
수행한다. 변경 영역·알려진 차이·사용자 지적 영역의 실제 모양·장평·미세 간격과 앞뒤 내용까지
대조한 결과를 self-review에 기록하며, 점수만으로 준비 완료를 판정하지 않는다.

## 8.2 문서와 오늘할일

review 문서는 처음부터 archive 경로에 둔다.

~~~text
mydocs/pr/archives/pr_N_review.md
mydocs/pr/archives/pr_N_review_impl.md
mydocs/pr/archives/pr_N_report.md          # 필요 시
mydocs/orders/YYYYMMDD.md                  # 갱신이 필요한 경우
~~~

### 8.2.1 PR 채번과 오늘할일 생성·갱신 시점

오늘할일은 이슈 등록·branch 생성·조사·계획·구현 중간에는 만들거나 갱신하지 않는다.
구현과 로컬 검증이 끝나고 작업지시자가 remote push와 PR 생성을 승인하면 먼저 코드 후보로
PR을 만든다. **같은 PR의 정확한 code head에 대한 GitHub Actions CI가 성공한 뒤에만**
PR review와 오늘할일을 작성·갱신하고 문서-only trailing commit으로 반영한다.

1. 검증을 마친 후보 commit을 원격 작업 branch에 push한다.
2. Draft 지시가 없으면 Open PR을 생성해 번호 `N`을 받는다.
3. 번호 `N`의 code head SHA를 고정하고 해당 SHA의 required check와 변경 범위상 필요한
   GitHub Actions가 모두 완료되어 성공했는지 확인한다. 진행 중·실패·취소·미실행 상태에서는
   review·오늘할일 trailing commit을 만들거나 push하지 않는다. 로컬 검증 성공으로 이 CI를
   대체하지 않는다. source·test가 바뀌면 새 code head의 CI를 다시 기다린다.
4. reviewer를 지정하지 않고 self-review를 `mydocs/pr/archives/pr_N_review.md`와 필요한
   `pr_N_review_impl.md`에 기록한다. 오늘할일에는 PR 번호, 검증한 code SHA, CI run URL과
   실제 결과, 남은 merge 조건을 함께 적는다.
5. review 문서와 오늘할일을 같은 source branch의 문서-only trailing commit으로 만든다.
   [push 전 병합·링크 검증](../pr_review_workflow.md#321-최신-devel-오늘할일을-보존하는-trailing-기록)을
   통과한 뒤 push해 PR diff에 포함한다. source·test 보정을 이 기록 commit에 섞지 않는다.
6. trailing head의 required check와 실제 fast-pass 판정을 다시 확인한 뒤 merge한다.
   code candidate의 CI 성공만으로 trailing head의 CI 성공을 가정하지 않는다.

PR 생성 전에 번호를 예측해 review 파일명을 만들지 않는다. 이미 active 경로에 만든 review 문서는
다음 PR에 임시로 동반하지 말고, 해당 PR 번호가 확정된 뒤 archive 경로와 파일명을 확정한다.

review 문서와 오늘할일에는 완료된 로컬 검증과 code head CI 결과를 과거형으로 적는다.
아직 실행 전인 trailing head의 GitHub Actions·작업지시자 승인·merge는 남은 조건으로 분리한다.

오늘할일 갱신이나 다른 PR의 `devel` 병합만을 이유로 검토 branch를 반복 merge/rebase하지
않는다. 최신 `upstream/devel`의 해당 오늘할일만 읽어 양쪽 기록을 보존하고, 검증된 code head
위에 문서-only trailing commit을 추가한다. 적용 순서와 실제 충돌·필수 최신화 조건의 예외는
[검토 중 base 전진과 오늘할일 갱신](review_only_fast_pass.md#a0-검토-중-base-전진과-오늘할일-갱신)을 따른다.

## 8.3 remote push

collaborator는 권한 제약이 없는 한 fork origin이 아니라 원본 remote upstream의 작업 branch로 push한다.

~~~bash
git push upstream HEAD:task_m100_<issue>
~~~

## 8.4 merge 전 조건

- 최신 PR head의 GitHub Actions가 통과한다.
- 필요한 review, review_impl, 오늘할일이 PR diff에 포함된다.
- draft·mergeable·head SHA·CI 상태는 작성 시점 참고값으로만 기록한다.
- 작업지시자 승인을 받는다.

### 8.4.1 명시 지시된 maintainer `--admin` merge 예외

일반 collaborator는 `--admin`으로 branch protection을 우회하지 않는다. 단, maintainer 권한을 가진
실행자가 collaborator self PR을 처리하면서 작업지시자로부터 **해당 PR의 `--admin` merge 명시 지시**를
받은 경우에는 다음 조건을 모두 만족할 때만 사용할 수 있다.

- code candidate와 review·오늘할일 trailing head 각각의 최신 GitHub Actions가 성공했고, 실패·대기 중인
  check가 없다.
- trailing head의 `mergeable`은 `MERGEABLE`, `mergeStateStatus`는 `CLEAN`이며, merge 직전에 다시
  조회한 head SHA가 명령의 `--match-head-commit` 값과 같다.
- trailing commit은 review, 오늘할일, stage·절차 문서만 추가한다. source, test, fixture, workflow,
  baseline, sample 변경이 trailing commit에 섞이면 이 예외를 적용하지 않는다.
- 이 옵션은 reviewer 부족, 실패한 검증, 오래된 code candidate, 충돌을 우회하는 용도로 사용하지 않는다.

위 조건에서는 다음과 같이 squash merge할 수 있다. 권한이 없는 collaborator의 토큰에서는 명령이 실패할
수 있으며, 그 경우 정상 merge 경로로 되돌아간다.

~~~bash
gh pr merge N --repo edwardkim/rhwp --squash --admin \
  --match-head-commit <latest-trailing-head>
~~~

merge 뒤에는 이 PR 자체가 review 기록을 포함했는지와 issue 상태를 확인하기 위해
[merge 후속 처리](post_merge.md)를 적용한다. 이 과정이 끝나면 이번 PR만을 위해 만든 local worktree는
clean 상태와 다른 작업의 소유 여부를 확인한 뒤 제거한다. 단순히 다음 작업에 재사용할 편의만으로 유지하지
않으며, 제거할 수 없는 활성 작업 또는 사용자 보존 지시가 있으면 그 사유와 경로를 최종 상태에 남긴다.

작업지시자가 이 경로의 PR 병합과 `merge 후 후속 처리`를 승인했다면, 그 승인은 이번 PR 전용의 clean한
local branch와 local worktree를 제거하는 데에도 적용된다. 따라서 조건을 만족한 뒤에는 별도의 "정리" 지시를
기다리지 않고 후속 처리에서 제거한다. 이번 작업에서 만든 PR 전용 임시 upstream head branch도
`post_merge.md` 7.7의 소유·최종 SHA·병합·CI·활성 작업 조건을 충족하면 별도 승인 질문 없이 자동 정리한다.
기본 작업공간, 공유 target, contributor fork 및 사용자·다른 도구의 branch/worktree는 자동 삭제하지 않는다.
