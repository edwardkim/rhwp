# Probe mismatches after colliding PROBES rewrite

Remaining mismatch count: **0**.
3x gate exits: **0, 0, 0**.

No production files edited. `tools/skill_router/gate_new_skill.py` was not touched.

측정: 2026-08-20. CWD `C:\Users\swsz9\rhwp-skill-router`. `PYTHONUTF8=1`, `PYTHONIOENCODING=utf-8`.

## Method

Imported `PROBES` from `tools/skill_router/gate_new_skill.py` and routed every tuple member through `route.route(request)`. Selected skill is `skillSelection[0].id`. A mismatch is `selected != catalog skill`.

27 catalog skills × 3 probes = 81 routes.

## Remaining mismatches (selected != catalog skill)

**None.** 81/81 selected == catalog skill.

## All PROBES tuples

| Catalog skill | # | Request | Selected | Intent | Result |
|---------------|---|---------|----------|--------|--------|
| rhwp-agent-surface | 1 | 새 MCP 도구 추가해줘 | rhwp-agent-surface | add-surface | match |
| rhwp-agent-surface | 2 | 드리프트 가드 확인해 | rhwp-agent-surface | add-surface | match |
| rhwp-agent-surface | 3 | capabilities 가 SSOT 인지 확인해 | rhwp-agent-surface | add-surface | match |
| rhwp-bug-hunter | 1 | 버그 찾아줘 실사용 기준으로 | rhwp-bug-hunter | hunt-bug | match |
| rhwp-bug-hunter | 2 | 정답지와 비교해 | rhwp-bug-hunter | hunt-bug | match |
| rhwp-bug-hunter | 3 | playbook 여정 실행해 | rhwp-bug-hunter | hunt-bug | match |
| rhwp-bulk-pipeline | 1 | 폴더 전체를 텍스트로 변환해 | rhwp-bulk-pipeline | bulk | match |
| rhwp-bulk-pipeline | 2 | rhwp batch the whole corpus | rhwp-bulk-pipeline | bulk | match |
| rhwp-bulk-pipeline | 3 | 여러 hwp 대량 처리해줘 | rhwp-bulk-pipeline | bulk | match |
| rhwp-chief | 1 | 요청 큐 돌려줘 | rhwp-chief | run-request-queue | match |
| rhwp-chief | 2 | 서비스 루프 감시해 | rhwp-chief | run-request-queue | match |
| rhwp-chief | 3 | needs-agent 수거해 | rhwp-chief | run-request-queue | match |
| rhwp-cli | 1 | 페이지네이션 조판부호 덤프 | rhwp-cli | inspect-cli | match |
| rhwp-cli | 2 | dump pagination and the render tree | rhwp-cli | inspect-cli | match |
| rhwp-cli | 3 | 레이아웃 겹침 버그 디버깅해 | rhwp-cli | inspect-cli | match |
| rhwp-codex | 1 | rhwp 사용법 전체 명령 보여줘 | rhwp-codex | codex | match |
| rhwp-codex | 2 | navigate the rhwp command codex | rhwp-codex | codex | match |
| rhwp-codex | 3 | 뭘 쓸지 모르겠어 | rhwp-codex | codex | match |
| rhwp-contributor | 1 | PR 올려 | rhwp-contributor | contribute | match |
| rhwp-contributor | 2 | open a pull request | rhwp-contributor | contribute | match |
| rhwp-contributor | 3 | 기여 절차 알려줘 | rhwp-contributor | contribute | match |
| rhwp-doc-triage | 1 | 이 hwp 뭔 문서야? | rhwp-doc-triage | triage | match |
| rhwp-doc-triage | 2 | summarize this document without reading it all | rhwp-doc-triage | triage | match |
| rhwp-doc-triage | 3 | 목차 뽑아줘 | rhwp-doc-triage | triage | match |
| rhwp-exam-ingest | 1 | 한글 시험지로 변환해줘 | rhwp-exam-ingest | exam-ingest | match |
| rhwp-exam-ingest | 2 | exam ingest this paper to hwpx | rhwp-exam-ingest | exam-ingest | match |
| rhwp-exam-ingest | 3 | 시험문제 변환해 | rhwp-exam-ingest | exam-ingest | match |
| rhwp-explore | 1 | 이 문서로 뭘 할 수 있어? | rhwp-explore | explore-doc | match |
| rhwp-explore | 2 | 어포던스 메뉴 보여줘 | rhwp-explore | explore-doc | match |
| rhwp-explore | 3 | 문서 탐색해봐 | rhwp-explore | explore-doc | match |
| rhwp-fde | 1 | 고객이 이 문서가 안 열린대 | rhwp-fde | triage-symptom | match |
| rhwp-fde | 2 | 현장 증상 트리아지해줘 | rhwp-fde | triage-symptom | match |
| rhwp-fde | 3 | 필드가 안 채워진대 대응해줘 | rhwp-fde | triage-symptom | match |
| rhwp-fidelity-compare | 1 | 한컴 PDF와 비교해줘 | rhwp-fidelity-compare | compare-fidelity | match |
| rhwp-fidelity-compare | 2 | run fidelity_compare against the official PDF | rhwp-fidelity-compare | compare-fidelity | match |
| rhwp-fidelity-compare | 3 | 한컴이 뽑은 PDF랑 rhwp가 같은지 | rhwp-fidelity-compare | compare-fidelity | match |
| rhwp-form-fill | 1 | 이 서식 채워줘 | rhwp-form-fill | fill-form | match |
| rhwp-form-fill | 2 | fill this form | rhwp-form-fill | fill-form | match |
| rhwp-form-fill | 3 | 누름틀에 값 넣어줘 | rhwp-form-fill | fill-form | match |
| rhwp-handoff | 1 | 세션 핸드오프 해줘 | rhwp-handoff | handoff-session | match |
| rhwp-handoff | 2 | 컨텍스트 바닥이라 핸드오프해 | rhwp-handoff | handoff-session | match |
| rhwp-handoff | 3 | 작업 인수인계 result.json 읽어 | rhwp-handoff | handoff-session | match |
| rhwp-knowledge-map | 1 | 지식 지도 어디 문서부터 | rhwp-knowledge-map | find-canonical | match |
| rhwp-knowledge-map | 2 | 이 필드가 뭐야 | rhwp-knowledge-map | find-canonical | match |
| rhwp-knowledge-map | 3 | llms.txt 다음이 뭐야 | rhwp-knowledge-map | find-canonical | match |
| rhwp-mcp-session | 1 | rhwp 를 MCP로 붙여줘 | rhwp-mcp-session | mcp | match |
| rhwp-mcp-session | 2 | start mcp-serve and list session tools | rhwp-mcp-session | mcp | match |
| rhwp-mcp-session | 3 | hwp_open 으로 문서 열어 | rhwp-mcp-session | mcp | match |
| rhwp-onboarding | 1 | rhwp 처음인데 온보딩해줘 | rhwp-onboarding | onboard | match |
| rhwp-onboarding | 2 | rhwp_doctor로 온보딩해 | rhwp-onboarding | onboard | match |
| rhwp-onboarding | 3 | .mcp.json 만들어줘 | rhwp-onboarding | onboard | match |
| rhwp-provenance | 1 | 이 값이 문서에서 온 건가 | rhwp-provenance | provenance | match |
| rhwp-provenance | 2 | mark untrustedFields provenance | rhwp-provenance | provenance | match |
| rhwp-provenance | 3 | 출처 모르는 문서 처리해 | rhwp-provenance | provenance | match |
| rhwp-recipes | 1 | 어떤 레시피로 가? | rhwp-recipes | pick-recipe | match |
| rhwp-recipes | 2 | 실무 플레이북 골라줘 | rhwp-recipes | pick-recipe | match |
| rhwp-recipes | 3 | 결번 레시피 07 08 없지 | rhwp-recipes | pick-recipe | match |
| rhwp-safe-edit | 1 | 안전하게 편집해줘 | rhwp-safe-edit | safe-edit | match |
| rhwp-safe-edit | 2 | dry-run the replace-text plan first | rhwp-safe-edit | safe-edit | match |
| rhwp-safe-edit | 3 | 여러 편집을 한 번에 원자적으로 | rhwp-safe-edit | safe-edit | match |
| rhwp-security-sweep | 1 | 이 문서 보내도 돼? | rhwp-security-sweep | security | match |
| rhwp-security-sweep | 2 | inspect hidden text and redact PII | rhwp-security-sweep | security | match |
| rhwp-security-sweep | 3 | 받은 첨부 안전한지 확인 | rhwp-security-sweep | security | match |
| rhwp-skill-author | 1 | 새 스킬 만들어 | rhwp-skill-author | author-skill | match |
| rhwp-skill-author | 2 | create a new SKILL.md with the 3-pass gate | rhwp-skill-author | author-skill | match |
| rhwp-skill-author | 3 | SKILL.md 작성해줘 | rhwp-skill-author | author-skill | match |
| rhwp-skill-router | 1 | 어떤 스킬을 쓰지 | rhwp-skill-router | route | match |
| rhwp-skill-router | 2 | route this request through the execution graph | rhwp-skill-router | route | match |
| rhwp-skill-router | 3 | 라우터에 통과시켜줘 | rhwp-skill-router | route | match |
| rhwp-strategist | 1 | 이 문서들로 전략 보고서 만들어 | rhwp-strategist | build-strategy | match |
| rhwp-strategist | 2 | 근거 대장에 주장마다 좌표 | rhwp-strategist | build-strategy | match |
| rhwp-strategist | 3 | 정부과제 수주 근거 모아줘 | rhwp-strategist | build-strategy | match |
| rhwp-table-exchange | 1 | 표를 CSV로 뽑아줘 | rhwp-table-exchange | table-csv | match |
| rhwp-table-exchange | 2 | csv-to-table 로 되돌려 | rhwp-table-exchange | table-csv | match |
| rhwp-table-exchange | 3 | 표 셀 하나만 고쳐줘 | rhwp-table-exchange | table-csv | match |
| rhwp-visual-regression | 1 | 편집 전후 화면 비교해 | rhwp-visual-regression | visual | match |
| rhwp-visual-regression | 2 | run render-diff for visual regression | rhwp-visual-regression | visual | match |
| rhwp-visual-regression | 3 | 레이아웃 회귀 깨졌는지 확인 | rhwp-visual-regression | visual | match |
| rhwp-work-receipt | 1 | 이 작업 영수증 남겨 | rhwp-work-receipt | receipt | match |
| rhwp-work-receipt | 2 | replay the work capsule and audit lineage | rhwp-work-receipt | receipt | match |
| rhwp-work-receipt | 3 | 재현율 검증해줘 | rhwp-work-receipt | receipt | match |

Counts: 81 routed, 81 match, 0 mismatch.

## `python tools/skill_router/gate_new_skill.py` (3 repeats)

| Repeat | Exit | FAIL lines | Summary |
|--------|------|------------|---------|
| 1 | 0 | (none) | OK: 27 skills x 3 scans, catalog=27, route_probes=81, rhwp=207 |
| 2 | 0 | (none) | OK: 27 skills x 3 scans, catalog=27, route_probes=81, rhwp=207 |
| 3 | 0 | (none) | OK: 27 skills x 3 scans, catalog=27, route_probes=81, rhwp=207 |

Counts: 3/3 pass. No FAIL lines.

Each run: 27 skills × 3 scans pass; live rhwp `target/release/rhwp.exe`, `rhwp commands: pass (207 known, 189 refs)`; catalog 27/27 pass; `catalog probes: pass (27 skills)`; all 81 route probes pass (selected == expected).
