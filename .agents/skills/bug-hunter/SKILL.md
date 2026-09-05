---
name: bug-hunter
description: 실사례 사용자 여정을 처음부터 끝까지 실행하고 한컴 공식 출력·법정 서식·실제 제출 요건의 정답지와 대조해 재현 가능한 rhwp 결함을 찾는다. "버그 찾아줘(실사용 기준)", "정답지와 비교해", "playbook 여정 실행" 요청에 사용한다. 단순 무작위 스윕이나 구현 수정만이 목표인 작업에는 사용하지 않는다.
---

# bug-hunter — 실사례 여정 기반 탑다운 버그 헌팅

권위 방법론은 [버그 헌팅 playbook](../../../mydocs/manual/bug_hunting_playbook.md)이다.
이 Skill은 그 playbook을 **실행**하기 위한 진입점이다. 방법론을 복제하거나 별도의 판정 기준·두 번째 루브릭을 만들지 않는다. gym 이 아니고, 새 CLI 명령을 발명하지
않는다. 헌팅 스킬이지 버그픽스가 아니다. DocumentCore 를 고치지 않는다.

상세는 `references/` 를 단계별로 연다. SKILL.md 는 인덱스와 정지 규칙만 담는다.

## 실행 계약 (playbook 6단 — 강제 순회)

1. playbook의 사용자 가치 순서에 따라 실사례 여정 하나를 선택한다. 없으면 실제 정부 양식,
   법정 서식, 실제 공고 등 정답지가 존재하는 여정을 정의한다.
2. 한컴 출력 PDF, 법정 서식, 실제 제출 요건처럼 원 구현과 독립된 비교 기준을 **먼저** 확보한다.
   한컴 출력물은 도구·버전·출력 경로·폰트와 원본/산출물 경로를 기록한다. 비교 기준이 없으면
   render-diff 같은 자기 일관성 검사까지만 수행하고 그 한계를 기록한다.
3. `rhwp` CLI로 여정을 중간에 멈추지 말고 최종 산출물까지 실행한다. 명령, 입력과 산출물을
   재현 가능하게 남긴다.
4. 픽셀/시각 대조, 기준 PDF 텍스트층 ↔ SVG `<text>`의 쪽별 문자 멀티셋 대조, 기록값 재독,
   종료 코드와 JSON 계약 검증을 우선해 비교 기준과 대조한다. 문자 멀티셋의 기준본 전용 문자는
   소실, SVG 전용 문자는 과잉, 같은 쪽의 양쪽 차이는 치환 후보로 분류한다. 기록값·종료 코드·
   JSON 계약은 기계적으로 판정한다. 텍스트층 차이·픽셀 diff·sweep은 후보 검출·무회귀 근거이며,
   최종 시각 판정은 작업지시자/maintainer가 한다.
5. 격차마다 재현 명령, 관련 코드 경로(파일:라인), 정답지 대비 근거를 갖춘 이슈를 작성한다.
   증상만 기록하지 않는다.
6. 발견이 없을 때까지 같은 여정의 다음 격차를 확인하고, 결과를 playbook의 예시로 추가할지
   제안한다. 코드 수정은 요청받은 경우에만 별도 작업으로 분리한다.

## 사다리 (질문이 이미 답이면 다음 단 금지)

```
1. 여정 선택          playbook 카탈로그 / 실물 정부·법정 서식 (F01)
2. 정답지 확보        한컴 PDF·법정 서식·제출 요건 + provenance (F02·F03)
     └ 정답지 없음 ──▶ render-diff 자기 일관성만 + 한계 기록 (F04). 충실도 이슈 금지
3. 최종 산출물까지    기존 rhwp CLI 만. 중간 정지 금지 (F05)
4. 대조               fidelity_compare · 재독 · exit/JSON (F06~F10)
5. 이슈화             재현 명령 + 파일:라인 + 정답지 근거 (F11)
6. 다음 격차          같은 여정 → 다음 여정. 수정은 별도 PR (F12)
```

살아 있는 동사는 기존 CLI 와 `tools/fidelity_compare` 뿐이다.

```
rhwp info --json <파일>
rhwp fields --json <파일>
rhwp export-tables --json <파일>
rhwp edit set-cell|fill-fields|replace-text … -o <산출> --json
rhwp export-svg|export-png|export-pdf <산출>
rhwp export-hwpx <원본> <변환> --verify --verify-pages
rhwp ir-diff <A> <B> --json
rhwp render-diff <파일> [--via hwpx|hwp]
rhwp dump / dump-pages / capabilities
venv/bin/python tools/fidelity_compare/fidelity_compare.py <키> <시작> <끝> --out-dir <외부>
```

## 요청 → 명령

| 사용자 요청 | 명령 | 레퍼런스 |
| --- | --- | --- |
| playbook 여정 실행 | 카탈로그에서 하나 고르고 6단을 끝까지 | 01_playbook_authority.md · 03_journey_selection.md |
| 정답지부터 잡아 | 한컴 PDF / 법정 서식 / 제출 요건 + provenance | 04_ground_truth.md · 05_hangul_pdf_provenance.md |
| 한컴 PDF 와 비교 | `fidelity_compare.py` + `export-svg` | 08_pixel_visual.md · 09_text_multiset.md · 12_fidelity_compare.md |
| 정답지가 없다 | `render-diff` 자기 일관성만. 한계 기록 | 06_self_consistency_limit.md |
| 값 맞나 | `export-tables` / `fields` 재독 | 10_reread_values.md |
| 이슈로 남겨 | 재현·파일:라인·정답지 근거 | 13_issue_template.md |
| 콘솔이 깨져 보여 | UTF-8 파일 비교. 콘솔은 결함 아님 | 15_utf8_console.md |
| 접수까지 자동화 | 거부. 제출 직전 산출물까지만 | 14_no_filing.md |

## 정지 규칙

| ID | 언제 | 행동 |
| --- | --- | --- |
| F01 | 여정이 실물 정답지 없이 samples/ 무작위 스윕 | 중단. playbook 카탈로그로 되돌림 |
| F02 | 한컴 PDF / 법정 서식 / 제출 요건을 아직 안 확보 | 여정 실행 금지. 정답지부터 |
| F03 | 한컴 출력 provenance(도구·버전·경로·폰트) 미기록 | 비교 시트를 이슈 근거로 쓰지 않음 |
| F04 | 독립 기준이 없음 | render-diff 자기 일관성만. 충실도 결함으로 이슈화 금지 |
| F05 | 중간 단계에서 멈춤 | 최종 산출물까지 이어서 실행 |
| F06 | 문자 멀티셋 기준본 전용 | 소실 후보. 단독 최종 판정 금지 |
| F07 | 문자 멀티셋 SVG 전용 | 과잉 후보. 단독 최종 판정 금지 |
| F08 | 같은 쪽에 양쪽 차이 | 치환 후보. 사람 감사 |
| F09 | `--verify` 4/4 통과 | 멈추지 않음. ZIP 엔트리 이름 집합·태그 개수 대조 (함정 1) |
| F10 | 콘솔에서 한글이 깨져 보임 | 결함 아님. UTF-8 파일로 재비교 |
| F11 | 증상만 있는 이슈 초안 | 재현 명령·파일:라인·정답지 근거가 생길 때까지 올리지 않음 |
| F12 | 수정을 이 스킬 안에서 시작 | 별도 작업으로 분리. DocumentCore 금지 |
| F13 | 실제 접수·로그인·실명인증을 자동화하려 함 | 즉시 거부. 가상 데이터·제출 직전까지만 |
| F14 | devel 에서 이미 고쳐진 결함 | 이슈를 새로 열지 않음 (함정 2) |
| F15 | 표본 1건으로 포맷 계약을 단정 | N건 중 M건·반례 수가 생길 때까지 가설 (함정 3) |
| F16 | 가설만 있고 구현 기각이 없음 | 음성 결과도 이슈에 남김 (함정 4) |

**금지 기본값**

- 새 CLI 발명 (`bug-hunt`, `oracle-check`, `fidelity-diff`, `ground-truth` 하위명령)
- gym pack / gym 과제 / 채점기
- 두 번째 헌팅 루브릭 (playbook 이 유일한 권위)
- DocumentCore · 엔진 버그픽스
- 실제 접수 / 로그인 / 실명인증 자동화
- 콘솔 cp949 착시를 결함으로 이슈화
- 정답지 없이 "그럴듯하다"로 통과
- 이웃 스킬(onboarding / mcp-session / safe-edit / provenance / doc-triage / form-fill / visual-regression) 재작성

## 인계

- 채움 수단 자체 → `rhwp-form-fill` (채운 뒤 여기로 돌아와 정답지 대조)
- 전후 레이아웃 숫자만 → `rhwp-visual-regression` (`render-diff` 는 자기 일관성. 한컴 충실도가 아님)
- 표 CSV 왕복 → `rhwp-table-exchange`
- 배포 전 숨은 글 → `rhwp-security-sweep`
- 미지 문서 파악만 → `rhwp-doc-triage`
- 수정 PR 절차 → `rhwp-contributor` (요청받은 뒤에만)

상세: [21_handoff.md](references/21_handoff.md)

## 비교 분류 (playbook 문자 멀티셋)

| 관측 | 분류 | 이슈 초안 |
| --- | --- | --- |
| 기준 PDF 텍스트층에만 있는 문자 | 소실 (`reference_only`) | 후보. 최종 시각 판정 전 |
| SVG `<text>` 에만 있는 문자 | 과잉 (`svg_only`) | 후보. 숨김 대상 과잉 출력 의심 |
| 같은 쪽에 양쪽 차이 | 치환 (`substitution`) | PUA·폰트 대체 후보 |
| 기록값 재독 불일치 | 기계 확정 | 이슈화 가능 |
| 종료 코드·JSON 계약 위반 | 기계 확정 | 이슈화 가능 |
| 픽셀 diff% 상위 | 후보 검출 | 사람 감사. 절대 오라클 아님 |

표: [20_classification.md](references/20_classification.md)

## 저장소 규칙

- 빌드·CLI 사용은 [개발 환경 가이드](../../../mydocs/manual/dev_environment_guide.md)와
  [CLI 명령어 매뉴얼](../../../mydocs/manual/cli_commands.md)을 따른다.
- 정합 대조 도구와 사용법은
  [`tools/fidelity_compare`](../../../tools/fidelity_compare/README.md)를 따른다.
- 한글 콘솔 인코딩 착시를 결함으로 오인하지 않도록, 문자열 비교와 검증은 UTF-8 파일 기반으로
  수행한다.
- 실제 서비스 접수, 로그인, 실명인증은 자동화하거나 수행하지 않는다. 가상 데이터로 문서 작성과
  제출 직전 산출물까지를 검증 범위로 한다.

## 완료 기준

각 발견은 재현, 원인 경로, 정답지 대비 근거를 포함한 이슈로 남긴다. 발견이 없으면 실행한
여정·정답지·검증 범위와 남은 한계를 기록한다.

## 레퍼런스 목차

1. [00_tree.md](references/00_tree.md) — 판단 트리
2. [01_playbook_authority.md](references/01_playbook_authority.md) — playbook 이 유일한 권위
3. [02_judgment_traps.md](references/02_judgment_traps.md) — 판정 함정 4종
4. [03_journey_selection.md](references/03_journey_selection.md) — 여정 선택
5. [04_ground_truth.md](references/04_ground_truth.md) — 정답지 먼저
6. [05_hangul_pdf_provenance.md](references/05_hangul_pdf_provenance.md) — 한컴 PDF provenance
7. [06_self_consistency_limit.md](references/06_self_consistency_limit.md) — 기준 없을 때
8. [07_run_to_final.md](references/07_run_to_final.md) — 최종 산출물까지
9. [08_pixel_visual.md](references/08_pixel_visual.md) — 픽셀/시각
10. [09_text_multiset.md](references/09_text_multiset.md) — 쪽별 문자 멀티셋
11. [10_reread_values.md](references/10_reread_values.md) — 기록값 재독
12. [11_exit_json_contract.md](references/11_exit_json_contract.md) — 종료 코드·JSON
13. [12_fidelity_compare.md](references/12_fidelity_compare.md) — fidelity_compare
14. [13_issue_template.md](references/13_issue_template.md) — 이슈 템플릿
15. [14_no_filing.md](references/14_no_filing.md) — 접수 자동화 금지
16. [15_utf8_console.md](references/15_utf8_console.md) — UTF-8 파일 비교
17. [16_pitfalls.md](references/16_pitfalls.md) — 헌팅 함정
18. [17_journeys.md](references/17_journeys.md) — 여정 카탈로그
19. [18_worked_traces.md](references/18_worked_traces.md) — 재현 트레이스
20. [19_intent_matrix.md](references/19_intent_matrix.md) — 발화 → 명령
21. [20_classification.md](references/20_classification.md) — 비교 분류표
22. [21_handoff.md](references/21_handoff.md) — 다른 스킬로
23. [22_failure_signals.md](references/22_failure_signals.md) — 신호 → 처방
24. [23_gate_recipes.md](references/23_gate_recipes.md) — 게이트 레시피
25. [24_existing_cli.md](references/24_existing_cli.md) — 기존 CLI 만

예제: [examples/](examples/). 기계 가독 픽스처: [fixtures/](fixtures/).
Claude 진입 포인터: [`.agents/skills/rhwp-bug-hunter/`](../../../.agents/skills/rhwp-bug-hunter/SKILL.md).

## 권위

- [`mydocs/manual/bug_hunting_playbook.md`](../../../mydocs/manual/bug_hunting_playbook.md) — **유일한 루브릭**
- [`tools/fidelity_compare/README.md`](../../../tools/fidelity_compare/README.md)
- [`mydocs/manual/cli_commands.md`](../../../mydocs/manual/cli_commands.md)
- [`mydocs/manual/verification/visual_verification_governance.md`](../../../mydocs/manual/verification/visual_verification_governance.md)
- 처리 결과: [`mydocs/working/agent_bug_hunter.md`](../../../mydocs/working/archives/agent_bug_hunter.md)
