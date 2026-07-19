---
kind: guide
status: active
canonical: mydocs/README.md
last_verified: 2026-07-17
---

# rhwp 문서 지도와 canonical manifest

이 문서는 저장소 문서의 진입점과 권위 관계를 기록한다. 상세 절차는
[manual 문서 지도](manual/README.md), 기술 근거는 [tech 문서 지도](tech/README.md)에서 찾는다.

## 시작 순서

1. 저장소 루트의 `AGENTS.md`와 `CLAUDE.md`를 읽는다.
2. 이 문서에서 작업 종류에 맞는 canonical 문서를 고른다.
3. 해당 문서가 가리키는 상세 가이드·기술 조사·트러블슈팅만 추가로 읽는다.

## 메타 규칙

- `kind`: 문서의 역할. `canonical`, `guide`, `reference`, `investigation`, `decision`, `snapshot`, `memory` 중 하나다.
- `status`: 생명주기. `active`, `historical`, `superseded` 중 하나다.
- `canonical`: 상세 문서가 따르는 권위 문서의 저장소 상대 경로다.
- `last_verified`: 문서의 역할, canonical 관계, 현재 진입점과 해당 감사에서 명시한 사실을 마지막으로
  확인한 날짜다. historical snapshot에서는 당시 사실을 현재 구현으로 다시 보증한다는 뜻이 아니다.

front matter는 `mydocs/manual`, `mydocs/tech`, `mydocs/troubleshootings`의 모든 Markdown과 이 문서에
필수다. 정보구조를 변경할 때는 디렉터리 단위 로컬 검사로 역할, 생명주기, canonical 경로와 확인일을
확인한다. 일반 Markdown 추가·수정에는 자동 CI를 실행하지 않는다.

## 채택한 실제 구조

이슈 초안의 디렉터리 이름을 먼저 만들고 문서를 일괄 이동하지 않았다. 문서 역할과 현행성을 내용으로
감사한 뒤 현재 탐색 비용을 실제로 줄이는 다음 경계만 채택했다.

- `manual/verification/`: 시각 검증 정책과 실행 가이드
- `manual/codex/`: 저장소 부트스트랩과 현행 문서·Git 절차
- `manual/memory/`: 과거 피드백과 memory 출처
- `tech/investigations/issue-####/`: 특정 이슈의 가설·실험·관찰
- `tech/archive/`: 대체된 계획·설계와 역사 자료
- `tech/webhwp/`: 특정 webhwp bundle 역분석 기록

`manual/workflow`, `manual/cli`, `manual/release`, `manual/api`, `tech/spec`, `tech/architecture`,
`tech/domains`, `tech/decisions` 같은 빈 분류 계층은 만들지 않았다. 루트 문서 지도와 front matter만으로
권위 관계가 분명한 문서는 기존 안정 경로를 유지한다. 이후 이동도 파일명 패턴이 아니라 내용과 참조
비용을 확인한 독립 커밋으로 수행한다.

## Canonical manifest

| 경로 | kind | status | canonical | last_verified |
| --- | --- | --- | --- | --- |
| [문서·Git 워크플로](manual/codex/docs_and_git_workflow.md) | canonical | active | `manual/codex/docs_and_git_workflow.md` | 2026-07-16 |
| [Codex 문서 지도](manual/codex/README.md) | guide | active | `manual/codex/README.md` | 2026-07-17 |
| [Claude memory dump 색인](manual/memory/MEMORY.md) | memory | historical | `manual/codex/docs_and_git_workflow.md` | 2026-07-17 |
| [PR 리뷰·통합 워크플로](manual/pr_review_workflow.md) | canonical | active | `manual/pr_review_workflow.md` | 2026-07-16 |
| [개발 환경 가이드](manual/dev_environment_guide.md) | guide | active | `manual/dev_environment_guide.md` | 2026-07-17 |
| [온보딩 가이드](manual/onboarding_guide.md) | guide | active | `manual/README.md` | 2026-07-17 |
| [배포 가이드](manual/publish_guide.md) | guide | active | `manual/publish_guide.md` | 2026-07-17 |
| [Hyper-Waterfall 문서 체계](manual/hyper_waterfall_docs_guide.md) | guide | active | `manual/codex/docs_and_git_workflow.md` | 2026-07-17 |
| [AI 페어프로그래밍 기록](manual/ai_pair_programming_guide.md) | reference | historical | `manual/codex/docs_and_git_workflow.md` | 2026-07-17 |
| [CLI 명령어 매뉴얼](manual/cli_commands.md) | canonical | active | `manual/cli_commands.md` | 2026-07-16 |
| [문서 링크와 메타데이터 로컬 검사](manual/markdown_link_check_guide.md) | guide | active | `manual/markdown_link_check_guide.md` | 2026-07-17 |
| [시각 검증 문서 지도](manual/verification/README.md) | guide | active | `manual/verification/README.md` | 2026-07-16 |
| [시각 검증 거버넌스](manual/verification/visual_verification_governance.md) | canonical | active | `manual/verification/visual_verification_governance.md` | 2026-07-16 |
| [HWP 2020 MCP 사용법](manual/mcp_hwp2020Convert_usage.md) | guide | active | `manual/mcp_hwp2020Convert_usage.md` | 2026-07-16 |
| [HWP 5.0 스펙 문서 정오표](tech/hwp_spec_errata.md) | canonical | active | `tech/hwp_spec_errata.md` | 2026-07-16 |
| [한글 문서 파일 형식 5.0 개정 1.3](tech/한글문서파일형식_5.0_revision1.3.md) | reference | active | `tech/hwp_spec_errata.md` | 2026-07-16 |
| [Document IR LineSeg 표준](tech/document_ir_lineseg_standard.md) | canonical | active | `tech/document_ir_lineseg_standard.md` | 2026-07-16 |
| [렌더링 엔진 설계](tech/rendering_engine_design.md) | canonical | active | `tech/rendering_engine_design.md` | 2026-07-16 |
| [표 레이아웃 규칙](tech/table_layout_rules.md) | canonical | active | `tech/table_layout_rules.md` | 2026-07-16 |
| [폰트 fallback 전략](tech/font_fallback_strategy.md) | canonical | active | `tech/font_fallback_strategy.md` | 2026-07-16 |
| [편집 action undo/redo 아키텍처](tech/edit_action_undo_redo_architecture.md) | canonical | active | `tech/edit_action_undo_redo_architecture.md` | 2026-07-16 |
| [포맷 파서와 공통 Document IR 경계](tech/parser_architecture.md) | canonical | active | `tech/parser_architecture.md` | 2026-07-17 |
| [ThorVG 결정 기록](tech/thorvg_decision.md) | decision | active | `tech/thorvg_decision.md` | 2026-07-16 |
| [이전 개발 로드맵](tech/archive/dev_roadmap_v1_backup.md) | snapshot | historical | `tech/archive/README.md` | 2026-07-16 |
| [이슈별 기술 조사 지도](tech/investigations/README.md) | guide | active | `tech/investigations/README.md` | 2026-07-17 |
| [트러블슈팅 문서 지도](troubleshootings/README.md) | guide | active | `troubleshootings/README.md` | 2026-07-16 |
| [rhwp-studio UI 명칭과 CSS 접두어](manual/rhwp_studio_ui_conventions.md) | reference | active | `manual/rhwp_studio_ui_conventions.md` | 2026-07-17 |

## Reference 자산

| 경로 | kind | status | canonical | last_verified |
| --- | --- | --- | --- | --- |
| [OWPML XML 스키마 reference](manual/owpml_schema_reference.md) | reference | active | `tech/hwpx_hancom_reference.md` | 2026-07-16 |

## 이동 규칙

문서 이동은 역할·현행성·참조 빈도를 검토한 독립 commit에서만 수행한다. 모든 내부 참조를 새 경로로
갱신하고, 외부 이력 호환이 필요한 옛 경로만 같은 commit의 redirect stub으로 남긴다. 기존 문서를 매번
자동 재검사하거나 migration 목록을 별도 파일에 하드코딩하지 않는다. 필요한 작업에서만
[문서 링크와 메타데이터 로컬 검사 가이드](manual/markdown_link_check_guide.md)에 따라 검사한다.

이슈별 조사 문서는 `tech/investigations/issue-####/`에 두며, 이슈별 기준선이나 진단이 장기 기술
계약으로 확정되면 해당 canonical 문서에 결론을 반영한다.
