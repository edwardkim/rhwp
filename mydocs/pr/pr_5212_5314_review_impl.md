---
kind: review_plan
status: local-validation-passed
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-18
---

# kevin9327 누적 체리픽 통합 기록 — PR #5212–#5314

기준은 `upstream/devel@0bc05ef81107ac61ec38d622f71b44a44d1b4821`이며, 검토 브랜치는 `review/kevin9327-5212-5314-20260818`다. 2026-08-18 최종 fetch에서도 기준 SHA는 변하지 않았다. 27개 열린 원 PR의 기능 commit만 순서대로 적용했고, 원 branch에는 push하지 않았다.

## 원 source와 누적 적용 SHA

| 원 PR | 원 기능 source | 검토 브랜치 적용 commit |
| --- | --- | --- |
| #5212 | `db2ef47` | `8f179edc` |
| #5212 (2) | `5bf0bd6` | `211328fd` |
| #5212 (3) | `3283651` | `c81792bb` |
| #5213 | `eb8ad67` | `cf976c40` |
| #5213 (2) | `9c203b2` | `c5827239` |
| #5213 (3) | `f8e5733` | `9402dcc2` |
| #5213 (4) | `04989f9` | `461488d1` |
| #5222 | `13a4ec9` | `e318223c` |
| #5222 (2) | `d5fc42e` | `9aac4c0d` |
| #5225 | `9512ea9` | `5c7c4d32` |
| #5225 (2) | `37fedd4` | `12467e43` |
| #5226 | `054ff31` | `e593df8d` |
| #5226 (2) | `74fe7fa` | `15d11d2c` |
| #5232 | `5851bd3` | `799106e9` |
| #5232 (2) | `0f8483f` | `cde46411` |
| #5238 | `262d2cd` | `e92deaf5` |
| #5238 (2) | `14748e4` | `f7b9148d` |
| #5239 | `e47f35a` | `3a5556c0` |
| #5239 (2) | `aa97347` | `c07fa67c` |
| #5241 | `58c7c5f` | `fd12a046` |
| #5241 (2) | `abbd2db` | `a2ad6ef1` |
| #5244 | `1b01c74` | `3c78cacb` |
| #5244 (2) | `79158e0` | `19a5ef1a` |
| #5266 | `3e539a1` | `ba75294e` |
| #5267 | `ed4d082` | `3cb32825` |
| #5268 | `c5470cb` | `79a0173a` |
| #5269 | `31d37a0` | `a71f7cf7` |
| #5271 | `6555f1b` | `6497bc29` |
| #5276 | `15f484b` | `5d273b33` |
| #5280 | `572e3c1` | `ae51afd4` |
| #5286 | `c885c3e` | `34fc9d18` |
| #5288 | `8429b7c` | `ffba03d1` |
| #5290 | `b41f99f` | `117e3479` |
| #5299 | `cf73acd` | `a5bf2c16` |
| #5302 | `6bb5e3b` | `9467caa0` |
| #5303 | `9b1feae` | `7142f2ea` |
| #5304 | `f4ba71a` | `886ae2e2` |
| #5305 | `4842ffd` | `c3b966d0` |
| #5310 | `dcabeb4` | `7aa8303b` |
| #5314 | `2de37bb` | `a2a08dd9` |

## 충돌과 메인터너 보정

- #5303 registry 충돌은 기존 CAP-5293과 새 CAP-5295를 보존했다. #5310 충돌은 CAP-5293·5295·5296·5300을 모두 보존했다.
- `be0fa75ba9`: 통합된 text fixture의 LF·EOF 정규화.
- `f2ffa13173`: form-fill integration contract를 `tests/cases/`로 이동해 PR-base manifest 정책을 충족.
- `5ac70d87f`: BO14/FJ45 check 명 중복과 gym pack 허용 목록·profiles 문서 계약 보정.
- `bb9ef8b46`: Windows text pipe가 Bash block의 LF를 CRLF로 바꾸지 않도록 CodeQL workflow test 보정.
- `ea9218b52`: 새 Rust contract의 clippy manual-contains/manual-range-patterns 보정.
- CAP-5300 권위 문서 칸: recipe 링크 3개를 서식 자동화 가이드 1개로 정규화해 capability registry의 단일 내부 Markdown 링크 규칙을 충족.

## 검증 결과

1. `git diff --check upstream/devel...HEAD`와 `cargo fmt --all -- --check` 통과.
2. 변경 Python 모듈 1,637개, CI/CodeQL/review-only Python 54개 통과.
3. CI impact Node 62개, Rust manifest 16개, Rust unit tier 11개 통과.
4. `cargo clippy --all-targets --target-dir target/pr-review -- -D warnings` 통과.
5. 선택 regression suite 004·006·010·014·018·021·024·026: 723 passed, 6 skipped.
6. `cargo test --doc --target-dir target/pr-review`: 8 passed, 2 ignored.
7. 전체 `cargo nextest run --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 4 --no-fail-fast`는 대형 `issue2063_huge_cellbreak_table.hwp` fixture를 포함해 완료 프로세스까지 관찰했다. 실행 래퍼의 시간 제한으로 최종 stdout/exit code를 회수하지 못했으므로, 위 선택 suite의 명시적 723/723 성공으로 이번 변경 source의 직접 검증 근거를 보강했다.

## 원격 후속 경계

현재 원 PR들은 OPEN / MERGEABLE / BLOCKED 상태다. 통합 후보는 아직 push하지 않았으므로 BLOCKED는 원격 CI 전 상태다. 작업지시자 push 승인을 받은 뒤에만 새 head branch를 upstream에 push하고 devel 대상 통합 PR을 만들며, 최신 CI·CodeQL이 통과한 뒤 별도 merge 승인에 따라 merge·원 PR 처리·오늘할일 archive를 진행한다.
