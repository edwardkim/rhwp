# Task M100 #5177 Stage 4 - CI PR-base 계보 조회 보정

## 관찰

PR #5179의 `Validate Rust test suite manifest` 단계는 PR base SHA를 깊이 1로만 가져온 뒤
`base...HEAD` 3-way diff를 수행했다. 이 checkout에는 base와 HEAD를 잇는 공통 조상이 없어
`fatal: <base>...HEAD: no merge base`로 fail-closed 됐다.

## 수정

- lint job checkout에 `fetch-depth: 0`을 지정해 PR base와 head의 전체 Git 계보를 확보한다.
- 별도 shallow base fetch를 제거하고, 기존 `base...HEAD` 비교를 유지한다.
- workflow 계약 테스트가 lint checkout의 전체 계보 조회와 shallow base fetch 제거를 검증한다.

## 기대 결과

PR commit에 포함된 파생 suite 산출물만 정확히 거부하고, checkout 깊이 부족으로 인한 정책 검사 실패는
발생하지 않는다.
