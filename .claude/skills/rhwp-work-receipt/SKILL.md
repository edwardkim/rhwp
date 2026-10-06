---
name: rhwp-work-receipt
description: rhwp 검증 사다리(작업 영수증·감사·계보)로 에이전트 노동을 증명 가능하게 만듭니다. replay 로 영수증(입력·계획·산출 SHA-256 3종) 발급과 제3자 재현 검증, --capsule/--parent 로 작업 캡슐과 해시 체인 구축, audit 로 캡슐 폴더 전수 재현율 회계, lineage 로 연대기(부모 산출=자식 입력) 무결 판정까지 수행합니다. 트리거 — 사용자가 "이 작업 증명해/영수증 남겨", "타인 산출물 재현 검증", "작업 캡슐/체인 만들어", "캡슐 폴더 감사", "계보 검증", "재현율", "rhwp replay/audit/lineage" 등을 요청할 때.
---

# rhwp-work-receipt — 검증 사다리 실행 규약 Skill

## 목적

에이전트가 한 일을 **말이 아니라 재실행으로** 증명한다. 사다리 3단을 상황에
맞게 고른다:

| 단 | 명령 | 증명하는 것 | 언제 |
|---|---|---|---|
| 영수증 | `rhwp replay` | 작업 **하나**가 사실 (3해시) | 산출물 하나를 넘기거나 받을 때 |
| 감사 | `rhwp audit` | 캡슐 **폴더**의 재현율 회계 | 축적된 작업을 조직 단위로 재검증할 때 |
| 계보 | `rhwp lineage` | 작업 **역사**의 무결 (해시 체인) | 여러 작업이 이어질 때 (이전 산출 = 다음 입력) |

전제는 **결정론**이다 — 같은 계획은 같은 바이트를 낸다(저장소 실측 고정).
그래서 "재실행해서 해시가 같다"가 증명이 된다.

이 스킬은 **새 CLI 를 만들지 않는다.** 이미 devel 에 있는 `replay` /
`--capsule` / `--parent` / `audit` / `lineage` 를 에이전트가 잘못 조립하지
않게 배선한다. gym 트레이스가 아니라 실작업 증명이다.

## 자식 문서 (이 스킬의 본문)

SKILL.md 는 라우터다. 작업 종류에 맞는 자식을 **읽고 나서** 명령을 조립한다.

| 작업 | 읽기 | 경로 |
|------|------|------|
| 단건 발급·제3자 검증 | 영수증 | [references/replay-attest.md](references/replay-attest.md) |
| 캡슐·부모 해시 체인 | 캡슐 | [references/capsule-chain.md](references/capsule-chain.md) |
| 폴더 전수 재현율 | 감사 | [references/audit-accounting.md](references/audit-accounting.md) |
| 연대기 3축 | 계보 | [references/lineage-chronicle.md](references/lineage-chronicle.md) |
| exit 3/1/2 | 종료 코드 | [references/exit-codes.md](references/exit-codes.md) |
| toolVersion·귀속 금지 | 함정 | [references/pitfalls.md](references/pitfalls.md) |
| 요청 → 단 고르기 | 판단 트리 | [references/decision-tree.md](references/decision-tree.md) |
| 봉투 키 사전 | 카탈로그 | [references/envelope-field-catalog.md](references/envelope-field-catalog.md) |
| 레시피 색인 | 색인 | [references/recipe-index.md](references/recipe-index.md) |

실측 워크스루는 [examples/](examples/README.md) 다.
기계가 읽는 픽스처는 [fixtures/catalog.json](fixtures/catalog.json) 다.

## 판정 규약 (모든 단 공통)

- 판정은 예외가 아니라 **봉투 데이터**다: `reproduced`·`reproducedRate`·
  `valid`·`brokenAt`. 재현 실패·깨진 체인 = **exit 3** (도구 고장 아님).
- IO 실패 exit 1, 사용법 exit 2. 실패 경로 stdout 은 0바이트다.

## 절차 1 — 단건 영수증 (발급 → 제3자 검증)

```bash
# 발급(attest): 계획을 임시 산출로 재실행해 3해시 영수증. 사용자 경로는 건드리지 않는다.
rhwp replay --plan-json '{"planVersion":"1.0","input":"원본.hwp","output":"산출.hwp","steps":[…]}' --json
# → { inputSha256, planSha256, outputSha256, toolVersion, steps, mode:"attest" }

# 제3자 검증(verify): 상대가 주장한 산출 해시를 재현으로 대조.
rhwp replay --plan-json '<같은 계획>' --expect-output-sha256 <상대가 준 64hex> --json
# → reproduced:true(exit 0) / false(exit 3 — 주장 기각, 근거는 봉투)
```

함정: `--plan-json` 대신 계획 파일 경로 위치 인자도 된다. `output` 경로는
영수증 발급 중 **생성되지 않는다**(임시 재실행) — 실산출이 필요하면 `rhwp run`
을 따로 실행하라.

## 절차 2 — 작업 캡슐과 해시 체인

```bash
# 캡슐: 계획+영수증의 자기완결 교환 파일 (제3자가 이것만 받으면 재현 가능).
rhwp replay --plan-json '<계획A>' --capsule a.capsule.json --json

# 체인: 다음 작업의 입력이 이전 작업의 실산출일 때, --parent 로 부모를 지목.
rhwp run planA.json --json                     # 실산출 O1 생성
rhwp replay --plan-json '<계획B: input=O1>' --capsule b.capsule.json --parent a.capsule.json --json
```

함정 실록:
- **캡슐은 발급 후 불변이다.** 에디터·포맷터로 열어 저장하는 순간 부모 해시
  대조가 깨진다 — 의도된 동작이다(변조 검출). 수정하려면 재발급하라.
- `--parent` 의 상대 경로는 **캡슐 파일 기준**으로 저장·해석된다(호출 cwd
  아님). 같은 폴더에 두는 것이 가장 단순하다.
- `--capsule` 과 `--parent` 가 같은 파일을 가리키면 거부된다(부모 덮어쓰기 방지).

## 절차 3 — 폴더 감사 (재현율 회계)

```bash
rhwp audit <캡슐 폴더> --json
# → { total, reproduced, reproducedRate, failed:[{capsule, 기대/실측 해시 또는 사유}] }
```

- 대상은 폴더 직속 `*.capsule.json` (비재귀). 0개면 exit 2.
- `failed` 가 하나라도 있으면 exit 3 — **회계는 봉투로 읽고**, 실패 캡슐만
  절차 1의 verify 로 개별 추적하라.

## 절차 4 — 계보 검증 (연대기 무결)

```bash
rhwp lineage b.capsule.json --json          # 머리(최신) 캡슐에서 뿌리까지 거슬러 판정
rhwp lineage b.capsule.json --deep --json   # 링크마다 재실행 재현까지 (비용: 링크 수)
```

링크 판정 3축: `parentOk`(부모 파일이 발급 당시 그대로인가) ·
`lineageOk`(**부모 산출 해시 == 자식 입력 해시** — 연대기의 정의) ·
`reproduced`(`--deep`). 하나라도 false 면 `brokenAt` 이 어느 캡슐인지 가리키고
exit 3. 머리 캡슐 없음은 exit 1(IO)이다.

## 권장 흐름 (요청별)

- "이 편집 증명해 줘" → 절차 1 발급 → 영수증 JSON 을 산출물과 함께 전달.
- "받은 작업이 진짜인지 확인" → 절차 1 verify (계획과 주장 해시를 요구하라).
- "작업들을 이어서 기록" → 절차 2 체인 → 마지막에 절차 4 로 전체 판정.
- "쌓인 캡슐 전수 점검" → 절차 3 → 실패분만 개별 verify.

자세한 분기는 [references/decision-tree.md](references/decision-tree.md).

## 경계 (정직)

- 캡슐·영수증은 **누가** 했는지는 증명하지 않는다 — 귀속(서명) 축은 4년 축
  구현(#4511) 착지 후 이 스킬의 2부로 확장된다. **attribution/signature claim 없음**.
- 영수증의 `toolVersion` 이 다르면 재현 불일치가 날 수 있다 — 판정 전에 버전
  부터 대조하라.
- 새 `receipt` / `work-receipt` / `prove` 명령을 발명하지 않는다.
- gym pack·채점·admission 을 이 경로에 끌어들이지 않는다.
- 온보딩·MCP 세션·출처 표지·안전 편집·문서 트리아지 스킬 본문을 이 파동에서
  고치지 않는다.
- DocumentCore 편집 구현을 건드리지 않는다.

## 상세 레퍼런스

- 영수증: [references/replay-attest.md](references/replay-attest.md)
- 캡슐: [references/capsule-chain.md](references/capsule-chain.md)
- 감사: [references/audit-accounting.md](references/audit-accounting.md)
- 계보: [references/lineage-chronicle.md](references/lineage-chronicle.md)
- 종료 코드: [references/exit-codes.md](references/exit-codes.md)
- 함정: [references/pitfalls.md](references/pitfalls.md)
- 판단 트리: [references/decision-tree.md](references/decision-tree.md)
- 필드 카탈로그: [references/envelope-field-catalog.md](references/envelope-field-catalog.md)
- 레시피 색인: [references/recipe-index.md](references/recipe-index.md)
- 워크스루: [examples/README.md](examples/README.md)
- 픽스처: [fixtures/catalog.json](fixtures/catalog.json)
- 작업 기록: [`mydocs/working/agent_work_receipt.md`](../../../mydocs/working/archives/agent_work_receipt.md)
- 명령 정본: [`mydocs/manual/cli_commands.md`](../../../mydocs/manual/cli_commands.md)
- 지식 지도 §영수증·감사·계보: [`mydocs/manual/agent_knowledge_map.md`](../../../mydocs/manual/agent_knowledge_map.md)
