---
kind: report
status: active
canonical: mydocs/report/agent_metrics_2026-08-11.md
last_verified: 2026-08-11
---

# 에이전트 축 주도 실측 — 2026-08-11 (기준 origin/devel @ 8ea92cdad)

> `tools/agent_dominance_metrics.py` 가 git 병합 이력과 바이너리
> 자기서술에서 산출했다. 모집단은 devel 병합분(열린 PR 불포함),
> 귀속은 git author. 같은 커밋에서 재실행하면 같은 숫자가 나온다.

## 1. 총괄 — 에이전트 축 전체 (경로군 합집합)

| 지표 | kevin9327 | 전체 | 점유 |
|---|---|---|---|
| 커밋 | 145 | 251 | **58%** |
| 추가 줄 | 53,420 | 69,039 | **77%** |

상위 기여자 (추가 줄): kevin9327 53,420 · jangster77 6,150 · kevin 4,218 · Taesup Jang 4,000 · edward 571

## 2. 경로군별 점유

| 경로군 | 커밋 (점유) | 추가 줄 (점유) |
|---|---|---|
| 로드맵·조망 | 27/36 (75%) | 5,117/5,602 (91%) |
| 지식지도·통합 문서 | 14/26 (54%) | 1,351/1,447 (93%) |
| 스킬 | 3/6 (50%) | 1,409/2,006 (70%) |
| 에이전트 코어(src) | 37/59 (63%) | 2,491/3,427 (73%) |
| 계약 가드(tests) | 92/168 (55%) | 24,223/33,362 (73%) |
| 바인딩(node) | 18/26 (69%) | 18,043/22,048 (82%) |
| 하네스 도구 | 3/5 (60%) | 786/1,147 (69%) |

경로군 정의(원문): {"로드맵·조망": ["mydocs/tech/agent_roadmap"], "지식지도·통합 문서": ["mydocs/manual/agent_knowledge_map.md", "mydocs/manual/mcp_integration_guide.md", "mydocs/manual/agent_codex"], "스킬": [".claude/skills"], "에이전트 코어(src)": ["src/mcp_serve.rs", "src/agent_profiles.rs", "src/provenance.rs", "src/schema_registry.rs", "src/capsule_sign.rs"], "계약 가드(tests)": ["tests/*contract*.rs"], "바인딩(node)": ["bindings/node"], "하네스 도구": ["tools/roadmap_progress.py", "tools/agent_preflight.py", "tools/gen_agent_codex.py", "tools/agent_dominance_metrics.py"]}

## 3. 최근 30일 동적 (--since 2026-07-12)

| 지표(30일) | kevin9327 | 전체 | 점유 |
|---|---|---|---|
| 커밋 | 145 | 240 | **60%** |
| 추가 줄 | 53,420 | 68,013 | **79%** |

## 4. 표면 절대 수치 (바이너리·저장소 실측)

| 표면 | 수치 |
|---|---|
| 계약 가드 테스트 함수 | 720 |
| 계약 가드 파일 | 86 |
| 봉투 recordFields 유니크 | 185 |
| 스킬 수 | 11 |
| 자기서술 명령 수 | 71 |
| 지식지도 사전 필드 | 188 |

## 5. 읽는 법 (정직 조항)

- 점유율은 **작업량 귀속**이지 가치 서열이 아니다 — 리뷰·머지 판단은
  메인테이너의 몫이며 이 표에 잡히지 않는다.
- 경로군 밖 기여(렌더러·파서 본체)는 이 축의 모집단이 아니다.
- 외부 도구와의 비교는 원리로만 말한다: 자기서술(capabilities)·
  봉투 계약·출처 표지·검증 사다리·표류 가드·살아있는 교본을 함께
  갖춘 문서 CLI 관행은 표준이 아니다 — 이 표면 자체가 차별점이다.
