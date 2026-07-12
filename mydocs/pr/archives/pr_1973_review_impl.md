# PR #1973 처리 계획 - raw IME/iOS page-local refresh 힌트 보강

## 커밋

- `f8b47af12b8b4be9b7b618741360b3041558b5fa` - `task 1964: raw IME page-local refresh 힌트 보강`

## Stage 1. 구현 검토

완료.

- `afterTextInputEdit()`가 `PageLocalTextEditOptions`를 받아 `shouldUsePageLocalRefresh()`에 전달한다.
- IME 조합 입력은 `insertedText`, `beforePageIndex`, `afterPageIndex`를 전달한다.
- iOS fallback 입력은 첫 입력 앵커 기준 `beforePageIndex`를 보존하고 마지막 입력 후 디바운스 렌더링에 사용한다.
- 기존 `isPageLocalTextEditCommand()` 정책을 재사용한다.

## Stage 2. 로컬 검증

완료.

```bash
cd rhwp-studio
node --test tests/input-edit-invalidation.test.ts
npm run build
cd ..
git diff --check
```

결과:

- `node --test tests/input-edit-invalidation.test.ts`: 7개 통과
- `npm run build`: 통과
- `git diff --check`: 통과

보조 확인:

- `npm test -- input-edit-invalidation.test.ts`: 166개 통과

## Stage 3. 옵션 1 문서 반영

진행.

- `mydocs/pr/archives/pr_1973_review.md`
- `mydocs/pr/archives/pr_1973_review_impl.md`
- `mydocs/orders/20260706.md`

위 문서를 PR head에 포함한다.

## Stage 4. merge 전 확인

대기.

- PR head 최신 커밋 기준 GitHub Actions 통과 확인
- 작업지시자 승인 확인

## Stage 5. merge 후 후속 처리

대기.

- #1964 상태 확인
- #1964에 처리 완료 코멘트 작성
- auto-close 여부와 관계없이 후속 코멘트 기록
- `devel` 동기화
- 로컬/원격 PR 브랜치 정리
