## 변경 요약

실사용 에이전트가 rhwp 기여를 공식 절차대로 완주하도록
`.claude/skills/rhwp-contributor/` 를 레시피·예외·픽스처·계약 시험으로 고도화한다.
HARD GATE 는 `cargo fmt --all -- --check` 다. `cargo fmt --check` 는 낡은 표기다.

## 관련 이슈

closes #5322

## 테스트

- [x] `cargo fmt --all -- --check` 통과 (PR 생성·push 직전 필수. `cargo fmt --check` 만으로는 부족)
- [x] `cargo clippy -- -D warnings` (관련 범위)
- [x] 관련 `cargo test --test agent_contributor_skill_contract`
- [x] `python -m unittest scripts.tests.test_agent_contributor`
- [x] 시각 근거: N/A (렌더/레이아웃 변경 없음)
- [ ] 작업 증빙 `rhwp replay --capsule` (문서 편집이 있으면 권장)

## 성능 영향 및 측정 결과

- 예상 영향: 영향 없음
- 재현·측정: 미측정 (스킬·문서·계약 시험)
