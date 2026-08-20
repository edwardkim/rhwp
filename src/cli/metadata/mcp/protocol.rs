use super::{tool, tool_with_optional_args};

pub(super) fn extend(tools: &mut Vec<serde_json::Value>) {
    tools.extend([
        tool(
            "hwp_run_plan",
            "[#3703] 선언적 편집 계획(JSON)을 정적 선검증→원자 실행→저널로 수행한다. 도구 호출을 체이닝하는 대신 의도를 계획서 하나로 선언하면, 전 step 의 실행 가능성을 미리 판정하고(불가 시 실행 0·invalid[]·exit 2) 인메모리로 적용해 단언(verify 자기검증) 통과 시에만 단 한 번 저장한다 — 실패 시 디스크 무변경. fill_fields step 은 화면상 구별되지 않는 필드 이름을 steps[].confusable 로 경고한다. steps: fill_fields{data} · replace_text{find,replace[,occurrence]} · set_cell{table,row,col,text[,keepStyle]} · set_checkbox{occurrence}. [#3719 §6-8] 각 step 은 선택 필드 if 로 조건을 달 수 있고(fieldExists·fieldEquals·textFound), 조건이 거짓이면 그 step 만 건너뛰며 저널에 skipped:true 로 남는다. 계획서의 정확한 문법은 hwp_export_plan_schema 로 먼저 받아 보라.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "plan": {
                        "type": "object",
                        "description": "계획서. { planVersion:\"1.0\", input:<원본 경로>, output:<산출 경로>, steps:[{action:…, if?:{…}}…], assertions:{ notFoundEmpty?, verify? }, dryRun?:true } — dryRun:true 면 선검증만 하고 preview 저널을 낸다(디스크 무변경). 계획을 실행 전에 검사할 때 쓴다. 전체 JSON Schema 는 hwp_export_plan_schema 참조"
                    }
                },
                "required": ["plan"],
            }),
            "run",
            serde_json::json!(["run", "--plan-json", "{plan}", "--json"]),
            &["schemaVersion", "planVersion", "input", "output", "outputFormat", "steps", "steps[].confusable", "steps[].skipped", "verify", "invalid", "changedPages", "dryRun", "preview", "inputSha256", "outputSha256"],
        ),
        tool_with_optional_args(
            "hwp_replay",
            "[#4391] 작업 영수증 — 계획을 **임시 산출**로 재실행해 (입력·계획·산출) SHA-256 3종 영수증을 발급(attest)하거나, expectOutputSha256 을 주면 타인의 작업 주장을 재현 검증한다(verify — 불일치 exit 3, reproduced:false). 사용자 파일은 절대 건드리지 않는다(계획의 output 은 임시 경로로 대체). 전제는 결정론: 같은 계획의 재실행은 같은 산출 바이트를 낸다(replay_contract 가 고정).",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "plan": {
                        "type": "object",
                        "description": "hwp_run_plan 과 같은 계획서. output 경로는 영수증 발급 시 무시(임시 산출로 대체)되고 확장자만 산출 형식 결정에 쓰인다"
                    },
                    "expectOutputSha256": {
                        "type": "string",
                        "description": "검증 모드 — 주장된 산출의 SHA-256(64자리 16진). 재현 산출과 다르면 exit 3"
                    }
                },
                "required": ["plan"],
            }),
            "replay",
            serde_json::json!(["replay", "--plan-json", "{plan}", "--json"]),
            serde_json::json!([{ "when": "expectOutputSha256", "args": ["--expect-output-sha256", "{expectOutputSha256}"] }]),
            &["schemaVersion", "mode", "input", "inputSha256", "planSha256", "outputSha256", "toolVersion", "steps", "reproduced", "expectedOutputSha256"],
        ),
        tool_with_optional_args(
            "hwp_lineage",
            "[#4401] 작업 계보 검증 — 캡슐 해시 체인을 머리부터 거슬러 부모 파일 무결(기록 해시 대조)·계보 불변식(부모 산출=자식 입력)을 판정하고, deep 이면 링크마다 재실행 재현까지 확인한다. 깨진 체인은 exit 3, 봉투의 brokenAt·links[] 가 어느 링크가 왜 깨졌는지 명세.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "capsule": { "type": "string", "description": "체인의 머리(최신) 캡슐 경로" },
                    "deep": { "type": "boolean", "description": "링크마다 재실행 재현까지 확인" }
                },
                "required": ["capsule"],
            }),
            "lineage",
            serde_json::json!(["lineage", "{capsule}", "--json"]),
            serde_json::json!([{ "when": "deep", "args": ["--deep"] }]),
            &["schemaVersion", "head", "depth", "valid", "brokenAt", "links"],
        ),
        tool(
            "hwp_keygen",
            "[#4509] Ed25519 서명키 파일 발급 — 캡슐 귀속의 시작점. 비밀키가 파일에 담기므로 덮어쓰기 금지·보관 책임은 소유자. keyId 관례는 '소유 주체/용도#세대'.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "keyId": { "type": "string", "description": "키 식별자 — 예: org.example/agent-7#2026" },
                    "out": { "type": "string", "description": "키 파일 저장 경로 (기존 파일이면 거부)" }
                },
                "required": ["keyId", "out"],
            }),
            "keygen",
            serde_json::json!(["keygen", "--key-id", "{keyId}", "--out", "{out}", "--json"]),
            &["schemaVersion", "keyId", "publicKey", "keyFile"],
        ),
        tool_with_optional_args(
            "hwp_verify_signature",
            "[#4509] 캡슐 분리 서명 검증 — <캡슐>.sig.json 을 캡슐 파일 바이트·키 등록부와 대조한다. verdict(valid|invalid|unknownKey|revoked|malformed)는 봉투 데이터이고 유효하지 않으면 exit 3. 서명 시점 증명은 이 축 밖(5년 축).",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "capsule": { "type": "string", "description": "검증할 캡슐 경로" },
                    "keyring": { "type": "string", "description": "키 등록부(keyring.json) 경로" },
                    "sig": { "type": "string", "description": "서명 파일 경로 (기본: <캡슐>.sig.json)" }
                },
                "required": ["capsule", "keyring"],
            }),
            "verify-signature",
            serde_json::json!(["verify-signature", "{capsule}", "--keyring", "{keyring}", "--json"]),
            serde_json::json!([{ "when": "sig", "args": ["--sig", "{sig}"] }]),
            &["schemaVersion", "capsule", "sigPath", "capsuleSha256", "capsuleShaMatches", "signatureOk", "keyId", "keyKnown", "revoked", "verdict"],
        ),
        tool_with_optional_args(
            "hwp_harness_wrap",
            "[#4537] 하네스 한 방 루프 — 계획을 실산출로 실행하고 영수증·캡슐(연번)·직전 캡슐 자동 부모 연결·(signKey) 서명까지 한 호출로 만든다. 에이전트가 매 작업을 이 도구로 돌리면 작업장의 해시 체인이 스스로 자란다.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "plan": { "type": "string", "description": "run 계획 JSON 문자열 (또는 @경로)" },
                    "dir": { "type": "string", "description": "harness init 로 만든 작업장" },
                    "signKey": { "type": "string", "description": "서명키 파일 (선택)" }
                },
                "required": ["plan", "dir"],
            }),
            "harness",
            serde_json::json!(["harness", "wrap", "--plan", "{plan}", "--dir", "{dir}", "--json"]),
            serde_json::json!([{ "when": "signKey", "args": ["--sign-key", "{signKey}"] }]),
            &["schemaVersion", "dir", "capsule", "output", "outputSha256", "parent", "signed"],
        ),
        tool_with_optional_args(
            "hwp_harness_status",
            "[#4537] 작업장 통합 판정 — 캡슐 체인 무결·(keyring) 서명 집계·(deep) 전수 재현을 한 봉투로. 하나라도 깨지면 exit 3, brokenAt 이 원인 캡슐을 가리킨다.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "dir": { "type": "string", "description": "작업장 폴더" },
                    "keyring": { "type": "string", "description": "키 등록부 (선택)" },
                    "deep": { "type": "boolean", "description": "캡슐마다 재실행 재현까지" }
                },
                "required": ["dir"],
            }),
            "harness-status",
            serde_json::json!(["harness-status", "{dir}", "--json"]),
            serde_json::json!([
                { "when": "keyring", "args": ["--keyring", "{keyring}"] },
                { "when": "deep", "args": ["--deep"] }
            ]),
            &["schemaVersion", "dir", "capsules", "chainValid", "brokenAt", "signed", "reproduced", "verdict"],
        ),
        tool(
            "hwp_anchor_add",
            "[#4543] 앵커 등재 — 캡슐 해시를 append-only 투명성 로그 끝에 더한다. 등재 전 로그 자기 무결을 검사하며, 깨진 로그에는 등재를 거부한다(exit 3). T7(역사 전체 재작성) 방어의 시작점.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "capsule": { "type": "string", "description": "등재할 캡슐 경로" },
                    "log": { "type": "string", "description": "anchor.ndjson 로그 경로 (없으면 생성)" }
                },
                "required": ["capsule", "log"],
            }),
            "anchor",
            serde_json::json!(["anchor", "add", "{capsule}", "--log", "{log}", "--json"]),
            &["schemaVersion", "log", "capsuleSha256", "seq"],
        ),
        tool_with_optional_args(
            "hwp_anchor_verify",
            "[#4543] 앵커 검증 — 캡슐이 로그에 등재됐고 로그가 무결하며 (checkpoint 지정 시) 머클 경로가 루트에 닿는지 판정한다. 아니면 exit 3. 체크포인트 공표는 도구 밖 운영 절차임을 봉투가 주장하지 않는다.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "capsule": { "type": "string", "description": "검증할 캡슐 경로" },
                    "log": { "type": "string", "description": "anchor.ndjson 로그 경로" },
                    "checkpoint": { "type": "string", "description": "체크포인트 파일 (선택)" }
                },
                "required": ["capsule", "log"],
            }),
            "anchor",
            serde_json::json!(["anchor", "verify", "{capsule}", "--log", "{log}", "--json"]),
            serde_json::json!([{ "when": "checkpoint", "args": ["--checkpoint", "{checkpoint}"] }]),
            &["schemaVersion", "capsule", "log", "capsuleSha256", "logChainOk", "logged", "seq", "inCheckpoint", "merklePath"],
        ),
        tool_with_optional_args(
            "hwp_gate",
            "[#4545] 반입 정책 기계 판정 — admissionPolicy 를 캡슐에 적용한다. 판정 재료는 자기 신고가 아니라 재계산(계보 걷기·서명 검증·앵커 조회·deep 재실행)이며, 규칙이 참조하는 판정만 지연 계산한다. 거부 = exit 3, violations[] 가 규칙·기대·실측을 명세.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "capsule": { "type": "string", "description": "판정 대상 캡슐" },
                    "policy": { "type": "string", "description": "admissionPolicy JSON 경로" },
                    "keyring": { "type": "string", "description": "서명 판정용 키 등록부 (signer* 규칙 시)" },
                    "anchorLog": { "type": "string", "description": "앵커 로그 (anchoredOk 규칙 시)" },
                    "deep": { "type": "boolean", "description": "reproduced 규칙의 재실행 재계산" }
                },
                "required": ["capsule", "policy"],
            }),
            "gate",
            serde_json::json!(["gate", "{capsule}", "--policy", "{policy}", "--json"]),
            serde_json::json!([
                { "when": "keyring", "args": ["--keyring", "{keyring}"] },
                { "when": "anchorLog", "args": ["--anchor-log", "{anchorLog}"] },
                { "when": "deep", "args": ["--deep"] }
            ]),
            &["schemaVersion", "policy", "policySigned", "target", "targetSha256", "verdict", "evaluated", "violations"],
        ),
        tool_with_optional_args(
            "hwp_bundle_export",
            "[#4549] 연합 번들 내보내기 — 머리 캡슐의 계보 폐쇄집합 전체를 서명·머클 증명과 함께 zip 하나로 만든다. 수신자는 이 파일 하나로 오프라인 전건 검증이 가능하다.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "head": { "type": "string", "description": "머리(최신) 캡슐 경로" },
                    "out": { "type": "string", "description": "산출 번들 경로 (*.lineage-bundle)" },
                    "anchorLog": { "type": "string", "description": "앵커 로그 (증명 동봉 시)" },
                    "checkpoint": { "type": "string", "description": "체크포인트 파일 (증명 동봉 시)" },
                    "domain": { "type": "string", "description": "발신 도메인 파일 (참고 동봉)" }
                },
                "required": ["head", "out"],
            }),
            "bundle",
            serde_json::json!(["bundle", "export", "{head}", "-o", "{out}", "--json"]),
            serde_json::json!([
                { "when": "anchorLog", "args": ["--anchor-log", "{anchorLog}"] },
                { "when": "checkpoint", "args": ["--checkpoint", "{checkpoint}"] },
                { "when": "domain", "args": ["--domain", "{domain}"] }
            ]),
            &["schemaVersion", "bundle", "head", "capsules", "signatures", "proofs"],
        ),
        tool(
            "hwp_bundle_verify",
            "[#4549] 연합 번들 검증 — 5단 오프라인 판정: 컨테이너 해시·폐쇄집합 완전성·계보 걷기·서명(수신자가 자기 경로로 받은 trust-domain 의 keyring 으로만 — 동봉 keyring 불신)·앵커(머클 루트가 도메인 선언 체크포인트와 일치). 깨짐 = exit 3 + brokenAt.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "bundle": { "type": "string", "description": "*.lineage-bundle 경로" },
                    "trustDomain": { "type": "string", "description": "수신자 보유 trust-domain 파일" }
                },
                "required": ["bundle", "trustDomain"],
            }),
            "bundle",
            serde_json::json!(["bundle", "verify", "{bundle}", "--trust-domain", "{trustDomain}", "--json"]),
            &["schemaVersion", "bundle", "trustDomain", "containerOk", "closureOk", "lineageValid", "capsules", "signed", "anchored", "brokenAt", "verdict"],
        ),
        tool(
            "hwp_disclose_redact",
            "[#4551] 가림 캡슐 발급 — plan 의 문자열 잎 전부를 salt 커밋으로 치환하고(구조 골격은 공개), 값·salt·원본 planText 는 비밀 개봉 파일로 분리한다. 해시 축 검증(체인·앵커)은 가림본에도 그대로 돈다.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "capsule": { "type": "string", "description": "원본 캡슐 경로" },
                    "out": { "type": "string", "description": "가림 캡슐 저장 경로" },
                    "openingOut": { "type": "string", "description": "비밀 개봉 파일 저장 경로" }
                },
                "required": ["capsule", "out", "openingOut"],
            }),
            "disclose",
            serde_json::json!(["disclose", "redact", "{capsule}", "-o", "{out}", "--opening-out", "{openingOut}", "--json"]),
            &["schemaVersion", "capsule", "redacted", "opening", "committedFields", "originalCapsuleSha256"],
        ),
        tool(
            "hwp_disclose_verify",
            "[#4551] 부분 개봉 검증 — 개봉된 필드만 커밋과 대조한다. verifiedFields/mismatched/unopened 가 협상의 단위이고, 불일치는 exit 3(위조 또는 값 변경).",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "redacted": { "type": "string", "description": "가림 캡슐 경로" },
                    "opening": { "type": "string", "description": "(부분) 개봉 파일 경로" }
                },
                "required": ["redacted", "opening"],
            }),
            "disclose",
            serde_json::json!(["disclose", "verify", "{redacted}", "--opening", "{opening}", "--json"]),
            &["schemaVersion", "redacted", "verifiedFields", "mismatched", "unopened", "verdict"],
        ),
        tool(
            "hwp_settle_propose",
            "[#4553] 정산 청구 발급 — 작업 명세서(workorder)·작업 캡슐·게이트 판정 봉투를 파일 바이트 sha256 셋으로 고정한 settlementClaim 을 만든다. 청구 후 산출물 바꿔치기·명세서 갖다붙이기·판정 위조가 전부 해시 불일치로 환원된다. 돈은 움직이지 않는다.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "workorder": { "type": "string", "description": "작업 명세서 경로 (acceptancePolicy 필수)" },
                    "capsule": { "type": "string", "description": "작업 캡슐 경로" },
                    "gateEnvelope": { "type": "string", "description": "게이트 판정 봉투 경로" },
                    "out": { "type": "string", "description": "청구 저장 경로" }
                },
                "required": ["workorder", "capsule", "gateEnvelope", "out"],
            }),
            "settle",
            serde_json::json!(["settle", "propose", "--workorder", "{workorder}", "--capsule", "{capsule}", "--gate-envelope", "{gateEnvelope}", "-o", "{out}", "--json"]),
            &["schemaVersion", "claim", "workorderSha256", "capsuleSha256", "gateEnvelopeSha256", "signed"],
        ),
        tool_with_optional_args(
            "hwp_settle_verify",
            "[#4553] 정산 청구 검증 — 3해시 대조 + 게이트 verdict 재확인. keyring 을 주면 청구·명세서 서명 판정, ledger 를 주면 이중 청구 검사까지. 실패는 exit 3 이고 어떤 축이 무너졌는지는 봉투가 말한다.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "claim": { "type": "string", "description": "청구 파일 경로" },
                    "workorder": { "type": "string", "description": "작업 명세서 경로" },
                    "capsule": { "type": "string", "description": "작업 캡슐 경로" },
                    "gateEnvelope": { "type": "string", "description": "게이트 판정 봉투 경로" },
                    "keyring": { "type": "string", "description": "서명 판정 keyring (opt-in)" },
                    "ledger": { "type": "string", "description": "이중 청구 검사 원장 (opt-in)" }
                },
                "required": ["claim", "workorder", "capsule", "gateEnvelope"],
            }),
            "settle",
            serde_json::json!(["settle", "verify", "{claim}", "--workorder", "{workorder}", "--capsule", "{capsule}", "--gate-envelope", "{gateEnvelope}", "--json"]),
            serde_json::json!([
                { "when": "keyring", "args": ["--keyring", "{keyring}"] },
                { "when": "ledger", "args": ["--ledger", "{ledger}"] }
            ]),
            &["schemaVersion", "claim", "workorderOk", "capsuleOk", "gateOk", "gateVerdict", "signerOk", "workorderSignerOk", "ledgerOk", "duplicate", "verdict"],
        ),
        tool(
            "hwp_settle_record",
            "[#4553] 원장 기입 — 5년 앵커 로그와 동형인 append-only 해시 체인에 청구를 등재한다. 같은 캡슐의 accepted 가 이미 있으면 이중 청구로 거부(exit 3, existingSeq 보고). 깨진 원장에는 기입하지 않는다.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "claim": { "type": "string", "description": "청구 파일 경로" },
                    "ledger": { "type": "string", "description": "원장 ndjson 경로 (없으면 생성)" }
                },
                "required": ["claim", "ledger"],
            }),
            "settle",
            serde_json::json!(["settle", "record", "{claim}", "--ledger", "{ledger}", "--json"]),
            &["schemaVersion", "ledger", "seq", "claimSha256", "capsuleSha256", "verdict", "duplicate", "existingSeq"],
        ),
        tool_with_optional_args(
            "hwp_audit_report",
            "[#4558] 감사 보고 표준 — 캡슐 폴더의 계보·귀속·앵커·게이트 수치를 기존 축 검증의 기계 합산으로 산출한 agentLaborAuditReport 를 생성한다. 전 수치는 재계산 가능하고 보고서 자체를 서명할 수 있다.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "dir": { "type": "string", "description": "*.capsule.json 폴더 (비재귀)" },
                    "out": { "type": "string", "description": "보고서 저장 경로" },
                    "keyring": { "type": "string", "description": "귀속 절 keyring (opt-in)" },
                    "anchorLog": { "type": "string", "description": "앵커 절 로그 (opt-in)" },
                    "policy": { "type": "string", "description": "게이트 절 정책 (opt-in)" }
                },
                "required": ["dir", "out"],
            }),
            "audit-report",
            serde_json::json!(["audit-report", "{dir}", "-o", "{out}", "--json"]),
            serde_json::json!([
                { "when": "keyring", "args": ["--keyring", "{keyring}"] },
                { "when": "anchorLog", "args": ["--anchor-log", "{anchorLog}"] },
                { "when": "policy", "args": ["--policy", "{policy}"] }
            ]),
            &["schemaVersion", "report", "capsules", "reproduction", "lineage", "attribution", "anchoring", "gate", "toolVersions", "signed"],
        ),
        tool_with_optional_args(
            "hwp_recall_scope",
            "[#4558] 오염 리콜 범위 — 오염 캡슐의 후손 폐쇄집합(영향 전건)과 미영향 계수를 계보 걷기로 계산한다. ledger 를 주면 영향 캡슐의 정산 청구 좌표까지 보고한다.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "contaminated": { "type": "string", "description": "오염 캡슐 경로 또는 파일 sha256" },
                    "among": { "type": "string", "description": "수색 대상 캡슐 폴더" },
                    "ledger": { "type": "string", "description": "정산 원장 (opt-in — 회계 연결)" }
                },
                "required": ["contaminated", "among"],
            }),
            "recall-scope",
            serde_json::json!(["recall-scope", "--contaminated", "{contaminated}", "--among", "{among}", "--json"]),
            serde_json::json!([
                { "when": "ledger", "args": ["--ledger", "{ledger}"] }
            ]),
            &["schemaVersion", "contaminated", "affected", "unaffected", "claims"],
        ),
        tool(
            "hwp_conformance",
            "[#4558] 적합성 자가진단 — L1(영수증)~L5(원장) 누적 요건을 기존 판정기 재사용으로 검사한다. 미달은 exit 3, 항목별 판정은 checks 배열이 말한다. L3+ 는 keyring/anchorLog, L4+ 는 policy, L5 는 ledger 가 필수다.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "dir": { "type": "string", "description": "*.capsule.json 폴더 (비재귀)" },
                    "level": { "type": "string", "enum": ["L1", "L2", "L3", "L4", "L5"], "description": "목표 등급" }
                },
                "required": ["dir", "level"],
            }),
            "conformance",
            serde_json::json!(["conformance", "{dir}", "--level", "{level}", "--json"]),
            &["schemaVersion", "level", "capsules", "checks", "achieved", "verdict"],
        ),
        tool(
            "hwp_audit",
            "[#4393] 에이전트 노동 감사 — 작업 캡슐(*.capsule.json) 폴더를 전수 재실행해 재현율을 회계한다. 개별 검증은 hwp_replay, 조직 규모 일괄은 이 도구. 불일치 1건 = exit 3, failed[] 에 캡슐별 기대/실제 해시.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "dir": { "type": "string", "description": "*.capsule.json 이 담긴 폴더 (비재귀)" }
                },
                "required": ["dir"],
            }),
            "audit",
            serde_json::json!(["audit", "{dir}", "--json"]),
            &["schemaVersion", "root", "total", "reproduced", "failed", "reproducedRate"],
        ),
        tool_with_optional_args(
            "hwp_export_plan_schema",
            "[#3719 §6-4] hwp_run_plan 이 받는 **계획서 자체**의 JSON Schema 를 돌려준다. hwp_run_plan 이 계획을 실행한다면 이 도구는 계획을 어떻게 쓰는지 알려준다 — step 4종의 필수·선택 필드, 조건절 if 의 문법, assertions 의 뜻이 판별 유니온으로 적혀 있다. 계획을 처음 만들 때 한 번 받아 두면 필드명을 지어내 invalid[] 로 되돌아오는 왕복을 없앨 수 있다. 문서를 입력으로 받지 않는다(계획서 문법의 서술이지 특정 문서의 속성이 아니다).",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "bare": {
                        "type": "boolean",
                        "description": "참이면 봉투 없이 계획 스키마 본문만 (JSON Schema 검증기에 바로 먹일 때)"
                    }
                },
                // 문서를 받지 않으므로 필수 인자가 없다 — 그래도 빈 배열을 선언한다.
                // 소비자가 required 의 부재와 "필수 없음"을 구분할 수 없으면 안 된다.
                "required": [],
            }),
            "export-plan-schema",
            serde_json::json!(["export-plan-schema", "--json"]),
            serde_json::json!([{ "when": "bare", "args": ["--bare"] }]),
            &["schemaVersion", "planSchemaVersion", "dialect", "definitionCount", "schema"],
        ),
        tool_with_optional_args(
            "hwp_export_capabilities_schema",
            "[#3776] capabilities 자기서술 **자체**의 JSON Schema 를 돌려준다. capabilities 가 명령 표면을 설명한다면 이것은 그 설명의 모양을 설명한다 — 외부 바인딩·코드 생성기가 commands[].recordFields·flags·exitCodes 를 안전하게 읽으려면 이 모양이 고정돼야 한다. 문서를 입력으로 받지 않는다(명령 표면의 서술이지 특정 문서의 속성이 아니다). 봉투는 capabilities 스키마(schema)와 capabilities --mcp 매니페스트 스키마(mcpSchema)를 함께 싣는다.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "bare": {
                        "type": "boolean",
                        "description": "참이면 봉투 없이 capabilities 스키마 본문만 (JSON Schema 도구에 바로 먹일 때)"
                    }
                },
                // 문서를 받지 않으므로 필수 인자가 없다 — 그래도 빈 배열을 선언한다.
                // 소비자가 required 의 부재와 "필수 없음"을 구분할 수 없으면 안 된다.
                "required": [],
            }),
            "export-capabilities-schema",
            serde_json::json!(["export-capabilities-schema", "--json"]),
            serde_json::json!([{ "when": "bare", "args": ["--bare"] }]),
            &["schemaVersion", "capabilitiesSchemaVersion", "dialect", "definitionCount", "schema", "mcpSchema"],
        ),
        tool_with_optional_args(
            "hwp_export_ontology",
            "[#3907 O1] rhwp 의 자기서술(IR 스키마·capabilities·MCP 도구 정의·봉투 출처 지도)에서 실행 시점에 기계 유도한 JSON-LD 온톨로지를 돌려준다. @graph 에 IR 타입 = 클래스(rdfs:Class), IR 필드 = 속성(rdf:Property, 도메인·레인지 유도), 명령·MCP 도구 = 행위(schema:Action), 출처 지도의 문서 파생 경로 = 신뢰 술어(rhwp:untrustedFields)가 실린다. 손으로 쓴 목록이 없어 원천 선언이 바뀌면 온톨로지가 함께 바뀐다 — 지식그래프·시맨틱 소비자가 단일 출처로 쓴다. 문서를 입력으로 받지 않는다(도구 자신의 서술이지 특정 문서의 속성이 아니다).",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "bare": {
                        "type": "boolean",
                        "description": "참이면 봉투 없이 JSON-LD 본문(@context·@graph)만 (RDF/JSON-LD 도구에 바로 먹일 때)"
                    }
                },
                // 문서를 받지 않으므로 필수 인자가 없다 — 그래도 빈 배열을 선언한다.
                // 소비자가 required 의 부재와 "필수 없음"을 구분할 수 없으면 안 된다.
                "required": [],
            }),
            "export-ontology",
            serde_json::json!(["export-ontology", "--json"]),
            serde_json::json!([{ "when": "bare", "args": ["--bare"] }]),
            &["schemaVersion", "ontology", "classCount", "propertyCount", "actionCount"],
        ),
    ]);
}
