> **PR base 브랜치가 `devel` 인지 확인해주세요** (`main` 아님 — GitHub 기본 선택이 main 일 수 있습니다).
> 작업 브랜치는 최신 `upstream/devel` 에서 생성합니다. 상세: [CONTRIBUTING.md](../CONTRIBUTING.md)

## 변경 요약

이 PR이 해결하는 문제와 변경 내용을 간결하게 설명해주세요.

## 관련 이슈

closes #

## 테스트

- [ ] **`cargo fmt --all -- --check` 통과** (PR 생성·push 직전 필수. CI Lint Format check 와 동일. `cargo fmt --check` 만으로는 안 됨. 테스트만 고친 커밋도 다시 돌릴 것. 실패 시 `cargo fmt --all` 후 다시 `--check`)
- [ ] 새 integration test는 원본을 `tests/cases/*.rs`에만 추가했고 `tests/generated/`, `tests/suites/manifest.json`, 일반 PR의 Cargo generated test target을 포함하지 않음 (`--sync-cargo-targets` 메인터너 registry PR은 marker 블록만 예외)
- [ ] `src/**`의 `#[cfg(test)]`를 변경한 경우 `node scripts/rust-unit-test-tiers.mjs --check` 통과 (파생 inventory 생성·stage 불필요)
- [ ] `cargo test` 통과
- [ ] `cargo clippy -- -D warnings` 통과
- [ ] 관련 샘플 파일로 SVG 내보내기 확인
- [ ] 웹(WASM) 렌더링 확인 (해당하는 경우)
- [ ] rhwp-studio 편집·UI 변경 시: e2e 시나리오(`rhwp-studio/e2e/…`) 또는 편집 커맨드 리뷰 체크리스트(`mydocs/manual/edit_command_review_checklist.md`) 통과 · 링크:
- [ ] (에이전트 보조 작업 권장) 작업 증빙 첨부 — `rhwp replay --capsule` 영수증 캡슐 또는 관련 `--json` 봉투 원문 ([AGENTS.md 작업 증빙 절](../AGENTS.md#작업-증빙--에이전트-기본-경로-권장))
- [ ] `.agents/agents/`, `.agents/skills/`, `.agents/skills/` 변경 시: [capability 카탈로그](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/agent_capability_registry.md)의 등록·검증 규칙을 반영

> `node scripts/rust-test-suite-manifest.mjs --prepare`와 이어지는 `--check`는 파생 suite를
> 준비한 PR review worktree와 CI의 절차입니다. 일반 기여자의 source PR checkout에서는
> 실행하거나 결과를 커밋하지 마세요. 새 `tests/cases/*.rs` 원본은 검토·CI에서 weight 기준으로
> 자동 배정됩니다. 상세 절차는 [CONTRIBUTING.md](../CONTRIBUTING.md)를 따릅니다.

## 성능 영향 및 측정 결과 (해당하는 경우)

- 예상 영향: <!-- 개선 / 회귀 가능성 / 영향 없음 / 미확인 -->
- 재현·측정: <!-- 공개 sample, 명령, 환경, 변경 전후 관측값. 측정 환경이 없으면 "미측정" -->

> 특정 장비의 절대 성능 수치나 메인테이너 전용·비공개 벤치마크 통과는 PR 제출 조건이 아닙니다.
> 공개된 결정적 성능 회귀 테스트와 GitHub required checks는 기존과 같이 적용됩니다.

## 스크린샷

변경 전후 비교가 필요한 경우 첨부해주세요.
