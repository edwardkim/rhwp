# [#4551] 8년 축 선택적 공개 — 처리 결과 (stage 1)

> 이슈: [#4551](https://github.com/edwardkim/rhwp/issues/4551) · 좌표 이슈: [#4463](https://github.com/edwardkim/rhwp/issues/4463)
> 브랜치: `task_m100_4551` (스택: `task_m100_4549` 위) · 설계서: `mydocs/tech/agent_verification_decade.md` 8년 절

## 1. 무엇을 만들었나 — 한 문단

계보 증명은 공개하고 내용은 비밀로 하고 싶다는 요구(8년 축)를 **가림 캡슐 + 비밀
개봉 파일**의 분리로 구현했다. `disclose redact` 는 캡슐 `plan` 의 문자열 잎
전부를 `sha256(값‖salt)` 커밋으로 치환한 가림 캡슐과, 값·salt·원본 `planText`
를 담은 개봉 파일을 발급한다. `disclose verify` 는 개봉된 필드만 커밋과
대조하는 **부분 개봉**(verifiedFields/mismatched/unopened)이고, `disclose
restore` 는 전체 개봉으로 캡슐을 **바이트 단위 원본과 동일하게** 되살린다 —
그래서 원본의 Ed25519 분리 서명(4년 축)이 복원본에서 그대로 valid 다. ZK 증명은
범위 밖(설계서 정직 조항 승계): 이 축이 파는 것은 영지식이 아니라 **salt 커밋
기반의 선택적 공개와 바이트 복원 보증**이다.

## 2. 설계 결정 — 왜 이 모양인가

| 결정 | 선택 | 기각한 대안과 이유 |
|---|---|---|
| 커밋 형태 | `sha256(값 UTF-8 ‖ salt hex ASCII)` — 필드마다 독립 32바이트 난수 salt, 도구 강제 생성 | 무salt `sha256(값)` — 저엔트로피 필드(짧은 문구·경로)가 사전 대입으로 전수 복원된다. 수동 salt 입력 — 재사용·저엔트로피 salt 사고를 도구가 막을 수 없다 |
| 가림 단위 | 문자열 잎 전부(옵트아웃 없음), 구조 골격 `planVersion`·`action` 만 평문 | 필드 선택형 가림 — "무엇을 가릴지"의 협상이 발급 시점으로 앞당겨져 개봉 협상(부분 공개)과 중복되고, 빠뜨린 필드가 그대로 샌다. 전부 가리고 **공개를 개봉 쪽에서 고르는** 편이 실수 여지가 없다 |
| 좌표계 | JSON 포인터(`/steps/0/find`) | 자체 경로 문법 발명 — 이 저장소 결(기존 표준 재사용, HWPX=zip 선례)에 반한다. serde_json `Value::pointer` 가 이미 있다 |
| 복원 보증 | 개봉 파일이 원본 `planText` 원문을 보관 → restore 가 **바이트 동일** 복원 | plan 재직렬화 복원 — 키 순서·공백이 달라져 원본 서명이 깨진다. 가림·복원이 서명 축과 정합하는 유일한 길은 원문 보관이다(개봉은 어차피 비밀 보관물) |
| 실패 언어 | 커밋 불일치·커버리지 부족 = exit 3(판정 데이터), kind 오류 = exit 2(사용법) | 전부 exit 1 — #2707 종료 코드 사전과 어긋난다. 검증 계열(verify-signature·gate·bundle)과 같은 문장 |
| 부분 개봉으로 restore | 거부(exit 3) + **산출물 미생성** | 열린 필드만 채운 부분 복원 — "복원"의 의미가 흐려지고 반쪽 캡슐이 유통된다. 부분은 verify 의 세계, 복원은 전체 개봉의 세계로 갈랐다 |

## 3. 구현 명세

### 3-1. 코어 `src/disclose.rs`

- `OPENING_KIND = "capsuleOpening"` — 개봉 파일 kind.
- `commit(value, salt_hex)` — 커밋 계산 한 곳.
- `redact_plan(plan, path, key_hint, openings)` — 재귀 걷기. 문자열 잎 →
  `{"committed": hex}` 치환 + `(포인터, 값, salt)` 수집. 숫자·불리언은 그대로
  (구조 정보), `planVersion`·`action` 은 골격 공개.
- `committed_at(plan, pointer)` / `committed_count(plan)` — 검증·unopened 계수.

### 3-2. 명령 3개 (`src/main.rs`)

```
rhwp disclose redact  <캡슐> -o <가림> --opening-out <개봉> [--json]
rhwp disclose verify  <가림> --opening <부분개봉> [--json]        # 불일치 exit 3
rhwp disclose restore <가림> --opening <전체개봉> -o <복원> [--json]  # 비동일 exit 3
```

- redact: 원본 sha256 을 가림 캡슐(`originalCapsuleSha256`)과 개봉 양쪽에 박아
  restore 의 성공 기준점을 만든다. `planText` 원문은 개봉으로 이사하고 가림본에는
  자리표시 문자열 + `planRedacted: true`.
- verify: 개봉 항목마다 `commit(값, salt) == committed` 대조 →
  `verifiedFields`/`mismatched`/`unopened`. 형식 오류 개봉 항목도 mismatched 로
  계수(무결성 실패는 조용히 넘기지 않는다).
- restore: 전체 커버리지 선검사(`개봉 수 < committed 잎 수` = exit 3, **파일
  미생성**) → planText 원문 재이식 → `planRedacted`/`originalCapsuleSha256`
  제거 → 바이트 동일 검사.

### 3-3. 7+1 등재 표면 (전판 규약 그대로)

| 표면 | 내용 |
|---|---|
| capabilities | `disclose` 우산 항목 — recordFields 13개 선언 |
| MCP | `hwp_disclose_redact` · `hwp_disclose_verify` (restore 는 개봉 전체가 필요한 복구 작업이라 MCP 노출 보류 — 에이전트 루프의 단위는 발급·검증) |
| help | 3행 (`disclose redact/verify/restore`) |
| 품질검증 프로파일 | `hwp_disclose_redact`·`hwp_disclose_verify` 추가 |
| node 래퍼 | `discloseRedact`/`discloseVerify`/`discloseRestore` + 패리티 우산 등재 |
| 지식 지도 §2-2 | 신규 필드 10개 — 사전 226 → **236** |
| provenance | MAP 항목(untrusted: NONE — 값 원문은 개봉 파일에만, 봉투에 싣지 않는 것이 이 축의 존재 이유) + 스윕 레시피 4개(replay 발급→redact→verify→restore 연쇄) |
| (+1) 대전 | 검증_사다리 가족 4항목 — **78 명령**, 재생성 δ=2장, `--check` 0 |

## 4. 검증 실측

| 게이트 | 결과 |
|---|---|
| `tests/disclose_contract.rs` (신규 2) | **2/2 첫 시도 그린** — ①왕복(누설검사·골격공개·find 커밋화) ②부분개봉(verifiedFields/unopened 정확) ③위조검출(값 변조=mismatch·exit 3) ④바이트복원(sha 동일 + **원본 사이드카 valid**) ⑤방어(부분개봉 restore 거부+파일 미생성·kind exit 2·비캡슐 exit 2) |
| `provenance_contract` | 10/10 — 신규 레시피 4개 포함, 선언 필드 전수 봉투 실측 |
| `cli_json` + `agent_codex` + `agent_profile_router` | 2 + 8 + 31 전부 그린 |
| node 바인딩 | **427/427** — disclose 우산·래퍼 3개 패리티 포함 |
| clippy | 신규 경고 **0** (기존 2건은 스택 이전 코드: anchor_log merkle·run_plan_engine) |
| fmt | 적용 완료 (`cargo fmt` 가 os error 206 으로 실패 → 변경 파일 직접 `rustfmt --edition 2021`, §5 함정 참조) |

### 시각 증거 (실문서 왕복 — 자기 검증 완료)

`mydocs/report/edit_demo_4551/`:

1. `01_document_edit.png` — `samples/table-001.hwp` 원본 vs `set_cell` 편집 후
   (셀(0,0) = "기밀 단가 3,200원/셀") — 캡슐 아래에서 **실제로 일어난** 편집.
2. `02_redact_roundtrip.png` — 같은 캡슐의 네 시점: 평문 plan → salt 커밋 가림
   (`비밀 원문 포함: False` 실측) → 부분 개봉(/input 만, unopened 2) → 전체 복원
   (`byteIdentical: true` + 원본 서명 `valid`).

실측 판정 원문:

```json
{"committedFields": 3,
 "partialVerify": {"verdict": "ok", "verified": ["/input"], "unopened": 2},
 "restore": {"byteIdentical": true}, "signature": "valid", "secretLeaked": false}
```

## 5. 부딪힌 함정 (정직 기록)

1. **`cargo fmt` os error 206** — Windows 명령줄 길이 한계. 저장소 파일 수가
   늘며 `cargo fmt` 가 전 파일 경로를 한 줄에 싣다 터졌다. 우회: 변경 파일만
   `rustfmt --edition 2021 <files>`. CI(리눅스)는 무관.
2. **replay 는 산출물 임시-전용** — 시각 증거 렌더에 쓸 출력 파일이 없어
   `export-svg` 가 os error 2. gym T14 에서 이미 만난 함정의 재현 —
   `run --plan-json` 으로 실물 산출을 별도 생성해 렌더.
3. **`export-svg -o` 는 디렉터리 규약** — 페이지 파일이 `<이름>.svg` 로 그 안에
   생긴다. 단일 파일 경로를 주면 그 이름의 폴더가 생긴다.
4. **heredoc 한글 파손 재재현** — bash heredoc 에 한글 포함 패치를 넣으면 인코딩
   파손 + 이스케이프 접힘. 확립 규약(파이썬 파일을 Write 로 만들고 실행) 준수로
   복구. 패처는 전 앵커 `assert count==1` 원자 적용이라 파일 오염은 없었다.

## 6. 이 축이 사다리에서 하는 일

- **1~3년(영수증·감사·계보)** 이 만든 해시 사슬은 가림 캡슐에서도 그대로 돈다 —
  커밋 치환은 `plan` 내부 잎의 값만 바꾸고, 검증 대상 해시는 파일 바이트 기준으로
  새로 계산되는 가림본 자신의 것이므로 축이 충돌하지 않는다.
- **4년(서명)** 과의 정합이 급소 — 바이트 완전 복원이므로 원본 사이드카가 복원본에
  그대로 valid. 가림을 위해 서명을 다시 받을 필요가 없다.
- **7년(연합)** 의 조직 간 교환에서 "조상 캡슐은 넘기되 내용은 비밀" 요구를 이제
  가림 캡슐로 답할 수 있다(번들에 가림본을 담는 통합은 후속 — §7).
- **9년(정산)·10년(감사 표준)** 이 다음 계단: 부분 개봉의 "몇 개를 열었나"가
  정산 증빙의 수량 언어(`unopened`)로 이어진다.

## 7. 남은 것 (후속 이슈 후보)

- 번들(7년)에 가림 캡슐 + 개봉 별송의 공식 통합 — 지금도 파일로는 가능하나
  `bundle export --redact` 한 방이 없다.
- 개봉 파일 자체의 서명(개봉 위조 방지의 2층) — 지금은 커밋 대조가 위조를 잡지만
  "누가 개봉을 발급했나"는 답하지 않는다.
- Y9 정산 증빙(workorder/claim/ledger) — 설계서 `086ff2958` 절, 다음 착공.
