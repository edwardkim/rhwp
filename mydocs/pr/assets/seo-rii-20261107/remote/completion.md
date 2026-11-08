# seo-rii Ready 29건 통합 수용 — PR #8739

[통합 PR #8739](https://github.com/edwardkim/rhwp/pull/8739)의 head `0d2548fd88ea76b10e53eec841376b4bcbecbc4a`에서 필수 CI·self-review를 완료했고, 원 기여 이력을 보존하는 merge `f5f0fbee95b5fb0e32e99f8f639d228ae55389dd`로 수용했다. 작업지시자는 기존 eq-01 본문 굵기·수식 간격/분수 모양 및 picture-crop 가는 테두리 차이에 한정한 예외와 Draft #8646의 간접 MERGED 효과를 승인했다. 새 숫자 오류·회귀·필수 검사 실패는 예외에 포함하지 않았다. #536·#8701은 OPEN을 유지하며 #8337 완료·원 PR별 후속 댓글·정리는 정본 순서에 따라 이어 간다.

- [실제 CI](ci-success.json) · [self-review](self-review.json) · [승인](approval.json) · [merge](merge.json).
- CodeQL #478·#483·#484는 명시 승인 뒤 오탐으로 분류했다. check는 자동 성공으로 갱신됐으며 재실행하지 않았다. [판정 근거](security-alerts-review.md) · [실행 결과](security-mutation.json).
- GitHub Archive 실제 12,631 passed·45 skipped와 Studio 2,745 passed·2 skipped를 로그로 확인했다. 로컬 전체 회귀와 CI archive 집계는 서로 다른 구성이다.
- 로컬 전체 12,825·Native 공식 4,077·Studio 2,745 통과. Native/fresh WASM 각 24쪽 모두 raw 90% 이상, 기존 대조군 PNG 48쌍 동일. 원 face 부재 실제 숫자 의미 보정과 정상 대조를 구분했다.
- Metal은 승인된 기본 비활성 실험 기반이며 Apple SDK 미검증. Windows D3D12 producer 실행과 최종 Flutter consumer·지속 표시 미검증을 구분한다.
- 이 기록을 devel에 반영한 뒤 #8337 완료·원 PR별 댓글·기간 메트릭 확인·종료 정리를 완료한다.

원 Ready PR 29건과 선행 Draft #8646은 GitHub에서 모두 간접 MERGED로 확인했다. 별도의 Draft 전환이나 수동 PR close는 하지 않았다. [상태 원장](original-prs-after-merge.json) · [merge SHA 고정 이미지 18개 검증](merge-raw-image-verification.json) · [게시한 self-review](https://github.com/edwardkim/rhwp/pull/8739#issuecomment-6705036349).

병합 후 [소요시간 집계](https://github.com/edwardkim/rhwp/actions/runs/42359744228)는 성공했다. [실제 결과](duration-observation.json)를 확인했으며 검증 CI를 추가 실행하지 않았다.

## 이슈·댓글 후속 처리 완료

#8337은 devel 반영 뒤 완료 close했고 #536·#8701은 OPEN이다. 원 PR 29건과 이슈 1건의 댓글 30개를 게시해 API 본문을 일치 확인했다. [댓글 원장](posted-followups.json), [이슈 상태](final-issue-states.json). 시각 댓글 4개의 이미지 22회 표시(고유 18개)를 실제 GitHub 브라우저 화면에서 확인했다. [표시 결과](posted-image-display.json).

작업 전용 branch·worktree·Windows 도구·Docker 이미지와 루트 output·공유 debug의 종료 정리를 이어 간다.

## 종료 확인

작업 전용 upstream/local branch·worktree·fetch ref 29개·Windows 임시 도구·Docker 이미지를 제거했다. 저장소 루트 `output/` 및 `target/pr-review/debug` 디렉터리 자체의 부재를 확인했다. WSL 파일시스템 여유 공간은 38.67 GiB 증가해 197.60 GiB가 됐다. [정리 로그](cleanup.json). 기본 Studio JS/WASM과 node_modules, 기존 Vite, 다른 worktree·Draft 및 contributor fork branch는 보존했다. 공유 release·release-test·WASM profile도 유지했다.

필수 로컬 검증·동일 head GitHub CI·self-review·merge·운영 기록·이슈와 댓글·종료 정리를 모두 완료했다. 제품 CI 재실행 및 새 제품 코드 변경 없이 CodeQL 세 건의 승인된 분류와 자동 check 갱신으로 원격 절차를 마쳤다.
