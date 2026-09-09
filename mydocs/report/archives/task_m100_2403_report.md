# 최종 보고 — #2403 Stage 1: SourceProvenance / LayoutCompatibilityProfile

- PR #2408 merge (2026-07-19, admin). 단계별 승인 4회(수행계획서 → 0~3단계).
- 원칙 준수: **observable behavior 불변** — 전 단계 bit-identical 게이트,
  치환만(해체 없음, 마스터 플랜 §1).

## 결과

| 지표 | before | after |
|------|--------|-------|
| 소스분기 계열 참조 (src) | 176 | **87** |
| renderer 코어 필드 읽기 | 77 | **0** |
| document_core 필드 읽기 | 40 | **0** |
| rendering.rs 파생 중복 (Issue #1770 규칙) | 2곳 | **1소유** (Document::layout_profile) |

잔존 87 = 파서 확정 지점(원점) 18 · 모델 shim 선언 · 테스트 초기화 · 값 전달
파라미터 · 이력 주석 — 전부 계획 범주.

## 게이트 이력

- advisory 3종(API 표면/CLI output/render-tree 해시) — 단계마다 대조,
  CLI·render-tree 전 단계 무변동, API 는 의도 추가분만(스냅샷 기록)
- 전체 스위트 284 바이너리 0 실패 × 단계 3회, clippy --all-targets 0, 연결맵
  414쪽 유지, 등가성 가드 6케이스(tests/issue_2403_provenance_stage1.rs)
- PR CI 전 항목 green — **#2393 warm 첫 실측 겸 (wall 9.6분, 10분 목표 달성)**

## 구조 성과

- 소스분기 판단의 단일 소유: 파서가 provenance 확정 → `Document::
  layout_profile()` 질의 4종 — 두-경로 정정 반복(feedback_fix_scope_check_
  two_paths)의 구조적 원인이던 파생 중복 소멸
- 신규 규약 명문화: parser_architecture.md 정책 절 + CONTRIBUTING — 신규
  분기는 profile 질의로만
- #2373 잔여 판별자 트랙의 수용 좌석 마련 (SourceProvenance 서명 필드)

## 후속 (범위 외 명시분)

- shim 필드(is_hwp3_variant/is_hwpx_variant) 제거 — API 호환 판단 후 별도
- 분기 밀집 구간 해체 — 마스터 플랜 Phase 후속
- #2373 문서군 판별자 — 한글 COM 재저장 실측 후 서명 필드 + profile 질의 추가
