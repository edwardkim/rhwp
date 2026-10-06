# task_m100_4401 stage1 — 작업 계보: 캡슐 해시 체인(--parent) + lineage 연대기 검증

- 이슈: #4401 (3년 선행 축 — #3907 조망, #4399 workCapsule 적층)
- 브랜치: task_m100_4401 (base: task_m100_4393 — 캡슐 형식이 필요)

## 무엇을 만들었나

에이전트 노동의 **연대기**를 검증 가능한 데이터 구조로 만들었다. 영수증(#4391)이
"이 작업 하나가 사실"임을, 감사(#4399)가 "이 폴더의 작업들이 재현됨"을 증명한다면,
계보(#4401)는 **"이 작업들이 이 순서로 이어졌고 중간이 바뀌지 않았음"** 을 증명한다.

### 1. `replay --capsule ... --parent <이전캡슐.json>`

캡슐 발급 시 부모 캡슐을 지목하면, 부모 **파일 바이트의 SHA-256** 을 캡슐에 내장한다:

```json
"parent": { "capsule": "a.capsule.json", "sha256": "9f2c…" }
```

부모가 나중에 한 글자라도 바뀌면 이 기록 해시가 폭로한다 — git 커밋 체인과 같은
원리를 작업 캡슐에 적용한 것이다. 부모 없음은 `parent: null`(계보의 뿌리).

### 2. `rhwp lineage <캡슐.json> [--deep] [--json]`

머리(최신) 캡슐부터 체인을 거슬러 올라가며 링크마다 3중 판정:

| 축 | 질문 | 판정 |
|---|---|---|
| `parentOk` | 부모 파일이 발급 당시 그대로인가 | 기록 해시 == 실물 파일 해시 |
| `lineageOk` | **부모의 산출이 자식의 입력인가** | 부모 receipt.outputSha256 == 자식 receipt.inputSha256 |
| `reproduced` (`--deep`) | 그 링크가 지금도 재현되는가 | 재실행 산출 해시 == 영수증 해시 |

`lineageOk` 가 계보의 정의다 — "이전 작업의 산출물이 다음 작업의 입력물"이라는
연대기 불변식. 봉투는 `{schemaVersion, head, depth, valid, brokenAt, links[]}`,
깨진 체인은 exit 3(#2707 검증 단언 실패), 어느 링크가 왜 깨졌는지 `brokenAt` 과
링크별 축이 명세한다. 상대 경로 부모는 자식 캡슐 파일 기준으로 해석하고, 순환은
길이 1000 가드로 끊는다.

## 실측 증거 (tests/lineage_contract.rs, 2/2 green)

1. **2링크 왕복**: `run` 으로 O1 실산출 → 캡슐 A 발급 → O1 을 입력으로 캡슐 B
   발급(`--parent` A) → `lineage B` = depth 2 · valid · lineageOk true.
   - 이 lineageOk true 는 부수 수확이 아니라 **run↔replay 교차 결정론의 직접
     증거**다: `run` 이 디스크에 쓴 바이트와 `replay` 의 임시 재실행 바이트가
     같은 해시로 떨어져야만 참이 된다. 동일 명령 2회 결정론(#4391)에서 한 단계
     더 나간 실측이다.
2. **변조 폭로**: 부모 캡슐의 영수증 해시를 0으로 바꿔 저장 → `lineage` exit 3,
   `brokenAt` = 부모 경로, `links[1].parentOk` false.
3. **--deep**: 두 링크 모두 reproduced true.
4. **실패 규약**: 무인자/미지 옵션 exit 2, 머리 캡슐 없음 exit 1 + stdout 0바이트.

## 등재 4종 + 가드

- capabilities: `lineage` 항목 신설 + `replay` flags 에 `--parent` 추가
- MCP: `hwp_lineage` (tool_with_optional_args, deep 선택) — 39→40번째 도구
- help: `lineage <캡슐.json> [--deep] [--json]` 줄
- 프로필: 품질검증 += hwp_lineage
- 가드 green: cli_json_contract 31 · agent_profile_router 8 · capabilities_schema 2
  · replay 4 · audit 17(파일 전체) · Node gen:types/gen:check (봉투 40, 멱등)
- clippy --bin rhwp 0 · rustfmt 적용(변경분은 git diff --numstat 로 확인 — CRLF 소음 배제)

## 왜 3년 선행인가

- 1년 축(영수증·#4391): 작업 **하나**의 사실 증명 — 빅테크 로드맵의 "attestation" 방향.
- 2년 축(감사·캡슐·#4399): 작업 **집합**의 재현율 회계 — 아직 어느 런타임도 없는 축.
- 3년 축(계보·#4401): 작업 **역사**의 무결성 — 멀티 에이전트가 서로의 산출물을
  이어받는 시대에 "누가 무엇을 어떤 순서로 했고 중간에 바뀌지 않았다"를 제3자가
  오프라인 검증하는 인프라. 해시 체인이라 중앙 서버·신뢰 기관이 필요 없다.

## 한계·후속

- 계보는 단일 사슬(부모 1개)이다. 합류(여러 부모, DAG)는 다중 입력 계획이 생기면
  `parents[]` 로 확장한다 — 스키마에 여지를 남겼다(현재 값은 객체 하나 또는 null).
- `audit` 는 폴더 전수를 재현율로, `lineage` 는 사슬을 무결성으로 본다 — 폴더 안에
  체인이 여럿일 때 "전 체인 일괄 계보 감사"는 후속(#4401 논의에 적음).
