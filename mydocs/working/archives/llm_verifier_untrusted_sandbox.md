---
kind: working
status: active
issue: 5491
claim: V-nonce
---

# V-nonce: 문서 텍스트가 검증 기준이 되지 못하게 (#5491)

작업 브랜치: `feat/v-nonce-sandbox`
대상: `tools/llm_verifier/untrusted_sandbox/`
이슈: [V-nonce: 문서 텍스트가 검증 기준이 되지 못하게](https://github.com/edwardkim/rhwp/issues/5491)

## 1. 한 줄

LLM-as-verifier 축 6. `untrustedContent` / `untrustedFields` 로 표지된
문서 파생 텍스트는 nonce 경계 안의 데이터로만 다루고, 검증 기준·지시
자리로 새면 차단한다. provenance 스킬은 다시 쓰지 않는다. 새 rhwp CLI
는 없다.

## 2. 이슈가 요구한 것 / 하지 말라는 것

요구:

- 문서 파생 텍스트가 검증 기준·프롬프트 지시가 되지 못하게 nonce 경계
- 파일 소유: `tools/llm_verifier/untrusted_sandbox/` + 이 문서
- 코퍼스 열: `(excerpt, nonce, slot, leaked_into_criteria, expected_block)`
- 서로 다른 행. 주석 패딩 금지
- `git diff --shortstat upstream/devel` ≥ 100000 insertions
- HARD GATE: `cargo fmt --all -- --check`
- isolation worktree, 브랜치 `feat/v-nonce-sandbox` from `upstream/devel`
- origin `kevin9327`, 한국어 PR, base `devel`, `closes #5491`

금지:

- `.agents/skills/rhwp-provenance` 재작성
- 새 rhwp CLI 발명
- 다른 V-\* (`verdict_protocol`, `claim_bind`, `best_of_n`,
  `process_steps`, `oracle_vs_self`) 디렉터리 작성
- `gym/`
- `git add -A`
- 금지 워크트리: `rhwp`, `rhwp-desk*`, `rhwp-handoff`,
  `rhwp-scaffold-final`, `rhwp-doc-repro`

## 3. 왜 이 축인가

봉투의 `pages[].text` · `matches[].context` · `title` 은 문서 작성자가
정한 값이다. 그 문자열을 검증기 프롬프트의 기준 칸이나 시스템 지시에
붙이면 문서가 자기 합격 조건을 쓰게 된다. rhwp 는 원문을 바꾸지 않고
표지만 싣는다. 격리는 검증기 쪽 샌드박스의 몫이다.

## 4. 계약 요약

허용 자리:

- `user_display` — 사람에게 보여 주는 화면
- `llm_data_block` — nonce 시작/끝 표지가 온전한 데이터 블록

그 외 자리(`criteria`, `system_prompt`, `tool_arg_path`, `tool_name`,
`shell_command`, `url_body`, `run_plan`, `authorization`)는 전부 차단.

`leaked_into_criteria=yes` 이면 자리와 관계없이 차단. nonce 가 비었거나
정적 표지이거나 본문에 이미 있으면 wrap 실패(닫힌 실패). `source_label`
에 `title` 을 쓰면 표지 줄 자체가 공격면이다.

분류 규칙은 `decide.py` 가 정본이다. 이 문서는 포인터다.

## 5. 검증

```text
python tools/llm_verifier/untrusted_sandbox/generate_corpus.py
python tools/llm_verifier/untrusted_sandbox/verify_corpus.py
python -m unittest discover -s tools/llm_verifier/untrusted_sandbox/tests -v
cargo fmt --all -- --check
```

코퍼스 모든 행의 `expected_block` 은 `decide()` 재계산과 같다.
closed-set 축 표는 10개 slot × leak × nonce_kind × source_label_kind 를
덮는다.

## 6. 소유 경로

- `tools/llm_verifier/untrusted_sandbox/`
- `mydocs/working/llm_verifier_untrusted_sandbox.md`
