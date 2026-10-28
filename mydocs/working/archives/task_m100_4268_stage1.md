# Task M100 #4268 Stage 1 — 컨트리뷰터 PR 성능 검증 책임 경계

- 이슈: [#4268](https://github.com/edwardkim/rhwp/issues/4268)
- 기준 브랜치: `upstream/devel`
- 작업 브랜치: `docs/issue-4268-contributor-performance-policy`
- 작성일: 2026-08-09 KST
- 상태: 구현 및 로컬 문서 검증 완료

## 목표

환경 의존적인 성능 측정을 컨트리뷰터의 공통 PR 제출 조건으로 오해하지 않도록 공개 제출 계약과
maintainer의 merge 판정 책임을 분리한다. 다만 저장소에 공개된 결정적 성능 회귀 테스트와 GitHub
required checks는 기존 merge gate로 유지한다.

## 반영 내용

- `CONTRIBUTING.md`
  - 특정 장비의 절대 수치, 비공개 코퍼스와 maintainer 전용 벤치마크 통과는 PR 제출 조건이 아님을
    명시했다.
  - 성능 영향 가능성이 있는 PR에는 예상 영향, 공개 재현 절차와 가능한 범위의 동일 환경 전후 관측값을
    요청하되 측정 환경이 없으면 `미측정`으로 제출할 수 있게 했다.
  - 공개된 결정적 성능 테스트와 required checks는 merge gate로 유지하고, 성능 회귀로 보류할 때는
    maintainer가 공개 가능한 재현 근거를 제공하도록 했다.
- `.github/pull_request_template.md`
  - 성능 영향과 재현·측정 결과를 선택적으로 기록하는 입력란을 추가했다.
  - 환경 의존 절대 수치와 공개 자동 gate의 차이를 템플릿 안에서도 바로 확인할 수 있게 했다.
- `mydocs/manual/pr_review/local_validation.md`
  - CI timeout을 제품 성능 목표로 해석하지 않는 원칙과 동일 환경 전후 비교 우선순위를 명시했다.
  - 비공개 자료에서 발견한 회귀는 자료나 식별 파일 목록을 공개하지 않고 최소 공개 재현 또는 비식별
    집계 근거로 전환하도록 했다.
  - blocking 시 maintainer 보정, 공개 재현 제공 또는 후속 issue 분리 중 처리 경로를 남기도록 했다.

## 변경하지 않은 범위

- CI workflow, timeout, required check와 성능 임계치
- 비공개 코퍼스의 공개 범위
- Rust, WASM, Studio와 렌더링 코드

## 검증

- `python3 scripts/check_markdown_links.py CONTRIBUTING.md .github/pull_request_template.md mydocs/manual/pr_review/local_validation.md mydocs/plans/task_m100_4268.md mydocs/orders/20260809.md mydocs/working/task_m100_4268_stage1.md`
  - 통과
- `git diff --check`
  - 통과
- 공개 기여 가이드와 maintainer 검증 문서 문구 대조
  - 제출 조건과 merge gate가 분리되어 있고 서로 모순되지 않음을 확인

저장소 전체 메타데이터 검사는 이 변경과 무관한 기존 기술 문서 두 파일의 오류 3건 때문에 실패한다.
이 Stage의 메타데이터 대상 변경 파일인 `local_validation.md`의 front matter는 기존 canonical을 유지하고
`last_verified`만 2026-08-09로 갱신했다.

일반 Markdown 정책 변경이므로 Cargo, WASM, Studio 빌드와 시각 검증은 수행하지 않았다.

## 결과

이슈 #4268의 문서 후보를 준비했다. 후보 커밋을 만든 뒤 Open PR 생성은 별도 승인 게이트에서 진행한다.
