# PR #8178 self-review — 선택 실행 증적 보고서

## 최종 판정

**머지 보류** — 사용자가 직접 확인할 보고서 읽기 편의성·공유 전 정보 제외 판정이 미확인이다.
코드 후보의 로컬 계약은 통과했으며, 이 기록은 GitHub approve나 merge 승인이 아니다.
해제 조건은 사용자 수동 판정과 최신 PR head의 required checks 확인, 별도 merge 지시다.

## 접수와 범위

- PR: https://github.com/edwardkim/rhwp/pull/8178
- Issue: #7449 (참조만 하며 자동 종료하지 않음)
- 작성자: postmelee. 권한 조회에서 push 가능. 본인 self-review이므로 reviewer assign 없음.
- base: devel `def947c2149df5bc812d801e6e79be19a403b4d1`
- 검증한 code candidate: `e22fda49fab94fd6bdb64092aed870912ffff9fd`
- 최초 code diff: 11개 파일, +620/-9. 후속은 이 review와 결과보고의 문서 기록만 추가한다.
- base route: collaborator_self_merge. modifiers: intake_and_review, local_validation.
- loaded documents: pr_review_workflow.md, pr_review/README.md, collaborator_self_merge.md,
  intake_and_review.md, review_template.md, local_validation.md, CONTRIBUTING.md 및 문서·Git 절차.
- #3512/#7283의 병합 결과만 소비한다. 다른 후보 PR의 미병합 변경·다른 이슈 구현·자동 CI·배포 없음.
- 본인 기존 열린 PR과 #7449 중복 PR 없음으로 확인하고 생성했다. 원격 push·PR 생성은 사용자가 명시 승인했다.

## 검증과 근거

[결과보고](../../report/task_m100_7449_report.md)와
[매뉴얼](../../manual/chrome_edge_extension_build_deploy.md#310-선택-실행-증적-보고서-7449)에 명령·검증·미검증·공유 절차를 기록했다.

- code candidate commit 후 동일 Node 계약: 32 PASS / 0 FAIL / 0 SKIP.
- 수정 전 base run.mjs에 같은 실행 소속 assertion: runId undefined로 FAIL. 수정 후 PASS.
- pass/fail/error/not-run/incomplete fixture, schemaVersion·후보 inventory·도구 SHA·기대 해시,
  로그/PNG/JSON 누락·오류·다른 실행·symlink·불완전 입력·공유 제외·수동 기록·동일 CLI 재현을 확인했다.
- error/not-run과 수동 미확인을 성공으로 바꾸지 않는다. 해시 검사는 내부 일관성이지 출처 인증이 아니다.
- Chromium 151의 동일 HTML 표시와 1200px/390px overflow 없음 확인. 실제 확장 전체 E2E 재실행과 구분했다.
- code candidate의 merge-tree exit 0; tree `406b2005704c0de5e6073dfd1546c8f382df4ed5`는 code tree와 동일.
- commit 범위 `git diff --check`, 변경 3개 Markdown 상대 링크 검사 통과.
- CI·mergeable은 변동하는 참고값이며 이 문서로 완료를 단정하지 않는다. 최종 PR 본문/작업 반환의 최신 조회를 따른다.

## 공통 적용 여부와 미검증

- 조판 원칙: 비해당. 실행 증적과 로컬 보고서만 변경하며 렌더·조판·페이지·baseline 변경이 없다.
- 검증 입력 커밋 확인: 비해당. HWP/HWPX/PDF 입력을 사용하지 않고 테스트 코드에서 합성 JSON/PNG를 생성한다.
- Cargo·fresh WASM·Native/Visual Sweep: 비해당. Rust/Studio/제품 렌더/빌드 경로를 바꾸지 않았다.
- 실제 Chrome 확장 전체 E2E: 미실행. Chrome for Testing 다운로드가 EAI_AGAIN으로 실패했다.
  시스템 Chromium의 HTML 확인을 그 대체 증거로 승격하지 않는다.
- 스토어·Edge/Firefox·실제 사용자 문서 품질·배포 판단은 미검증이며 도구 보증 범위 밖이다.
- 보고서 읽기 편의성과 공유 정보 제외에 대한 사용자 판정: 미확인. 매뉴얼 예제로 직접 확인 후 수동 기록을 남긴다.
- merge·release·production 배포·이슈 종료·GitHub review/comment 게시를 수행하지 않는다.
