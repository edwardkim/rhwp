# rhwp 작업 지침

이 파일은 저장소 안에서 재현 가능한 작업 부트스트랩이다. 세부 절차는 아래 권위 문서를 우선한다.

> **에이전트 진입점**: rhwp 를 도구로 부리는 에이전트는 루트 [`llms.txt`](llms.txt)와
> [에이전트 지식 지도](mydocs/manual/agent_knowledge_map.md)에서 시작한다.

## 문서 로딩 순서

1. `CLAUDE.md`
2. Codex는 프로젝트 메모리 덤프 `mydocs/manual/codex/MEMORY.md`
3. `mydocs/README.md`
4. 작업 성격에 맞는 `mydocs/manual/README.md` 또는 `mydocs/tech/README.md`
5. GitHub Actions·저장소 설정·branch protection·cache·runner 운영은
   `mydocs/manual/github_operations.md`
6. 개발·문서·Git 작업은 `mydocs/manual/codex/docs_and_git_workflow.md`
7. PR 검토·merge·후속 처리는 `mydocs/manual/pr_review_workflow.md`를 먼저 읽고,
   `mydocs/manual/pr_review/README.md`의 선택표가 지정한 기본·보조 자식 문서를 작업 전에 모두 읽는다.
   rhwp 첫 외부 contributor PR이면
   `mydocs/manual/pr_review/first_time_contributor.md`를 추가로 읽는다.
8. 로컬 빌드·WASM 검증은 `mydocs/manual/dev_environment_guide.md`,
   변경 범위별 검증 게이트는 `mydocs/manual/pr_review/local_validation.md`의 4.3
9. CLI 작업은 `mydocs/manual/cli_commands.md`
10. 시각 검증은 `mydocs/manual/verification/visual_verification_governance.md`와 `mydocs/manual/verification/visual_sweep_guide.md`

더 구체적인 문서가 이 요약과 다르면 그 문서를 따른다.

## 공통 원칙

- 구현 전에 관련 이슈, 기존 계획·보고서·트러블슈팅을 확인한다.
- 사용자 또는 다른 도구가 만든 변경은 임의로 되돌리거나 삭제하지 않는다.
- 작업 브랜치는 최신 `upstream/devel`을 기준으로 만들고, 일반 변경은 PR로 통합한다.
- collaborator·maintainer의 예외 처리와 오늘할일·PR review 문서는 `pr_review_workflow.md`의
  라우팅 결과에 해당하는 역할별 자식 절차를 따른다. 모 문서만 읽고 세부 경로를 추정해 진행하지 않는다.
- 작업 단계가 바뀌면 현재 단계의 변경을 커밋한 뒤 다음 단계 문서를 시작한다.
- GitHub comment, remote push, PR 생성은 사용자 승인을 받은 뒤 수행한다.
- Windows PowerShell에서 한글을 포함한 여러 단락의 PR·review·comment 본문은 here-string을
  `gh --body-file -`로 직접 pipe하지 않는다. UTF-8 **without BOM** 임시 Markdown 파일을
  `--body-file`로 전달하고, 게시 뒤 API로 한글·선두 BOM·`??` 치환 여부를 확인한다. 정확한
  명령과 정리 절차는 `mydocs/manual/pr_review_workflow.md`의 3.4.1을 따른다.

## 문서와 검증

- **Rust source 또는 Rust test/baseline helper를 바꾼 모든 PR·push 직전 필수**: 포맷만 확인하고
  Clippy를 CI에 넘기지 않는다. PR review worktree에서 파생 integration suite를 준비한 뒤 아래
  Rust lint 묶음을 **순차로** 모두 통과시킨다. `cargo clippy -- -D warnings`만으로는
  WASM 전용 cfg와 workspace member·integration target을 놓치므로 CI `Lint (fmt, clippy, WASM
  check)`의 세 Clippy 단계를 각각 확인한다.
  ```
  node scripts/rust-test-suite-manifest.mjs --prepare
  cargo fmt --all
  cargo fmt --all -- --check
  cargo clippy --locked --target-dir target/pr-review -- -D warnings
  cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown \
    --target-dir target/pr-review -- -D warnings
  cargo build --locked --workspace --target-dir target/pr-review
  cargo clippy --locked --workspace --all-targets --target-dir target/pr-review -- -D warnings
  node scripts/rust-test-suite-manifest.mjs --check
  ```
  새 integration test source를 추가한 경우 `--prepare`가 만든 파생 파일은 검증 뒤 review
  worktree에서만 복원하고 PR에 stage하지 않는다. 한 단계라도 실패하면 수정·재실행 전에는 push 또는
  PR을 만들지 않는다. 세부 범위와 예외는 `mydocs/manual/pr_review/local_validation.md`의 4.3을
  따른다.
- **source-side test 변경 시 추가**: `src/**`의 `#[cfg(test)]`를 변경하면
  `node scripts/rust-unit-test-tiers.mjs --check`를 실행한다. 이 검사는 source와 정책만 읽고
  파생 inventory를 만들지 않는다. 진단용 `--generate` 결과는 `tests/generated/unit-test-tiers.json`에
  남으며 커밋하지 않는다.
- **review·maintainer worktree와 CI 전용**: 새 integration source는 `tests/cases/` 원본만 PR에
  포함한다. `node scripts/rust-test-suite-manifest.mjs --prepare`와 manifest `--check`는
  파생 suite를 준비한 review worktree와 CI에서만 수행한다. generated suite·manifest는
  검증 증적일 뿐 source PR에 stage하지 않는다. 기본 `--prepare`는 root `Cargo.toml`을 바꾸지 않으며,
  통합 불가 예외 target registry를 갱신하는 메인터너 전용 PR만 `--sync-cargo-targets`로 Cargo marker
  블록을 동기화할 수 있다. 세부 절차는 `CONTRIBUTING.md`와
  `mydocs/manual/pr_review/local_validation.md`의 integration test 절을 따른다.
- 문서 역할·생명주기·canonical 관계는 `mydocs/README.md`의 manifest를 따른다.
- 문서 이동·정보구조 리팩토링의 링크와 메타데이터 검사는
  `mydocs/manual/markdown_link_check_guide.md`를 따른다. 일반 Markdown 추가·수정에는 자동 CI를 실행하지 않는다.
- 렌더링·레이아웃 변경은 시각 검증 정책에 따라 PDF/SVG 또는 동등한 근거를 남긴다.

## 작업 증빙 — 에이전트 기본 경로 (권장)

에이전트가 문서를 편집·변환했다면 판정 근거를 PR 본문에 남긴다 — 관련 `--json`
봉투 원문(종료 코드 포함)을 붙이고, 리뷰어는 주장 대신 재계산으로 확인한다.
증빙의 수준 정의는 [에이전트 작업 표준(AWS) 1.0](mydocs/tech/standards/agent_work_standard.md)을
따른다(기계용 정본: [`agent_work_standard.json`](mydocs/tech/standards/agent_work_standard.json)).
기여 절차 전체는 Claude Code 스킬 `rhwp-contributor` 가 체크리스트로 안내한다.
