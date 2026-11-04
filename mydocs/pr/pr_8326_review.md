# PR #8326 검토 — fix(hwpx): 내보낸 HWPX의 칸 배경 그림과 HWP 그림 글머리표가 다른 그림을 가리키지 않게 한다

## 최종 판정

**머지 보류 — 검증 진행 중.** 동일 입력의 최신 base/누적 후보 실행과 필수 검증을 완료한 뒤 판정을 갱신한다. 이 잠정 상태를 기여자 재작업 결론이나 새 회귀 검출로 사용하지 않는다.

## 접수와 범위

- 원 PR: https://github.com/edwardkim/rhwp/pull/8326, 작성자 `semanticist21`, Open·non-draft, base `devel`.
- 원 head: `32c2a1e9febb303ac0662021279cf9171b346cb8`, reviewer `edwardkim`, 지정 시각 `2026-11-05T00:26:44.159666+09:00`; API 재확인 완료.
- 실제 검증 base: `22947f6a90c998e95fa51238d5b907a0ebd37162`. 충돌 simulation: CONFLICT (content): Merge conflict in src/model/style.rs; CONFLICT (content): Merge conflict in src/parser/hwpx/mod.rs; CONFLICT (content): Merge conflict in src/serializer/doc_info.rs; CONFLICT (content): Merge conflict in src/serializer/hwpx/header.rs.
- [공통 수행 계획](semanticist21_20261105_review_plan.md). CI 참고값: 동일 원 head의 제출된 check에 failure/cancelled 없음.
- 원 contributor의 주장과 독립 측정·메인터너 보정 결과는 별도로 기록한다. 조판 원칙·입력 commit·전쪽 Native/fresh WASM·실행/반례·필수 로컬 gate는 현재 검토 중이며 아직 완료로 판정하지 않는다.
