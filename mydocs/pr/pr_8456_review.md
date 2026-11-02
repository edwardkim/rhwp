# PR #8456 리뷰 — skia-safe/skia-bindings 0.153.3 → 0.156.0

## 최종 판정

**메인터너 보정 후 수용 가능 — Skia 의존성 호환성의 로컬 검증 완료.**

이는 로컬 review 판정이며 GitHub approve·push·comment·merge 승인을 대신하지 않는다. 다음 조건은 작업지시자의 원격 처리 승인, merge 직전 최신 head·CI·mergeability 재확인이다.

## 접수 정보

- PR: [#8456](https://github.com/edwardkim/rhwp/pull/8456), 작성자 `dependabot[bot]`, base `devel`, Open·non-draft.
- 원 head: `54d0255c594241d1593adff0367a236a9b6f3ebc`. reviewer `edwardkim` 지정과 API 재확인을 완료했다.
- 기준 devel: `d3f7209bbb0fdb1711c5f40a70ad22f0ca0be66d`. 누적 검증 후보: `e58024b15e11438a0e1162a25d836d3c3c4ff7a9`.
- 기본 경로 `maintainer_general`; 보조 `intake_and_review`, `local_validation`, `multi_pr_update_branch`, `rework_and_exceptions` 및 해당 renderer 증적 절차.
- 충돌 simulation: 독립·누적 모두 clean. 작성 시 mergeable `MERGEABLE`, merge state `BLOCKED`는 참고값이며 승인 뒤 다시 조회한다.

## 변경과 검증

Native Skia의 글꼴·그림·PNG/PDF 소비 경로를 검토했다. 원 CI의 두 실패는 bindgen 0.73.2가 syn 3을 고른 상태에서 prettyplease 0.2.37이 syn 2의 File을 요구한 E0277이다. Canvas 비교도 native CLI build에서 멈췄으며 실제 Canvas 차이 실패가 아니다.

prettyplease 0.3.0·syn 3으로 정렬한 lockfile 보정 뒤 실제 Native build, 모든 적용 lint, 최소 글꼴 전체 회귀·Native 검사·fresh optimized WASM, 실제 Studio 브라우저와 전쪽 시각 검증을 완료했다.

공유 검증은 원 PR별 결과와 구분했다. 녹색 원 head의 broad default/Native 회귀는 4.3.0 조건을 충족하는 적용 범위에서 재사용하고, 모든 변경을 합친 후보의 필수 검증은 Skia 보정 때문에 한 번 별도로 수행했다. 같은 전체 검사를 9번 반복한 결과로 표시하지 않는다. [실제 명령·종료 코드·로그 요약](assets/dependabot-20261102/README.md).

현재 base로 반영한 #8309의 저장 후 Bottom 정렬은 정식 focused와 독립 Print 전쪽 Native/fresh WASM에서 보존됨을 확인했다. 제품 source·test·golden·baseline·자원 한도 변경은 없다.

## 보안·공급망과 CI

OSV querybatch로 변경·추가된 모든 Cargo 버전을 포함한 15개 정확한 버전을 조회했으며 advisory ID가 반환되지 않았다. 전체 의존성·Skia의 C/C++ 라이브러리에 대한 완전한 보안 감사를 주장하지 않는다.

- [Canvas visual diff](https://github.com/edwardkim/rhwp/actions/runs/41379744686/job/125069110998): FAILURE
- [Native Skia tests](https://github.com/edwardkim/rhwp/actions/runs/41379744712/job/125070083215): FAILURE
- [Build & Test](https://github.com/edwardkim/rhwp/actions/runs/41379744712/job/125081496872): FAILURE
- [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/41384282620): FAILURE

## 조판 원칙과 시각 판독

실제 영향을 받는 Native/fresh WASM 출력은 [전쪽 TSV·producer·직접 판독](assets/dependabot-20261102/README.md)로 확인했다. 4문서 5쪽의 기준/후보 출력은 각 backend에서 RGBA 픽셀이 모두 같고, raw 최저 Native 94.40%·WASM 98.77%, 90% 미만·누락·2%p 초과 하락은 없다. 이 업데이트가 기존 Print 대비 글꼴/획 차이를 해결했다는 뜻은 아니다.

줄 구성·좌표 생산/소비·pagination 컷 규칙을 바꾸는 구현이 없어 해당 새 규칙 입증은 비해당이다. 실제 소비 backend와 기존 입력·독립 Print 계약은 검증했다. 새 회귀·fixture·절대 좌표 핀 추가나 기존 기대값 변경은 없다. 수식이 없는 입력에는 수식 모양 검사를 수행했다고 쓰지 않는다.

## 메인터너 보정과 제출 순서

원 head는 그대로는 머지하지 않는다. `8777e11c165636b19fe4f5db10eb0a7ee7ed06cf`에서 Cargo.lock의 prettyplease 버전·checksum·syn 연결 3줄만 보정했다. bindgen의 허용 범위 `>=0.2.7,<0.4` 안에서 syn 3 타입을 공유한다. Skia의 MSRV 1.87은 프로젝트 1.93.1 이내이며, 이전·새 GitHub release 모두 prerelease 표시를 사용한다.

제출 source branch는 원 bot commit·현재 base와 누적 검증 graph를 보존한다. 8개 변경 없는 원 PR을 먼저 merge한 뒤 실제 최신 devel과 정렬하며, 최종 runtime·Cargo.lock이 검증한 `e58024b15e11438a0e1162a25d836d3c3c4ff7a9`와 같은지 확인한다. 그 후 승인된 한 번의 push와 새 exact-head CI·self-review를 거쳐 마지막으로 원 #8456을 merge한다. 지금 원격은 아직 원 head이고 새 head CI는 미실행이다.

- [bindgen manifest](https://github.com/rust-lang/rust-bindgen/blob/v0.73.2/bindgen/Cargo.toml)
- [prettyplease 0.3.0](https://docs.rs/crate/prettyplease/0.3.0)
- [Skia release](https://github.com/rust-skia/rust-skia/releases/tag/0.156.0)

## 원격 처리·후속 계획

승인 뒤 원 PR을 개별 처리한다. source SHA를 재확인하고 최신 devel merge-tree와 해당 head의 실제 required checks를 대조한 뒤 `--match-head-commit`으로 merge한다. 승인되지 않은 workflow 재실행·취소나 보안 metadata 변경은 하지 않는다. merge SHA·CI를 기록하고, 허용된 후속 댓글·archive 기록·기본 devel 동기화와 소유한 worktree/output/debug 정리를 수행한다. 사용자 기존 Vite·Studio 배포 WASM·다른 작업은 보존한다.
