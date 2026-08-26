---
kind: pr-review
status: accepted-pending-trailing-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-25
---

# PR #6071 self-review — 한컴 호환 화면 확대·쪽 모양·쪽 이동 (#6039)

## 라우팅

- base route: `collaborator_self_merge.md`
- modifiers: `intake_and_review.md`, `local_validation.md`, `visual_fixture_evidence.md`,
  `review_only_fast_pass.md`, `rework_and_exceptions.md`
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`와 위 자식 문서
- current code candidate: `09c3e6559dca64c148c244a615643ce16d82d25d`

작성자 self-review이므로 reviewer는 지정하지 않았다. 1,000줄을 넘는 대형 PR이어서 즉시 merge하지 않고,
code candidate와 후행 review-only head의 CI를 각각 확인한 뒤 작업지시자 판단을 받는다.

## 접수 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#6071](https://github.com/edwardkim/rhwp/pull/6071) |
| Issue | [#6039](https://github.com/edwardkim/rhwp/issues/6039) |
| 작성자·검토자 | [@postmelee](https://github.com/postmelee) collaborator self-review |
| base | `devel@898e75930` |
| code candidate | `09c3e6559dca64c148c244a615643ce16d82d25d` |
| code candidate 확정 상태 | Open, `MERGEABLE/CLEAN`, reviewer 미지정, CI 13 success/12 policy skip |
| 규모 | 48 files, +3,356/-123, 13 commits |

## 결론

**수용 가능 — code candidate CI 녹색, review-only trailing head의 최신 GitHub Actions와
작업지시자 merge 승인 조건부**.

기존 자동 다중 열을 `자동`으로 보존하면서 배율과 독립적인 쪽 배치 상태, 한컴식 확대/축소 대화상자,
세로·가로 쪽 이동, 상태 표시줄 배율 조작을 하나의 사용자 보기 계약으로 연결했다. 문서 모델·undo·
HWP/HWPX 직렬화에는 보기 설정을 넣지 않았다.

## 이슈 완료 조건 대사

| #6039 완료 조건 | self-review 판정 |
| --- | --- |
| 자동 기본값과 기존 저배율 다중 열 보존 | `PageArrangement.auto` 정규화와 기존 50% 임계값 테스트로 고정 |
| 한 쪽·두 쪽·맞쪽·여러 쪽 배치 | 행/열·맞쪽 빈 슬롯·1×1~8×8 배율 계약 테스트 통과 |
| 상태 표시줄과 보기 메뉴의 같은 설정 상태 | 공통 확대/축소 명령과 대화상자 통합 테스트 통과 |
| 현재 쪽·클릭·PageUp/PageDown·앵커 정합 | 행 첫 쪽·키 이동·클릭 좌표·줌 앵커 focused test 통과 |
| 문서 dirty·저장 데이터 비영향 | 사용자 설정 저장·보기 전용 이벤트 테스트 통과 |
| 한컴식 쪽 이동과 배율 조작 | 가로 축 잠금, 10~500%, 중앙 100% 스냅, 플랫폼 단축키 검증 |

## 변경 범위와 대형 PR 판단

- 상태·순수 레이아웃, CanvasView 전환, 대화상자 UI, 쪽 이동, 상태 표시줄 보정을 10개 단계와 독립
  commit으로 나눴다. 각 commit은 계획·focused test·단계 보고와 대응한다.
- UI만 별도 PR로 떼면 상태 모델·레이아웃·좌표 계약 없이 동작하지 않고, 레이아웃만 떼면 사용자 선택 및
  저장 계약이 노출되지 않는다. #6039 수용 기준을 동작 가능한 수직 단위로 유지한다.
- 자동 줌 중 Canvas 교체, 적응형 렌더 해상도, 페이지 캐시·프리페치는 각각 #6040, #6041, #6042로
  분리해 이 PR의 성능 범위를 확장하지 않았다.
- 최신 `upstream/devel@898e75930`을 충돌 없이 병합했고 병합 후 전체 Studio 검증을 다시 수행했다.

## 렌더·시각 영향 판정

`CanvasView`와 `VirtualScroll`의 브라우저 내 페이지 배치·가시 범위가 바뀌므로 사용자-visible layout
변경이다. 다만 문서 내부 paint, PDF/SVG export, WASM renderer, HWP/HWPX fixture와 기준 PDF는 바꾸지 않아
PDF visual sweep은 이번 주장과 맞지 않는다. 대신 실제 로컬 브라우저에서 다음을 확인했다.

- 상태 표시줄의 폭 맞춤·쪽 맞춤·축소·배율 가로바·확대·통합 배율 버튼 6개 노출
- `자동`, `한 쪽`, `두 쪽`, `맞쪽`, `여러 쪽`, `세로 방향`, `가로 방향` 설정 노출과 적용
- 단일·다중 페이지 가로 이동에서 우세 휠 축의 수평 잠금
- 10~500% 가로바, 100% 중앙 스냅, 통합 배율 버튼의 68px 고정 폭과 2px 내부 간격

검증 상세 수치와 단계별 근거는
[`task_m100_6039_report.md`](../../report/task_m100_6039_report.md)의 브라우저 검증에 있다. visual sweep
asset을 merge 판단에 사용하지 않았으므로 별도 PDF·review PNG는 추가하지 않았다.

## 로컬 검증

code candidate `09c3e6559`와 최신 `devel` 병합 tree에서 다음을 완료했다.

| 명령·검증 | 결과 |
| --- | --- |
| `npx tsc --noEmit` | 통과 |
| `npm test` | 1,123 pass, 1 skip, 실패 0 |
| `npm run build` | 통과 |
| 실제 브라우저 UI·대화상자·배치·스크롤 확인 | 통과 |
| `cargo fmt --all` | review worktree에서 generated suite 32개 준비 후 통과 |
| `cargo fmt --all -- --check` | 같은 review worktree에서 통과 |
| `git diff --check upstream/devel...HEAD` | 통과 |

Rust source, WASM renderer, fixture 변경이 없어 release-test, Native Skia, clippy, PDF/SVG export는 실행하지
않았다. `tests/generated/`, suite manifest와 Cargo generated target은 source PR에 포함하지 않았다.

## GitHub Actions

code candidate `09c3e6559` 기준 GitHub Actions는 13 success, 12 policy skip, 실패·대기 0으로
완료됐다. PR은 `MERGEABLE/CLEAN`이고 조회한 head SHA는 code candidate와 일치한다.

- [CI](https://github.com/edwardkim/rhwp/actions/runs/32860101834): Frontend package gates와
  Build & Test aggregate 성공, Rust·WASM·Native Skia 중량 lane은 frontend-only 영향 분류에 따라 skip
- [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/32860101407): Canvas visual diff와
  selected CanvasKit readiness gate 성공
- CodeQL JavaScript/TypeScript·Python·Rust, proptest roundtrip, adapter inter-diff 성공

후행 commit은 이 review 문서와 오늘할일만 변경하므로 `review-only fast pass` 후보다.
최종 merge 조건은 후행 head의 preflight·required aggregate와 `MERGEABLE/CLEAN` 재확인이다.

## 위험과 후속 범위

- 자동 열 경계를 통과하는 핀치 줌은 여전히 정착 시 Canvas 교체 비용이 있다: #6040.
- 작은 배율에서도 고해상도 Canvas를 만들 수 있다: #6041.
- 대량 페이지를 반복 스크롤할 때 렌더 제거·재생성이 발생할 수 있다: #6042.
- `쪽 윤곽`은 실제 용지 경계 렌더 의미가 필요한 별도 기능으로 남겼다.

## 남은 절차

1. 이 review 문서와 오늘할일만 single-parent trailing commit으로 push한다.
2. 최신 trailing head의 preflight·Build & Test aggregate와 review-only fast-pass 판정을 확인한다.
3. 최신 head의 `MERGEABLE/CLEAN`, required checks와 작업지시자 승인을 다시 확인한 뒤 merge를 판단한다.
