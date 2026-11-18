# PR #9128 리뷰 — 프로세스 Metal context admission

## 최종 판정

**승인 — 원 기여의 로컬 검토 완료.** 이 단계에서 제품 보정이 필요한 결함은 발견하지 않았다. 기존 macOS 기여자 주장 수용 결정의 범위와 실제 실행 한계를 유지한다. 작업지시자의 원 PR 머지·후속 처리 승인 뒤 승인 리뷰와 누적 PR #9132의 원 head merge를 완료했다. 댓글과 종료 정리는 운영 기록 반영 후 아래 순서로 실행한다.

## 접수와 원 기여

- 원 PR: [#9128](https://github.com/edwardkim/rhwp/pull/9128) / `seo-rii` / `devel` / Ready.
- 원 head: `207476213889668e17b2ea6f64eb80c735632f46`. 접수 당시 user/team reviewer가 없어 작업지시자 지시에 따라 `edwardkim`을 지정하고 API로 재확인했다.
- 고정 devel: `443dafb08107152e066689544e88d90988339567`. 원 head의 current-base merge simulation exit 0, 충돌 없음.
- 단계별 비교 시작: `41e8618fdff2a55a16ab60fc8c667f9255692179`. 모든 원 head는 최종 #9132 head `4379aa080b01a5da26f03f154287057ff6fb8269`의 조상이다. 원 SHA·fork branch를 재작성하지 않았다.
- 실제 누적 실행 checkout: `4f6fdb689c2fe45964080074c7a230d2023b4e53`. 원 head+최신 base를 자동 병합한 로컬 검증용 tree이며 제품 보정은 없다.
- 접수 시 원 head check: success 31 / 정상 skipped 5, failure 0 / cancelled 0. 전체 검증 로그·skip 조건과 실행 여부는 구분한다. 최신 head와 required CI는 원격 승인 직전 다시 확인한다.
- 부모 [#536](https://github.com/edwardkim/rhwp/issues/536)·[#8701](https://github.com/edwardkim/rhwp/issues/8701)은 전체 로드맵이므로 OPEN을 유지한다.

## 변경과 실제 소비 검토

Native ABI 생성 예약 → 성공 직후 mark_live → 정상 presenter release_confirmed 또는 영구 quarantine를 추적했다. 한 atomic word에 live+quarantine 수를 저장하며 프로세스 cap 4를 thread 재생성으로 우회하지 못한다. 생성 실패만 미노출 예약을 반환한다.

선행 #8952–#8954 → 수용 #9037 / merge `2c7644c53171e21c27b8a532ce4c53f01c40ddff`의 실제 통합과 메인터너 CMake junction·실행 파일 identity·messenger 종료 보정을 추적했다. 해당 소스를 보존하고 실제 Linux·Windows 실행·빌드·정상 종료로 대조했다. 최신 base의 #9134 의존성 업데이트와 #9136 공통 dialog 보정은 자동 병합됐으며 Native는 이 최신 dependency lock을 사용해 두 OS에서 새로 빌드했다. Studio 파일을 수정·덮어쓰지 않았다.

## 검증 근거

실제 정책 include focused 8개, 동시 생성·panic·abandon·release 경계; Apple SDK/Metal 실기는 미실행·기존 사용자 결정 적용.

[누적 검증 원장](../assets/seo-rii-20261116/README.md)에 실행 source·SDK·Native artifact 해시, 실제 타일 consumer·CPU replay·자원 종료·추가 lifecycle 검사와 실패했던 검증 코드 가정까지 분리해 보존했다. [원 head/지정](../assets/seo-rii-20261116/intake.json), [각 merge simulation](../assets/seo-rii-20261116/simulation.json)도 연결한다. 기여자의 assertion을 메인터너 실기 실행 결과로 옮겨 적지 않았다.

이 단계는 문서 parser/IR·조판·PageLayerTree/PaintOp·HWP/HWPX 저장·WASM Canvas producer를 바꾸지 않는 opt-in Native GPU 자원/소비·호스트 경로다. 새로운 실물 문서 fixture·golden·절대 좌표 회귀를 추가하지 않았다. 실물 문서의 한컴 일치/전쪽 Native·fresh WASM 재측정 또는 편람 조판 개선을 주장하지 않는다. 실제 consumer에서 같은 불변 snapshot의 독립 CPU replay와 native seam/transform을 대조한 결과로 판정한다.

macOS는 환경 부재로 기여자의 주장을 수용한 기존 작업지시자 결정을 적용한다. Apple SDK compile/link·Metal 실기는 실행하지 않았고 portable 정책 결과로 대신하지 않는다. 소프트웨어 GL·실제 Windows API 결과 역시 물리 GPU 가속·consumer fence·VRAM/RSS·FPS 개선의 증거가 아니다.

## 원격 후속 조건

최종 [누적 검토](seo_rii_20261116_review.md)와 [실제 원격 처리안](../assets/seo-rii-20261116/remote-action-plan.md)에 따라 작업지시자의 승인을 요청한다. 최신 원 head/required checks·mergeability와 현재 base 병합을 다시 확인한다. 원 기여 보존 merge 뒤 각 원 PR의 MERGED 상태·merge SHA를 개별 조회하고 감사 댓글에 실제 검증과 한계를 기재한다. 부모 이슈는 닫지 않는다. 운영 기록·동기화·종료 정리는 canonical 후속 순서를 따른다.

## 원 head CI 재사용과 최종 판독

메인터너 source/test/fixture/workflow 보정이 없으므로 원 head 자체가 각 PR의 code candidate다. `pr_review_workflow.md` 3.2.2에 따라 원 code head의 녹색 Full CI를 재사용한다. 로컬 current-base merge는 제출 head가 아니라 최신 base 기능과의 공존을 실행한 검증용 tree다. 실제 remote 대상 #9132는 모든 앞 head를 그대로 포함하며, 새 통합 PR이나 fork push가 필요하지 않다. 원 PR의 exact head와 최신 devel을 병합한 tree가 실제 실행 tree와 같음을 재확인했다.

로컬에서는 최소 글꼴 격리의 Rust focused 21개, fmt·Native/WASM Clippy·suite/unit 정책과 Flutter Native 연결 360개를 추가 실행했다. 전체 Rust/Native 광범위 회귀는 GitHub 결과 재사용이며 로컬 PASS라고 보고하지 않는다. 시작했던 선택적 중복 workspace 빌드는 중지했고 그 결과도 보존했다. 실제 PNG의 색상·타일 경계·resize/복구 후 내용과 공개 front 보존을 직접 확인했다. 독립 한컴 조판 일치 수치나 새 물리 GPU 성능을 추정하지 않았다.

## 원격 통합 확정 — 2026-11-16 KST

- [승인 리뷰](https://github.com/edwardkim/rhwp/pull/9128#pullrequestreview-5919917318)는 원 head `207476213889668e17b2ea6f64eb80c735632f46`에 제출했고 API로 APPROVED·본문·commit을 재확인했다.
- [누적 PR #9132](https://github.com/edwardkim/rhwp/pull/9132)의 실제 merge commit: `97575816999577744f0e2e7f2e16a41d5961bebf`. 실제 tree `cb9c303399b1769d137751da9fe5c1ccdd8c1842`는 최종 검증 tree와 같다.
- 이 원 PR의 GitHub 상태는 **MERGED**다. 원 API의 `merge_commit_sha`는 `92268674482f78cdb0e0b91bb0bd23c7fd548ea4`이며 간접 수용 PR의 이 값과 devel 통합 commit을 구분한다. 원 head의 devel 포함을 별도로 확인했다. contributor fork/source branch는 보존했다.
- [원격 영수증](../assets/seo-rii-20261116/postmerge-verification.json)·[마지막 CI와 병합 gate](../assets/seo-rii-20261116/exact-merge-gate.json). 부모 #536·#8701은 OPEN 유지다.

## Merge 후 contributor PR comment 계획

운영 기록을 devel에 반영하고 동기화한 뒤, 실제 code merge `97575816999577744f0e2e7f2e16a41d5961bebf`·승인 리뷰·CI 링크·원 PR 범위의 검증 요약을 한국어 존댓말 감사 댓글로 게시한다. 본문은 UTF-8/LF 파일로 전달하고 API로 본문·줄바꿈·BOM·한글을 재확인한다. [확정 문안](../assets/seo-rii-20261116/merge-comment-drafts.md)을 사용하며 부모 이슈를 닫지 않는다.

이 단계의 주장은 자원 admission/polling/진단 계약이며 한컴 문서 시각 일치를 주장하지 않는다. 관련 없는 화면을 이 단계만의 시각 증거로 삽입하지 않으며 실제 소비 검증과 플랫폼 한계는 원장으로 연결한다.
