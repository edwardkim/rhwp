# PR #8457 리뷰 — taiki-e/install-action 2.87.31 → 2.88.1

## 최종 판정

**승인 — 변경 범위와 current-base 결합 호환성 검토 완료.**

이는 로컬 review 판정이며 GitHub approve·push·comment·merge 승인을 대신하지 않는다. 다음 조건은 작업지시자의 원격 처리 승인, merge 직전 최신 head·CI·mergeability 재확인이다.

## 접수 정보

- PR: [#8457](https://github.com/edwardkim/rhwp/pull/8457), 작성자 `dependabot[bot]`, base `devel`, Open·non-draft.
- 원 head: `cd0ccf51995d9e985f8137d211db6077d7003a8b`. reviewer `edwardkim` 지정과 API 재확인을 완료했다.
- 기준 devel: `d3f7209bbb0fdb1711c5f40a70ad22f0ca0be66d`. 누적 검증 후보: `e58024b15e11438a0e1162a25d836d3c3c4ff7a9`.
- 기본 경로 `maintainer_general`; 보조 `intake_and_review`, `local_validation`, `multi_pr_update_branch`, `rework_and_exceptions` 및 해당 renderer 증적 절차.
- 충돌 simulation: 독립·누적 모두 clean. 작성 시 mergeable `MERGEABLE`, merge state `CLEAN`는 참고값이며 승인 뒤 다시 조회한다.

## 변경과 검증

O3 action 의존성 변경이다. build-nextest-archives.yml과 run-nextest-archives.yml의 full SHA 두 곳만 바뀐다. 새 SHA 68a389c788b2cd04016ce8c80ee23b80d2ab3d68과 v2.88.1 tag를 대조했다. 실행 스크립트와 권한·event·job 명령은 바뀌지 않고 설치 manifest가 갱신됐다.

nextest archive workflow 계약 18건, 원 head의 실제 GitHub archive build/test 성공. Archive A 로그에서 새 action SHA와 cargo-nextest 0.9.150 다운로드·checksum 검증·실행을 확인했다. 로컬 nextest 0.9.137을 새 버전 실행 증거로 쓰지 않는다.

공유 검증은 원 PR별 결과와 구분했다. 녹색 원 head의 broad default/Native 회귀는 4.3.0 조건을 충족하는 적용 범위에서 재사용하고, 모든 변경을 합친 후보의 필수 검증은 Skia 보정 때문에 한 번 별도로 수행했다. 같은 전체 검사를 9번 반복한 결과로 표시하지 않는다. [실제 명령·종료 코드·로그 요약](assets/dependabot-20261102/README.md).

현재 base로 반영한 #8309의 저장 후 Bottom 정렬은 정식 focused와 독립 Print 전쪽 Native/fresh WASM에서 보존됨을 확인했다. 제품 source·test·golden·baseline·자원 한도 변경은 없다.

## 보안·공급망과 CI

외부 action은 full SHA 고정과 버전 주석을 유지한다. fork 신뢰 경계·token 전달·write 권한을 확대하지 않는다. upstream action 실행 코드가 바뀌지 않았고 manifest delta와 실제 설치 로그를 대조했다.

- [Native Skia tests](https://github.com/edwardkim/rhwp/actions/runs/41379931125/job/125071501965): SUCCESS
- [Build & Test](https://github.com/edwardkim/rhwp/actions/runs/41379931125/job/125082364715): SUCCESS
- [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/41384556475): SUCCESS

## 조판 원칙과 시각 판독

이번 diff는 조판 규칙·문서 모델·serializer를 수정하지 않는다. 공통 Native/fresh WASM 시각 대조는 다른 Cargo 변경과의 결합 호환성 근거이며, 이 PR 자체의 시각 개선으로 해석하지 않는다.

줄 구성·좌표 생산/소비·pagination 컷 규칙을 바꾸는 구현이 없어 해당 새 규칙 입증은 비해당이다. 실제 소비 backend와 기존 입력·독립 Print 계약은 검증했다. 새 회귀·fixture·절대 좌표 핀 추가나 기존 기대값 변경은 없다. 수식이 없는 입력에는 수식 모양 검사를 수행했다고 쓰지 않는다.

## 원격 처리·후속 계획

승인 뒤 원 PR을 개별 처리한다. source SHA를 재확인하고 최신 devel merge-tree와 해당 head의 실제 required checks를 대조한 뒤 `--match-head-commit`으로 merge한다. 승인되지 않은 workflow 재실행·취소나 보안 metadata 변경은 하지 않는다. merge SHA·CI를 기록하고, 허용된 후속 댓글·archive 기록·기본 devel 동기화와 소유한 worktree/output/debug 정리를 수행한다. 사용자 기존 Vite·Studio 배포 WASM·다른 작업은 보존한다.
