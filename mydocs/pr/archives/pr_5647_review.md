---
kind: pr-review
status: blocked
pr: 5647
issue: 5447
---

# PR #5647 검토 기록 - B2 구조 편집 스파이크와 한컴 판정 번들

- PR: [#5647](https://github.com/edwardkim/rhwp/pull/5647) `test: B2 구조 편집 스파이크 - 정책 3종 한컴 판정과 판정 번들`
- 관련 이슈: [#5447](https://github.com/edwardkim/rhwp/issues/5447), 부모 [#3683](https://github.com/edwardkim/rhwp/issues/3683)
- 작성자: `@johndoekim` external contributor, `maintainer_can_modify=true`
- source code candidate: `e565202890822c7f55e3be97b4d5a6d3522642d7`
- source base: `upstream/devel@52d8bf8eb3c3351cbabba00ce2b4e299d1930c01`
- 검토 기준: `upstream/devel@2d897ca04dc80819a9833cf96f2a971c3ae792a1`
- 라우팅: `collaborator_external_pr` direct source head + `intake_and_review` + `local_validation` + `review_only_fast_pass` + `rework_and_exceptions`
- 로드 문서: `pr_review_workflow.md`, `pr_review/README.md`, `collaborator_external_pr.md`, `intake_and_review.md`, `local_validation.md`, `review_only_fast_pass.md`, `rework_and_exceptions.md`, `visual_fixture_evidence.md`

## 검토 범위

- 기존 `tests/issue_4100_chart_data_edit.rs`에 B2 구조 변종 생성기와 상시 회귀 두 건을 추가한다.
- 행/점 삽입·삭제, 계열 복제·삭제, 계열명·카테고리 라벨 변경에서 `c:f` 무갱신,
  `c:ptCount` 재계산, `c:pt idx` 전수 재번호의 세 정책을 바이트 수술로 검증한다.
- 프로덕션 `src/`, renderer, Studio, 샘플 fixture는 변경하지 않는다. 생성 판정 번들은
  gitignored output으로 남기며, 새 보고서가 한컴 2022 판정 결과와 B2 구현 권고를 서술한다.

## 검증 근거

- 최신 `devel`과 source head의 merge tree `bac684f3121dd0b23e673f125cace121ead23aa4`는
  텍스트 충돌 없이 생성됐다.
- 최신 `devel`을 임시 merge한 동일 작업공간에서 suite manifest prepare/check, source-side
  unit tier check, `cargo fmt --all`, `cargo fmt --all -- --check`, `git diff --check`를 통과했다.
- `cargo nextest run --cargo-profile release-test --target-dir target/review-5647-johndoekim-20260820 --test issue_4100_chart_data_edit --no-fail-fast`는
  2분 58초에 37 passed, 2 skipped로 통과했다. B2 행 수술 SVG 렌더 회귀와 계열 채번·재개방
  회귀도 이 결과에 포함된다.
- source candidate의 required [Build & Test](https://github.com/edwardkim/rhwp/actions/runs/32250581010/job/96069918078)는 성공했다.

## 차단 사항

보고서의 핵심 결론은 한컴 2022에서 변환한 38개 결과와 사람이 확인한 렌더·편집기 판정이다.
그러나 PR diff와 #5447에는 그 결론을 재계산할 원본 판정 bundle, 한컴 변환 PDF, PPM 해시 입력,
대표 시각 asset의 안정 경로가 없다. 생성기는 gitignored output만 만들며, 보고서의 해시 표만으로는
어떤 파일을 어떤 한컴 결과와 비교했는지 제3자가 확인할 수 없다.

`b2_category_row_surgery_roundtrips_and_renders`의 SVG 계약은 rhwp 내부 렌더 회귀 근거로
유효하지만 한컴 2022 판정의 대체 증적은 아니다. 이 PR은 향후 B2 본구현에서 `c:f`를 갱신하지
않는 설계 결정을 고정하므로, 다음을 보존하기 전에는 그 결론을 승인할 수 없다.

- 생성 입력과 한컴 반환물을 재현할 수 있는 bundle 또는 대형 자산 보관 경로, 파일별 SHA-256,
  38건 판정 매니페스트를 남긴다.
- 정상 변종과 두 fail-closed 경계 변종의 대표 PDF/PNG를 시각 증적 정책의 안정 경로에 보존하고,
  보고서에서 원본·변환본·대표 asset의 역할과 경로를 구분한다.
- 저장소 용량 제한이 있으면 `pdf-large`/LFS 정책에 맞는 자산 위치와 재생성 절차를 명시한다.

## 결론

**보류.** 최신 `devel`에서 Rust 회귀와 정책 호환은 확인했고 코드 수준 차단 결함은 발견하지 못했다.
다만 한컴 판정 자산이 보존되기 전에는 정책 3종과 fail-closed 경계의 사용자-visible 결론을 merge
근거로 승인하지 않는다. #5447은 B2 본구현 추적 이슈이므로 닫지 않는다.
## 2026-08-20 최종 검토

- 최신 `upstream/devel` 위 병합 시뮬레이션이 충돌 없이 통과했다. 원 PR head의 녹색 CI와 함께,
  검토 전용 재적용 head에서 `cargo fmt --all -- --check`, manifest/unit-tier 정책, `git diff --check`를
  통과했다.
- #5683의 Cargo 격리 계약 4건을 통과했고, review worktree의 `--prepare` 전후 `Cargo.toml`과
  `Cargo.lock` SHA-256은 동일했다. 파생 suite 준비가 원 PR source manifest를 변경하지 않는다.
- `issue_4100_chart_data_edit`는 38 passed, 2 ignored였다. `pdftoppm` 144dpi로 한컴 PDF 판정 원장을
  재계산해 38개 파일, 262개 검사를 전건 일치로 확인했고, 정상 행 추가·원형 계열 무시·주식형 의미 깨짐의
  시각 근거도 원장 판정과 일치한다.

**승인.** 이 기록보다 앞선 증적 보류 판정은 추가 증적 이전 상태이며, 본 절이 최종 검토 결과다.
