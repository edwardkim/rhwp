# Stage 2 완료 보고 — Task M100 #6791 worktree와 원본 제출 순서

- Issue: [#6791](https://github.com/edwardkim/rhwp/issues/6791)
- 근거: baba9811의 [PR #6786](https://github.com/edwardkim/rhwp/pull/6786) 본문 「외부 기여자 검증 절차 확인」
- 계획: [수행계획](../plans/task_m100_6791.md), [구현계획](../plans/task_m100_6791_impl.md)
- 승인: 2026-09-06 사용자 “진행해줘”, 기록 commit `c210cb7e7`
- 기준 devel: `ff1ce007b428547da74e0d6b7e9a196592c60ff6`
- 작업 브랜치: `codex/6791-contributing-validation`
- 상태: Stage 2 문서 구현 완료, Stage 3 실검증·최종 보고·로컬 PR 준비 승인 대기

## 변경 결과

[CONTRIBUTING.md](../../CONTRIBUTING.md)와 [PR 템플릿](../../.github/pull_request_template.md)을 수정했다.

1. clean clone의 빠른 시작과 Fork 흐름을 원본 commit → 해당 SHA의 별도 worktree 검증 → 동일 HEAD push로
   연결했다. suite 준비 없이 전체 Cargo 검사를 시작하도록 하던 빠른 시작 예제를 제거했다.
2. 공개 Rust 검증 절을 ① worktree 준비·전체 fmt check ② 세 Clippy와 workspace build ③ 해당 policy·focused/
   전체/Native Skia 회귀 ④ manifest 확인·검증 SHA의 원본 제출로 구성했다. 필요한 도구, 셸, 실행 장소,
   같은 셸에서 변수 유지, 실패 시 중단, 다음 단계 조건을 설명했다.
3. 기여자 본인이 원본 commit의 별도 worktree에서 prepare할 수 있음을 명시했다. source checkout에서는
   생성하지 않고, generated harness·manifest를 stage하지 않는다. 기본 prepare와 maintainer 전용
   registry 동기화·rebalance의 경계를 유지했다.
4. suite 누락 실패와 실제 fmt diff를 구분했다. fmt 보정은 필요한 원본 변경만 source branch의 새 commit에
   반영한 뒤 다시 검증하며, dirty review worktree의 성공을 보정 전 HEAD의 성공으로 기록하지 않는다.
5. 포맷 명령 실행과 무관한 전체 포맷 정규화 변경 제출을 구분했다. 기능 PR에 전체 정규화를 섞지 않는
   기존 목적을 유지하고, ignored 파일이 `git restore`로 지워진다는 잘못된 정리 설명을 제거했다.
6. 회귀 파일명 관례를 `tests/cases/`와 일치시키고 Studio 회귀 위치를 별도로 적었다. 코드 스타일과
   렌더링 검증의 축약 lint 안내를 공개 Rust 검증 절과 연결했다.
7. 템플릿에서 범위·검증 SHA·실제 명령·결과·해당 없음 사유를 기록하게 하고 무조건 fmt·cargo test·native
   Clippy만 체크하던 항목을 범위별 검증으로 바꿨다. 실패·미실행을 PASS로 표시하지 않는 계약을 유지했다.

## 같은 범위 안의 추가 정정

기존 `cargo test --test svg_snapshot` 예제는 현재 Cargo target과 맞지 않았다. `Cargo.toml`은
`autotests = false`이며, `svg_snapshot`은 직접 등록된 target이 아니라 generated suite의 module이다.
따라서 선택 실행 예제를 기존 `run-rust-test.mjs svg_snapshot` wrapper로 정정했다. 이 수정은 요청한
회귀·포맷 정책 문단의 실행 가능성 정렬에 포함되며 변경 파일이나 CI 정책을 확장하지 않았다.

## 검증 결과

| 확인 | 실제 결과 |
| --- | --- |
| 공개 2파일 `check_markdown_links.py` | exit 0, 상대 링크 이상 없음 |
| 공개 2파일 내부·교차 anchor와 실제 heading 대조 | 21개 참조 모두 존재 |
| Rust 검증 절 bash 코드 블록 `bash -n` | 10개 구문 검사 통과. 명령 실행은 하지 않음 |
| `resolveCasePlan()`으로 회귀 이름 해석 | svg_snapshot, issue_2225_missing_picture_placeholder, render_p37_direct_pdf_export 모두 실제 suite로 해석 |
| `git diff --check` | exit 0 |
| Rust lint·Native Skia·WASM 표준/진단 명령 대조 | local_validation 4.3·개발 환경 가이드의 범위·명령 유지 |
| 변경 범위 확인 | 공개 2파일과 #6791 계획·보고 문서. Rust·Studio 제품 코드, Cargo, 생성기, CI 수정 없음 |

`resolveCasePlan()`은 `deriveManifest(... persist: false, report: false)`로 메모리에서 배정을 계산한
읽기 전용 확인이다. Cargo 회귀를 실행하거나 generated 파일을 준비하지 않았다. 이번 단계에서 전체
Rust/Studio build·Clippy·nextest는 실행하지 않았으며, 셸 구문 통과를 실검증 통과로 기록하지 않는다.

## 남은 작업과 승인

Stage 3에서는 이 단계 commit의 별도 clean worktree에서 공개된 준비·fmt·manifest 절차를 실제 실행하고,
tracked Rust·Cargo 파일의 SHA-256 불변과 generated 미제출을 확인한다. 그 결과, 최종 보고서와 실제
검증 SHA를 담은 PR 본문 초안·push/생성 명령을 로컬에 준비한다.

공개 예제 안의 push 명령은 이번 단계에서 실행하지 않았다. 원격 push·PR 생성·merge·이슈 close는
별도 승인 대상으로 유지하며 #6786에는 head·본문·review·comment·merge 변경을 하지 않았다.
