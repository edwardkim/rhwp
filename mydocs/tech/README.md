---
kind: guide
status: active
canonical: mydocs/tech/README.md
last_verified: 2026-07-19
---

# tech 문서 지도

`mydocs/tech/`은 rhwp의 **기술 사실, 설계, 결정, 조사 근거**를 보존하는 문서 공간이다.
반복 작업 절차와 도구 사용법은 [manual 문서 지도](../manual/README.md)에서 찾는다.

## 권위 문서와 진입점

| 주제 | 권위 또는 우선 진입점 | 관련 상세 문서 |
| --- | --- | --- |
| HWP 5.0 구현 차이 | [HWP 5.0 스펙 문서 정오표](hwp_spec_errata.md) | [한글 문서 파일 형식 5.0 개정 1.3](한글문서파일형식_5.0_revision1.3.md), [HWP 제어 데이터](hwp_ctrl_data.md) |
| 포맷 파서와 공통 IR 책임 경계 | [포맷 파서와 공통 Document IR 경계](parser_architecture.md) | [HWP/HWPX IR 차이](hwp_hwpx_ir_differences.md) |
| IR 저장·직렬화와 무효화 계약 | [HWP 저장 기술 가이드](hwp_save_guide.md) | [직렬화 passthrough·무효화 계약](serialization_passthrough_contract.md) |
| HWPX와 HWP IR 차이 | [HWP/HWPX IR 차이](hwp_hwpx_ir_differences.md) | [HWPX 한컴 참조](hwpx_hancom_reference.md), [HWPX DVC 참조](hwpx_dvc_reference.md), [로컬 OWPML XML 스키마](../manual/owpml_schema_reference.md) |
| Document IR와 LineSeg 계약 | [Document IR LineSeg 표준](document_ir_lineseg_standard.md) | [Issue #310 LineSeg vpos 조사](investigations/issue-310/README.md), [HWPX LineSeg 검증](hwpx_lineseg_validation.md) |
| 렌더링 엔진 | [렌더링 엔진 설계](rendering_engine_design.md) | [Issue #516 다층 렌더링 후보 조사](investigations/issue-516/README.md), [Issue #124 캔버스·폰트 측정 조사](investigations/issue-124/README.md) |
| 표 레이아웃 | [표 레이아웃 규칙](table_layout_rules.md) | [HWP 표 렌더링](hwp_table_rendering.md), [Issue #101 부분 표 흐름 조사](investigations/issue-101/README.md) |
| 폰트 대체와 충실도 | [폰트 fallback 전략](font_fallback_strategy.md) | [Issue #124 캔버스·폰트 측정 조사](investigations/issue-124/README.md), [Issue #2125 font ownership 조사](investigations/issue-2125/README.md) |
| 편집 undo/redo | [편집 action undo/redo 아키텍처](edit_action_undo_redo_architecture.md) | 이슈별 실동작 조사 문서 |
| 기술 채택·비채택 | [ThorVG 결정 기록](thorvg_decision.md) | [Issue #112-115 ThorVG PoC 조사](investigations/issue-112/README.md) |
| 초소형 모델용 매크로 도구 축 | [초소형 모델용 매크로 도구 축 설계 결정](tiny_model_macro_tools.md) | 계약 테스트 `tests/digest_macro_contract.rs`, CLI 명령은 [CLI 명령 레퍼런스](../manual/cli_commands.md) 재확인 |
| OLE chart renderer | [OLE chart renderer 선택 결정](hwp_ole_chart_renderer_architecture_decision_1251.md) | [Issue #1251 시각 차이 조사](investigations/issue-1251/README.md) |
| CI cache 정책 이력 | [Issue #1664 cache 정책 결정](ci_cache_policy_1664.md) | 현재 동작은 `.github/workflows/ci.yml` 재확인 |
| WASM toolchain 버전 | [wasm-pack 버전 고정 정책](wasm_pack_version_policy.md) | 현재 설치 동작은 `.github/actions/install-wasm-pack/action.yml`과 `Dockerfile` 재확인 |
| 에이전트 표면 내성 설계 | [경량 에이전트 내성 — CLI·MCP 계약 확장 4건](weak_agent_proofing.md) | [에이전트 실패 사전](../manual/agent_troubleshooting_guide.md), [에이전트 표면 플레이북](../manual/agent_surface_playbook.md) |
| 에이전트 표면의 층과 순서 | [에이전트 아키텍처 지도](agent_architecture/README.md) | [4층 모델](agent_architecture/layer_model.md), [로드맵 지도](agent_architecture/roadmap_atlas.md), [불변식](agent_architecture/invariants.md), [결정 이력](agent_architecture/decision_log.md), [미해결 공백](agent_architecture/open_gaps.md), [생태계 지표](agent_architecture/ecosystem_metrics.md) — 탑다운 로드맵 #3880 |
| 자율 유지보수 — 병렬 세션·드리프트·엔드게임 | [자율 유지보수 지도](autonomous_maintenance/README.md) | [병렬 세션 규약](autonomous_maintenance/parallel_session_protocol.md), [선등재 패턴](autonomous_maintenance/pre_registration_pattern.md), [드리프트 감지](autonomous_maintenance/drift_detection.md), [엔드게임 판정](autonomous_maintenance/endgame_criteria.md) — 로드맵 #3907 J그룹 |
| 에이전트 보안 — 문서가 에이전트를 조종하는 경로 | [에이전트 보안 문서 지도](agent_security/README.md) | [위협 모델](agent_security/threat_model.md), [공격 표면](agent_security/attack_surface.md), [소비 에이전트 가이드](agent_security/consumer_guide.md), 로드맵 #3793·구현 #3787 |
| 신뢰할 수 없는 문서에 대한 경계 | [에이전트 경계 무결성 계약 — 경로·교정단서·자원한계·핸들](agent_boundary_contract.md) | 회귀 `tests/boundary_integrity_contract.rs`, 처리 결과 [task_sec_boundary](../report/task_sec_boundary/README.md) |
| 에이전트-네이티브 로드맵 R1~R100 (전체 조망의 상세) | [에이전트 로드맵 문서 지도](agent_roadmap/README.md) | 트랙 [A 봉투무결](agent_roadmap/track_a_envelope.md)·[B 가드보안](agent_roadmap/track_b_guards_security.md)·[C 동시성](agent_roadmap/track_c_concurrency.md)·[D 발견](agent_roadmap/track_d_discovery.md)·[E 능력](agent_roadmap/track_e_capabilities.md)·[F 규모](agent_roadmap/track_f_scale_perf.md)·[G 바인딩](agent_roadmap/track_g_bindings.md)·[H MCP](agent_roadmap/track_h_mcp_server.md)·[I 표준](agent_roadmap/track_i_standards.md)·[J 자율](agent_roadmap/track_j_autonomy.md), 조망 이슈 #3907·층 모델 #3880 |
| 외부 바인딩 공통 기반(M18~M20) | [IR 스키마 버저닝·표면 판단·파이썬 1호 명세](bindings_foundation.md) | 로드맵 #3608 M18~M20, [에이전트 표면 플레이북](../manual/agent_surface_playbook.md) |
| WASM/브라우저 에이전트 표면(M24) | [WASM 에이전트 표면 문서 지도](wasm_agent_surface/README.md) | [WASM capabilities 자기서술](wasm_agent_surface/self_description.md), [브라우저 MCP-유사 브리지](wasm_agent_surface/browser_bridge.md), [설치 0 온보딩](wasm_agent_surface/zero_install_onboarding.md), 로드맵 #3608 M24·#3869 |
| 문서 지능 서버(M25) — 파일 감시·워크스페이스·참조 조회 | [문서 지능 서버 문서 지도](document_intelligence/README.md) | [파일 감시와 증분 재파싱](document_intelligence/incremental_reparse.md), [다문서 워크스페이스 핸들](document_intelligence/workspace_handles.md), [참조 조회](document_intelligence/reference_queries.md), 로드맵 #3608 M25 |
| 바인딩들 사이의 동등성 계약 | [바인딩 동등성 계약](bindings/parity_contract.md) | [바인딩 문서 지도](bindings/README.md), [새 언어 바인딩 추가 절차](bindings/new_binding_guide.md), [파이썬·Node 실측 대조](bindings/python_node_comparison.md) |
| 이슈별 기술 조사 | [이슈별 기술 조사 지도](investigations/README.md) | [Issue #511 IR wrap 조사](investigations/issue-511/README.md), [Issue #1151 picture TAC 조사](investigations/issue-1151/README.md), [Issue #1584 이후 HWPX 잔여 IR 차이 조사](investigations/issue-1584/README.md), [Issue #1658 페이지네이션 조사](investigations/issue-1658/README.md), [Issue #1772 잔여 OVER 조사](investigations/issue-1772/README.md), [Issue #2125 font ownership 조사](investigations/issue-2125/README.md) |

## 현재 구조를 읽는 법

- `hwp_*`, `hwpx_*`, `document_ir_*`, `rendering_*`, `table_*`, `font_*` 문서는 장기 참조 후보이지만,
  파일명만으로 권위 문서라고 가정하지 않는다. 위 표 또는 각 문서의 명시적 링크를 우선한다.
- `task_m100_*`, `*_root_cause`, `*_diagnosis`, `*_investigation`은 이슈별 조사일 가능성이 높다. 다만
  장기 계약·기준선·설계 결론을 담은 문서는 `investigations/`로 자동 이동하지 않고 현행성 감사를 거쳐 분류한다.
- 현재 분리된 이슈별 조사는 `investigations/issue-####/`에서 관리한다. 각 디렉터리의 README가 해당
  스냅샷과 진단의 당시 범위, 최신성 제한, 관련 문서를 설명한다.
- [webhwp/](webhwp/README.md) 하위 문서는 2026-02 번들을 역분석한 historical investigation 묶음이다.
- [agent_security/](agent_security/README.md) 하위 문서는 rhwp가 에이전트 도구로서 노출하는 보안
  표면의 계약이다. 파서 견고성이 아니라 **문서 내용이 에이전트 행동에 영향을 미치는 경로**를 다룬다.
  구현(#3787)이 진행 중이므로 각 문서는 "현재 있는 것"과 "설계된 것"을 구분해 표시한다.
- [document_intelligence/](document_intelligence/README.md) 하위 문서는 로드맵 #3608 M25 의 **설계**다.
  구현이 아직 없으므로 각 문서는 "지금 있는 것"(코드 경로)과 "설계된 것"(제안)을 갈라 표시한다.
  현재 동작의 권위는 [MCP 연동 가이드](../manual/mcp_integration_guide.md)와
  [에이전트 경계 무결성 계약](agent_boundary_contract.md)이다.
- [archive/](archive/README.md)의 v1 roadmap처럼 대체된 계획은 historical 자료로 취급하며, 새 작업의
  근거로 직접 사용하지 않는다.

## 조사와 트러블슈팅 경계

- `tech/investigations/`는 특정 이슈의 가설·실험·관찰과 미확정 또는 기각 결론을 보존한다. 반복 작업의
  지침이나 장기 계약의 권위 문서는 아니다.
- [mydocs/troubleshootings/](../troubleshootings/README.md)는 재현 가능한 증상, 확정 원인, 적용 가능한 대응과 검증 방법을 보존한다.
  이후 작업의 사전 점검 자료로 사용한다.
- 조사 또는 트러블슈팅에서 장기 계약·스펙 정정·운영 절차가 확정되면 canonical 문서에 반영하고, 원 문서는
  근거 링크로 남긴다.

## 문서 역할과 생명주기

문서 역할(`kind`)과 생명주기(`status`)는 독립이다. 메타 블록을 추가하거나 갱신할 때는 다음 스키마를
쓴다.

| 필드 | 허용 값 | 의미 |
| --- | --- | --- |
| `kind` | `canonical`, `guide`, `reference`, `investigation`, `decision`, `snapshot`, `memory` | 문서가 수행하는 역할 |
| `status` | `active`, `historical`, `superseded` | 현재 사용 가능성 |
| `canonical` | 저장소 상대 경로 | 해당 상세 문서가 따르는 권위 문서 |
| `last_verified` | `YYYY-MM-DD` | 기술 사실을 마지막으로 확인한 날짜 |

`mydocs/tech`의 모든 Markdown 문서는 이 스키마로 분류한다. 장기 계약은 `active`, 이슈별 비교·실험과
대체된 설계는 `historical`, 이동 안내 문서는 `superseded`로 구분한다. 대규모 이동이나 정보구조 변경에서는
로컬 검사기로 누락과 잘못된 canonical 경로를 확인한다.

## 링크와 이동 규칙

내부 참조는 이동 커밋에서 새 경로로 직접 바꾼다. redirect stub은 외부 이력 호환이 필요한 문서만
allowlist로 제한한다. 검사 시점과 명령은 [문서 링크와 메타데이터 로컬 검사 가이드](../manual/markdown_link_check_guide.md)를
따른다. 일반 Markdown 추가나 본문 수정은 자동 CI를 실행하지 않으며 고정 migration 목록도 두지 않는다.
