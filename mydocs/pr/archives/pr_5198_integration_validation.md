---
kind: pr-review
status: local-validation-passed-pending-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-18
---

# PR #5198 누적 통합 후보 — 메인터너 보정 및 로컬 검증

## 기준과 범위

- 검토 브랜치: `review/kevin9327-stack-20260817-r2`
- 기준: `upstream/devel` `e0851908bbe568e850c4610986247494203b75d5`
- 대상: #4885, #4887, #4888, #4982, #4983, #4985, #4986, #4987, #4988, #4989,
  #5042, #5056, #5068, #5080, #5100, #5116, #5117, #5119, #5123, #5125, #5130,
  #5131, #5139, #5144, #5145, #5146, #5147, #5157, #5161

## 메인터너 보정

리베이스 뒤 CLI 자기서술과 실제 표면이 어긋난 부분을 정합시켰다.

- `charts`를 capabilities, `--help`, MCP/ontology/provenance 경로와 자동 에이전트 문서에 연결했다.
- `form-value`와 수식·도형 편집 하위 명령을 `edit` 도움말·명령 계약에 맞췄다.
- 지식 지도 §2-2에 양식·수식 필드 8개를 추가하고, 310개 선언 필드/313개 사전 필드로 갱신했다.

## 로컬 검증

- `cargo nextest run --tests …focused…`: **128/128 통과**.
- `cargo nextest run --tests --test-threads 4 --no-fail-fast`: **6,798/6,798 통과**, slow 8, skipped 38.
  이전 `provenance_contract`의 남은 Windows 임시 원장은 삭제하지 않고 `target\pr-review` 하위
  격리 `TEMP`/`TMP`로 우회해 재현 가능한 깨끗한 실행을 확보했다.
- `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`,
  `git diff --check`: 통과.
- `rust-unit-test-tiers` 생성·자체 테스트·검사 및 `rust-test-suite-manifest` 준비·자체 테스트·검사: 통과.
- `python tools/gen_agent_codex.py --check`: 변경 0으로 통과.

## CI 상태와 다음 단계

[#5198의 실패 run](https://github.com/edwardkim/rhwp/actions/runs/32052155032/job/95454214512)은
이전 head `522b7aae`의 Format check 실패였다. 원격 최신 head `1d6df036`에는 이미
`style: PR head Rust 포맷 정규화`가 포함돼 있다. 이 검토 브랜치의 후속 메인터너 보정은 아직
원격에 push하지 않았으므로, 새 CI run은 없다.

사용자 승인 전에는 push, CI 재실행, PR comment/merge/close를 하지 않는다. 승인 후에는 이 후보를
push한 뒤 새 head의 required CI를 확인하고, 녹색일 때만 merge 및 원 PR 후속 처리를 진행한다.
