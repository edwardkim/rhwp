---
kind: pr_review
status: approved
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-04
pr: 6720
issue: 6717
author: edwardkim
---

# PR #6720 self-review — Frontend package gates 설치 지연 정상화

## 결론

**승인.** PR #6720은 네 frontend package의 직렬 `npm ci`가 network 응답 변동으로 20분 job
예산을 잠식해 실제 회귀 gate까지 취소되던 운영 결함을 제한된 O3 변경으로 정상화한다.

code candidate `8702ac8fcc392f8320824b555d252c62c05b6f2e`를 재검토한 결과 변경은
`frontend-package-gates`의 install option·timeout과 이를 고정하는 계약 테스트에 한정됐다.
required check 이름, job ID·권한·실행 단계·실패 전파는 유지됐고 제품 source·manifest·lockfile은
바뀌지 않았다. 로컬 검증과 exact candidate의 Full CI·CodeQL은 모두 성공했으며 blocker나 범위
누출은 발견하지 않았다.

이 문서의 승인은 작성자 self-review 판정이며 GitHub approve event가 아니다. 자기 PR이므로 reviewer를
지정하지 않는다. 이 review와 오늘 기록만 추가한 trailing head는 workflow 변경 PR이므로 자체
preflight만 신뢰하지 않고 trusted controller의 review-only A.1 증명과 최신 required check를 다시
확인해야 한다. merge는 별도 메인테이너 승인 뒤에만 수행한다.

## 라우팅과 메타데이터

- 기본 경로: `collaborator_self_merge.md`
- 보조 경로: `intake_and_review.md`, `local_validation.md`, `review_only_fast_pass.md`
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`와 위 자식 문서
- `review_impl`은 추가하지 않는다. [수행계획](../../plans/task_m100_6717.md)과
  [Stage 1 기록](../../working/task_m100_6717_stage1.md)이 원인·구현·검증·rollback 순서를 고정한다.

| 항목 | 값 |
| --- | --- |
| PR / 작성자 | [#6720](https://github.com/edwardkim/rhwp/pull/6720) / @edwardkim |
| 관련 이슈 | [#6717](https://github.com/edwardkim/rhwp/issues/6717) (`Closes #6717`) |
| base | `devel@009e30fe1f6812b046862589783c68f890b4d363` |
| code candidate | `8702ac8fcc392f8320824b555d252c62c05b6f2e` |
| 규모 | 5 files, `+228/-6`, 1 commit |
| 작성 시점 GitHub 상태 | Open, 비 Draft, `MERGEABLE`·`CLEAN`; candidate check 전부 완료 |
| reviewer | self PR이므로 지정하지 않음 |

최신 fetch에서 `upstream/devel`은 candidate의 직접 조상이었고 divergence는 `1/0`이었다.
`git merge-tree --write-tree upstream/devel HEAD`의 결과 tree는 candidate tree와 같았으며
merge tree의 `git diff --check`도 통과했다.

## 변경 범위 검토

### workflow 실행 계약

- 네 install 명령은 Studio, Chrome, Firefox, VS Code 순서를 유지하면서 각각
  `npm ci --no-audit --prefer-offline`을 사용한다.
- `--no-audit`은 install 중 자동 advisory 조회만 끈다. lockfile clean install, lifecycle script,
  package test/build와 CodeQL은 그대로다.
- `--prefer-offline`은 cache를 우선하지만 cache miss의 network 접근을 허용한다. 완전 offline이나
  `--ignore-scripts`를 사용하지 않는다.
- timeout은 20분에서 30분으로 늘었다. 지연 뒤 회귀 gate가 실행될 여유를 주되 hang은 여전히 유한
  시간 안에 실패한다.
- `frontend-package-gates`, `Frontend package gates`, `contents: read`, `Build & Test` aggregate
  연결과 후속 gate의 실행 순서는 바뀌지 않았다.

### 계약 테스트

새 테스트는 package job의 30분 상한과 install step의 정확한 네 명령·순서를 단언한다. 또한
`--offline`과 `--ignore-scripts`의 도입을 거부한다. 단순 문자열 하나의 존재만 확인하지 않고 install
명령 집합 전체를 비교하므로 package 누락·중복과 option 이탈을 함께 탐지한다.

문서 3개는 #6717의 원인·보안 경계·검증·후속 #6715 복구 순서를 기록한다. renderer, HWP/HWPX/PDF,
sample·golden·fixture 변경이 없으므로 시각 검증은 비대상이다.

## 로컬 검증

| 검증 | 결과 |
| --- | --- |
| PyYAML workflow parse | 통과 |
| `python3 -m unittest scripts.tests.test_ci_impact_workflow` | 34건 통과 |
| `python3 -m unittest discover -s scripts/tests -p 'test_*workflow.py'` | 152건 통과 |
| 네 `npm ci --no-audit --prefer-offline` | 6.05초, 2.16초, 0.95초, 2.71초; 전부 성공 |
| 표준 Docker WASM build | 성공, 전체 453.59초 |
| frontend 계약·unit·build 묶음 | 전부 성공, 전체 22.21초 |
| Markdown 상대 링크 | 오류 0 |
| `git diff --check`와 merge tree 검사 | 통과 |
| package manifest·lockfile 변경 | 0 |

새 worktree의 첫 WASM binding 검사는 `pkg/rhwp.d.ts`가 없어 `ENOENT`로 중단됐다. CI와 같은 선행조건인
Docker WASM build를 완료한 뒤 전체 frontend 묶음을 처음부터 재실행해 모두 통과했다. 이 최초 실패를
제품 회귀나 최종 통과로 오인하지 않았다.

호스트에 `actionlint`가 없어 PyYAML과 repository workflow 계약 suite를 사용했다. metadata 검사의
기존 16건은 diff 밖의 네 historical 파일에서 재현됐고 해당 파일은 `upstream/devel`과 차이가 없으므로
신규 오류가 아니다. Rust source·test가 없어 Cargo lint 묶음은 로컬 변경 범위에서 비대상이다.

## exact candidate GitHub Actions

| 검증 | 결과 |
| --- | --- |
| [CI run 33846409061](https://github.com/edwardkim/rhwp/actions/runs/33846409061) | Full CI 성공, `Build & Test` 성공 |
| `Frontend package gates` | 6분 52초 성공; install 14초 |
| undo depth / responsive gate | 1분 53초 / 1분 44초 성공 |
| Studio·Chrome·Firefox·VS Code build | 모두 실제 실행 후 성공 |
| [CodeQL 33846409056](https://github.com/edwardkim/rhwp/actions/runs/33846409056) | Rust, Python, JavaScript/TypeScript 성공 |
| [Adapter 33846409173](https://github.com/edwardkim/rhwp/actions/runs/33846409173) | 성공 |
| [Proptest 33846409068](https://github.com/edwardkim/rhwp/actions/runs/33846409068) | 성공 |
| [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/33847557964) | 성공 |

이 한 번의 녹색 run은 registry·runner 지연의 재발 가능성을 제거하지 않는다. 다만 수정된 install 명령이
실행되고, 이전 attempt 2에서 취소됐던 undo gate와 그 이후 모든 build까지 30분 안에 이어졌다는 적용
증거다.

## 잔여 위험과 rollback

- 실제 network hang이면 실패 감지 상한이 최대 10분 늘어난다. 30분은 성능 목표가 아니라 hang
  상한이며 무제한 대기를 허용하지 않는다.
- 네 package의 직렬 설치 구조 자체는 유지된다. 이번 범위는 registry 변동의 비용을 줄이고 후속 gate
  예산을 확보하는 최소 정정이며, 병렬화·package 통합은 측정 근거가 생길 때 별도 판단한다.
- 자동 audit 요약을 제거했지만 기존에도 취약점 발견 시 실패시키는 독립 gate는 아니었다. 차단형 정책은
  별도 `npm audit` job·severity·예외·실패 계약을 승인받아야 한다.
- 회귀가 확인되면 #6717 commit을 revert해 기존 install과 20분 상한으로 복원할 수 있다.

## 최종 판정과 다음 조건

- 판정: **승인**
- 판정 대상: code candidate `8702ac8fcc392f8320824b555d252c62c05b6f2e`
- trailing 조건: 이 review, Stage 1 현행화와 오늘 기록만 추가한 최신 head에서 trusted controller의
  A.1 증명, required check 전부 성공, `MERGEABLE`·`CLEAN` 재확인
- merge 조건: 최신 head SHA 고정과 메인테이너의 별도 merge 승인
- GitHub review: self PR이므로 approve event와 reviewer 지정 없음
- merge 뒤: #6717 상태와 `devel` 반영을 확인한 뒤 PR #6715 branch를 최신 `devel`과 정합시켜
  exact-head full lane을 다시 실행한다.
