---
kind: review
status: active
last_verified: 2026-10-07
---

# PR #7651 리뷰 — 수동 CodeQL Actions 분석

## 최종 판정

**승인 — 코드와 정책 계약의 로컬 검증을 충족한다.** GitHub 최신 head의 전체 CI·Actions 실제 Analyze 및 분석 업로드 성공은 아직 미확정이며 병합 전 필수 조건으로 남긴다. GitHub review 승인이나 병합 완료를 의미하지 않는다.

## 접수와 경로

- [PR #7651](https://github.com/edwardkim/rhwp/pull/7651), 작성자 `edwardkim`, base `devel`, [Issue #7650](https://github.com/edwardkim/rhwp/issues/7650).
- base route: `collaborator_self_merge.md` (maintainer 자체 PR), modifiers: 없음.
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`, `collaborator_self_merge.md`, `intake_and_review.md`, `local_validation.md`, `review_template.md`, `post_merge.md`, `github_operations.md`.
- reviewer는 지정하지 않고 작성자 self-review로 기록한다.
- 코드 head `7e7733a4428db97aa70a26920500d9178d01885a`, 기준 devel `b3c3047db`.

## 변경과 계약 검토

O3 실행·보안 계약 변경이다. CodeQL matrix의 Actions 추가를 trusted classifier 8·policy 7의 full 선택, compact 축 `ac`, 세 workflow의 상태 consumer와 연결했다. 기존 trusted base의 세 언어 full 응답은 네 언어 전체 분석으로 확장한다. 선택적 Rust/JS/Python과 순서는 보존한다.

결과 소비 경로는 workflow의 language finalizer → SELECTED_LANGUAGES → non-Rust init/Analyze이며, 전체 fallback에도 Actions를 포함한다. workflow review-only 재사용의 requiredChecks, shared post-merge verifier의 fullLane 판정, policy의 expanded matrix/실제 Analyze 단계 감사와 main promotion requiredJobs에서 Actions를 요구한다. 기존 세 언어 성공만으로 신규 분석 범위를 대체하지 않는다.

기존 GHAS 보안 판정·최소 권한·base에서 읽는 classifier·branch push 제한을 유지한다. 제품 코드·Rust 소스·조판에 영향이 없어 조판 원칙과 Visual Sweep은 비해당이다. 입력 문서·기준 PDF·WASM 증거도 비해당이다.

## 검증

- 관련 Node suite 8개: 495 PASS. classifier/policy/report/controller/workflow evidence 및 post-merge/squash/review bridge를 포함한다.
- `python3 -m unittest discover -s scripts/tests -p 'test_*workflow.py'`: 185 PASS.
- `python3 -m unittest scripts/tests/test_workflow_promotion_preflight.py scripts/tests/test_workflow_promotion_evidence.py`: 35 PASS.
- 공식 checksum 확인한 actionlint 1.7.11로 변경 YAML 세 개 PASS (`-shellcheck=`; 로컬 shellcheck 미설치), YAML 파싱·Node 구문·diff 검사 PASS.
- 새 핵심 검사는 수정 전 의도한 원인으로 FAIL / 수정 후 PASS: Actions 전체 선택·policy 누락/실패/Analyze 생략, workflow의 legacy candidate 재사용, shared post-merge legacy coverage, main promotion의 Actions 누락·skipped·failure. invalid/중복/역순 선택은 full로 닫으며 canonical 16개 선택을 실제 Bash finalizer로 검사했다.
- fixture 변경은 classifier 버전과 full의 Actions 추가만 반영했다. 조판 baseline 또는 기존 선택적 작업 범위를 완화하지 않았다.
- ignored 증적: `output/pr-review/codeql-actions-20261007/`의 before/after 로그·actionlint 로그·이전 SARIF. 기본 작업공간의 `output/pr-review/codeql-config-cfee-20261007/`에 기존 구성·alert 상태를 보존했다.

## 원격 검증과 후속

등록 당시 code head의 CI/CodeQL/Render Diff/Adapter/Proptest와 policy가 시작됐다. 문서 trailing head의 exact CI와 merge-tree를 병합 직전 재조회한다. 실패·누락·pending이 있으면 통합하지 않는다.

메인테이너는 main 적용을 **다음 devel → main 통합 시점**으로 지정했다. 이번 범위는 devel 설정 변경이다. main의 새 Actions 분석과 업로드가 확인되기 전에는 오래된 자동 구성 `cfee1df40ac1d9e320f7ce8499b9a748e4bfcf478f8ebb561a0be6e7107c057f`를 삭제하지 않는다. 따라서 #7650은 이번 PR로 자동 종료하지 않는다. 다음 main 적용 때 새 분석을 확인하고 기존 Actions 구성만 정리한다.
