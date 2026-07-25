# PR #3312 구현·통합 계획

## 대상과 rollback 경계

- 대상: [#3312](https://github.com/edwardkim/rhwp/pull/3312), [#3309](https://github.com/edwardkim/rhwp/issues/3309)
- 구현 SHA: `7fed99fd2c0e294d41ef2d868e67cd26c54f9290`
- 문서 trailing commit: 이 파일, `pr_3312_review.md`, `20260725.md`

workflow 세 파일은 같은 candidate 안전 규칙을 가져야 한다. fast-pass가 예상과 다르거나 최신 aggregate가
실패하면 merge하지 않고 해당 PR에서 원인을 보정한다. merge 뒤 문제를 발견하면 `devel` 직접 수정이 아니라
별도 후속 PR로 revert 또는 보정한다.

## 단계

1. 구현 SHA의 full CI, CodeQL, Render Diff 성공과 current-base 관계를 확인한다. 완료: `7fed99fd2`.
2. review·implementation·오늘할일만 추가한 문서 trailing commit을 push한다.
3. 새 head의 세 preflight가 `7fed99fd2`를 candidate로 선택하고 Build & Test aggregate, CodeQL, Render Diff가
   통과하는지 확인한다. heavy worker의 skipped는 이 경로에서 정상이다.
4. 최신 head SHA·mergeable·required check를 재확인한 뒤 작업지시자 merge 승인과 reviewer 판단을 기다린다.
5. merge 뒤 #3309 close, `upstream/devel` fast-forward, remote/local `task_m100_3309` branch 정리를 수행한다.

## 증적 파일 원칙

후속 기록에 `mydocs/**` 아래 PDF, HWP/HWPX, PNG 같은 증적 파일이 포함되어도 파일 상태·확장자 제한 없이
문서-only 허용 범위다. 반면 `samples/`·`pdf/`의 새 기준 자료와 source/test/workflow/Cargo.lock 변경은 각각
workflow allowlist와 full-CI fallback 규칙을 따른다.
