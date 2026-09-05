> **PR base 브랜치가 `devel` 인지 확인해주세요** (`main` 아님 — GitHub 기본 선택이 main 일 수 있습니다).
> 작업 브랜치는 최신 `upstream/devel` 에서 생성합니다. 상세: [CONTRIBUTING.md](../CONTRIBUTING.md)

## 변경 요약

이 PR이 해결하는 문제와 변경 내용을 간결하게 설명해주세요.

## 관련 이슈

closes #

## 테스트

<!-- 해당하는 항목만 선택하고 실행 명령·결과를 적어주세요. 해당 없음은 사유를 적고, 실패·미실행을 PASS로 표시하지 마세요. -->

- 변경 범위: <!-- Rust / Studio 단독 / 혼합 / package / 문서 / 기타 -->
- 검증한 commit SHA:
- 실행 명령·결과 및 해당 없음 사유:

- [ ] [변경 범위별 필수 검증](../CONTRIBUTING.md#pr-전-체크리스트)을 수행하고, 제출 HEAD가 검증한 commit과 같음을 확인
- [ ] Rust source·test/baseline helper·Rust 검증 입력 변경 시: [별도 worktree 준비](../CONTRIBUTING.md#rust-검증-worktree-준비와-실행) 후 `cargo fmt --all -- --check`, native·WASM32·workspace all-target Clippy 통과
- [ ] Rust 변경 시: 범위에 해당하는 focused·전체 integration·Native Skia 회귀 및 시각 검증 수행
- [ ] 새 integration test는 원본을 `tests/cases/*.rs`에만 추가했고 `tests/generated/`, `tests/suites/manifest.json`, 일반 PR의 Cargo generated test target을 포함하지 않음 (`--sync-cargo-targets` 메인터너 registry PR은 marker 블록만 예외)
- [ ] `src/**` 또는 `crates/*/src/**`의 `#[cfg(test)]` 변경 시: `node scripts/rust-unit-test-tiers.mjs --check` 통과 (무생성 검사)
- [ ] Studio 변경 시: [fresh WASM 준비 → TypeScript·단위·production build](../CONTRIBUTING.md#프런트엔드-변경-검증) 통과, 브라우저 동작 변경 시 관련 E2E·실제 동작 확인 · 명령/결과:
- [ ] npm/editor 변경 시: 해당 package·embed 계약 검사 수행
- [ ] 문서 변경 시: `git diff --check`, 링크·내용 정합성과 변경한 실행 절차 확인
- [ ] (에이전트 보조 작업 권장) 작업 증빙 첨부 — `rhwp replay --capsule` 영수증 캡슐 또는 관련 `--json` 봉투 원문 ([AGENTS.md 작업 증빙 절](../AGENTS.md#작업-증빙--에이전트-기본-경로-권장))
- [ ] `.claude/agents/`, `.claude/skills/`, `.agents/skills/` 변경 시: [capability 카탈로그](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/agent_capability_registry.md)의 등록·검증 규칙을 반영

> Rust 검증이 필요하면 기여자 본인이 원본 commit의 별도 review worktree를 만들어
> `--prepare` → fmt·lint·해당 회귀 → manifest `--check` 순서로 검증합니다. source 제출 checkout에서
> 생성하지 않으며, 파생 파일은 PR에 포함하지 않습니다. suite 누락으로 실패한 fmt를 PASS로 기록하지 마세요.
> Studio 단독 변경은 frontend 검증, 혼합 변경은 두 범위를 모두 적용합니다. 최신 GitHub required checks도
> 충족해야 합니다. 상세 순서는 [공개 검증 절차](../CONTRIBUTING.md#rust-검증-worktree-준비와-실행)를 따릅니다.

## 성능 영향 및 측정 결과 (해당하는 경우)

- 예상 영향: <!-- 개선 / 회귀 가능성 / 영향 없음 / 미확인 -->
- 재현·측정: <!-- 공개 sample, 명령, 환경, 변경 전후 관측값. 측정 환경이 없으면 "미측정" -->

> 특정 장비의 절대 성능 수치나 메인테이너 전용·비공개 벤치마크 통과는 PR 제출 조건이 아닙니다.
> 공개된 결정적 성능 회귀 테스트와 GitHub required checks는 기존과 같이 적용됩니다.

## 스크린샷

변경 전후 비교가 필요한 경우 첨부해주세요.
