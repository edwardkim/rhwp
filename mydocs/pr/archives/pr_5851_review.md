---
kind: pr-review
status: review-complete-pending-trailing-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-22
---

# PR #5851 검토 - DOCTYPE HWPML 2.1의 제한적 수용

## 접수 메타데이터

| 항목 | 작성 시점 확인값 |
| --- | --- |
| PR / 작성자 | [#5851](https://github.com/edwardkim/rhwp/pull/5851) / [@kevin9327](https://github.com/kevin9327) |
| 관련 issue | [#5848](https://github.com/edwardkim/rhwp/issues/5848) |
| base / contributor head | `devel` / `fix/5848-hwpml-doctype-version` |
| contributor 검토 기준 SHA | `7851be4bdf3d0c4eca443d7660bfa7d1388f1b26` |
| code candidate | `bf2803424dfc22647bb6982e7cb1acec8c6eaf1d` |
| 상태 | Open, non-draft, `MERGEABLE`, `CLEAN`, `maintainerCanModify=true` |
| 변경 규모 | 7 files, +379 / -7 (code candidate 기준) |
| 절차 | `collaborator_external_pr` 9.3.1.4 호환 보정 후 9.3.2 trailing review-only |

## 변경 범위와 메인터너 보정

- PR은 `Event::DocType`를 HWPML 감지에서 건너뛰고, HWPML `Version="2.1"`을 허용하며,
  안전한 내부 DTD 리터럴 엔티티의 제한적 수용과 본문 문자참조 해석을 추가한다.
- 내부 엔티티는 중첩 참조, 파라미터 엔티티, `SYSTEM`/`PUBLIC` 외부 엔티티를 수용하지 않는다.
  개수와 길이 상한도 유지해 XXE와 확장 폭발 경로를 열지 않는다.
- 검토 중 숫자 문자참조의 유효성 검사가 Rust `char` 생성 가능 여부에만 의존함을 발견했다.
  이 상태에서는 XML 1.0이 금지하는 `&#0;`, `&#x1F;`, `&#xFFFE;`도 내부 DTD 엔티티로 수용될 수 있다.
- 메인터너 보정 `bf2803424`은 XML 1.0 허용 범위
  `#x9 | #xA | #xD | #x20..#xD7FF | #xE000..#xFFFD | #x10000..#x10FFFF`만 통과시키고,
  case와 기존 HML parser suite에 거부 회귀를 추가했다. `&#160;`의 정상 수용은 유지한다.

## 검증 증적

code candidate `bf2803424`에서 다음을 완료했다.

- `node scripts/rust-test-suite-manifest.mjs --prepare`와 `--check`,
  `node scripts/rust-unit-test-tiers.mjs --check`, `cargo fmt --all -- --check`,
  `cargo clippy --locked --all-targets --target-dir target/pr-review -- -D warnings`,
  `git diff --check`가 통과했다.
- `issue_5848_hwpml_doctype_and_version` focused test는 **8 passed**,
  `hml_parser::rejects_invalid_xml_character_references_in_doctype_entities`는 generated
  `regression_suite_004`에서 **1 passed**다.
- 전체 검증은 `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review
  --tests --test-threads 12 --no-fail-fast`으로 **8,079 passed, 3 slow, 39 skipped**다.
- 실제 #5848 입력 `08462` HWPML 문서(SHA-256
  `897d70c9997ac52e89bdd9cad93a2a09682cbfb31c2125fa0ab7fc3f15cd6d96`)는 현재 head에서
  `rhwp info`가 exit 0으로 열고 HML `Version 2.1`, 1 section, **4 pages**를 보고했다.
  이는 issue에 기록된 한글 2022의 4쪽과 일치한다. 지원하지 않는 `PICTURE`/`HEADER`/`FOOTER`/
  `AUTONUM` 등 9개 warning은 남는다.
- GitHub Actions는 같은 SHA에서 [Build & Test](https://github.com/edwardkim/rhwp/actions/runs/32559956806/job/97001156683),
  [Lint](https://github.com/edwardkim/rhwp/actions/runs/32559956806/job/96999948957), archive build/shard,
  [CodeQL Rust 분석](https://github.com/edwardkim/rhwp/actions/runs/32559956686/job/96999950073),
  [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/32559956701/job/96999935524),
  [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/32559956685/job/96999941956)을 성공으로 완료했다.
  Native Skia와 frontend gates의 skip은 변경 범위 분류에 따른 정상 결과다.
- 최신 `upstream/devel@4bd9c5d60`과 code candidate의 merge-tree는 충돌 없이 생성됐고
  `git diff --check upstream/devel...HEAD`도 통과했다.

## 시각 범위와 잔여 한계

PR에 포함된 `mydocs/report/assets/task_m100_5848/after.png`은 문서가 렌더 경로까지 도달함을 보이는
참고 자료다. 다만 전체 쪽 비교나 기준 PDF visual sweep 증적은 아니며, 이미지 단독으로 완전한
layout fidelity를 주장할 수 없다. 이 PR의 수용 범위는 DOCTYPE HWPML 2.1의 감지·파싱·안전한 엔티티
수용과 실제 4쪽 개방까지다. 지원하지 않는 요소의 표현 충실도는 별도 fidelity 범위로 남긴다.

## 최종 판정

**수용 권고.** #5848의 접근 차단은 실제 문서의 4쪽 개방과 회귀·보안 가드로 해소됐고, 보정 뒤
Full CI와 CodeQL도 완료됐다. merge 전 조건은 이 review와 오늘할일만 담은 trailing docs commit의
review-only fast-pass 성공, 최신 head의 `MERGEABLE`/`CLEAN` 재확인, 그리고 메인테이너 merge다.
