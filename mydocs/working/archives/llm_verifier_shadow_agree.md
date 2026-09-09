---
kind: working
status: active
issue: 5510
claim: V-shadow
---

# V-shadow: 서로 다른 검사 두 개가 합의해야 합격 (#5510)

작업 브랜치: `feat/v-shadow-agree`
대상: `tools/llm_verifier/shadow_agree/`
이슈: [V-shadow: 서로 다른 검사 두 개가 합의해야 합격](https://github.com/edwardkim/rhwp/issues/5510)

## 1. 한 줄

이미 있는 기계 명령 두 개가 동시에 합격해야 합격이다. 한 명령만
합격하는 것은 합의가 아니다. 새 rhwp CLI 를 만들지 않는다.

## 2. 이슈가 요구한 것 / 하지 말라는 것

요구:

- LLM-as-Verifier 축 SHADOW AGREEMENT
- 파일 소유: `tools/llm_verifier/shadow_agree/` + 이 문서
- 코퍼스 열: `(check_a, check_b, a_pass, b_pass, expected_joint)`
- 서로 다른 행. 주석 패딩 금지
- `git diff --shortstat upstream/devel` ≥ 100000 insertions
- HARD GATE: `cargo fmt --all -- --check`
- isolation worktree, 브랜치 `feat/v-shadow-agree` from `upstream/devel`
- origin `kevin9327`, 한국어 PR, base `devel`, `closes #5510`

금지:

- 새 rhwp CLI 발명
- V-abstain (한 봉투 안 필드 모순) 구현
- `gym/`
- `git add -A`
- 금지 워크트리: `rhwp`, `rhwp-desk*`, `rhwp-handoff`, `rhwp-scaffold-final`, `rhwp-doc-repro`

## 3. 왜 이 축인가

`ir-diff identical` 만 보고 합격이라고 쓰면 쪽수가 갈려도 통과한다.
`layout-anomaly hasSignal=false` 만 보고 합격이라고 쓰면 채움 자기검증이
깨져도 통과한다. 그림자 합의는 서로 다른 명령의 신호를 AND 한다.

V-abstain 은 한 봉투에서 `identical:true` 와 `diffCount:3` 이 싸우면
기권한다. 여기는 명령 두 개의 합의이지 한 봉투의 침묵이 아니다.

## 4. 트리 요약

같은 `command_key` 를 두 칸에 넣으면 `SAME_CHECK_NOT_SHADOW`.
서로 다른 명령이고 둘 다 합격일 때만 `JOINT_PASS` / `expected_joint=1`.
한쪽만 합격이면 `SHADOW_A_ONLY` 또는 `SHADOW_B_ONLY`.

표본 쌍은 이슈가 든 그대로다.

- `rhwp ir-diff --json` `identical` 과 `rhwp verify --expect-pages` /
  `rhwp dump-pages --json` 쪽수
- `rhwp fill-fields --verify` `verify.identical` 과
  `rhwp layout-anomaly --json` `hasSignal=false`

## 5. 검증

```text
python -m unittest discover -s tools/llm_verifier/shadow_agree/tests -v
python tools/llm_verifier/shadow_agree/verify_corpus.py
cargo fmt --all -- --check
```

코퍼스 모든 행의 `expected_joint` / `expected_verdict_class` 는
`decide()` 재계산과 같다.

## 6. 소유 경로

- `tools/llm_verifier/shadow_agree/`
- `mydocs/working/llm_verifier_shadow_agree.md`
