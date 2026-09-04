# Task M100 #1425 구현계획서 - PR 리뷰 워크플로 문서 규칙 분리

- 이슈: #1425, 마일스톤 M100, 브랜치 `task_m100_1425`
- 작성일: 2026-06-21
- 수행계획서: `mydocs/plans/task_m100_1425.md`

## 0. 적용 원칙

이번 작업은 문서 전용 수정이다. 코드, CI 설정, GitHub branch protection, repository 권한 설정은
변경하지 않는다.

핵심 원칙은 다음과 같다.

1. maintainer가 외부 기여자 PR을 검토하는 일반 경로를 기본값으로 둔다.
2. collaborator self-merge 후보 경로는 예외 경로로 분리한다.
3. PR에 포함되는 review 문서는 merge 후에도 모순되지 않아야 한다.
4. `draft`, `mergeable`, `head SHA`, `CI 상태`는 확정 사실이 아니라 작성 시점 참고값 또는 merge 전
   최신 확인 조건으로만 기록한다.

## 1. 수정 대상

대상 파일은 `mydocs/manual/pr_review_workflow.md` 1개다.

작업 문서 산출물은 다음을 유지한다.

- `mydocs/orders/20260621.md`
- `mydocs/plans/task_m100_1425.md`
- `mydocs/plans/task_m100_1425_impl.md`
- `mydocs/report/task_m100_1425_report.md` (검증 후 작성)

## 2. 문서 구조 변경안

### 2.1 개요/대상 정리

현재 문서의 주 독자는 maintainer다. 문서 앞부분에 다음 구분을 추가한다.

- 기본 경로: maintainer가 외부 기여자 PR을 검토, merge, 후속 보고한다.
- 예외 경로: collaborator가 본인 PR을 self-merge 후보로 준비할 때 PR head에 운영 문서를 포함한다.

### 2.2 "리뷰 문서 작성" 절 복구

현재 일반 규칙처럼 쓰인 archive 직접 작성 문구를 maintainer 기본 경로로 되돌린다.

기본 경로:

```text
mydocs/pr/pr_{N}_review.md
mydocs/pr/pr_{N}_review_impl.md
```

작성 후 처리 완료 시 `mydocs/pr/archives/`로 이동한다고 명시한다.

### 2.3 collaborator self-merge 후보 섹션 신설

별도 섹션을 추가해 #1420에서 들어온 운영 규칙을 보존한다.

포함할 조건:

- collaborator 본인이 작성한 PR을 self-merge 후보로 준비하는 경우에만 적용한다.
- PR 번호가 생성된 뒤 review 문서와 처리 계획서를 PR head에 포함할 수 있다.
- merge 후 추가 문서 커밋을 만들지 않기 위해 처음부터 `mydocs/pr/archives/pr_{N}_review*.md`를
  사용할 수 있다.
- `upstream` 작업 브랜치 직접 push는 이 예외 경로에서만 기본 규칙으로 둔다.
- 작업지시자 승인과 최신 GitHub Actions 통과 없이는 ready 전환/merge 판단을 하지 않는다.

### 2.4 volatile 상태값 기록 규칙 추가

review 문서에 포함 가능한 값과 표현 방식을 제한한다.

확정 사실처럼 쓰지 않을 값:

- `draft`
- `mergeable`
- `head SHA`
- `CI 상태`

허용 표현:

- "문서 작성 시점 참고값"
- "merge 전 최신 상태 확인 필요"
- "최종 merge 조건: PR head 최신 커밋 기준 GitHub Actions 통과 + 작업지시자 승인"

금지 표현:

- merge 후에도 현재 상태처럼 읽히는 `draft: true`, `mergeable: CLEAN`, `현재 head: abc1234`
- 과거 CI 결과를 최종 merge 가능 판정처럼 쓰는 문장

### 2.5 후속 처리 절 복구

`7.5 리뷰 문서 archive 경로 유지 확인`은 maintainer 기본 경로와 맞지 않는다.

수정 방향:

- maintainer 기본 경로에서는 처리 완료 후 active 경로 문서를 `archives/`로 이동한다.
- collaborator self-merge 후보에서는 처음부터 archive 경로에 두므로 별도 이동 단계가 없다고
  예외 섹션에서 설명한다.

## 3. 단계별 구현

### 1단계 - 매뉴얼 개정

- `pr_review_workflow.md`의 리뷰 문서 작성 절을 maintainer 기본 경로 기준으로 수정한다.
- collaborator self-merge 후보 경로를 별도 섹션으로 분리한다.
- volatile 상태값 기록 규칙을 추가한다.
- 후속 처리 절에서 archive 이동/유지 규칙을 경로별로 분리한다.

### 2단계 - 검증과 최종 보고

- 핵심 규칙 검색으로 분리 상태를 확인한다.
- `git diff --check`를 실행한다.
- 최종 보고서 `mydocs/report/task_m100_1425_report.md`를 작성한다.

## 4. 검증 명령

```bash
git diff -- mydocs/manual/pr_review_workflow.md
rg -n "maintainer|collaborator|self-merge|archives|draft|mergeable|head SHA|CI 상태|최신 GitHub Actions" mydocs/manual/pr_review_workflow.md
git diff --check
```

## 5. 승인 게이트

1. 본 구현계획서 승인 후 `pr_review_workflow.md` 개정에 착수한다.
2. 매뉴얼 개정 후 검증 결과를 보고하고, 최종 보고서 작성 전 승인 요청한다.
3. 최종 보고서 작성 후 merge/close 여부는 작업지시자 승인을 별도로 받는다.

## 6. 위험과 대응

| 위험 | 대응 |
|------|------|
| collaborator self-merge 규칙이 삭제된 것처럼 보임 | 예외 섹션에 #1420 규칙을 명시적으로 보존 |
| maintainer 경로와 예외 경로가 다시 혼합됨 | "기본 경로"와 "예외 경로"라는 제목과 조건을 반복해 적용 |
| volatile 값 제한이 너무 추상적임 | 금지 표현과 허용 표현을 함께 적어 review 문서 작성자가 바로 적용 가능하게 함 |
| 문서 전용 작업에 과도한 검증 추가 | Markdown diff, 핵심 문구 검색, `git diff --check`로 제한 |

## 7. 완료 산출물

- `mydocs/manual/pr_review_workflow.md` 개정
- `mydocs/report/task_m100_1425_report.md`
- 검증 결과 기록
