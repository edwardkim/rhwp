# PR #2241 검토 — 거대 셀 편집 cache coherence + 경계 pagination 정합 (postmelee, #2214)

- 검토일: 2026-07-17 / head `1ee9ad0d` / 37파일 +7,807/−77 / CLEAN, CI 전 항목 green
- 경위: #2214 를 메인테이너·컨트리뷰터가 독립 진단 → 동일 근인(cold/warm
  셀 캐시) 수렴 → 작업지시자 결정(2안)으로 메인테이너 정정은
  `local/task2214` 보류, 컨트리뷰터 PR 대기. CI 실패 2회(최신 devel
  상호작용)를 Stage 7~8 로 정리 후 CLEAN 도달.

## 대조 검토 (보류 브랜치 `local/task2214` 와)

| 축 | 메인테이너 정정 (b6cff702, 보류) | 본 PR |
|----|------|------|
| 접근 | `invalidate_page_tree_cache_from` 에 `clear_layout_caches()` 1줄 — 전체 셀 캐시 클리어 | **표적 coherence** — 편집 문단만 render_normalized 에서 교체, 무관 셀 캐시 보존 |
| 정확성 | FAILED→PASS 실증 (7/12) | **동치 확인** — 메인테이너 표적 프로브(조회-오염 시퀀스, 상대 단언 교정판)를 본 PR head 에 얹어 **2/2 통과** |
| 성능 | 매 편집 전체 셀 유닛 재적재 (거대 셀에서 비용) | #1949 O(pages×cell) 보호를 편집 중에도 유지 — **우월** |
| 가드 | native 2 + browser E2E 1 | `page_local_repaint` 3건(warm_target_layout = 동일 오염 시퀀스 커버) + cache_matrix_probe + studio 검증 |

**판정: 본 PR 이 supersede** — 접근이 더 정밀하고 성능 보존적이며, 가드
커버리지도 동등 이상. 보류 브랜치는 merge 후 폐기.

> 프로브 이식 시 절대 좌표 단언이 최신 devel 의 pagination 정정들로 3.7px
> 시프트되어 실패 — stale 캐시가 아닌 기하 전진임을 확인하고 상대 단언
> (최종 캐럿 = 중간 캐럿 + 한 줄)으로 교정 후 통과. 좌표 핀 테스트의
> 취약성 교훈.

## 게이트 (로컬 재실증)

| 게이트 | 결과 |
|--------|------|
| 전수 `--tests --no-fail-fast` | **3,242/0** |
| PR 표적 (page_local_repaint 3, cache_matrix) | 통과 |
| **메인테이너 프로브 이식** (조회-오염) | **2/2 통과 (동치 실증)** |
| studio npm ci + tsc + test | 클린 / **306/0** |
| fmt / clippy | 통과 / 0 |
| 원격 CI | 전 항목 green (CLEAN) |

## 판단

**approve → merge 수용 권고** (CLEAN — 일반 merge). merge 후:
`local/task2214` 폐기, #2214 close 는 작업지시자 승인 대기.
