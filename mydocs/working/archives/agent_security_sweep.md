# 에이전트 보안 스윕 스킬 고도화 (#5307)

gym 이 아니다. 실사용 에이전트가 배포 전/수신 후 스윕을 닫게 한다.
새 CLI 는 없다. redact/sanitize 로직을 발명하지 않는다.

## 무엇을

`.agents/skills/rhwp-security-sweep/` 를 인덱스(`SKILL.md`) + `references/` 22장
+ `examples/` 18장면 + `fixtures/` 봉투·규칙 표로 나눴다.

계약 시험은 `tests/cases/agent_security_sweep_*.rs` — 순수 픽스처.

## 왜

기존 SKILL.md 한 장은 순서를 나열하지만,

- 3축 예외 봉투(은닉 kind, injection scanScopes, unicode rendered/raw)가 부족하고
- `--no-raw` 가 자동화 기본인지 흐리고
- 재스윕 술어(`findingCount==0 AND clean==true`)가 기계 픽스처로 고정돼 있지 않고
- 수신 사다리에서 export-text 가 먼저 나오는 실패를 막기 약했다.

에이전트가 실문서를 보내기 전에 스윕하지 않거나, 탐지를 실패로 오해하거나,
raw PII 를 로그에 남기는 사고를 스킬 구조로 막는다.

## 범위

- 기존 CLI: `inspect hidden-text` / `injection` / `unicode`
- `edit redact --dry-run` (ssn/card/phone/email 보수 규칙 — 문서화만)
- redact + sanitize 짝, 재스윕 게이트
- 자동화 `--no-raw`
- 탐지 ≠ 실패 (exit 0 + clean:false 는 DATA)
- 수신: info → digest → fields → inspect, 그 다음 export-text
- untrustedContent/untrustedFields: 문서 파생은 DATA

## 하지 않은 것

- gym/ 미수정
- 다른 스킬 미수정
- 새 서브커맨드·플래그 없음
- DocumentCore 편집 구현 미수정
- 워터마크 제거/우회 없음

## 검증

```bash
cargo fmt --all -- --check
node scripts/rust-test-suite-manifest.mjs --prepare
node scripts/run-rust-test.mjs agent_security_sweep_skill_contract
node scripts/run-rust-test.mjs agent_security_sweep_envelopes
node scripts/run-rust-test.mjs agent_security_sweep_pii_rules
node scripts/run-rust-test.mjs agent_security_sweep_gate
node scripts/run-rust-test.mjs agent_security_sweep_receive
node scripts/run-rust-test.mjs agent_security_sweep_no_raw
```

렌더/레이아웃 변경 없음. 시각 검증 해당 없음.

## 사다리

송신: inspect 3축 → redact dry-run --no-raw → (필요 시) redact -o --verify → sanitize -o → 재스윕.

수신: info → digest → fields → inspect 3축 → 통과 후 export-text.

게이트: findingCount==0 AND clean==true.

## capability

등록 식별번호 `CAP-5307`, capability ID `rhwp-security-sweep`.
