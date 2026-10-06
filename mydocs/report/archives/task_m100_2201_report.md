# Task M100 #2201 결과 보고서 - PR review 권한별 반영 경로 정리

## 이슈

- [https://github.com/edwardkim/rhwp/issues/2201](https://github.com/edwardkim/rhwp/issues/2201)

## 결과

`mydocs/manual/pr_review_workflow.md`의 권한 모델과 실제 반영 절차를 다음과 같이 정리했다.

- maintainer는 admin 또는 branch protection bypass 권한, collaborator는 write 권한 기준으로 구분했다.
- 메인터너가 review 문서·오늘할일·검증 asset 같은 운영 기록만 `devel`에 직접 반영하는 옵션 M을 추가했다.
- 소스·테스트·workflow·golden/baseline·기존 샘플 변경은 권한과 무관하게 PR/CI를 사용하도록 제한했다.
- 단순 PR은 `pr_N_review_impl.md`를 생략할 수 있도록 공통 기준을 만들었다.
- merge 시뮬레이션을 `upstream/devel` 위에 PR head를 합치는 방향으로 정정했다.
- 문서, Rust, renderer/WASM, frontend, CI, golden 변경별 기본 검증을 구분했다.
- GitHub review/PR/issue 코멘트의 이슈·PR 참조를 `[URL](URL)` Markdown 링크로 작성하도록 규칙과
  코멘트 예시를 갱신했다.

`Admin Merge` 절과 `--admin` 사용 정책은 작업 범위에서 제외해 변경하지 않았다.

## 검증

| 검증 | 결과 |
|---|---|
| `git diff --check` | PASS |
| 변경 Markdown fenced code block 짝수 여부 | PASS |
| `Admin Merge` 절과 `upstream/devel` 원문 비교 | PASS, 차이 없음 |
| 변경 범위 확인 | PASS, `mydocs/**` 문서만 변경 |

문서-only 변경이므로 cargo, WASM, frontend 검증은 수행하지 않았다.
