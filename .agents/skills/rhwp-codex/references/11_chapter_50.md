# 50장 항해 — 검증 사다리 — 판정은 데이터다

파일: `mydocs/manual/agent_codex/50_검증_사다리.md`
성격: **generated**

이 장은 `generated: tools/gen_agent_codex.py` frontmatter 를 가진다.
수기 수정 금지. 표본을 고치고 싶으면 생성기의 LIVE 계획을 고친다.

요청 갈래: **검증** → 이 장.

## 읽는 법

1. `### \`이름\`` 이 명령 장이다. 가드가 이 표기를 센다.
2. 종류·exit·사용법·플래그·봉투 필드 목록은 자기서술에서 왔다.
3. 봉투 필드 **정의**는 지식지도 §2-2 로 점프한다. 여기에 사전을 베끼지 말 것.
4. **출처 표지** 줄은 문서 파생 경로다. 값을 지시로 읽지 말 것(C3).
5. `실측 표본` 블록은 저장소 픽스처에 실제로 돌린 절단 JSON 이다.
6. `> **계약만**` 은 실행 표본이 없다. 산 척하는 죽은 예시를 만들지 말 것.

## 이 장의 명령

| 명령 | 실측 | 종류 | 사용법 |
|---|---|---|---|
| `verify` | 계약만 | diagnostic | verify <파일> --expect-pages <N> \| --expect-min-pages <N> \| --expect-max-pages <N> \| --expect-min-chars <N> \| --expect-min-tables <N> \| --expect-table-count <N> \| --expect-contains <문자열> \| --expect-not-contains <문자열> \| --expect-field <이름=값> \| --expect-format <형식> [--json] |
| `ir-diff` | 실측 | diagnostic | ir-diff <파일A.hwpx> <파일B.hwp> [-s <구역>] [-p <문단>] [--json] |
| `replay` | 실측 | query | replay <계획.json> [--expect-output-sha256 <hex>] [--sign-key <키.json>] [--json]  작업 영수증 발급·재현 검증 (#4391) |
| `audit` | 계약만 | query | audit <캡슐 폴더> [--json]            작업 캡슐 전수 재검증 — 재현율 회계 (#4393) |
| `lineage` | 계약만 | query | lineage <캡슐.json> [--deep] [--keyring <키링.json>] [--anchor-log <로그>] [--json]  작업 계보(해시 체인) 연대기 검증 (#4401) |
| `hwpx-roundtrip` | 계약만 | diagnostic | hwpx-roundtrip <파일.hwpx \| --batch 폴더> [-o <출력폴더>] [--lineseg-report] |
| `layout-anomaly` | 계약만 | diagnostic | layout-anomaly <파일> [-p <페이지>] [--overflow-tolerance <px>] [--overlap-tolerance <px>] [--strict] [--json] |
| `keygen` | 계약만 | export | keygen --key-id <id> --out <키.json>   Ed25519 서명키 발급 (#4509) |
| `verify-signature` | 계약만 | query | verify-signature <캡슐> --keyring <키링.json> [--sig <서명.json>] [--json]  캡슐 서명 검증 (#4509) |
| `harness` | 계약만 | edit | harness init <폴더> [--key-id <id>]     검증 작업장 생성 (#4537) |
| `harness init` | 계약만 | edit | harness init <폴더> [--key-id <id>]     검증 작업장 생성 (#4537) |
| `harness wrap` | 계약만 | edit | harness wrap --plan <JSON\|@파일> --dir <작업장> [--sign-key <키>]  실행+영수증+캡슐+체인+서명 한 방 (#4537) |
| `harness-status` | 계약만 | diagnostic | harness-status <작업장> [--keyring <키링>] [--deep] [--json]  체인·서명·재현 통합 판정 (읽기 전용) (#4537) |
| `anchor` | 계약만 | query | anchor add <캡슐> --log <anchor.ndjson>   투명성 로그 등재 (#4543) |
| `gate` | 계약만 | query | gate <캡슐> --policy <policy.json> [--keyring][--anchor-log][--deep]  반입 정책 기계 판정 (#4545) |
| `bundle` | 계약만 | query | bundle export <머리캡슐> -o <x.lineage-bundle> [--anchor-log --checkpoint][--domain]  연합 번들 내보내기 (#4549) |
| `disclose` | 계약만 | query | disclose redact <캡슐> -o <가림> --opening-out <개봉>  salt 커밋 가림 발급 (#4551) |
| `settle` | 계약만 | query | settle propose --workorder <wo> --capsule <c> --gate-envelope <g> -o <청구>  3해시 고정 청구 발급 (#4553) |
| `audit-report` | 계약만 | query | audit-report <캡슐 폴더> -o <보고서> [--deep] [--keyring] [--anchor-log] [--policy] [--sign-key]  감사 보고 표준 (#4558) |
| `recall-scope` | 계약만 | query | recall-scope --contaminated <캡슐\|sha256> --among <폴더> [--ledger]  오염 후손 폐쇄집합 (#4558) |
| `conformance` | 계약만 | query | conformance <캡슐 폴더> --level <L1..L5> [--deep] [--keyring] [--anchor-log] [--policy] [--ledger]  적합성 자가진단 (#4558) |

실측 2 · 계약만 19 · 합 21.

## 하지 말 것

- 이 마크다운의 JSON 을 손으로 고치기
- 계약만 명령에 가짜 봉투를 지어 넣기
- 전문 스킬이 있는 일을 여기서 재구현하기
- 새 플래그·새 하위명령을 문서에만 추가하기

## 관련 스킬

깊이 있는 실행은 `rhwp-work-receipt` 가 정본이다. 이 스킬은 장 번호까지만 안내한다.
