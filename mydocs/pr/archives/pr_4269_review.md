---
kind: pr_review
status: review-ready
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-09
---

# PR #4269 검토 — 컨트리뷰터 PR 성능 검증 책임 경계

## 결론

**수용 권고.** [PR #4269](https://github.com/edwardkim/rhwp/pull/4269)은 특정 로컬 장비의
절대 성능 수치, 비공개 코퍼스와 maintainer 전용 벤치마크 통과를 컨트리뷰터의 공통 제출 조건에서
제외하면서도, 공개된 결정적 성능 회귀 테스트와 GitHub required checks는 merge gate로 유지한다.

공개 기여 계약, PR 템플릿과 maintainer 로컬 검증 계약이 같은 책임 경계를 가리키며 코드·CI threshold·
workflow를 변경하지 않는다. 문구상 blocking 근거와 후속 처리 책임도 maintainer에게 남아 있어 성능 회귀를
무조건 수용하는 정책으로 읽히지 않는다.

## 검토 경로

```text
base route: collaborator_self_merge.md
modifiers: intake_and_review.md, local_validation.md,
           review_only_fast_pass.md
loaded documents: pr_review_workflow.md, pr_review/README.md,
                  collaborator_self_merge.md, intake_and_review.md,
                  local_validation.md, review_only_fast_pass.md
devel base: fc8eb90d670e9ad0f07ef117c2d37f155b051ab2
review candidate: 0c1717f40c0f6b8ec5e928c795f8ac512f88b0ce
```

별도 `pr_4269_review_impl.md`는 만들지 않는다. 코드 보정, conflict 해결과 다중 PR 통합이 없는 단일
정책 문서 PR이므로 이 문서의 merge 조건만으로 실행 순서와 rollback 범위가 명확하다.

## 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#4269](https://github.com/edwardkim/rhwp/pull/4269) |
| 관련 이슈 | [#4268](https://github.com/edwardkim/rhwp/issues/4268), PR 본문 `Closes #4268` 확인 |
| 작성자 / assignee | `edwardkim` / `edwardkim` |
| reviewer | 작업지시자 지정 `edwardkim` maintainer self-review |
| GitHub review request | 작성자 자기 PR에는 review를 요청할 수 없어 REST API가 HTTP 422로 거부함. 별도 request 미등록 |
| milestone / labels | `v1.0.0` / `documentation`, `performance` |
| base / head | `devel` / `docs/issue-4268-contributor-performance-policy` |
| 접수 시점 head | `0c1717f40c0f6b8ec5e928c795f8ac512f88b0ce` |
| 접수 시점 규모 | 2 commits, 6 files, +175 / -0 |
| 접수 시점 상태 | open, non-draft, mergeable; `BLOCKED`는 CI 진행 중 참고값 |

위 head·규모·merge 상태는 review 기록 작성 시점 참고값이다. 이 review 문서를 push한 최신 head의 CI와
mergeability를 merge 직전에 다시 확인한다.

## 변경 범위와 검토

1. `CONTRIBUTING.md`는 환경 의존 절대 수치와 비공개 검증을 PR 제출 의무에서 제외하되, 성능 영향·공개
   재현·가능한 전후 관측값을 요청한다. 측정 장비가 없으면 `미측정`으로 제출할 수 있다.
2. `.github/pull_request_template.md`는 같은 정보를 선택형 입력란으로 제공하고, 공개 결정적 테스트와
   required checks가 면제되지 않음을 바로 아래에 명시한다.
3. `mydocs/manual/pr_review/local_validation.md`는 maintainer가 같은 환경의 전후 비교와 결정적 관측을
   우선하고, blocking 시 공개 가능한 sample·명령·환경·관측 또는 최소 재현을 남기게 한다.
4. 계획·Stage·오늘할일 문서는 이슈 #4268, 실제 변경 범위와 검증 결과를 같은 결론으로 기록한다.

CI workflow, timeout, 성능 임계치, source, test, fixture, WASM, Studio와 renderer는 변경하지 않았다.
비공개 코퍼스 자체나 식별 가능한 파일 목록도 포함하지 않았다.

## 렌더 및 시각 영향

시각·fixture 증적 보조 경로는 적용하지 않는다. renderer/layout/paint, HWP/HWPX sample, PDF, golden과
화면 UI 구현을 변경하지 않는 정책 문서 PR이므로 시각 검증 대상이 아니다.

## 로컬 검증

review candidate `0c1717f40`에서 다음을 확인했다.

| 검증 | 결과 |
| --- | --- |
| `upstream/devel` 조상 확인 | `fc8eb90d6`가 candidate의 조상, 별도 merge commit 없이 clean |
| 영향 문서 Markdown 링크 검사 | candidate 6개와 review 기록을 포함한 7개 문서, 내부 상대 링크 이상 없음 |
| `git diff --check upstream/devel...HEAD` | PASS |
| 변경 범위 | 문서 6개만 변경, source·test·workflow·fixture 변경 없음 |
| 공개 계약 대조 | 제출 조건과 merge gate가 분리되고 세 정책 문서가 서로 모순되지 않음 |

저장소 전체 `scripts/check_document_metadata.py`는 이 변경과 무관한 기존 기술 문서 2개의 오류 3건을
재현했다. 해당 파일은 PR diff에 없고, 이 PR의 메타데이터 변경 파일은 기존 canonical을 유지한다.

Cargo, WASM, Studio 빌드와 시각 검증은 실행하지 않았다. 문서 정책만 변경하므로 결과에 영향을 주는 실행
경로가 없고, 생략 범위를 PR 본문과 Stage 기록에 모두 남겼다.

## CI 경로와 남은 게이트

PR 전체가 문서이지만 `CONTRIBUTING.md`와 `.github/pull_request_template.md`는 review-only fast-pass B의
허용 경로인 `mydocs/` 밖에 있다. 따라서 B 경로로 단정하지 않고 GitHub가 선택한 일반 CI 결과를 기다린다.

review 기록 전 candidate의 CI run `31266657480`은 preflight와 Frontend package gates가 성공했고 Lint가
진행 중이었다. CodeQL run `31266657430`도 JavaScript/TypeScript·Python 분석은 성공하고 Rust 분석은 진행
중이었다. 이 값은 작성 시점 참고값이며, review 기록을 추가한 최신 head의 required checks가 최종 기준이다.

GitHub는 PR 작성자의 자기 `APPROVE` review와 reviewer request를 허용하지 않는다. 작업지시자가 지정한
maintainer self-review 결과는 최신 CI 성공 뒤 `COMMENT` review로 게시할 수 있으며, 실제 게시와 merge는
각각 작업지시자의 승인을 받는다.

## 발견한 문제와 위험

blocking finding은 없다. 정책 문구가 환경 의존 성능 검증을 면제하는 범위를 명확히 제한하고, 공개된 자동
gate와 심각한 회귀의 공개 재현 책임을 유지한다. rollback은 이 PR의 문서 변경만 되돌리면 되며 제품 코드나
검증 threshold의 복원 작업은 없다.

## 최종 권고

변경 내용과 검증 범위는 타당하므로 merge를 권고한다. 다음 조건을 모두 충족한 뒤 진행한다.

1. 이 review 기록을 포함한 최신 PR head의 required checks가 성공한다.
2. 최신 head가 mergeable인지 다시 확인한다.
3. 작업지시자 승인 뒤 self-review `COMMENT`를 게시한다.
4. 별도 merge 승인을 받은 뒤 PR을 merge한다. `Closes #4268`에 따라 이슈 상태를 merge 후 확인한다.
