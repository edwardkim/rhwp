# PR #1490 self-merge 실행 계획

- PR: https://github.com/edwardkim/rhwp/pull/1490
- 작성일: 2026-06-23
- 경로: collaborator self-merge 후보
- 문서 작성 시점 참고 head: `01eeacf7` (본 review 문서/오늘할일 커밋 전)

## 1. 목적

PR #1490은 #1487 내부망/오프라인 확장 사용 피드백에 대응해
확장 옵션에서 외부 웹폰트 사용을 끌 수 있도록 하는 collaborator self-merge 후보 PR이다.

`mydocs/manual/pr_review_workflow.md` 8장에 따라 review 문서와 오늘할일 문서를 PR head에 포함해,
merge 후 별도 문서 커밋을 만들지 않도록 한다.

## 2. 커밋 목록

문서 작성 시점 로컬/원격 head 기준 커밋:

| 순서 | SHA | 제목 |
|------|-----|------|
| 1 | `32da0761` | `task 1487: 외부 웹폰트 비활성 옵션 추가` |
| 2 | `de77297f` | `task 1487: 빈 뷰어 클릭 오류 방지` |
| 3 | `31d2483a` | `task 1487: 로컬 글꼴 모달 문구 정리` |
| 4 | `f7577932` | `task 1487: PR 검증 공백 정리` |
| 5 | `01eeacf7` | `task 1487: CodeQL 테스트 URL 검사 경고 수정` |

최종 문서 커밋 예정:

- `mydocs/pr/archives/pr_1490_review.md`
- `mydocs/pr/archives/pr_1490_review_impl.md`
- `mydocs/orders/20260623.md`

## 3. 진행 단계

### Stage A - 작업 범위 확정

1. #1487 이슈와 PR #1490 메타를 확인한다.
2. PR diff가 확장 옵션, viewer 설정, font loader, 로컬 글꼴 모달, 빈 viewer 클릭 방어, 회귀 테스트와 stage 문서로 구성되어 있음을 확인한다.
3. GitHub Advanced Security 코멘트가 URL 부분 문자열 검사 권고였고, 최신 코드에서 URL 파싱 기반 비교로 반영되었음을 확인한다.

### Stage B - review 문서 작성

1. `mydocs/pr/archives/pr_1490_review.md`를 작성한다.
2. `mydocs/pr/archives/pr_1490_review_impl.md`를 작성한다.
3. `mydocs/orders/20260623.md`에 #1487 PR 문서화와 merge 전 조건을 기록한다.

### Stage C - 문서 검증과 push

문서만 수정하므로 cargo 빌드/테스트는 기계적으로 반복하지 않는다.
다음 검증을 수행한다.

```bash
git diff --check
git status --short
```

검증 후 다음 커밋으로 기존 PR head 브랜치에 push한다.

```bash
git add mydocs/pr/archives/pr_1490_review.md \
        mydocs/pr/archives/pr_1490_review_impl.md \
        mydocs/orders/20260623.md
git commit -m "task 1487: PR 검토 문서와 오늘할일 갱신"
git push upstream task_m100_1487
```

### Stage D - GitHub Actions 및 merge 준비

push 후 다음을 최신 PR head 기준으로 확인한다.

- GitHub Actions 전체 통과
- review 문서 2건과 오늘할일 문서가 PR diff에 포함됨
- GitHub Advanced Security의 기존 코멘트가 최신 코드에서 재발하지 않음
- merge 가능한 상태
- 작업지시자 승인

## 4. merge 후 후속 처리

- PR #1490 merge 결과 확인
- 이슈 #1487 close 여부 확인
- 필요 시 이슈 close + 코멘트
- PR에 merge 완료 및 검증 결과 코멘트
- `upstream/devel` 동기화
- `upstream/task_m100_1487` 원격 작업 브랜치 삭제
- 로컬 작업 브랜치 정리
