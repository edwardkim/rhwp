# rhwp 작업 지침

이 파일은 저장소 안에서 재현 가능한 작업 부트스트랩이다. 세부 절차는 아래 권위 문서를 우선한다.

## 문서 로딩 순서

1. `CLAUDE.md`
2. `mydocs/README.md`
3. 작업 성격에 맞는 `mydocs/manual/README.md` 또는 `mydocs/tech/README.md`
4. 개발·문서·Git 작업은 `mydocs/manual/codex/docs_and_git_workflow.md`
5. PR 검토·merge·후속 처리는 `mydocs/manual/pr_review_workflow.md`
6. 로컬 빌드·WASM 검증은 `mydocs/manual/dev_environment_guide.md`
7. CLI 작업은 `mydocs/manual/cli_commands.md`
8. 시각 검증은 `mydocs/manual/verification/visual_verification_governance.md`와 `mydocs/manual/verification/visual_sweep_guide.md`

더 구체적인 문서가 이 요약과 다르면 그 문서를 따른다.

## 공통 원칙

- 구현 전에 관련 이슈, 기존 계획·보고서·트러블슈팅을 확인한다.
- 사용자 또는 다른 도구가 만든 변경은 임의로 되돌리거나 삭제하지 않는다.
- 작업 브랜치는 최신 `upstream/devel`을 기준으로 만들고, 일반 변경은 PR로 통합한다.
- collaborator·maintainer의 예외 처리와 오늘할일·PR review 문서는 `pr_review_workflow.md`의 역할별 절차를 따른다.
- 작업 단계가 바뀌면 현재 단계의 변경을 커밋한 뒤 다음 단계 문서를 시작한다.
- GitHub comment, remote push, PR 생성은 사용자 승인을 받은 뒤 수행한다.

## 문서와 검증

- 문서 역할·생명주기·canonical 관계는 `mydocs/README.md`의 manifest를 따른다.
- 문서 이동·정보구조 리팩토링의 링크와 메타데이터 검사는
  `mydocs/manual/markdown_link_check_guide.md`를 따른다. 일반 Markdown 추가·수정에는 자동 CI를 실행하지 않는다.
- 렌더링·레이아웃 변경은 시각 검증 정책에 따라 PDF/SVG 또는 동등한 근거를 남긴다.
