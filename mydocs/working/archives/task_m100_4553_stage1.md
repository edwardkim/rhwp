# [#4553] 9년 축 정산 증빙 — 처리 결과 (stage 1)

> 이슈: [#4553](https://github.com/edwardkim/rhwp/issues/4553) · 좌표 이슈: [#4463](https://github.com/edwardkim/rhwp/issues/4463)
> 브랜치: `task_m100_4553` (스택: `task_m100_4551` 위) · 설계서: `mydocs/tech/agent_roadmap/horizon_year9_settlement.md` (#4452)

## 1. 무엇을 만들었나 — 한 문단

(요구 명세 → 캡슐 → 게이트 판정 → 원장 기입)을 전부 검증 가능한 데이터로 잇는
9년 축을 구현했다. `settle propose` 는 명세서·캡슐·게이트 봉투의 **파일 바이트
sha256 셋**을 청구(settlementClaim)에 고정하고(청구자 서명 선택), `settle
verify` 는 3해시 대조 + 게이트 verdict 재확인 + 서명·이중 청구 opt-in 축을
밟으며, `settle record` 는 5년 앵커 로그와 **동형인** append-only 해시 체인
원장에 기입하되 **원장 전역에서 같은 capsuleSha256 는 한 번만 accepted** 된다
(이중 청구 exit 3). 새 검증 메커니즘 발명 0 — 검수는 6년 게이트, 원장 체인은
`anchor_log::load_kind` 재사용, 서명은 4년 사이드카다. 돈은 움직이지 않는다:
금액은 문자열로 운반만 하고, 산출물은 "지불 근거가 되는 제3자 검증 가능한
증빙"뿐이다(설계서 범위 경계 그대로).

## 2. 설계 결정 — 왜 이 모양인가

| 결정 | 선택 | 기각한 대안과 이유 |
|---|---|---|
| 원장 재사용 방식 | `anchor_log::load` 를 `load_kind(path, kind)` 로 매개변수화 — 검증 코드 경로 공유 | 원장 전용 체인 구현 복제 — "같은 형식, 같은 검증 코드 경로"(설계서 §2.3)가 문장 그대로 코드가 되게 했다. 기존 `load` 는 위임 한 줄이라 anchor 회귀 0 |
| 원장 항목에 capsuleSha256 포함 | 설계서 예시(claimSha256 만)에 **추가** | claim 파일을 열어야 이중 청구를 검사할 수 있는 구조 — 원장은 자립 검증이 결이다. 원장만으로 P3 전역 검사가 닫히려면 항목이 캡슐 해시를 직접 들어야 한다 (구현 세부의 정직 기록) |
| verify 의 파일 재공급 | `--workorder/--capsule/--gate-envelope` 필수 | 설계서 CLI 예시는 claim+ledger+keyring 만 보였지만, 3해시 "대조"는 실물 파일 재해시가 있어야 성립한다. 해시는 파일의 주장이 아니라 검증자의 재계산이다 |
| gateVerdict 재확인 | 해시 일치와 **별도로** 봉투 verdict == allow 요구 | 해시만 보면 "deny 봉투에 정직하게 고정한 청구"가 통과한다 — 검수 통과가 지불 조건이라는 축의 정의가 무너진다 |
| 미서명 명세서 | `workorderSignerOk: null` (보고), false 만 실패 | 필수화 — 발주자 서명 강제는 운영 성숙도를 앞서간다. 부재(null)와 위조(false)를 갈라 보고하고, 정책은 수신자의 몫 |
| 청구 사이드카 부재 | `--keyring` 지정 시 `signerOk: false` | null 보고 — 청구 귀속은 이 축의 본질이라 명세서와 달리 봐준다는 선택지가 없다 |
| rejected 기입 | `--verdict rejected` 허용, 이중 검사는 accepted 만 | rejected 도 중복 차단 — 재작업 후 재청구(다른 캡슐)가 정상 경로인데 rejected 기록이 캡슐을 소모하면 안 된다 |

## 3. 구현 명세

### 3-1. 코어 (`src/settle.rs` + `src/anchor_log.rs` 리팩터)

- `WORKORDER_KIND`/`CLAIM_KIND`/`LEDGER_KIND` 상수, `sha256_hex`.
- `parse_workorder` — kind·workorderId·**acceptancePolicy 필수**(검수 기준 없는
  명세서는 발급 단계 거부: 분쟁을 산문으로 되돌리지 않는다).
- `find_accepted(ledger, capsule_sha)` — P3 전역 검사 단일 구현.
- `make_ledger_line` — seq 연번·prevEntryHash(직전 줄 바이트 sha256)·at.
- `anchor_log::load_kind(path, kind)` 신설, `load` 는 위임 — anchor 계약 3/3
  회귀 그린으로 무해 실증.

### 3-2. 명령 3개

```
rhwp settle propose --workorder <wo> --capsule <c> --gate-envelope <g> -o <claim> [--sign-key <키>]
rhwp settle verify  <claim> --workorder <wo> --capsule <c> --gate-envelope <g>
                    [--keyring <k>] [--ledger <l>] [--sig <서명>]     # 실패 exit 3
rhwp settle record  <claim> --ledger <l> [--verdict accepted|rejected]  # 이중 청구 exit 3
```

- propose: 3해시 계산→claim 발급(+선택 서명 사이드카 `<claim>.sig.json`).
- verify: 축별 판정을 봉투로 — `workorderOk`/`capsuleOk`/`gateOk`(재해시 대조),
  `gateVerdict`(봉투 verdict 재확인), `signerOk`/`workorderSignerOk`(--keyring
  opt-in), `ledgerOk`/`duplicate`(--ledger opt-in). 종합 실패 = exit 3.
- record: 깨진 원장 기입 거부(anchor add 와 같은 문장), accepted 이중 청구
  거부(`existingSeq` 보고), append-only 기입.

### 3-3. 7+1 표면

| 표면 | 내용 |
|---|---|
| capabilities | `settle` 우산 — recordFields 19 선언 |
| MCP | `hwp_settle_propose`·`hwp_settle_verify`(keyring/ledger optionalArgs)·`hwp_settle_record` |
| help | 3행 |
| 품질검증 프로파일 | 3 도구 추가 (라우터 가드 31/31) |
| node | `settlePropose/Verify/Record` + 옵션 인터페이스 3종 + 패리티 우산 등재 |
| 지식 지도 §2-2 | 신규 필드 14 — 사전 236 → **250** |
| provenance | MAP(untrusted: NONE — 명세서 제목·금액 같은 문서 유래 문자열은 봉투에 싣지 않는다) + 스윕 레시피 4연쇄(propose→verify(전 축)→record→record 이중) |
| (+1) 대전 | 검증_사다리 가족 4항 — **79 명령**, δ=2장, `--check` 0 |

## 4. 검증 실측

| 게이트 | 결과 |
|---|---|
| `tests/settle_contract.rs` (신규 2) | 2/2 그린 — ①3해시 왕복(발급 봉투 해시 == 수동 재계산) ②전 축 verify ok ③record seq 0 → 이중 청구 exit 3(existingSeq 0) + 기입 후 verify --ledger 도 duplicate true ④P1 캡슐 변조 = capsuleOk false(타 축 무사) ⑤P4 명세서 금액 변조 = workorderOk false ⑥deny 봉투 청구는 해시 일치여도 rejected ⑦acceptancePolicy 없는 명세서 propose 거부 exit 2 ⑧깨진 원장 기입 거부 ⑨서명 후 청구 변조 signerOk false |
| `provenance_contract` | 10/10 (settle 레시피 4 포함) |
| `cli_json`/`agent_codex`/`router`/`anchor` | 2+8+3+31 전부 그린 (anchor 회귀 = load_kind 무해) |
| node 바인딩 | **427/427** — settle 우산·래퍼 3종 패리티 포함 |
| clippy | 신규 경고 0 (기존 2건 유지) |

### 시각 증거 (`mydocs/report/edit_demo_4553/`) — 전부 실물, 자기 검증 완료

1. `01_delivery.png` — 발주 대상 vs 납품물(셀(0,0) = "납품 v1.2 — 검수 통과분",
   서명 캡슐로 발급).
2. `02_settle_roundtrip.png` — 명세서(정책 인라인·문자열 금액) → **실제
   `rhwp gate --deep` 실행 allow**(수제 봉투 아님 — 재현 판정+서명 판정 실측) →
   3해시 고정 청구 → 전 축 green verify → 원장 seq 0 기입 → 같은 캡슐 재청구
   duplicate·existingSeq 0·exit 3.

실측 원문:

```json
{"verify": {"workorderOk": true, "capsuleOk": true, "gateOk": true,
            "gateVerdict": "allow", "signerOk": true, "duplicate": false, "verdict": "ok"},
 "record": {"seq": 0}, "dup": {"duplicate": true, "existingSeq": 0}}
```

## 5. 부딪힌 함정 (정직 기록)

1. **replay 는 빈 steps 를 exit 2 로 거부** — 픽스처 플랜에 실존 문자열 무해
   치환 스텝을 넣어야 한다.
2. **원장 마지막 줄 내용 변조는 체인이 못 잡는다** — prevEntryHash 는 후속 줄이
   봉인하는 구조라 꼬리는 미봉인(5년 동형의 한계 그대로 — 최종 봉인은 체크포인트
   공표). 테스트를 "체인이 잡는 변조"(kind 구조 파손)로 정정하고 한계를 주석에
   명시했다.
3. **admissionPolicy 규칙은 `{id, require: {key: {op: value}}}` 형** — 설계서
   예시의 축약(`{key, op, value}`)과 다르다. 시각 증거의 실제 gate 실행이 이걸
   잡아냈다(수제 봉투였으면 못 잡았다).
4. **gate 의 `reproduced` 판정은 `--deep` opt-in** — 재현은 비싸서 기본
   미계산(unavailable → deny). 검수 정책에 reproduced 를 넣는 발주자는 검수
   실행이 `--deep` 여야 한다는 운영 사실을 데모가 실측으로 남겼다.

## 6. 사다리에서의 자리와 다음

- 1~8년이 만든 전 축이 이 축의 부품이다: 캡슐(1~3년)·서명(4년)·체인(5년 동형)·
  게이트(6년)·그리고 가림 캡슐(8년)로 "내용 비밀 납품"까지 조합 가능.
- 다음: **Y10 감사 표준**(audit-report/recall-scope/conformance — 설계서
  b573d11e9) — 사다리 마지막 계단. 그 뒤는 조합 통합(bundle --redact, gym 4부
  정산 과제, 리더보드).
