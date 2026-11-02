# PR #8453 리뷰 — webpack 5.112.1 → 5.112.2

## 최종 판정

**승인 — 원 PR 정상 merge 완료.**

작업지시자의 9건 원격 처리 승인 뒤 정확한 최종 head CI와 최신 devel의 merge-tree를 재확인하고 정상 merge했다. 아래 원 head의 진단·작성 시점 상태는 이력으로 보존하며, 확정된 원격 결과는 마지막 절에 기록한다.

## 접수 정보

- PR: [#8453](https://github.com/edwardkim/rhwp/pull/8453), 작성자 `dependabot[bot]`, base `devel`, Open·non-draft.
- 원 head: `dc8b52ad418594a42b52560c7b706ca0fe4f1783`. reviewer `edwardkim` 지정과 API 재확인을 완료했다.
- 기준 devel: `d3f7209bbb0fdb1711c5f40a70ad22f0ca0be66d`. 누적 검증 후보: `e58024b15e11438a0e1162a25d836d3c3c4ff7a9`.
- 기본 경로 `maintainer_general`; 보조 `intake_and_review`, `local_validation`, `multi_pr_update_branch`, `rework_and_exceptions` 및 해당 renderer 증적 절차.
- 충돌 simulation: 독립·누적 모두 clean. 작성 시 mergeable `MERGEABLE`, merge state `CLEAN`는 참고값이며 승인 뒤 다시 조회한다.

## 변경과 검증

VS Code extension 개발용 bundler의 patch 업데이트다. manifest/lock의 해당 패키지 외 전이 변경은 없다. extension과 webview bundle을 실제 새 webpack으로 생성했다.

rhwp-vscode npm ci 및 npm run compile. webpack 5.112.2의 실제 compile 성공을 확인했다. 별도의 VS Code extension host 수동 조작을 수행한 것으로 표시하지 않는다.

공유 검증은 원 PR별 결과와 구분했다. 녹색 원 head의 broad default/Native 회귀는 4.3.0 조건을 충족하는 적용 범위에서 재사용하고, 모든 변경을 합친 후보의 필수 검증은 Skia 보정 때문에 한 번 별도로 수행했다. 같은 전체 검사를 9번 반복한 결과로 표시하지 않는다. [실제 명령·종료 코드·로그 요약](../assets/dependabot-20261102/README.md).

현재 base로 반영한 #8309의 저장 후 Bottom 정렬은 정식 focused와 독립 Print 전쪽 Native/fresh WASM에서 보존됨을 확인했다. 제품 source·test·golden·baseline·자원 한도 변경은 없다.

## 보안·공급망과 CI

NPMScan은 실제 변경 lock entry 각 1개와 root만 대조했다. 신규 취약점·설치 스크립트·출처 변경·identity mismatch는 없고 전이 delta는 없다. 축약 입력의 다른 root dependency 미포함 경고를 lockfile 결손으로 해석하지 않는다. 전체 npm 그래프 감사가 아니다.

- [Native Skia tests](https://github.com/edwardkim/rhwp/actions/runs/41379707378/job/125067706092): SKIPPED
- [Build & Test](https://github.com/edwardkim/rhwp/actions/runs/41379707378/job/125072471946): SUCCESS
- [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/41381374777): SUCCESS

CodeQL app의 NEUTRAL은 이 scoped 실행에 Python/Rust 두 설정이 없다는 집계 메시지이고 annotation은 0건이다. 실제 JS/actions 분석과 CI Impact Policy는 성공했다. 경고를 dismiss하거나 탐지 설정을 낮추지 않았다. [원문](../assets/dependabot-20261102/pr-8453-codeql-neutral.json).

## 조판 원칙과 시각 판독

이번 diff는 조판 규칙·문서 모델·serializer를 수정하지 않는다. 공통 Native/fresh WASM 시각 대조는 다른 Cargo 변경과의 결합 호환성 근거이며, 이 PR 자체의 시각 개선으로 해석하지 않는다.

줄 구성·좌표 생산/소비·pagination 컷 규칙을 바꾸는 구현이 없어 해당 새 규칙 입증은 비해당이다. 실제 소비 backend와 기존 입력·독립 Print 계약은 검증했다. 새 회귀·fixture·절대 좌표 핀 추가나 기존 기대값 변경은 없다. 수식이 없는 입력에는 수식 모양 검사를 수행했다고 쓰지 않는다.

## 원격 처리·후속 계획

승인 뒤 원 PR을 개별 처리한다. source SHA를 재확인하고 최신 devel merge-tree와 해당 head의 실제 required checks를 대조한 뒤 `--match-head-commit`으로 merge한다. 승인되지 않은 workflow 재실행·취소나 보안 metadata 변경은 하지 않는다. merge SHA·CI를 기록하고, 허용된 후속 댓글·archive 기록·기본 devel 동기화와 소유한 worktree/output/debug 정리를 수행한다. 사용자 기존 Vite·Studio 배포 WASM·다른 작업은 보존한다.

## 원격 통합과 후속 처리

- 최종 head `dc8b52ad418594a42b52560c7b706ca0fe4f1783`, merge `72f8cafa505bf058a60ed35f1d4096a5700e2f0f`, 실제 merge 시각 `2026-11-02T06:07:32Z`.
- 최신 대상 head의 [Build & Test](https://github.com/edwardkim/rhwp/actions/runs/41379707378/job/125072471946)와 [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/41381374777) 성공을 확인했다. 적용 범위의 원 CI·skip/NEUTRAL은 [원격 증적](../assets/dependabot-20261102/remote-integration.json)에 구분한다.
- merge 직전 실제 base `e2f97b8c7d01a6d1487e41bd401fb5420c53d25b`에서 clean merge-tree를 확인했고 실제 merge의 부모·tree도 대조했다. source head를 고정한 normal merge이며 원 bot commit을 보존했다.
- 관련 closing issue는 없다. 검증 CI는 PR에서 완료했고 merge 뒤에는 duration metadata만 관찰했다. 자료 부족·이전 pending의 자동 대체는 검증 CI 재실행으로 우회하지 않았다.
- 유지보수 기록은 archive review·asset·오늘할일의 직접 반영으로 결정했다. 기본 devel 동기화·댓글·종료 정리는 승인 범위이며 실제 완료 상태와 permalink는 `/home/edward/.local/state/rhwp/dependabot-cleanup-20261102.json`에 기록한다.

## Merge 후 contributor PR comment 계획

[이 PR의 정식 초안](../assets/dependabot-20261102/comment-drafts/pr-8453-merge-comment-draft.md)을 사용한다.
원 PR의 merge SHA와 source head CI를 실제 값으로 채우고 증적은 후속 Skia merge `feb092532a03f7ebba4e7a9517d25c2b1b5714b5`를 사용한다.
4문서 전쪽 5쪽의 Native/fresh WASM 기준·후보 픽셀 동일성과 raw 최저 94.40%/98.77%,
직접 2배 판독·기존 face/획 차이를 구분한다. 이 raw 값은 2px 내용 실루엣 자동 일치율이며
사람 판정 정확도가 아니다. 시각 소비 PR에는 Visual Sweep 정본과 해당 merge의 실제 비교 PNG를
이미지로 표시한다. `--body-file` 게시 뒤 API로 한글·치환 완료·본문 일치를 확인하고 중복 게시하지 않는다.
