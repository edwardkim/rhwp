# Stage 1 완료 보고 — Task M100 #6791 공개 검증 범위 정렬

- Issue: [#6791](https://github.com/edwardkim/rhwp/issues/6791)
- 근거: baba9811의 [PR #6786](https://github.com/edwardkim/rhwp/pull/6786) 본문 「외부 기여자 검증 절차 확인」
- 계획: [수행계획](../plans/task_m100_6791.md), [구현계획](../plans/task_m100_6791_impl.md)
- 승인: 2026-09-06 사용자의 구현계획·Stage 1 진행 승인, 기록 commit `16e2ea171`
- 기준 devel: `ff1ce007b428547da74e0d6b7e9a196592c60ff6`
- 작업 브랜치: `codex/6791-contributing-validation`
- 상태: Stage 1 완료, 다음 단계 승인 대기. 전체 문서 개선·PR 준비 완료를 의미하지 않는다.

## 변경 결과

[CONTRIBUTING.md](../../CONTRIBUTING.md)의 상단 차단 안내, PR 전 체크리스트와 frontend 검증 절을 정렬했다.

1. 상단은 변경 범위 표로 연결한다. 해당 검증의 실패는 계속 push·PR 차단 사유로 유지한다.
2. Rust source, Rust test/baseline helper, renderer/WASM, Studio 단독, npm/editor, 혼합, 기존 fixture data,
   문서 변경을 구분한다. Cargo·toolchain·Rust 빌드 설정 변경은 Studio 단독으로 분류하지 않는다.
3. Rust lint는 전체 fmt check, native root Clippy, WASM32 lib Clippy, workspace build와 all-target Clippy를
   유지한다. 테스트 전용 변경도 이 묶음을 생략하지 않는다. 조건부 무생성 policy 검사와 전체 nextest
   회귀를 적용 범위별 코드 블록으로 나누고 중복된 축약 Clippy 명령을 제거했다.
4. 기여자 본인도 원본 commit의 별도 review worktree를 만들 수 있다고 명시했다. 파생 파일의 PR 미제출
   계약을 유지하고, 자세한 복사 가능한 worktree 명령은 Stage 2에서 작성한다.
5. frontend는 의존성 설치 → 해당 commit의 fresh WASM → TypeScript·단위·production build → 관련 E2E·실제
   브라우저 확인으로 연결했다. WASM 준비와 Rust 전체 lint·회귀 의무를 구분했다.
6. 검증 SHA·실제 명령·결과·해당 없음 사유를 기록하도록 했다. 실패나 미완료를 PASS로 쓰지 않는다.
7. 상단과 체크리스트에서 내부 local_validation 해석을 요구하던 링크를 공개 문서 안의 범위·회귀 안내로
   대체했다. CI의 최신 required checks는 그대로 충족해야 한다고 명시했다.

## 검증 결과

| 확인 | 결과 |
| --- | --- |
| `python3 scripts/check_markdown_links.py CONTRIBUTING.md` | exit 0, 상대 링크 이상 없음 |
| 같은 문서 내부 anchor와 실제 heading 대조 | 8개 참조 모두 대상 존재 |
| `git diff --check` | exit 0 |
| local_validation 4.3과 범위 표·Rust lint 명령 대조 | fmt·세 Clippy, helper의 focused/snapshot, renderer 추가 검증 유지 |
| `.github/workflows/ci.yml` frontend package gate 대조 | fresh WASM이 frontend 검사에 선행함을 확인 |
| 공개 변경 범위 확인 | CONTRIBUTING.md만 수정. PR 템플릿은 Stage 2 대상 |

실제 검사 명령은 위 문서 검사다. 이번 단계에서 Rust·Studio build/Clippy/회귀를 실행하지 않았다.
수행계획 전 clean worktree 실험 결과를 재사용해 원인 근거로 삼았으며, 최종 공개 prepare·fmt·manifest
명령 검증은 Stage 3에 남아 있다. source·Cargo·CI·generated 파일은 변경하지 않았다.

## 다음 단계

Stage 2에서는 Rust 검증 worktree의 복사 가능한 준비 명령, 원본 commit → 검증 → push 순서,
fmt 실패 후 원본 보정·재검증 절차, 회귀·포맷 정책 및 PR 템플릿을 정렬한다. 지금 남아 있는 quickstart,
포맷 정책과 PR 템플릿의 기존 표현은 그 단계에서 함께 수정한다. 따라서 이 중간 commit을 제출 준비가
완료된 후보로 취급하지 않는다.

Stage 1 변경과 이 보고서를 함께 로컬 commit하고 Stage 2 진행 승인을 요청한다. #6786에는 원격 조치를
하지 않았으며, #6791의 push·PR 생성·merge·이슈 close도 아직 수행하지 않았다.
