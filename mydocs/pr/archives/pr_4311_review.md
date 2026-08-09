---
kind: pr_review
status: review-ready
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-09
---

# PR #4311 검토 — 기술 문서 메타데이터 역할 정합화

## 결론

**수용 권고.** [PR #4311](https://github.com/edwardkim/rhwp/pull/4311)은 기술 문서의
실제 역할을 기존 메타데이터 스키마에 맞게 분류해 저장소 전체 메타데이터 검사 오류 3건을
해소한다. 스키마를 확장하지 않고 권위 문서는 `canonical`, 지속되는 구현 결정 문서는
`decision`으로 정리한 범위가 타당하다.

## 검토 경로

```text
base route: collaborator_self_merge.md
modifier: review_only_fast_pass.md
loaded documents: pr_review_workflow.md, pr_review/README.md,
                  collaborator_self_merge.md, review_only_fast_pass.md
devel base: e8aa8272a0da0e572820657ee4b24ef4f2bbf604
code candidate: 2592034ba3bdc66d9ceb6f4ec7a98162f4e97317
```

코드 보정, conflict 해결과 다중 PR 통합이 없는 메타데이터 전용 PR이므로 별도
`pr_4311_review_impl.md`는 만들지 않는다.

## 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#4311](https://github.com/edwardkim/rhwp/pull/4311) |
| 작성자 / assignee | `edwardkim` / `edwardkim` |
| reviewer | `edwardkim` maintainer self-review (`COMMENT` 방식) |
| milestone / labels | `v1.0.0` / `documentation` |
| base / head | `devel` / `docs/fix-tech-metadata-20260809` |
| 접수 시점 head | `2592034ba3bdc66d9ceb6f4ec7a98162f4e97317` |
| 접수 시점 규모 | 1 commit, 2 files, +3 / -2 |
| 접수 시점 상태 | open, non-draft |

이 검토 기록을 포함한 최신 head의 CI와 mergeability를 merge 직전에 다시 확인한다.

## 변경 범위와 검토

1. `envelope_provenance.md`는 Envelope 구조와 도입·소비 지점을 한곳에서 정의하고 다른 문서가
   참조하는 단일 진실 공급원이므로 `kind: canonical`이 문서 역할과 일치한다.
2. `task_m100_3604_password_encryption_cpp_review.md`는 제품 비밀번호 암호화 구현을 Rust 공통
   모듈에 두고 C++ private ABI는 교차 검증에만 사용한다는 지속되는 선택을 기록하므로
   `kind: decision`이 적절하다.
3. 두 번째 문서에는 자체 canonical 경로를 추가해 현재 권위 문서의 위치를 명시했다.
4. 기술 사실을 다시 검증한 변경이 아니므로 두 문서의 기존 `last_verified` 값은 갱신하지 않았다.

허용 kind 목록에 `contract`나 `technical-note`를 추가하지 않았다. 따라서 역할이 불명확한 새
분류를 도입하거나 다른 문서의 분류 계약을 느슨하게 만들지 않는다.

## 영향과 검증

제품 코드, 테스트, CI workflow, fixture, sample, WASM, Studio와 renderer는 변경하지 않는다.
따라서 빌드·런타임·렌더 결과와 사용자 화면에는 영향이 없으며 시각 검증 대상도 아니다.

| 검증 | 결과 |
| --- | --- |
| `python3 scripts/check_document_metadata.py` | PASS, 520개 문서, 오류 없음 |
| `python3 scripts/check_markdown_links.py` | PASS, 525개 문서, 상대 링크 오류 없음 |
| `git diff --check upstream/devel...HEAD` | PASS |
| 변경 범위 | `mydocs/` 아래 Markdown 문서만 변경 |

## CI 경로와 남은 게이트

PR 전체가 `mydocs/` 아래에 있으므로 review-only fast-pass B 대상이다. 최신 head에서 preflight와
Build & Test aggregate 결과를 확인하고, 문서 전용 분류가 유지되는지 검증한다. Rust·WASM·Studio
전체 실행이 skip되면 변경 범위에 따른 정상 라우팅으로 판정한다.

GitHub는 PR 작성자의 자기 `APPROVE` review를 허용하지 않는다. maintainer self-review 결과는
blocking finding이 없다는 `COMMENT` review로 게시한다.

## 최종 권고

blocking finding은 없다. 다음 조건을 충족하면 merge한다.

1. 이 검토 기록을 포함한 최신 PR head의 required checks가 성공한다.
2. 최신 head가 mergeable인지 확인한다.
3. self-review `COMMENT`를 게시한 뒤 merge하고 최신 `devel` 반영을 확인한다.
