---
name: project_v080_hwpx_save_milestone
description: 다음 릴리즈는 0.7→0.8 MINOR 분기점 — HWPX 직접 저장+양방향 포맷 선택이 근거
metadata: 
  node_type: memory
  type: project
  originSessionId: 2560b31a-9f1c-4764-bbf1-7ba5fc27c7ce
---

다음 rhwp 릴리즈는 **0.7.x → 0.8.0 MINOR 버전 업**으로 올린다 (2026-06-28 작업지시자 결정).

**분기점 근거**: HWPX 직접 저장(#1532/PR #1533, #196 베타 해제) + 저장 시 출력 포맷(HWP/HWPX)
양방향 사용자 선택(#1613)으로, 읽기 전용을 넘어 **쓰기/포맷 변환 기능이 확립**된 의미 있는 분기점.
HWPX 직렬화 무손실(#1597 등)과 #197 완전 변환기 완료가 전제.

**릴리즈 시 적용** (publish_guide MINOR 규칙):
- Cargo.toml(기준) + rhwp-vscode/package.json + npm/editor/package.json + rhwp-studio/package.json
  을 모두 `0.8.0` 으로 동기화.
- MINOR 라 **git 태그(v0.8.0) 생성** (PATCH 와 달리).
- 확장(chrome/firefox)은 라이브러리와 독립 버전(현 0.2.x) — MINOR 업이 확장 버전 업을 강제하지 않음.

**주의 — studio WASM stale**: rhwp-studio 는 `@wasm` → `../pkg` 로 루트 `pkg/` WASM 을 직접
참조한다(vite alias). studio TS 변경(#1613)은 pkg 재빌드와 독립이나, exportHwp/exportHwpx 동작은
pkg 에 의존하므로, 0.8.0 배포 시 **pkg 를 devel 기준으로 재빌드**해야 한다. (#1613 처리 중
"확장자만 hwp" 제보가 stale pkg 캐시 때문이었음.)

관련: [[project_hwpx_serializer_limits]] [[project_hwpx_to_hwp_adapter_limit]] [[feedback_milestone_notation]]
