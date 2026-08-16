---
kind: guide
status: active
canonical: mydocs/manual/planner_templates/README.md
last_verified: 2026-08-13
---

# 계획 템플릿 — 자가검증 편집을 시작점으로

`rhwp run <계획.json>` 이 받는 편집 계획서의 **재사용 가능한 시작점**이다. 에이전트가
매번 맨 계획을 짜는 대신, 검증 조건을 이미 품은 템플릿에서 출발한다 — 그래서 계획을
세우면 자연히 증명이 남는다([AGENTS.md 작업 증빙](../../../AGENTS.md#작업-증빙--에이전트-기본-경로-권장)의
AW-L1 영수증 축).

## 템플릿

| 파일 | 무엇을 | 지목 방식 |
|---|---|---|
| [`set_cell.plan.json`](set_cell.plan.json) | 표 칸 기록 | `table·row·col`(0 기준) |
| [`replace_text.plan.json`](replace_text.plan.json) | 본문 문자열 치환 | `find`(0건이면 선검증 거부) |
| [`fill_fields.plan.json`](fill_fields.plan.json) | 누름틀 채우기(메일머지) | `필드이름` 또는 `이름[순번]` |
| [`set_checkbox.plan.json`](set_checkbox.plan.json) | 빈 체크박스 표시 | `occurrence`(0 기준 □ 순번) |

`<...>` 는 채울 자리다. 지목 대상은 실측으로 찾는다: `rhwp fields <문서>`(필드),
`rhwp export-tables <문서>`(표 좌표), `rhwp search <문서> <문자열>`(치환 대상 존재).

## 쓰는 법 — 계획이 곧 증명이 되게

```bash
# 1) 템플릿을 복사해 <...> 를 채운다 → my.plan.json
# 2) 먼저 선검증만 (디스크 무변경)
rhwp run my.plan.json --dry-run --json

# 3) 실행 (전 step 인메모리 적용 → 단언 통과 시에만 한 번 저장)
rhwp run my.plan.json --json

# 4) AW-L1 영수증 — 같은 계획을 캡슐과 함께 남긴다(제3자 재현 검증 가능)
rhwp replay my.plan.json --capsule work.capsule.json --json
```

`run` 대신 `replay --capsule` 로 실행하면 입력·계획·산출 3해시가 캡슐 하나에 고정돼
[작업 증빙 사다리](../../../AGENTS.md#작업-증빙--에이전트-기본-경로-권장)의 첫 단(영수증)이
된다. 연속 작업은 `--parent 이전.capsule.json` 으로 잇는다(계보).

## 자가검증 — 단언(assertions)

각 템플릿은 `assertions.notFoundEmpty: true` 를 기본으로 든다 — 지목한 대상이 하나도
빠지지 않았음을 요구한다(없는 대상은 실행 전에 `invalid` 로 거절되므로 구조적으로
보장된다). "성공했다고 보고했는데 산출물이 깨진" 상태를 계획이 스스로 막는다.

**더 강한 검증(옵션)** `assertions.verify: true` — 저장 직전 산출 바이트를 다시 읽어
인메모리 IR 과 대조하고, 차이가 있으면 `exit 3` 이며 **디스크는 무변경**이다. 이는
편집이 저장 왕복(save→reload)을 바이트로 견디는 경우에만 통과한다 — 견디지 못하는
문서·편집에서는 실패하는데, 그것이 곧 저장 충실도 갭을 드러내는 쓸모다(무조건 켜지
않고, 왕복 안정성이 확인된 워크플로에서 켠다).

## 경합 유실 차단(옵션) — preconditions

계획 수립과 실행 사이에 다른 에이전트/사람이 문서를 바꿀 수 있다. `preconditions`
축을 더하면 그 유실을 차단한다:

```json
"preconditions": { "inputSha256": "<원본 SHA-256 64자리>" }
```

실행 시점의 실제 해시와 다르면 아무것도 적용하지 않고 거절한다 — **실행 0건 · 디스크
무변경 · `preconditionFailed{kind,expected,actual}` + `nextCall`(재계획 힌트) ·
`exit 3`**(#2707 의 "판정" 계열이다. 계획서는 옳고 틀린 것은 문서 쪽이라 사용법
오류가 아니다). `--dry-run` 도 **같은 대조·같은 판정**을 한다 — 예행이 통과했는데
실행이 거부되는 상태를 만들지 않기 위해서다. `nextCall` 은 기대 해시를 실제 해시로
갈아 끼운 계획을 `--dry-run` 으로 다시 선검증하는 **그대로 실행 가능한 호출**이다:
통과하면 `--dry-run` 만 빼고 재실행하고, `invalid` 가 나오면 문서를 다시 읽고
재계획한다. 해시는 `rhwp-agent fingerprint <문서>` 또는 `sha256sum` 으로 얻는다.

## 판(version) 규약

계획 문법 판번호는 `planVersion: "1.0"`(스키마 판번호와 별개 — `rhwp export-plan-schema`
의 `planSchemaVersion` 은 1.2). 실행기는 `"1.0"` 만 받는다. 스텝·단언·전제 형식의
단일 출처는 `rhwp export-plan-schema` 다.
