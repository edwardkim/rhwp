# 단계별 기록 S6 - 최초 배포 표면 검증 (M100 #2285)

- **이슈**: [edwardkim/rhwp#2285](https://github.com/edwardkim/rhwp/issues/2285)
- **대상 PR**: [edwardkim/rhwp#2286](https://github.com/edwardkim/rhwp/pull/2286)
- **브랜치**: `codex/task2285-handle-only-20260716`
- **단계**: S6 / S7에서 재검증
- **작성일**: 2026-07-16

## 최초 결과

Chrome extension build, VS Code webpack compile, npm SDK test/package dry-run은 모두 통과했다.
Chrome viewer의 `apple-touch-icon` `icons/icon-256.png` 누락 후보는 `upstream/devel`에도 있는
기존 packaging 문제로, 최근 문서 변경과 무관했다.

## 정정 사항

S6은 이후 철회된 S5 handle-only 소스를 빌드했다. 따라서 최근 문서 기능의 최종 검증 근거로는 충분하지
않다. S7에서 실제 Chrome URL-load를 다시 확인했고, meta-only 소스로 Studio 및 Chrome extension
build를 재실행해 통과했다. VS Code와 npm 표면은 Studio를 직접 import하지 않는다는 S6의 경로 분석과
기존 compile/test/package 결과를 유지한다.

최종 검토 기준은 [S7 기록](task_m100_2285_stage7.md)과
[PR 검토 정정](../pr/pr_2286_review.md)이다.
