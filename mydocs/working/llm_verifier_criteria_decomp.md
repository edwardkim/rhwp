---
kind: working
status: active
issue: 5504
---

# V-decomp: 검증 기준을 원자 항목으로 분해 (#5504)

작업 브랜치: `feat/v-decomp-criteria`
대상: `tools/llm_verifier/criteria_decomp/` · 본 문서
워크트리: `C:\Users\swsz9\rhwp-v-decomp-criteria` (`upstream/devel` 격리)

## 한 줄

LLM 검증을 한 덩어리 점수로 주지 않는다. 기존 rhwp `--json` 봉투 필드마다
원자 기준을 두고, 각 원자의 통과와 총점 채점이 그 실패를 가릴 수 있는지만 본다.

## 이슈가 요구한 것

- LLM-as-verifier 축: criteria decomposition
- 총점 하나가 아니라 원자 기준. 각 원자는 기존 봉투 필드에 결속
- 프롬프트 편향(한 줄 점수가 실패를 덮음)을 줄인다
- V-bon 순위·V-step 과정 보상과 파일 분리
- 새 CLI 발명 금지
- 파일 소유: `tools/llm_verifier/criteria_decomp/` + 본 문서
- `git diff --shortstat upstream/devel` 100000 insertions 이상
- `(task, criterion_id, envelope_field, atom_pass, holistic_would_hide)` 실픽스처
- 서로 다른 한국어 과업 행. 주석 패딩·로렘 금지
- `cargo fmt --all -- --check`, base `devel`, 한국어 PR, `closes #5504`

## 하지 말라는 것

- Best-of-N 순위 (`best_of_n`, V-bon)
- 과정 보상 (`process_steps`, V-step)
- 새 rhwp CLI
- `gym/`
- `git add -A`
- 금지 워크트리 (`rhwp`, `rhwp-desk*`, `rhwp-handoff`, `rhwp-scaffold-final`, `rhwp-doc-repro`)

## 만진 경로

- `tools/llm_verifier/criteria_decomp/` (라이브러리, 스키마, 봉투 표본, 120000행 코퍼스)
- `mydocs/working/llm_verifier_criteria_decomp.md`
- 워크스페이스 등록: `Cargo.toml` members 에 `tools/llm_verifier/criteria_decomp` (`fmt --all` 대상)

## 만지지 않은 경로

- `tools/llm_verifier/best_of_n/`
- `tools/llm_verifier/process_steps/`
- `tools/llm_verifier/claim_bind/`
- `tools/llm_verifier/verdict_protocol/`
- `gym/`, DocumentCore, rhwp CLI 명령

## 어떻게

1. 최신 `upstream/devel` 에서 `feat/v-decomp-criteria` 격리 worktree.
2. `llm-verifier-criteria-decomp` 라이브러리: 허용 필드 닫힌 집합, 원자 기대, 묶음 분해, 가림 판정.
3. 실패 종류: `invented_field` · `holistic_only` · `empty_task` · `empty_criterion` · `missing_field` · `atom_mismatch` · `bundle_shape`.
4. `scripts/gen_decomp_corpus.py` 가 누름틀·치환·보안 스윕·IR/시각 회귀·조판 이상 등 실무 과업 120000행을 NDJSON 샤드로 생성. 각 행은 고유 `rowId`·`criterionId` 와 (빈 과업 실패행 제외) 고유 `task`.
5. 통합 시험이 전 행을 다시 판정해 라벨과 일치하는지, 한국어인지, 로렘/패딩이 없는지 검사.

## 판정 계약

| 조건 | atom_pass | holistic_would_hide |
| --- | --- | --- |
| 허용 필드 + 기대 일치 | true | false |
| 기대 불일치이고 형제 통과 ≥ 절반 (묶음 ≥ 2) | false | true |
| 기대 불일치이고 묶음 대부분이 실패 | false | false |
| 봉투에 없는 키 (`holisticScore`·`bestOfN`·`processReward` 등) | false | false |
| 원자 없이 총점만 | false / `holistic_only` | false |
| 과업 문장 공백 | false / `empty_task` | false |
| 필드 누락 | false / `missing_field` | 형제 비율에 따름 |

총점 숫자(`holisticScore`)는 보고서 필드가 아니다. 가림 여부는 통과 개수/전체 개수의 절반 임계로만 계산한다.

`page` 는 봉투 그대로 0 기준. 없는 `humanPage`·`pdfPage` 를 만들지 않는다.

## 시험 명령

```bash
cargo test -p llm-verifier-criteria-decomp
cargo fmt --all -- --check
```

## PR 메모

- base `devel`
- `closes #5504`
- `--body-file` UTF-8 without BOM
- 제목·본문 한국어
- 첫 체크박스: `cargo fmt --all -- --check`
