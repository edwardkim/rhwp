# seo-rii P120–P130 PR 11건 누적 검토

**승인 — 11건의 로컬 검토 완료.** reviewer가 없던 Ready 11건에 `edwardkim`을 지정했고, 원 head를 보존해 최신 devel과 clean merge한 소스로 검증했다. 각 단계의 기능 판정은 아래 원 PR별 문서로 나누어 기록한다.

| 원 PR | 단계·범위 | 검토 |
| --- | --- | --- |
| #8962 | Linux 참조 앱 resize | [원 PR 검토](pr_8962_review.md) |
| #8963 | producer-ready EventChannel | [원 PR 검토](pr_8963_review.md) |
| #8964 | 창 종료와 GPU retirement | [원 PR 검토](pr_8964_review.md) |
| #8965 | 공통 GPU 타일 정책과 Metal cache | [원 PR 검토](pr_8965_review.md) |
| #8966 | D3D bounded tile cache | [원 PR 검토](pr_8966_review.md) |
| #8967 | 완료된 D3D private target 재사용 | [원 PR 검토](pr_8967_review.md) |
| #9128 | 프로세스 Metal context admission | [원 PR 검토](pr_9128_review.md) |
| #9129 | 유휴 상태 polling 억제 | [원 PR 검토](pr_9129_review.md) |
| #9130 | owner thread cache trim | [원 PR 검토](pr_9130_review.md) |
| #9131 | 메모리 압력과 typed trim receipt | [원 PR 검토](pr_9131_review.md) |
| #9132 | 게시 세대의 cache metrics와 JSON CLI | [원 PR 검토](pr_9132_review.md) |

[접수·계획](seo_rii_20261116_review_plan.md) · [검증 원장](../assets/seo-rii-20261116/README.md).

원 최종 head `4379aa080b01a5da26f03f154287057ff6fb8269`, 기준 devel `443dafb08107152e066689544e88d90988339567`, 실제 검증 checkout `4f6fdb689c2fe45964080074c7a230d2023b4e53`를 구분한다. 원 head 11개가 누적 스택에 그대로 들어 있고 현재까지 제품 source 보정은 없다. 검토 접수 시에는 reviewer 지정만 했으며, 이후 작업지시자 승인에 따라 아래 원격 통합을 완료했다.

## 완료한 검토와 원격 제안

Flutter Native 연결 360개·0 SKIP, Rust focused 21개, fmt·Native/WASM Clippy·suite/unit 정책과 실제 Linux GL·Windows D3D consumer·참조앱·cache trim·memory pressure·metrics 전달·정상 종료를 확인했다. Native 라이브러리는 최신 devel의 dependency lock을 반영한 자동 병합 source에서 두 OS별로 새로 빌드했다. 실제 CPU 기준과의 consumer 비교를 독립 한컴 Print 일치로 바꾸어 보고하지 않는다. 광범위 전체 Rust/Native CI는 정본 3.2.2의 **변경하지 않은 원 code head** 재사용 조건을 적용했다. 중지한 선택적 중복 workspace 빌드를 PASS로 세지 않았다.

모든 원 head가 최종 #9132의 조상이므로, 제품 보정이나 새 PR 없이 **각 원 PR 승인 리뷰 → #9132의 exact-head merge commit → 원 PR 11건의 MERGED/기여 포함 개별 확인**으로 처리할 수 있다. 그 뒤 maintainer 직접 반영 범위의 mydocs 운영 기록을 devel에 push·동기화하고 감사 댓글·종료 정리를 수행한다. [정확한 원격 처리안](../assets/seo-rii-20261116/remote-action-plan.md)과 [댓글 초안](../assets/seo-rii-20261116/merge-comment-drafts.md)을 준비했다. 원 PR 승인 리뷰·머지·후속 처리까지 작업지시자 승인을 받았고 아래 통합을 완료했다.

#536·#8701은 전체 로드맵이므로 OPEN을 유지한다. macOS는 환경 제공 불가로 기여자 주장을 수용한 기존 사용자 결정을 적용하고 Apple SDK·Metal 실기 미실행을 명시한다. 물리 가속·consumer GPU fence·driver VRAM/RSS·물리 FPS 개선·모든 문서의 fidelity 완료를 주장하지 않는다.

## 통합 확정 — 2026-11-16 KST

원 PR 11건의 exact-head APPROVED review를 각각 게시한 뒤 #9132를 merge commit `97575816999577744f0e2e7f2e16a41d5961bebf`로 통합했다. **원 PR 11건 모두 MERGED**, 원 head 11개 전부 devel 포함을 확인했다. 원 fork branch는 수정·삭제하지 않았다. 실제 merge tree는 검증한 tree와 같고 보호 설정·CI 기준 변경이나 우회 merge는 하지 않았다. [승인 리뷰](../assets/seo-rii-20261116/approved-reviews.json) · [통합 영수증](../assets/seo-rii-20261116/postmerge-verification.json).

후속 기록은 post_merge 7.1의 **maintainer 직접 반영**이다. source/test/workflow/새 LFS를 추가하지 않고 이 archive·mydocs/pr/assets·오늘할일만 devel에 반영한다. 그 뒤 devel 동기화 → 부모 OPEN 확인 → 원 PR별 감사 댓글 → 이번 작업의 branch/worktree·Windows 임시 checkout/runner·루트 output 전체·종료한 debug 정리를 수행한다. 상세 최종 실행은 ignored `local/pr-review-cleanup/seo-rii-20261116.json`에 삭제 경로·전후 공간·최종 확인 값으로 남긴다.

병합 후 자동 [duration 메타데이터 갱신](https://github.com/edwardkim/rhwp/actions/runs/43866524638)은 **success**다. 실제 step의 collect·prepare·publish 성공을 확인했다. [영수증](../assets/seo-rii-20261116/duration-refresh-result.json)을 보존했다. 후속 제품 CI·CodeQL·Render Diff·Adapter·Proptest·Oracle 재실행은 하지 않았다.
