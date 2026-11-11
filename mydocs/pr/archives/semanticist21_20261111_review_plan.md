# semanticist21 PR 2건 검토 계획

## 범위와 권한

2026-11-11 KST 작업지시자의 “semanticist21 기여자의 PR 처리를 시작합니다”에 따른 로컬 검토다.
원격 reviewer 지정은 접수 규칙에 따라 완료했다. 공개 댓글·push·PR 생성·merge 승인은 별도로 구분한다.
앞선 hyunmyungJaneLee 묶음의 원격 승인과 예외를 이 묶음에 적용하지 않는다.

- base route: maintainer_general
- modifiers: intake_and_review, local_validation, multi_pr_update_branch, post_merge
- loaded documents: pr_review_workflow.md, pr_review/README.md, maintainer_general.md,
  intake_and_review.md, review_template.md, local_validation.md 4.3, multi_pr_update_branch.md,
  post_merge.md, dev_environment_guide.md, browser_extension_dev_guide.md, e2e-cdp.md
- 고정 최신 devel: `e118de8abadd2dffb2798036b9ffd381b96ed7d5`.
- #8913: `66141a6c485844e37cdd133f7ac92890e9d8a2c3`, #8908 문단 부호 선택 사각형.
- #8915: `61198fcf3f4d185c8f78eac5b3259d99303b8845`, #8914 일반 SSR 페이지 DOM 주입.
- 두 건 모두 Ready·기존 reviewer 미지정이었다. `edwardkim` 지정 후 API 확인을 완료했다.

## 검증과 순서

1. 각 원 head의 diff·관련 이슈·호출 경로·독립 GUI 증거를 검토한다. 두 원 head의 CI 36개 check는
   success 또는 정상 skip·neutral이고 실패·진행 중 항목이 없다. 최신 base의 개별 병합 시뮬레이션도 clean이다.
2. 최신 base 위 검토 전용 branch에서 원 SHA를 보존해 누적 적용하고 source·test를 보정하지 않는다.
   원 head CI와 로컬 누적 검증 소스를 구분한다. 충돌이 생기면 보정 전에 해결 방침을 확인한다.
3. #8913은 선택 API 사각형 → WASM/native interaction → 편집기 선택 overlay 소비를 추적한다.
   조판·page tree·출력 paint의 소비 여부를 확인해 해당 검증을 결정한다. focused Rust 검사,
   실제 WASM/브라우저의 본문·빈 문단·끝점·줄바꿈·셀 선택을 확인한다.
4. #8915는 실제 확장 패키지의 일반 페이지 비간섭, 통합 opt-in, 동적 HWP 링크 배지와 Node 검사·build를 확인한다.
   가능하면 현재 base content script로 음성 대조를 실행한다. 성공한 제품 검사를 중복 실행하지 않는다.
5. 원 head를 고친 사실이 없고 current-base simulation이 clean이면 3.2.2의 정확한 녹색 GitHub head
   전체 회귀 재사용 조건을 원 PR별로 기록한다. 새 보정이 필요하면 새 head의 4.3 검증을 완료한다.
6. PR별 최종 판정·구체적인 후속 문안·미해결 사항을 로컬 기록한 뒤 필요한 원격 처리만 승인 범위에 맞춰 진행한다.

## 기존 작업 보존

기본 작업공간과 hyunmyungJaneLee·#8863·#8296 worktree를 변경·삭제하지 않는다.
빌드는 기본 작업공간 `target/pr-review`를 순차 공유하고, 임시 증적은 기본 작업공간의 ignored
`output/pr-review/semanticist21-20261111/`에 둔다. 필수 증거는 정식 mydocs 경로에 남긴다.
