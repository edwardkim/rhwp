---
kind: investigation
status: active
canonical: mydocs/tech/agent_roadmap/onboarding_cases_2026h2.md
last_verified: 2026-08-09
---

# 온보딩 사례집 2026H2 — 외부 소비자 실측 6건 (R86 씨앗, #4346)

R86("외부 에이전트 온보딩 사례집")의 첫 실물이다. 수집 원천은
[user_demand_survey_2026h2.md](user_demand_survey_2026h2.md) §3 의 실측(2026-08-09,
결정 근거 전건 원문 재검증)이고, 형식은 R86 설계 그대로 — 사례마다 **마찰점을
적고 이슈·PR 로 환류**한다. 이 문서는 사례를 늘리는 대장이며, 트랙 문서의 등급
판정은 여기서 하지 않는다(착지 보정은 조망 규칙대로 별도).

## 사례 대장

| # | 소비자 | 소비 표면 | 실측 근거 | 마찰점 |
|---|---|---|---|---|
| 1 | 에이전트 스킬 허브(NomaDamas/k-skill) | 상류 CLI 공식 래핑(export-svg·dump·diag·ir-diff·thumbnail·convert) | [rhwp-advanced.md](https://github.com/NomaDamas/k-skill/blob/main/docs/features/rhwp-advanced.md) 원문 | **스테일 버전 고정** — v0.7.3 기준 "편집 서브커맨드는 없다"로 편집 축을 자체 CLI 로 우회 |
| 2 | 서드파티 MCP 서버(treesoop/hwp-mcp) | `@rhwp/core` 래핑, 35도구, `npx` 원라인 | [저장소](https://github.com/treesoop/hwp-mcp) 원문 (67★) | **스테일 버전 고정** — "0.7.7 라운드트립 한계로 .hwp 쓰기 미지원" 명기 |
| 3 | 서드파티 파이썬 바인딩(DanMeon/rhwp-python) | PyO3 직바인딩 | [저장소](https://github.com/DanMeon/rhwp-python) · 제안 이슈 #227 | 1st-party 파이썬 배포 부재가 원인인 병행 구현 |
| 4 | 데스크톱 파생(HOP) | 파싱 엔진으로 채택 | [Threads 공개 글](https://www.threads.com/@golbin/post/DXUPMzSE2P_/) (9.7K 뷰) | 배포 신뢰(미서명 바이너리 OS 경고)·암호 문서 무음 실패 제보(여론 §1.1) |
| 5 | macOS Quick Look 파생(알한글) | 코어 재사용 | [GeekNews 29692](https://news.hada.io/topic?id=29692) | (수집 대기 — 접촉 후 갱신) |
| 6 | 플랫폼 통합 요청(ONLYOFFICE) | 편집기 통합 요청(영문) | [DocumentServer#3659](https://github.com/ONLYOFFICE/DocumentServer/issues/3659) (open) | 영문 표면 빈약이 검토 장벽(종전 README_EN 에 설치·MCP 절 부재) |

## 마찰점 → 환류 대장 (같은 날 조치 연결)

| 마찰점 | 원인 구조 | 환류 조치 (제출물) |
|---|---|---|
| 스테일 버전 고정 (사례 1·2) | 소비자가 상류 진화를 기계 대조할 채널 부재 | R67×R83 — capabilities `schemaRegistry` 노출 (PR #4330) · 최신 추종 안내 연락 문안 (#4343 B) |
| 설치 경로 부재 (사례 3·4) | 바이너리는 실재하나 도달 문서·채널·휠 부재 | README 설치 절 (PR #4332) · PyPI/npm 파이프라인 (PR #4337) · scoop/brew/winget (PR #4339) |
| 프레임워크 통합 부재 (사례 3 인접) | RAG 진입 질의에서 1st-party 도달 0 | `rhwp.integrations` 로더 (PR #4342) |
| 영문 표면 (사례 6) | 발신 0 이 원인인 도달 0 | README_EN 설치·MCP 절 (PR #4345) · 등재 문안 (#4343 A) |
| 암호 문서 무음 실패 (사례 4) | 자동화 시 판정 불가 UX | 재현·판정 조사 필요 — 후속 이슈 후보(미확정이라 등록 보류) |

## 수집 규약 (이 대장의 운영)

1. 새 사례는 **실측 근거(URL·날짜) 필수** — 전문(傳聞) 사례는 싣지 않는다.
2. 마찰점은 관찰로 적고, 조치는 이슈·PR 링크로만 잇는다(문서가 백로그를
   대체하지 않는다).
3. 소비자 접촉(#4343 승인 후)의 회신은 해당 사례 행에 갱신하고, 새 마찰점은
   이슈로 환류한다.
4. 사례 N건·마찰점 환류가 쌓여 R86 DoD("사례 N건 정리 + 마찰점 이슈 환류")를
   충족하는지의 판정은 트랙 문서 쪽에서 한다.
