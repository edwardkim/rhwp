> **PR base 브랜치가 `devel` 인지 확인해주세요** (`main` 아님 — GitHub 기본 선택이 main 일 수 있습니다).
> 작업 브랜치는 최신 `upstream/devel` 에서 생성합니다.

## 변경 요약

LLM-as-Verifier 축 V-lineage (작업 사슬). 구현자 산문은 증거가 아닙니다.
부모 `outputSha256` 이 자식 `inputSha256` 과 같을 때만 연대기를 인정합니다.
`rhwp lineage --json` 의 `parentOk` · `lineageOk` · `brokenAt` 을 소비하고,
그 필드가 해시 등식과 모순되면 기각합니다.

- `tools/llm_verifier/lineage_chain/decide.py` — 입력 5열
  `(parent_out, child_in, parentOk, lineageOk, brokenAt)` → 12개 `verdict`
- 뿌리(`parentOk`/`lineageOk` null)는 `ROOT_ONLY`. 사슬 주장이 아님
- `parentOk=false` 는 부모 파일 변조 (`PARENT_TAMPERED`)
- 산문/`source=prose` 는 `PROSE_NOT_EVIDENCE`
- `--deep`/`reproduced` 는 V-replay 축이라 읽어도 쓰지 않음
- `.claude/skills/rhwp-work-receipt` 는 재작성하지 않고 기존 필드 계약만 래핑
- 커밋된 결정 사례 코퍼스 126000행 (서로 다른 행, 주석 패딩 아님)

## 관련 이슈

closes #5516

## 테스트

- [x] **`cargo fmt --all -- --check` 통과** (PR 생성·push 직전 필수. CI Lint Format check 와 동일. `cargo fmt --check` 만으로는 안 됨)
- [x] `python -m unittest discover -s tools/llm_verifier/lineage_chain/tests -v`
- [x] `python tools/llm_verifier/lineage_chain/verify_corpus.py` 126000행 전부 `decide()` 와 일치, 유일
- [ ] `node scripts/rust-test-suite-manifest.mjs --check` — 이번 파동은 Rust 테스트 원본을 추가하지 않음
- [ ] `cargo test` — Python 검증기·코퍼스만 소유. 관련 시험은 위 unittest
- [ ] `cargo clippy -- -D warnings` — Rust 소스 변경 없음
- [ ] 관련 샘플 파일로 SVG 내보내기 확인 — N/A (렌더/레이아웃 변경 없음)
- [ ] 웹(WASM) 렌더링 확인 — N/A
- [ ] 작업 증빙 캡슐 — 문서 편집 CLI 를 돌리지 않음. 판정은 코퍼스 TSV 가 데이터

## 성능 영향 및 측정 결과

- 예상 영향: 영향 없음 (검증기 래퍼·픽스처만)
- 재현·측정: 미측정

## 소유 범위

- `tools/llm_verifier/lineage_chain/`
- `mydocs/working/llm_verifier_lineage_chain.md`

`third_party_replay`, `verdict_protocol`, `oracle_vs_self`, `claim_bind`,
`best_of_n`, `process_steps`, `untrusted_sandbox`, `gym/`,
`.claude/skills/rhwp-work-receipt` 는 건드리지 않았습니다.
