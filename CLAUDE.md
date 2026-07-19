# CLAUDE.md

이 파일은 Claude Code가 저장소의 권위 문서를 찾기 위한 짧은 부트로더다. 작업 절차와 명령을 이 파일에
중복 기록하지 않는다.

## 로딩 순서

1. 저장소 루트의 [`AGENTS.md`](AGENTS.md)
2. [`mydocs/README.md`](mydocs/README.md)
3. 작업 성격에 맞는 [`manual` 지도](mydocs/manual/README.md) 또는
   [`tech` 지도](mydocs/tech/README.md)
4. 각 지도가 지정한 canonical 문서

이 파일과 canonical 문서가 다르면 canonical 문서를 따른다.

## 프로젝트 개요

rhwp는 Rust로 HWP/HWPX/HWP3 문서를 읽고 편집·렌더링하며, WebAssembly로 브라우저에서도 동작하는
문서 엔진이다. 모든 포맷 파서는 공통 `Document` IR을 반환한다.

## 파일 포맷별 파서 구조와 HWP3 파서 규칙

파서 책임과 공통 IR 경계의 권위 문서는
[`mydocs/tech/parser_architecture.md`](mydocs/tech/parser_architecture.md)다. 특히 HWP3 전용 해석은
`src/parser/hwp3/` 안에서 끝내고 렌더러·레이아웃·문서 코어에 HWP3 전용 분기를 추가하지 않는다.

## 작업과 검증

- 문서·Git 작업: [`docs_and_git_workflow.md`](mydocs/manual/codex/docs_and_git_workflow.md)
- PR 리뷰·merge·후속 처리: [`pr_review_workflow.md`](mydocs/manual/pr_review_workflow.md)
- 로컬 빌드·테스트·WASM: [`dev_environment_guide.md`](mydocs/manual/dev_environment_guide.md)
- CLI 명령: [`cli_commands.md`](mydocs/manual/cli_commands.md)
- 시각 검증: [`verification/README.md`](mydocs/manual/verification/README.md)

## rhwp-studio UI 명칭과 CSS 접두어 규칙

UI 명칭과 CSS 접두어는
[`rhwp_studio_ui_conventions.md`](mydocs/manual/rhwp_studio_ui_conventions.md)를 따른다.
