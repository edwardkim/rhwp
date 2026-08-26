---
kind: pr-review
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-19
---

# PR #5525 검토 - 한글 2024 조판 호환 플래그

## 접수 메타데이터

| 항목 | 검토 시점 참고값 |
| --- | --- |
| PR / 작성자 | [#5525](https://github.com/edwardkim/rhwp/pull/5525) / planet6897 |
| base / contributor head | `devel` / `0a51e845741b8f9108dc43bb31cac589566386f7` |
| 가시성 branch | `review/planet6897-5525-20260818` |
| source trailing branch | `review/planet6897-5525-source-20260819` → `feat/5524-hangul2024-compat` |
| local cherry-pick | `b4da74118a6de7bf8f4cc3e566d013570cdfc42b` |
| 원격 상태 | OPEN, 비 draft, MERGEABLE, CLEAN |
| 검토 기준 | `upstream/devel@c3c35306b1428a2dcd97656d1cbe4a8c74c780a7` 위에 PR head 적용 |

## 판정 요약

차단 결함은 발견하지 못했다. TAC 표 앵커 선행 세그먼트를 회수하는 Δ1 구현과 기본값 보존은
실샘플·focused test·GitHub 전체 검증에서 일치했다.

다만 **메인터너 보정이 필요하다.** 새로 추가한 `dump-pages --compat 2022|2024`가 실행되는데도
도움말, capability 자기서술, JSON 계약 필드, 사용자 매뉴얼이 기존 옵션만 선언한다. 특히
`--compat 2024`는 페이지 배치를 실제로 바꾸지만 JSON 봉투에는 선택한 호환 모드가 남지 않아,
기계 소비자가 결과의 조판 조건을 재현하거나 식별하기 어렵다.

## 주요 검토 사항

### 보완 필요: CLI 계약과 문서가 새 옵션을 선언하지 않음

`src/main.rs`의 `dump_pages` 파서는 `--compat 2022|2024`를 허용하지만 다음 표면은 갱신되지
않았다.

- `capabilities`의 `dump-pages` flags가 `-p`, `--respect-vpos-reset`, `--json`만 반환한다
  (`src/main.rs:6398-6410`). record fields에도 선택한 compat 모드가 없다.
- 일반 도움말과 인자 오류 usage가 `--compat`를 표시하지 않는다
  (`src/main.rs:7271`, `src/main.rs:16705`).
- 사용자 문서와 agent 조회 문서가 기존 옵션만 기록한다
  (`mydocs/manual/cli_commands.md:532`, `mydocs/manual/agent_codex/10_조회.md:545-547`,
  `mydocs/manual/agent_knowledge_map.md:800-804`).
- 신규 통합 테스트는 `HwpDocument::set_hangul2024_compat`의 native 경로만 검증하고 CLI
  usage/capabilities/JSON의 계약을 검증하지 않는다.

검토 중 빌드한 binary로 기본값과 `--compat 2022`의 JSON SHA-256은 동일했고, `--compat 2024`는
샘플의 `paraIndex=13`을 0기준 1쪽에서 0쪽으로 이동시켰다. 그런데 세 결과 모두 봉투에
`respectVposReset`만 있고 `compat` 또는 동등한 실행 모드 필드가 없다. `--compat 2024`가 공개
진단 계약이라면 capability flags·usage·위 문서에 옵션을 추가하고, JSON 봉투에 선택값을 기록하는
focused contract test를 추가하는 것이 필요하다. 이는 현재 동작을 막는 런타임 결함은 아니지만,
사용자와 자동화 도구가 새 기능을 발견·재현하지 못하게 하는 계약 드리프트다.

### 비차단 확인: WASM 선택 표면은 아직 없음

`DocumentCore::set_hangul2024_compat`는 native/CLI에서 사용할 수 있지만 `src/wasm_api.rs`의
`HwpDocument`에는 대응하는 `wasm_bindgen` 메서드가 없다. PR 본문이 신규 표면을 CLI/core로
한정하고 WASM 렌더링 변경을 범위 밖으로 명시했으므로 이번 PR의 차단 사유로 보지 않았다. 향후
rhwp-studio에서 동일 호환 모드를 선택해야 한다면 별도 WASM API·Studio 검증이 필요하다.

## 변경 범위

- `LayoutCompatibilityProfile`에 opt-in 한글 2024 레이아웃 축을 추가했다.
- `DocumentCore` 세션 설정과 유효 profile 합성을 추가하고 설정 변경 시 전체 페이지네이션을
  무효화한다.
- `TypesetState`가 TAC 앵커 선행 세그먼트 회수량을 추적하고 저장 vpos 경계 신호의 재적합에
  사용한다. 기본값은 기존 2022 계열 경로를 유지한다.
- 실제 HWP fixture와 한글 2024 호환 native 통합 테스트를 추가했다.
- PR 고유 diff는 11개 파일, 365 additions, 7 deletions이며 workflow 변경은 없다.

## 충돌 및 메인터너 보정

- `upstream/devel`에서 visibility review branch를 만들고 PR head를 단일 cherry-pick했다.
- cherry-pick 충돌은 없었다.
- 검토 문서는 코드 candidate 뒤 별도 trailing docs-only commit으로 정리해 원 PR source branch에
  push했으며, contributor 원 history는 rewrite하지 않았다. 검토용 통합 branch의 기록 head는
  `upstream/pr/devel-planet6897-5525-review`에도 보존되어 있다.
- 위 CLI 계약·문서 보정은 메인터너가 처리할 후속 보완으로 기록한다.

## 검증

- `git diff --check upstream/devel...HEAD` 통과
- `cargo fmt --all -- --check` 통과
- `node scripts/rust-test-suite-manifest.mjs --prepare` 후 `--check` 통과
- `node scripts/rust-unit-test-tiers.mjs --check` 통과
- `cargo nextest run --cargo-profile release-test --target-dir target/pr-review --test regression_suite_032 --filter-expr 'test(issue_5524_hangul2024_compat::hangul2024_compat_reclaims_tac_anchor_line_and_default_stays_2022)' --no-fail-fast`
  결과: 1 passed, 88 skipped
- CLI 실샘플 검증: 기본값 2쪽, `--compat 2022` 2쪽, `--compat 2024` 2쪽이며
  `paraIndex=13`은 각각 0기준 1쪽, 1쪽, 0쪽에 배치됐다. 잘못된 compat 값은 exit 2로
  거부됐다.
- 같은 PR head `0a51e845741b8f9108dc43bb31cac589566386f7`의 GitHub Full CI·CodeQL·Native
  Skia·Canvas visual diff가 모두 통과했다. WASM Build와 Frontend unit gates는 해당 변경
  모드가 아니어서 skipped였다.
- 링크된 [Canvas visual diff 실패 job](https://github.com/edwardkim/rhwp/actions/runs/32145385958/job/95739445933)은
  Chromium snapshot `1660786` 다운로드의 HTTP 403으로 실패했다. 같은 run의 후속
  [Canvas 재실행 job](https://github.com/edwardkim/rhwp/actions/runs/32145385958/job/95742812080)은
  통과했으므로 PR 코드 결함으로 판정하지 않았다.

## 결론

기능 구현과 기본값 무회귀는 검증되었고 차단 결함은 없다. merge 전 메인터너가
`dump-pages --compat`의 도움말·capability·JSON 계약·관련 매뉴얼을 정합화하고 CLI 계약 테스트를
보강하는 것을 권고한다. GitHub review·승인·merge·comment는 아직 수행하지 않았다.
