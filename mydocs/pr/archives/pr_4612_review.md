---
kind: pr-review
status: pending-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-12
---

# PR #4612 self-review — v0.8.3 릴리즈 후보 준비

## 결론

**수용 권고.** [PR #4612](https://github.com/edwardkim/rhwp/pull/4612)는 최신 `devel`에
누적된 변경을 v0.8.3 배포 표면에 정렬하고, 릴리즈 게이트에서 발견한 Python 공식 바인딩
누락과 버전 정합 도구 결함을 테스트와 함께 보완한다. 코드·패키지·문서 범위가 크지만
릴리즈 후보 하나로 응집돼 있으며 self-review에서 blocking finding은 발견하지 않았다.

이 문서와 오늘할일을 포함한 trailing review-only commit의 fast-pass, 최신 mergeability,
`COMMENTED` self-review와 작업지시자의 명시적 merge 승인을 최종 조건으로 둔다. 이 PR은
`devel` 준비 단계이므로 관련 이슈 [#4601](https://github.com/edwardkim/rhwp/issues/4601)은
실제 태그·배포·채널 실측이 끝날 때까지 닫지 않는다.

## 검토 경로

```text
base route: collaborator_self_merge.md
modifiers: intake_and_review.md, local_validation.md,
           review_only_fast_pass.md, rework_and_exceptions.md
loaded documents: pr_review_workflow.md, pr_review/README.md,
                  collaborator_self_merge.md, intake_and_review.md,
                  local_validation.md, review_only_fast_pass.md,
                  rework_and_exceptions.md
devel base: 572786d0246915e435ab1cd94be8c72f49304bb5
code candidate: 8a58029b90b1a7585fdb9a20e3717fb700f0e94d
trailing review head: 이 문서와 오늘할일을 포함할 후속 docs-only commit
```

1,000줄을 넘는 대형 PR이므로 코드 검토, merge simulation, CI 확인과 merge 판단을 별도
cycle로 분리했다. 별도 maintainer 코드 보정이나 conflict 해결은 없고 기존
`task_m100_4601` 계획과 Stage 1~4 문서가 실행 순서를 기록하므로
`pr_4612_review_impl.md`는 추가하지 않는다.

## 메타데이터

| 항목 | 문서 작성 시점 참고값 |
| --- | --- |
| PR | [#4612](https://github.com/edwardkim/rhwp/pull/4612) |
| 관련 이슈 | [#4601](https://github.com/edwardkim/rhwp/issues/4601) |
| 작성자 / assignee | `edwardkim` / `edwardkim` |
| reviewer | `edwardkim` maintainer self-review (`COMMENTED` 방식); 작성자 본인 request는 비워 둠 |
| milestone | `v1.0.0` |
| labels | `documentation`, `ci`, `packaging`, `python`, `api`, `dependencies` |
| base / head | `devel` / `task/4601-release-v0.8.3` |
| code candidate | `8a58029b90b1a7585fdb9a20e3717fb700f0e94d` |
| 규모 | 51 files, +1,744 / -89 |
| 상태 | Open, non-draft, MERGEABLE / CLEAN; code candidate checks 성공 |

GitHub는 작성자가 자기 PR에 reviewer request나 `APPROVE`를 만드는 것을 허용하지 않는다.
따라서 별도 reviewer 요청은 만들지 않고, 최신 trailing head에서 blocking finding이 없다는
`COMMENTED` review를 게시한다. 이는 작업지시자의 merge 승인을 대체하지 않는다.

## 범위와 대형 PR 판단

변경량의 큰 부분은 한국어·영어 CHANGELOG, 스토어 reviewer note, 릴리즈 계획·Stage 기록,
Python argv 계약 테스트다. 제품 코드 변경의 중심은 다음 세 축이다.

1. Rust·Studio·웹 편집기·브라우저/VS Code 확장·설치 채널의 공개 버전을 `0.8.3`으로
   정렬한다.
2. 최신 CLI의 agent-value 명령군 11개를 Python 함수 20개로 노출하고, 선택 인자와
   `raise_on_verdict` 전달을 argv 단위 테스트로 고정한다.
3. `tools/set_package_version.py`가 Python·Node 패키지 메타데이터뿐 아니라 런타임
   `__version__`·`VERSION`도 함께 바꾸도록 보완하고 임시 파일 기반 회귀 테스트를 추가한다.

`upstream/main` ancestry 병합 commit `1ae2c393c`는 현재 `devel`의 workflow 내용을
유지해 첫 번째 부모와 tree가 같다. 따라서 PR commit 목록에 기존 main 계보가 보이지만
릴리즈 diff에 과거 main 구현을 다시 섞지 않는다.

Python API 보완은 독립 기능 확장이 아니라 릴리즈 게이트가 검출한 공식 바인딩 parity
결함의 해소다. wrapper는 기존 `run_json`과 `Envelope` 계약을 재사용하고, 각 하위 명령의
위치 인자·flag·JSON 출력·verdict 전달을 테스트하므로 CLI 문자열 조립의 침묵 드리프트를
막는다.

VS Code lockfile의 `fast-uri` 3.1.4 → 3.1.5는 개발 경로의 보안 패치이며 package audit
0건을 확인했다. 제품 runtime 의존 계약을 넓히지 않는다.

## merge simulation

2026-08-12 재조회에서 GitHub base와 로컬 `upstream/devel`은 모두
`572786d0246915e435ab1cd94be8c72f49304bb5`였다. `git merge-tree --write-tree
upstream/devel HEAD` 결과와 `HEAD^{tree}`가 모두
`a2d39be8a6c3a454503067ae951ccb58062d9bc9`로 같았다. 따라서 current base 기준
충돌이나 추가 conflict resolution은 없다.

## 완료한 로컬 검증

상세 명령·환경·산출 해시는
[Stage 4](../../working/task_m100_4601_stage4.md)에 기록했다.

| 게이트 | 결과 |
| --- | --- |
| Rust release binary | 통과, `rhwp v0.8.3` |
| Rust release lib | 3,498 passed / 13 ignored |
| release-test nextest | 5,767 passed / 36 skipped / 0 failed |
| Native Skia 3종 | 58 + 2 + 4 passed |
| fmt / clippy / doctest / diff-check | 통과 |
| Docker WASM + Studio | WASM 빌드·직접 적용, Studio 846 passed / 1 skipped |
| 호스트 CDP E2E | 49 passed / 0 failed |
| Python | 304 passed / 2 skipped, mypy·ruff 오류 0 |
| Node | 466 passed / 1 skipped, 재검증 unit 427 passed |
| VS Code·브라우저 확장 | VSIX·Chrome·Firefox 빌드, audit 0, web-ext error 0 |
| Docker CLI | 0.8.3, non-root `uid=10001` 실행 확인 |

Node·Studio 자식 프로세스 테스트의 샌드박스 `spawn` 차단은 한 번 진단한 뒤 호스트에서
같은 gate를 통과시켰다. 해당 EPERM 서명을 코드 실패로 반복 집계하지 않았다.

## GitHub Actions와 시각 영향

code candidate `8a58029b9`에서 CI, CodeQL, Render Diff, Canvas visual diff, Native Skia,
Node/Python binding, Action Self-test와 최종 Build & Test가 성공했다. 조건 분류상
`WASM Build`와 `Frontend unit gates` 일부 job은 skipped였으며, 로컬 공식 Docker WASM과
Studio·frontend 검증으로 해당 배포 표면을 별도 확인했다.

renderer, layout, typeset, paint, fixture와 sample은 이 PR에서 변경하지 않는다. 따라서 새
기준 PDF visual sweep 대상은 아니다. 자동 Canvas visual diff와 실제 Studio/CDP 경로의
성공을 릴리즈 후보 무회귀 근거로 사용한다.

## 의도적으로 남은 릴리즈 게이트

- crates.io는 10MB 선행 정비 기준을 넘으므로 기존 token·dry-run 이중 게이트가 게시를
  건너뛰는 비활성 채널로 유지한다.
- Safari 빌드와 deb/rpm/MSI·4플랫폼 archive는 태그 workflow의 OS runner에서 검증한다.
- Scoop·Homebrew·Winget·AUR 해시는 실제 `SHA256SUMS.txt` 생성 뒤 갱신한다.
- 위 항목은 이 `devel` 준비 PR의 blocking finding이 아니지만 #4601 종료 전 배포 실측
  대상이다.

## 최종 권고

blocking finding은 없다. 다음 순서로 수용한다.

1. 이 review 문서와 오늘할일만 추가한 trailing commit을 같은 PR branch에 push한다.
2. code candidate `8a58029b9`를 재사용한 review-only fast-pass와 최신 Build & Test를
   확인한다.
3. 최신 head의 MERGEABLE / CLEAN 상태를 재확인한다.
4. maintainer self-review를 `COMMENTED`로 게시한다.
5. 작업지시자의 별도 merge 승인을 받은 뒤 `devel`에 병합한다.
