---
kind: guide
status: active
canonical: mydocs/tech/archive/README.md
last_verified: 2026-07-16
---

# Historical 기술 문서

이 디렉터리는 현재 구현·계획의 권위 문서가 아닌 이전 roadmap, 대체된 설계, 당시 기준선의 보존본을 둔다.
새 작업의 근거로 직접 사용하지 말고, 연결된 현행 canonical 문서와 현재 `devel` 상태를 먼저 확인한다.

## 보존 문서

- [v1 개발 로드맵 백업](dev_roadmap_v1_backup.md): 2026-02-10 기준.
- [2026-03-24 개발 로드맵](dev_roadmap_20260324.md): 당시 기능 상태와 목표 수치의 snapshot이다.
- [프로젝트 비전](project_vision.md), [웹기안기 대체 전략](webgian_replacement_strategy.md):
  2026-02의 제품·시장 가설과 일정이다.
- [직접 인쇄 아키텍처](direct_printing_guideline.md): 구현 전 B-009 제안이며 현재 기능 계약이 아니다.
- [미주 순차-flow](endnote_seq_flow_redesign.md), [단일 패스 레이아웃](single_pass_layout_design.md):
  당시 레이아웃 개선 설계다. 현재 계약은 [렌더링 엔진 설계](../rendering_engine_design.md)를 따른다.
- [Hexagonal Architecture 검토](hexagonal_architecture_review.md): 2026-03의 전환 검토다. 현재 책임 경계는
  [파서 아키텍처](../parser_architecture.md)를 따른다.
- [HWP 오픈소스 비교](hwp_errata_public_comparison.md), [hwpers 분석](hwpers_analysis.md):
  작성 당시 외부 프로젝트 비교 snapshot이다.
- [VS Code 확장 설계](vscode_extension_design.md): 초기 읽기 전용 구조 기록이다. 현재 기능은
  [확장 README](../../../rhwp-vscode/README.md)를 따른다.
- [all-in-one-parser 시각 정합화 사전 전략](all_in_one_parser_fidelity_strategy.md): 이슈 채번 전 작성된
  전략 스냅샷이다.
- [문단 부호 표시 구현 계획](return_cat.md): 당시 구현 상태와 계획을 기록한 보존본이다.
- [iPad 네이티브 앱 브레인스토밍](brainstorm_ipad_app.md): 2026-04-09의 사고 실험과 시장 가설을
  기록한 snapshot이다.
