# 작업 2501 단계 1 - CI 액션 버전 통합

## 범위

- [#2501](https://github.com/edwardkim/rhwp/pull/2501)의 네 workflow에서 구버전 GitHub Action을
  현재 CI와 같은 버전으로 올린다.
- [#2488](https://github.com/edwardkim/rhwp/pull/2488),
  [#2490](https://github.com/edwardkim/rhwp/pull/2490),
  [#2499](https://github.com/edwardkim/rhwp/pull/2499)의 workflow 변경은 #2501에 포함됐으므로
  별도로 cherry-pick하지 않는다.
- #2499의 과거 오늘할일 변경은 최종 PR 직전 규칙에 맞지 않아 통합 대상에서 제외한다.

## 검토 근거

- #2501은 `full-renderer-sweep.yml`, `npm-publish.yml`, `release-binary.yml`,
  `render-diff.yml`의 `v4` action 참조를 각각 `v5`·`v7`·`v8`로 갱신한다.
- 현행 `ci.yml`, `deploy-pages.yml`도 `actions/cache@v5`, `actions/upload-artifact@v7`,
  `actions/download-artifact@v8`, `actions/checkout@v5`를 사용한다.
- GitHub Actions 공식 release의 최신 태그는 더 높을 수 있으나, 이번 범위는 저장소의 이미
  검증된 공통 버전으로 맞추는 것이다.

## 검증 계획

1. `actionlint`로 변경된 네 workflow를 검사한다.
2. YAML 파싱과 `git diff --check`를 수행한다.
3. 네 workflow의 action version이 기존 CI 정책과 일치하는지 검색으로 확인한다.

## 결과

- 네 workflow의 action 참조가 `actions/cache@v5`, `actions/upload-artifact@v7`,
  `actions/download-artifact@v8`, `actions/checkout@v5`로 현행 CI 정책과 일치한다.
- 변경된 YAML 네 파일은 정상 파싱됐고, `git diff --check`도 통과했다.
- `actionlint`는 `release-binary.yml`과 `render-diff.yml`에 이미 있던 shellcheck 정보 경고
  8건만 보고했다. 해당 baseline 경고를 제외한 actionlint 검사는 통과했으며, 이번 action version
  변경과 관련된 문법·정책 오류는 없었다.
