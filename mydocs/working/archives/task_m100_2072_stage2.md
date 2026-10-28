# Task M100 #2072 Stage 2 - 문서 클러스터 현행성 감사

## 목표

Stage 0의 링크 기준선과 Stage 1의 문서 지도를 바탕으로, 파일명만으로 문서를 이동하지 않고 실제
역할, 참조 관계, 이력 호환 위험을 기준으로 첫 이동 클러스터를 선정한다.

## 감사 기준

| 기준 | 확인 방법 | 이동 판단 |
| --- | --- | --- |
| 역할 | 제목, 목적, 사용 명령, 결과물 | 같은 반복 작업을 안내하는 문서만 한 클러스터로 묶는다. |
| 현행성 | 권위 문서의 상호 링크, 대체 표기 | historical 자료는 현행 절차와 분리한다. |
| 참조 위험 | 저장소 전체 Markdown 링크와 루트 안내 문서 | 외부 이력에서 널리 참조되는 옛 경로만 redirect 후보로 둔다. |
| 검증 가능성 | 상대 링크 검사와 diff 검사 | 이동 커밋에서 모든 새 내부 링크를 새 경로로 바꾼다. |

## `manual/` 감사 결과

| 클러스터 | 현재 문서 | 판정 | 후속 처리 |
| --- | --- | --- | --- |
| Codex/기억 | `codex/`, `memory/` | 이미 역할별 하위 디렉터리가 있고 부트스트랩 경로와 결합됨 | 이번 구조 변경에서 이동하지 않음 |
| 작업·PR 절차 | `codex/docs_and_git_workflow.md`, `pr_review_workflow.md`, `dev_environment_guide.md` | 현재 권위 문서가 명확함 | README 링크만 유지, 이동은 별도 판단 |
| CLI | `cli_commands.md`, `dump_command.md`, `export_png_command.md`, `ir_diff_command.md`, `rhwp_cli_skill_guide.md` | 명령 단위 안내가 흩어져 있으나 서로 다른 이용자를 가짐 | 내용 중복 감사를 먼저 수행 |
| 검증 | `visual_sweep_guide.md`, `visual_verification_governance.md`, `object_visual_regression.md`, `roundtrip_fidelity_harness.md`, `svg_regression_diff.md`, `visual_clipping_detector.md`, `hangul_pdf_baseline.md`, `hangul_page_oracle.md`, `verify_pi_page_vs_hangul.md` | PDF/SVG/페이지·개체 비교라는 공통 목적과 상호 링크가 확인됨 | 첫 이동 후보. 경로 호환 조사 후 `verification/`으로 이동 |
| 확장·플랫폼 | `browser_extension_dev_guide.md`, `chrome_edge_extension_build_deploy.md`, `vscode` 관련 안내 | 제품별 배포 경로가 달라 단순 병합 위험 | 현행성·중복 감사 후 별도 stage |

## `tech/` 감사 결과

| 클러스터 | 현재 문서 | 판정 | 후속 처리 |
| --- | --- | --- | --- |
| 포맷 스펙 | `한글문서파일형식_5.0_revision1.3.md`, `hwp_spec_errata.md`, `hwp_ctrl_data.md`, HWPX 참조 문서 | 장기 참조·정정 관계가 있음 | `spec/` 이동은 정오표와 원문 링크를 함께 검증하는 독립 stage |
| 렌더링·조판 설계 | `rendering_engine_design.md`, `document_ir_lineseg_standard.md`, `table_layout_rules.md`, `font_fallback_strategy.md` | 장기 계약·설계 문서 | README에서 진입점으로 유지, 대량 이동 금지 |
| 기술 결정 | `thorvg_decision.md`, 일부 아키텍처 검토 | 선택 이유와 대안 비교를 보존 | 결정·조사 경계 감사 뒤 선택 이동 |
| 이슈 조사 | `task_m100_*`, `*_root_cause`, `*_diagnosis`, `*_investigation` | 파일명만으로는 장기 계약 여부를 판정할 수 없음 | 내용·참조 빈도·현행성으로 개별 분류 |
| 과거 계획 | `dev_roadmap_v1_backup.md` | `dev_roadmap.md`에서 이전 로드맵으로 명시됨 | historical로 표시하되, 이번 stage에서는 경로 유지 |

## 첫 이동 클러스터 선정

다음 Stage는 `manual/verification/`만 대상으로 한다.

- 후보 문서는 시각 비교, 페이지 오라클, roundtrip, 개체 geometry 회귀처럼 검증 절차를 직접 설명한다.
- `visual_sweep_guide.md`와 `visual_verification_governance.md`는 루트 `CONTRIBUTING.md`와 여러 PR/보고서에서
  참조된다. 이동 여부와 별개로 기존 경로 redirect 필요성을 개별적으로 확인한다.
- 과거 `plans/`, `working/`, `report/`, `pr/archives/`의 역사적 링크는 새 내부 참조를 만들지 않는 한 일괄 수정하지
  않는다. 그 대신 이동 전용 stage에서 허용된 redirect 목록을 문서화한다.

## 제외 범위

- 내용이 비슷해 보인다는 이유만으로 문서를 합치거나 삭제하는 작업
- `tech/`의 `task_m100_*` 파일 일괄 이동
- 과거 이슈·PR 본문을 보존하기 위한 GitHub 외부 URL의 자동 치환

## 다음 단계

1. 검증 문서군의 내부·루트 안내 링크를 전수 확인한다.
2. 실제 외부 이력 호환이 필요한 옛 경로만 redirect allowlist로 확정한다.
3. `git mv`와 링크 갱신만 포함한 독립 이동 stage를 수행한다.
