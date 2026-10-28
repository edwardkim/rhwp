---
kind: working
status: active
issue: 5490
---

# V-step: 편집 스텝마다 기계 과정 검증 (#5490)

구현은 `tools/llm_verifier/process_steps/` 만 소유한다.
다른 V-\* (`verdict_protocol`, `oracle_vs_self`, `claim_bind`, `best_of_n`,
`untrusted_sandbox`) 와 파일을 겹치지 않는다. `gym/` 없음.

## 한 줄

LLM-as-verifier 축 5(PROCESS). 편집 한 스텝마다 기존
`verify` / `layout-anomaly` / `info.pageCount` / `edit fill-fields --verify`
를 돌려 과정 보상(process reward)을 남긴다. 새 rhwp CLI 없음.
Best-of-N 순위는 V-bon(#5489) — 여기 없다.

## 기계 계약

- 입력: `ProcessStep` = `{stepKind, stepIndex, checks[4], sourceTag}`
- 검사: `verify` · `layout-anomaly` · `pageCount`(info 봉투) · `fill-verify`
- 출력: `ProcessReward.pass` = 네 검사가 모두 통과했는가
- 유일키: `(stepKind, stepIndex, checkFingerprints, processReward, sourceTag)`
- 코퍼스: `corpus/shards/*.json` (`generate_corpus.py` 재생성)
- 순위 필드(`rank`, `score`, `bestOfN`) 금지

분류 규칙은 `src/score.rs` 가 정본이다. 이 문서는 포인터다.

## 과정 보상 규칙

| 검사 | 기존 명령 | 합격 | 불합격 |
|------|-----------|------|--------|
| verify | `rhwp verify --json` | exit 0, `verdict=pass`, `failCount=0` | exit 3 또는 fail 신호 |
| layout-anomaly | `rhwp layout-anomaly --json` | exit 0, strict 확정 신호 없음 | exit 3 + overflow/overlap |
| pageCount | `rhwp info --json` | `pageCount==expectedPageCount` | exit 4, `pageCountMismatch` |
| fill-verify | `rhwp edit fill-fields --verify --json` | `verify.identical=true` | `verify.identical=false` |

종료코드는 기존 0/1/2/3/4 만. 봉투 없는 exit 3/4 는 inconsistent.

## 소유 파일

- `tools/llm_verifier/process_steps/` (crate · schema · corpus · generator)
- `mydocs/working/llm_verifier_process_steps.md` (이 문서)
