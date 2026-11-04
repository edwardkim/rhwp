# PR #8363 검토 — fix(renderer): 칸 대각선 모양 값의 꼭짓점 비트만 대각선으로 그린다

## 최종 판정

**메인터너 보정 후 수용 가능 — 로컬 필수 검증 완료.** 원 기여 commit을 보존한 누적 후보에서 기능·전쪽 Native/fresh WASM·필수 회귀·lint를 완료했다. 원 PR의 오래된 head를 직접 강제 merge하는 판정이 아니며, 통합 후보의 원격 처리 승인과 해당 head CI를 선행한다.

## 접수와 범위

- 원 PR: https://github.com/edwardkim/rhwp/pull/8363, 작성자 `semanticist21`, Open·non-draft, base `devel`.
- 원 head: `9d6e420da55d4b9d9684c0e9c8f61cdf84ff29ff`, reviewer `edwardkim`, 지정 시각 `2026-11-05T00:27:02.842479+09:00`; API 재확인 완료.
- 실제 검증 base: `22947f6a90c998e95fa51238d5b907a0ebd37162`. 충돌 simulation: clean.
- [공통 수행 계획](semanticist21_20261105_review_plan.md). CI 참고값: 동일 원 head의 제출된 check에 failure/cancelled 없음.
- 기여자의 주장과 독립 측정·메인터너 보정 결과는 별도로 기록한다. 조판 원칙·입력 commit·전쪽 Native/fresh WASM·실행/반례와 로컬 gate는 아래 기록 및 [공통 최종 결과](assets/semanticist21-20261105/final-validation-summary.json)에 연결했다. 원 head의 CI 상태와 새 보정 후보의 실제 검증 결과를 혼동하지 않는다.

## 대각선 속성의 비트 계약

공통 border 해석에서 꼭짓점 대각선 bit `0b010`을 확인한다. 하위 필드 값 전체를 truthy로 읽어 중심선·꺾인선까지 꼭짓점 대각선으로 그리지 않는다. 다른 대각선 및 center-line 속성은 기존 해석을 유지한다. 공개 1쪽 문서의 각 shape code를 독립 Mac Print와 Native/fresh WASM에서 직접 대조했으며 양쪽 raw 100.00%다. 새 분기는 문서 ID·우연한 크기로 제한하지 않는다.

## 공통 최종 검증과 원격 조건

최종 runtime `2f44d7d0ab`에서 전체 12,499개·Native 공식 3종 4,064개·집중 21개·Studio 2,720개를 통과했다. Rust lint 묶음·TypeScript·production build·문서 예제·suite/unit 정책도 통과했다. [전체 검증 요약](assets/semanticist21-20261105/final-validation-summary.json), [최종 PNG와 해시](assets/semanticist21-20261105/representative-final-pngs.json), [원 기여 보존](assets/semanticist21-20261105/contribution-ancestry.json).

원격 push·새 통합 PR·merge·후속 댓글은 현재 아직 실행하지 않았다. 승인 뒤 해당 제출 head의 CI와 최신 base의 충돌 여부를 다시 확인한다.
