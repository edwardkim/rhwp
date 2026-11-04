# PR #8349 검토 — fix(renderer): 아래 일점쇄선을 글자 크기와 원본 run 위상에 맞춤

## 최종 판정

**메인터너 보정 후 수용 가능 — 로컬 필수 검증 완료.** 원 기여 commit을 보존한 누적 후보에서 기능·전쪽 Native/fresh WASM·필수 회귀·lint를 완료했다. 원 PR의 오래된 head를 직접 강제 merge하는 판정이 아니며, 통합 후보의 원격 처리 승인과 해당 head CI를 선행한다.

## 접수와 범위

- 원 PR: https://github.com/edwardkim/rhwp/pull/8349, 작성자 `semanticist21`, Open·non-draft, base `devel`.
- 원 head: `ffc0550e8b9c6db8627f9a82cc7f0f3ff39a5415`, reviewer `edwardkim`, 지정 시각 `2026-11-05T00:26:55.219970+09:00`; API 재확인 완료.
- 실제 검증 base: `22947f6a90c998e95fa51238d5b907a0ebd37162`. 충돌 simulation: clean.
- [공통 수행 계획](semanticist21_20261105_review_plan.md). CI 참고값: Build & Test: failure, build-test-archive-b / Build test archive (b): cancelled, Lint (fmt, clippy, WASM check): cancelled.
- 기여자의 주장과 독립 측정·메인터너 보정 결과는 별도로 기록한다. 조판 원칙·입력 commit·전쪽 Native/fresh WASM·실행/반례와 로컬 gate는 아래 기록 및 [공통 최종 결과](assets/semanticist21-20261105/final-validation-summary.json)에 연결했다. 원 head의 CI 상태와 새 보정 후보의 실제 검증 결과를 혼동하지 않는다.

## 실제 head와 공통 소비 지점

PR 본문의 다른 변경 축을 실제 head에 포함된 것으로 취급하지 않았다. `ffc0550e8b`가 제공하는 아래 일점쇄선(shape 3)을 통합한다. 공통 `text_decoration`이 글자 크기와 원본 run에서 이미 소비한 길이로 기하·위상을 생산하고 Native/SVG/Canvas2D가 이를 소비한다. JSON 계약의 `topUnderline` 필드를 아래 장식에도 재사용하며 Studio CanvasKit도 같은 기하를 소비한다. 임의의 `bottomUnderline` 필드를 추가하지 않는다.

Hancom 2024 Print가 있는 공개 대조군 20개를 각각 1쪽 전체로 비교했다. Native/fresh WASM 모두 90% 이상이며 목표 shape 3은 양쪽 100.00%다. 실제 SVG browser 출력도 100.00%다. 실제 최종 WASM layer tree의 장식 32개(아래 공통 기하 16개)를 production `CanvasKitLayerRenderer.renderTextDecoration`과 CanvasKit 0.42 software surface에서 실행했고 unsupported operation은 없었다. 이 검사는 CanvasKit 전체 글자 글꼴 일치를 주장하는 검사가 아니다.

기준 PDF·Native·Canvas2D·CanvasKit의 같은 배율 확대에서 run 경계의 장단선 위상을 확인했다. 색 장식 부분의 2px raw 보조값은 세 backend 모두 99.83003%였다. Studio 전체 2,722개 중 2,720개 통과·기존 skip 2개, 해당 CanvasKit 집중 21개 통과, TypeScript·production build 성공을 실제 실행 기록에 보존한다. 기존 형광펜/다른 PR을 이 head의 개선으로 기록하지 않는다.

## 공통 최종 검증과 원격 조건

최종 runtime `2f44d7d0ab`에서 전체 12,499개·Native 공식 3종 4,064개·집중 21개·Studio 2,720개를 통과했다. Rust lint 묶음·TypeScript·production build·문서 예제·suite/unit 정책도 통과했다. [전체 검증 요약](assets/semanticist21-20261105/final-validation-summary.json), [최종 PNG와 해시](assets/semanticist21-20261105/representative-final-pngs.json), [원 기여 보존](assets/semanticist21-20261105/contribution-ancestry.json).

원격 push·새 통합 PR·merge·후속 댓글은 현재 아직 실행하지 않았다. 승인 뒤 해당 제출 head의 CI와 최신 base의 충돌 여부를 다시 확인한다.
