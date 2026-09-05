---
kind: working
status: active
issue: 5516
claim: V-lineage
---

# V-lineage: 부모 산출=자식 입력 해시 사슬만 인정 (#5516)

작업 브랜치: `feat/v-lineage-chain`
대상: `tools/llm_verifier/lineage_chain/`
이슈: [V-lineage: 부모 산출=자식 입력 해시 사슬만 인정](https://github.com/edwardkim/rhwp/issues/5516)

## 1. 한 줄

작업 사슬은 부모 `outputSha256` 이 자식 `inputSha256` 과 같을 때만 인정한다.
`rhwp lineage` 가 이미 내는 `parentOk` · `lineageOk` · `brokenAt` 을 읽고,
그 정의와 모순되면 기각한다. 단건 재실행(`reproduced`)은 V-replay 축이다.

## 2. 이슈가 요구한 것 / 하지 말라는 것

요구:

- LLM-as-Verifier 축 LINEAGE CHAIN
- 부모 산출 해시 == 자식 입력 해시
- 기존 봉투 필드 `parentOk`, `lineageOk`, `brokenAt` 소비
- 파일 소유: `tools/llm_verifier/lineage_chain/` + 이 문서
- 결정 사례 코퍼스 열:
  `(parent_out, child_in, parentOk, lineageOk, brokenAt, verdict)`
- 서로 다른 행. 주석 패딩 금지
- `git diff --shortstat upstream/devel` ≥ 100000 insertions
- HARD GATE: `cargo fmt --all -- --check`
- isolation worktree, 브랜치 `feat/v-lineage-chain` from `upstream/devel`
- origin `kevin9327`, 한국어 PR, base `devel`, `closes #5516`

금지:

- V-replay(단건 `rhwp replay --expect-output-sha256` / `reproduced`) 재구현
- `.agents/skills/rhwp-work-receipt` 재작성
- `gym/`
- `git add -A`
- 금지 워크트리: `rhwp`, `rhwp-desk*`, `rhwp-handoff`, `rhwp-scaffold-final`, `rhwp-doc-repro`

## 3. 왜 이 축인가

V-replay 는 **작업 하나**가 같은 계획으로 같은 바이트를 냈는지를 본다.
V-lineage 는 **작업과 작업 사이**가 이어졌는지를 본다. 부모 산출 파일이
자식 입력 파일이 아니면 연대기가 아니다. 구현자가 “이어서 편집했다”고
써도 해시가 다르면 `LINEAGE_BROKEN` 이다.

기존 계약:

- `rhwp lineage --json` → `links[].parentOk`, `links[].lineageOk`, `brokenAt`
- `workCapsule.receipt.outputSha256` / `inputSha256`
- `parent: null` 은 합법 뿌리. `parent` 키 자체 없음은 깨진 캡슐
- `--deep` 의 `reproduced` 는 링크마다 단건 재실행 — V-replay

이 파동은 그 신호를 묶는 **사슬 판정 트리**만 소유한다.

## 4. 트리 요약

산문 · 사용법(exit 2) · 머리 IO(exit 1) · `kind != workCapsule` ·
`parent` 키 없음 · `parent.sha256` 결함은 해시 등식보다 먼저다.

`parentOk` 와 `lineageOk` 가 둘 다 null 이면 뿌리다 (`ROOT_ONLY`).
`parentOk=false` 는 부모 파일 변조 (`PARENT_TAMPERED`).

그 다음 64 hex 등식:

- 같음 + `lineageOk=true` + `brokenAt` 없음 → `CHAIN_ACCEPTED`
- 다름 + `lineageOk=false` + `brokenAt` 있음 → `LINEAGE_BROKEN`
- 봉투가 등식과 다르면 `ENVELOPE_CONTRADICTS`

`reproduced` 는 읽어도 쓰지 않는다.

## 5. 검증

```text
python -m unittest discover -s tools/llm_verifier/lineage_chain/tests -v
python tools/llm_verifier/lineage_chain/verify_corpus.py
cargo fmt --all -- --check
```
