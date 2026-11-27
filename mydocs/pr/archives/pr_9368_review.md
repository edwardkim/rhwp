# PR #9368 리뷰 — Render Diff 색인의 Git 계보 기록

## 최종 판정

**승인.** 작성자 자체 검토 대상 `797478f226db140c366bd0a1a1518cf79c8342e6`에서 Render Diff가 PR 이벤트 base, 검사한 tree, 확인된 merge commit의 첫 부모를 증적에 남기는 동작을 검증했다. 필수 검사와 전체 CI 확인 뒤 `a8904a608fe63d848316e7f6e63cc6bf2202ed51`로 devel에 병합됐다.

## 접수와 범위

[PR #9368](https://github.com/edwardkim/rhwp/pull/9368)은 [이슈 #9366](https://github.com/edwardkim/rhwp/issues/9366)의 CI 증적을 개선한다. 작성자와 자체 검토자는 postmelee다. [Render Diff workflow](../../../.github/workflows/render-diff.yml), [색인 생성기](../../../scripts/render_diff_evidence.py), 관련 Python 검사와 운영 설명이 변경됐다. 조판·렌더링 구현, 테스트 문서와 기준 PDF, 검사 트리거·권한은 변경 범위에 포함되지 않는다. CI workflow 변경이므로 [별도 기록 절차](../../manual/pr_review/post_merge.md#711-ci-변경-pr은-옵션-2로-후속-기록)를 적용했다.

## 검증 결과

- 로컬에서 Render Diff Python 검사 18건, PR 요약 Node 검사 9건, actionlint와 공백 검사를 통과했다. 실제 얕은 clone을 사용해 이벤트 base와 검사 커밋의 첫 부모가 다른 경우 두 값이 각각 유지되는 것을 확인했다. 일반 커밋과 과거 색인 재생도 검사했다.
- [전체 CI](https://github.com/edwardkim/rhwp/actions/runs/45189008189)와 [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/45189007877)가 정확한 검토 head에서 성공했다. 실제 [색인 artifact](https://github.com/edwardkim/rhwp/actions/runs/45189007877/artifacts/14827820126)의 run·attempt와 이벤트 base를 Actions 작업과 대조하고, 검사한 커밋 `7eeaf8c707472ff4c7e5a7928915e67411a4d5e0`의 tree `4f1dca748fd9fd29819738f9c1b7ce395b4dcf99`와 두 부모를 GitHub의 Git 객체에서 확인했다. 당시 이벤트 base와 실제 첫 부모는 모두 `321ab8f97791ecf58b56f178f49b7795a836fb79`였다.
- 병합 직전 devel `e7b45a7a71e9d038bd137a9dc2ac3ef37e0ac2e5`에서 병합 시뮬레이션과 공백 검사를 통과했다. 별도 문서 입력을 추가하거나 조판 원칙을 바꾸는 작업은 비해당이다.
