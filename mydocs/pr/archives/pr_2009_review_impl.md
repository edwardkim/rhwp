# PR #2009 처리 계획

## 커밋 구성

- `a800c662e task 2008: Studio 파일 로딩 진행률 표시`
  - Studio 상태창 단계 기반 0~100% 표시
  - 초기 cursor rect page tree cache 사용
  - 표 resize hover bbox page hint 적용
  - #2008 Stage 1 문서와 오늘할일 갱신
- 후속 문서 커밋
  - 옵션 1 누락 보정: `pr_2009_review.md`, `pr_2009_review_impl.md`, 오늘할일 PR 번호 반영

## 옵션 1 처리 방침

이 PR 은 collaborator self-merge 후보 경로로 처리한다.

- review 문서: `mydocs/pr/archives/pr_2009_review.md`
- 구현/처리 계획: `mydocs/pr/archives/pr_2009_review_impl.md`
- 오늘할일: `mydocs/orders/20260707.md`
- 별도 docs-only 후속 PR 은 만들지 않는다.

## 검증

로컬 검증은 `task_m100_2008_stage1.md` 와 review 문서에 기록했다.

CI 는 PR head 최신 커밋 기준으로 재확인한다.

## merge 후 후속 처리

1. PR #2009 merge SHA 확인
2. `devel` 을 `upstream/devel` 로 fast-forward sync
3. #2008 auto-close 여부를 시간을 두고 확인
4. auto-close 여부와 무관하게 #2008 에 검증/merge 후속 코멘트 작성
5. PR #2009 브랜치 `task_m100_2008_studio_load_progress` 로컬/원격 정리
