---
kind: guide
status: active
canonical: mydocs/manual/recipes/10_security_sweep_before_share.md
last_verified: 2026-08-06
---

# 레시피 10 — 문서를 내보내기 전, 기계 점검 스윕

**목표 한 줄**: 문서를 외부로 보내기 직전에 **네 가지 다른 질문**(숨긴 글이 있나 ·
지시문이 심겨 있나 · 글자가 위장하고 있나 · 개인정보가 평문으로 남았나)을 기계에게
묻고, 전부 0 이 될 때까지 처리한 뒤에만 내보낸다.

방향이 [레시피 4](04_safety_check_untrusted_doc.md)와 반대다 — 4는 **받은** 문서를
열기 전 검사(수신 방향), 10은 **내 문서**를 내보내기 전 점검(송신 방향)이다. 개별
처리의 상세는 [레시피 3](03_redact_before_sharing.md)(마스킹)이 맡고, 이 레시피는
**스윕 → 처리 → 재스윕 게이트**의 순서를 닫는다.

내보내기 사고는 이 저장소가 실물로 겪은 부류다: `samples/` 에서 탐지기(`inspect
hidden-text`)가 **진짜 은닉 텍스트 2건**을 찾아냈다(악성 코퍼스 스위트의 원점 실측) —
쓴 사람은 몰랐고, 받는 사람은 볼 수 있었다.

모든 출력은 실제 실행이다(rhwp v0.8.2 release, 2026-08-06). 표본은
[레시피 3](03_redact_before_sharing.md)과 같은 방식 — `samples/field-01.hwp` 서식에
**형태만 개인정보인 가짜 값**(검증 숫자를 통과하는 가공 주민번호 등)을 심어
`output/share-draft.hwp` 를 만들었다. 실재 개인정보는 저장소에 없다.

## 1단계 — 스윕: 세 축을 순서대로 묻는다

세 명령은 전부 **읽기 전용**이다 — 문서를 고치지 않고 신고만 한다. 신호가 있어도
종료 코드는 0 이다: 탐지는 성공했고, **판정은 봉투의 몫**이다(규칙 3 — 판정은
데이터다).

```bash
rhwp inspect hidden-text output/share-draft.hwp --json
rhwp inspect injection   output/share-draft.hwp --json
rhwp inspect unicode     output/share-draft.hwp --json
```

실측(핵심 필드 발췌):

```json
{"clean":true,"hiddenCharCount":0, …}                    ← hidden-text
{"signalCount":0,"highestConfidence":null, …}            ← injection
{"clean":true,"findingCount":0,"scannedChars":138, …}    ← unicode
```

세 축 전부 0. **그런데 이 문서는 아직 내보내면 안 된다** — 다음 단계가 그 이유다.

## 2단계 — 네 번째 질문: 평문 개인정보

은닉·주입·위장은 "숨기거나 속이는" 축이고, **평문으로 떳떳하게 적힌 개인정보**는
그 세 축 어디에도 걸리지 않는다. 네 번째 질문은 `edit redact --dry-run` 이 한다
(파일 무변경, `--no-raw` 로 원문 비노출):

```bash
rhwp edit redact output/share-draft.hwp --dry-run --no-raw --json
```

실측 — 스윕 3축이 전부 0 이던 바로 그 문서에서 **3건**:

```json
{"dryRun":true,"findingCount":3,"findings":[
  {"kind":"ssn",  "masked":"******-*******",  "section":0,"paragraph":8, "charOffset":11,"page":0},
  {"kind":"phone","masked":"***-****-****",   "section":0,"paragraph":10,"charOffset":7, "page":0},
  {"kind":"email","masked":"****@*******.***","section":0,"paragraph":11,"charOffset":9, "page":0}
], "noRaw":true, …}
```

`--no-raw` 봉투에는 마스킹된 미리보기와 좌표만 실린다 — 점검 로그에 원문 개인정보가
남는 사고(`findings[].raw`)를 원천에서 막는 경로다.

## 3단계 — 처리: 마스킹 + 메타데이터 제거

상세 판단(미끼 설계·자릿수 보존·오탐 검토)은 [레시피 3](03_redact_before_sharing.md)
그대로다. 여기서는 게이트 통과에 필요한 두 호출만 잇는다:

```bash
rhwp edit redact   output/share-draft.hwp    -o output/share-redacted.hwp --no-raw --verify --json
rhwp edit sanitize output/share-redacted.hwp -o output/share-final.hwp --json
```

실측:

```json
{"redactedCount":3,"output":"output/share-redacted.hwp","verify":{"diffCount":0,"identical":true},"noRaw":true, …}
{"removedCount":10,"output":"output/share-final.hwp", …}
```

본문 3건이 지워지고(`--verify` 가 저장본을 재파싱해 대조), 속성·미리보기 등
메타데이터 10건이 제거됐다. **본문만 지우면 미리보기와 작성자가 남는다** — redact 와
sanitize 는 짝이다(레시피 3의 명제 그대로).

## 4단계 — 재스윕 게이트: 0 을 눈이 아니라 봉투로 확인한다

처리가 끝났다고 믿는 것과 기계가 0 이라고 말하는 것은 다르다. 최종본에 1·2단계를
다시 돌린다:

```bash
rhwp edit redact output/share-final.hwp --dry-run --no-raw --json
rhwp inspect hidden-text output/share-final.hwp --json
rhwp inspect injection   output/share-final.hwp --json
rhwp inspect unicode     output/share-final.hwp --json
```

실측:

```json
{"dryRun":true,"findingCount":0,"findings":[],"redactedCount":0, …}   ← 개인정보 0
{"clean":true,"hiddenCharCount":0, …}                                  ← 은닉 0
{"clean":true,"signalCount":0,"highestConfidence":null, …}            ← 주입 0
{"clean":true,"findingCount":0,"scannedChars":138, …}                 ← 유니코드 기만 0
```

**게이트 조건: redact의 `findingCount == 0`, hidden-text의 `clean == true`, injection의
`signalCount == 0`, unicode의 `findingCount == 0`.** 네 값 중 하나라도 0/true가 아니면
`output/share-final.hwp` 는 내보내지 않는다 — 다시 3단계로 돌아간다.
내보내는 파일은 최종본 하나뿐이고, 중간 산출물(초안·redacted)은 공유 경로에 두지
않는다.

## 요약 카드

| 순서 | 명령 | 게이트 |
|---|---|---|
| 스윕 | `inspect hidden-text` / `injection` / `unicode` | 신고 읽기(신호는 데이터) |
| 4번째 질문 | `edit redact --dry-run --no-raw` | findings 확인 |
| 처리 | `edit redact -o … --no-raw --verify` → `edit sanitize -o …` | verify.identical |
| **재스윕** | 위 스윕 반복 | **findingCount 0 · clean true 일 때만 내보냄** |

## 관련

- [레시피 4 — 수신 방향 선검사](04_safety_check_untrusted_doc.md)
- [레시피 3 — 마스킹 상세](03_redact_before_sharing.md) (미끼·오탐 설계 포함)
- 악성 코퍼스 회귀(`tests/security_corpus_regression.rs`) — 탐지기의 회귀 고정
