# PR #8454 리뷰 — libc 0.2.189 → 0.2.191

## 최종 판정

**승인 — 변경 범위와 current-base 결합 호환성 검토 완료.**

이는 로컬 review 판정이며 GitHub approve·push·comment·merge 승인을 대신하지 않는다. 다음 조건은 작업지시자의 원격 처리 승인, merge 직전 최신 head·CI·mergeability 재확인이다.

## 접수 정보

- PR: [#8454](https://github.com/edwardkim/rhwp/pull/8454), 작성자 `dependabot[bot]`, base `devel`, Open·non-draft.
- 원 head: `4dd0e7eb196a1bfede636c4314fbe8fcc7bec8eb`. reviewer `edwardkim` 지정과 API 재확인을 완료했다.
- 기준 devel: `d3f7209bbb0fdb1711c5f40a70ad22f0ca0be66d`. 누적 검증 후보: `e58024b15e11438a0e1162a25d836d3c3c4ff7a9`.
- 기본 경로 `maintainer_general`; 보조 `intake_and_review`, `local_validation`, `multi_pr_update_branch`, `rework_and_exceptions` 및 해당 renderer 증적 절차.
- 충돌 simulation: 독립·누적 모두 clean. 작성 시 mergeable `MERGEABLE`, merge state `CLEAN`는 참고값이며 승인 뒤 다시 조회한다.

## 변경과 검증

Native 의존성의 플랫폼 FFI 선언 업데이트다. OS API 호출부나 지원 target 정책은 바꾸지 않는다. x86_64 Linux Native와 wasm32 의존 그래프가 유지되는지 확인했다.

current-base 누적 후보의 Native build·Clippy·Native 검사·전체 회귀와 wasm32 Clippy·최적화 WASM. 다른 OS의 로컬 실행을 수행한 것으로 표시하지 않는다.

공유 검증은 원 PR별 결과와 구분했다. 녹색 원 head의 broad default/Native 회귀는 4.3.0 조건을 충족하는 적용 범위에서 재사용하고, 모든 변경을 합친 후보의 필수 검증은 Skia 보정 때문에 한 번 별도로 수행했다. 같은 전체 검사를 9번 반복한 결과로 표시하지 않는다. [실제 명령·종료 코드·로그 요약](assets/dependabot-20261102/README.md).

현재 base로 반영한 #8309의 저장 후 Bottom 정렬은 정식 focused와 독립 Print 전쪽 Native/fresh WASM에서 보존됨을 확인했다. 제품 source·test·golden·baseline·자원 한도 변경은 없다.

## 보안·공급망과 CI

OSV querybatch로 변경·추가된 모든 Cargo 버전을 포함한 15개 정확한 버전을 조회했으며 advisory ID가 반환되지 않았다. 전체 의존성·Skia의 C/C++ 라이브러리에 대한 완전한 보안 감사를 주장하지 않는다.

- [Canvas visual diff](https://github.com/edwardkim/rhwp/actions/runs/41379713424/job/125067660173): SUCCESS
- [Native Skia tests](https://github.com/edwardkim/rhwp/actions/runs/41379713467/job/125067744107): SUCCESS
- [Build & Test](https://github.com/edwardkim/rhwp/actions/runs/41379713467/job/125078335678): SUCCESS
- [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/41383255278): SUCCESS

## 조판 원칙과 시각 판독

실제 영향을 받는 Native/fresh WASM 출력은 [전쪽 TSV·producer·직접 판독](assets/dependabot-20261102/README.md)로 확인했다. 4문서 5쪽의 기준/후보 출력은 각 backend에서 RGBA 픽셀이 모두 같고, raw 최저 Native 94.40%·WASM 98.77%, 90% 미만·누락·2%p 초과 하락은 없다. 이 업데이트가 기존 Print 대비 글꼴/획 차이를 해결했다는 뜻은 아니다.

줄 구성·좌표 생산/소비·pagination 컷 규칙을 바꾸는 구현이 없어 해당 새 규칙 입증은 비해당이다. 실제 소비 backend와 기존 입력·독립 Print 계약은 검증했다. 새 회귀·fixture·절대 좌표 핀 추가나 기존 기대값 변경은 없다. 수식이 없는 입력에는 수식 모양 검사를 수행했다고 쓰지 않는다.

## 원격 처리·후속 계획

승인 뒤 원 PR을 개별 처리한다. source SHA를 재확인하고 최신 devel merge-tree와 해당 head의 실제 required checks를 대조한 뒤 `--match-head-commit`으로 merge한다. 승인되지 않은 workflow 재실행·취소나 보안 metadata 변경은 하지 않는다. merge SHA·CI를 기록하고, 허용된 후속 댓글·archive 기록·기본 devel 동기화와 소유한 worktree/output/debug 정리를 수행한다. 사용자 기존 Vite·Studio 배포 WASM·다른 작업은 보존한다.
