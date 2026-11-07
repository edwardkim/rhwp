# seo-rii PR 29건 검토·통합 수행 계획

## 현재 범위와 권한

2026-11-07 작업지시자 지시: seo-rii 기여자의 PR을 묶어서 처리한다. 기본 경로는 `maintainer_general`, 보조 경로는 `intake_and_review`, `local_validation`, `multi_pr_update_branch`, `visual_fixture_evidence`, `rework_and_exceptions`, `post_merge`다. reviewer는 원 PR마다 edwardkim으로 지정·API 확인했다. 이 지시와 reviewer 배정은 공개 댓글·원격 push·새 PR·merge·close 또는 workflow 재실행 승인이 아니다.

## 재현 가능한 검토 기준

- 기본 작업공간은 `/home/edward/mygithub/rhwp`이며 devel `b8dd67c5001dfb0410e9e192528676396cdd6a4e`와 원격이 같고 미커밋 변경이 없다. 기존 pr-8296 worktree와 사용자 Vite는 보존한다.
- 검토 공간: `/home/edward/mygithub/rhwp-worktrees/review-integration-536-seo-rii-native-hosts`, branch `review/integration-536-seo-rii-native-hosts`. 기본 작업공간의 branch·배포용 WASM을 바꾸지 않는다.
- source 마지막 head `629f204f82e95c4f2f1a12d318698d1a01fe1cf7`는 다른 28개 source head를 모두 조상으로 보존한다. 최신 devel과 29개 원 head의 merge-tree는 전부 clean이다. 최종 누적 diff는 177개 파일·38,403줄 추가·1,038줄 삭제이며 대형 PR 경로를 적용한다.
- 열린 PR 30개 중 Draft #8646은 정식 처리 대상에서 제외했다. 그러나 P73 source `cb59b901caf1e5a3c70c385f8b711424cbaaabc8`·`8d12fe501dc7f5d04da5d82e71e0439730f930b9`가 Ready 29개 모두에 포함돼 선행 코드의 검토가 필요하다. 원 history를 그대로 통합하면 Draft도 간접 MERGED로 바뀔 가능성이 있으므로 **원격 수용 전에 이 효과를 명시해 범위를 확정한다.** 검토 중 Draft 상태·head를 바꾸지 않는다.
- 기여자 본문의 과거 사용자 지시·후속 검증 경계는 출처 설명이며, 현재 메인테이너 검증 면제나 원격 승인으로 전용하지 않는다.
- 사전 목록 조회에서 상세 GraphQL 필드가 reviewer 지정 전 함께 조회된 순서 오류가 있었다. 이후 원 PR별 reviewer 배정과 API 확인을 완료한 뒤에 source fetch·merge simulation을 시작했고 head 29개가 접수 SHA와 같음을 확인했다. source 변경이나 공개 댓글은 없었다.

## 대상과 단계별 변화

| 원 PR | 단계 | 고정 source head | 직전 단계 대비 경로 수 | 접수 CI |
| --- | --- | --- | --- | --- |
| [8666](https://github.com/edwardkim/rhwp/pull/8666) | P74 | `67d46ba6b5a1` | 13 | 시간 초과 |
| [8667](https://github.com/edwardkim/rhwp/pull/8667) | P75 | `ca1f63c15740` | 5 | 성공 |
| [8668](https://github.com/edwardkim/rhwp/pull/8668) | P76 | `8a679ed90b1f` | 8 | 성공 |
| [8669](https://github.com/edwardkim/rhwp/pull/8669) | P77 | `fd2df4070400` | 9 | 성공 |
| [8670](https://github.com/edwardkim/rhwp/pull/8670) | P78 | `5847f6f25627` | 8 | 성공 |
| [8671](https://github.com/edwardkim/rhwp/pull/8671) | P79 | `a4e839d1c504` | 4 | 시간 초과 |
| [8672](https://github.com/edwardkim/rhwp/pull/8672) | P80 | `2fb8d6f0a70a` | 3 | 성공 |
| [8673](https://github.com/edwardkim/rhwp/pull/8673) | P81 | `47bdb1caee3f` | 4 | 성공 |
| [8674](https://github.com/edwardkim/rhwp/pull/8674) | P82 | `0d70e975806e` | 8 | 성공 |
| [8675](https://github.com/edwardkim/rhwp/pull/8675) | P83 | `a0b8824f5fb5` | 6 | 성공 |
| [8702](https://github.com/edwardkim/rhwp/pull/8702) | P84 | `c22449e8b592` | 912 | 성공 |
| [8703](https://github.com/edwardkim/rhwp/pull/8703) | P85 | `03a235362d2c` | 7 | 성공 |
| [8704](https://github.com/edwardkim/rhwp/pull/8704) | P86 | `7dadd5afd6c8` | 6 | 성공 |
| [8705](https://github.com/edwardkim/rhwp/pull/8705) | P87 | `fc27113c3e95` | 4 | 성공 |
| [8706](https://github.com/edwardkim/rhwp/pull/8706) | P88 | `16a15d2aa19b` | 5 | 성공 |
| [8707](https://github.com/edwardkim/rhwp/pull/8707) | P89 | `48382f79cdf2` | 17 | 성공 |
| [8712](https://github.com/edwardkim/rhwp/pull/8712) | P90 | `101899925c90` | 20 | 성공 |
| [8713](https://github.com/edwardkim/rhwp/pull/8713) | P91 | `f4ded9d0d358` | 7 | 성공 |
| [8714](https://github.com/edwardkim/rhwp/pull/8714) | P92 | `91cb75e870e7` | 9 | 성공 |
| [8716](https://github.com/edwardkim/rhwp/pull/8716) | P93 | `18149728631d` | 21 | 성공 |
| [8718](https://github.com/edwardkim/rhwp/pull/8718) | P94 | `8b69fde69046` | 3 | 성공 |
| [8720](https://github.com/edwardkim/rhwp/pull/8720) | P95 | `21fac65baabd` | 17 | 성공 |
| [8721](https://github.com/edwardkim/rhwp/pull/8721) | P96 | `4ac991f97bce` | 7 | 성공 |
| [8722](https://github.com/edwardkim/rhwp/pull/8722) | P97 | `6098e175eb8f` | 21 | 성공 |
| [8723](https://github.com/edwardkim/rhwp/pull/8723) | P98 | `0f629e936bd6` | 9 | 성공 |
| [8724](https://github.com/edwardkim/rhwp/pull/8724) | P99 | `9d8b56ecef4d` | 11 | 성공 |
| [8726](https://github.com/edwardkim/rhwp/pull/8726) | P100 | `d9570e04cc5e` | 16 | 성공 |
| [8727](https://github.com/edwardkim/rhwp/pull/8727) | P101 | `ac393935254c` | 24 | 성공 |
| [8728](https://github.com/edwardkim/rhwp/pull/8728) | P102 | `629f204f82e9` | 29 | 성공 |

정확한 원 head·선행 head·단계 commit·변경 경로·리뷰어 확인 시각·CI 스냅샷은 [접수 원장](../assets/seo-rii-20261107/intake.json)을 따른다. 원 PR별 review 파일에서 결과·처분을 분리하며 하나의 통합 승인으로 모든 원 PR을 자동 승인하지 않는다.

## 독립 조사에서 확인한 문제와 결정 경계

1. #8666 Native job은 기존 30분, #8671 Native job은 30분·Lint job은 45분 상한으로 취소됐다. 실제 job/step·로그를 확인했다. compiler/test failure로 단정하지 않으며 성공한 후속 prefix CI와 구분한다. 원격 재실행은 하지 않았다. 새 통합 후보에서는 공통 최신 base·실제 검증을 먼저 수행하고 원격 처리 승인 시 CI 범위를 확정한다.
2. 기여자가 완료했다고 밝힌 계약 검사와 실제 한컴 Native/fresh WASM 전쪽·최종 host sampling 검증은 다르다. 렌더링 변경의 독립 Print·전체 TSV·직접 판독·입력 commit을 메인테이너가 확인하기 전 수용하지 않는다. 기존 공개 입력·기준 PDF를 우선 재사용한다.
3. #8726(P100)·#8727(P101)은 제목과 본문에서 bounded foundation/WIP이며 **macOS Metal·Windows D3D SDK/backend compile·link 미실행**을 명시한다. Linux의 disabled ABI와 portable 수명 검사로 대상 플랫폼 빌드 성공을 대신하지 않는다. 최대 4 frame·64MiB와 consumer 완료 계약 부재, `continuousPresentationSupported:false`를 실제 코드와 대조한다. 대상 플랫폼 증거 또는 메인테이너의 명시적인 제한 수용 결정 전 최종 수용을 판정하지 않는다.
4. #8728은 P100/P101을 포함한 누적 head다. 두 플랫폼 코드를 보류할 경우 전체 뒤쪽 변경을 누락시키지 않고 실제 의존성에 따라 Linux/공통 기능의 분리 가능성을 조사한다. 공개 선언을 조용히 제거하거나 미검증을 완료로 바꾸지 않는다.

## 수행 순서와 완료 조건

### A. 접수·원인·계획

원 PR 29개의 reviewer 지정·source 고정·현재 base merge-tree·최소 CI 로그 조사를 완료했다. [#536](https://github.com/edwardkim/rhwp/issues/536)의 P69–P72 통합·P73–P83, [#8701](https://github.com/edwardkim/rhwp/issues/8701)의 P84–P102 계약·논의와 원 해결 PR·실제 통합 SHA·독립 증거를 대조한다. 이 단계 문서를 로컬 commit한 뒤 source 통합·검증 단계로 이동한다.

### B. 원 source 보존과 작은 경계 검증

기존 29개 head가 최종 tip의 조상이므로 같은 검토 branch에서 clean merge로 원 commit과 base를 보존한다. 중복 체리픽·원 fork push·기여자 history 재작성은 하지 않는다. P73–P83의 타일 cache/quota/cancel, glyph 진단·수식 fallback·radial PDF·culling·upright의 실제 경계와 정상 대조를 먼저 실행한다. 확정 결함이나 부족한 시각 증거의 메인테이너 보정은 범위·독립 근거·음성 대조를 구체화하고 필요한 작업지시자 결정을 따른다.

### C. Native·Qt·Flutter 실제 소비 검증

P84–P98의 문서 교체·저장 손실 승인·외부 변경 충돌·snapshot/lease·owner thread·source/device 세대·GL 복구·IME·접근성·자동 저장을 실제 공개 ABI와 예제로 실행한다. 대역과 실제 Native 실행을 구분한다. P99–P102는 platform texture 등록/가림/새 revision/close와 consumer 소유권·credit을 검증한다. CPU fallback, 잘못된 context/세대·오류 후 정상 복구·자원 통계 0과 GL 상태 복원을 확인한다. 지원 SDK·runtime 존재 여부를 먼저 확인하고 비활성 stub 통과를 실물 지원으로 보고하지 않는다.

### D. 필수 공통 검증·증거

[로컬 사전 검증](../../manual/pr_review/local_validation.md) 4.3의 Rust lint·focused·전체 nextest·Native 공식 3종, 필요한 Native feature·FFI·C/C++·Qt·Flutter 검사, host 최적화 fresh WASM·실제 브라우저를 순차 실행한다. 고정 공용 target은 `/home/edward/mygithub/rhwp/target/pr-review`다. 같은 cache에서 Cargo 명령을 겹치지 않는다. 새 통합 source/보정이 생기면 과거 source CI로 필수 검증을 대체하지 않는다.

렌더/조판 영향 사례는 같은 독립 한컴 Print와 Native/fresh WASM 전쪽 TSV·직접 확대 판독, 최저 90%·base 대비 2%p 초과 하락 0·쪽수/내용 보존을 확인한다. 완료 쪽의 고정 좌표는 98% 조건을 지킨다. 최종 조판 영향 후보는 편람 Native 383쪽 `--handbook-gate`를 실행한다. 최신 base에서 보존할 최근 그림 편집·#8711 선행 정의 저장 재열기의 실제 동작·출력도 함께 확인한다.

### E. 원격 판단과 후속

원 PR마다 검증한 source/통합 head와 판정을 갱신하고 공통 증거는 원장으로 공유한다. 승인 가능한 범위·보정/미완료 범위·Draft P73 간접 수용 효과를 명시한 구체적인 원격 처리안과 댓글을 준비한 뒤 별도 승인을 요청한다. 승인 뒤에만 push·Open PR·새 head CI·self-review·merge·원 PR/관련 이슈 후속·기록/devel 동기화·종료 정리를 수행한다. #536·#8701은 추적 이슈의 전체 남은 범위까지 확인하고 임의 close하지 않는다. 최종 종료 시 전용 branch/worktree/ref·루트 output 전체·공유 debug를 정본에 따라 정리한다.

## 2026-11-07 플랫폼 검증 범위 결정

작업지시자는 Windows 11 호스트 자원의 사용을 지시했고 실제 MSVC·Windows SDK 검증을 진행했다. #8726은 추가 답변 “Metal 실험 기반에 한정해 예외 수용”에 따라 default-disabled 실험 기반으로만 제한 수용한다. macOS SDK·실물 Metal 검증 미실행과 연속 표시 차단을 숨기지 않으며, portable/비활성 ABI·공통 필수 검증은 계속한다. [결정 원장](../assets/seo-rii-20261107/metal-maintainer-decision.json)을 따른다. 이는 원격 처리 승인이 아니다.

## 최종 로컬 검증 완료

제품 보정 `cda7c8cb33`·회귀 source `263b1e753c`에서 [최종 17개 묶음](../assets/seo-rii-20261107/local-gates.json)을 통과했다. 전체 nextest 12,825건, Native 집중 194건, Native 공식 3종 4,077건, Studio 2,745건이다. 신규 실제 producer 숫자 검사의 수정 전 실패·수정 후 통과와 실제 Windows SDK/D3D·Linux Qt/Flutter·독립 Vulkan 실행을 완료했다. 플랫폼의 미검증 범위는 [실행·재사용 범위](../assets/seo-rii-20261107/validation-scope.md)에 명시했다.

Native/fresh WASM 각 전 24쪽이 raw 90% 이상이고 base/후보 PNG 48쌍이 같다. 편람 Native 383쪽은 최저 99.96%·기준 대비 회귀 0쪽이다. 실제 Direct PDF 6쪽은 raw 100%다. [전쪽 TSV·대표 이미지](../assets/seo-rii-20261107/visual/README.md)와 독립 확대 판독을 보존했다.

검증 종료 후 11개 테스트의 첫머리 module documentation만 보완했다. 실제 Git source와 [본문 hash](../assets/seo-rii-20261107/test-header-metadata-receipt.json)를 대조해 프로그램·assertion·fixture의 동일성을 확인했고, fmt·현재 자동 수집 48/48·변경 Markdown 58개 링크와 metadata를 통과했다. 이 주석 보완을 새 기능 실행으로 표시하지 않는다.

Ready 원 PR별 판정은 메인터너 보정 후 수용 가능으로 정리했다. [원격 처리안](../assets/seo-rii-20261107/remote-action-plan.md)의 기존 대조군 차이에 한정한 예외·Draft #8646 간접 수용 효과·원격 범위는 아직 승인 전이다. 별도 사용자 Draft #8738에는 실패 check가 없어 변경하지 않았다. 원격 조치는 별도 승인 뒤 진행한다.

## 원격 수용·merge 확정

[통합 PR #8739](https://github.com/edwardkim/rhwp/pull/8739)의 head `0d2548fd88ea76b10e53eec841376b4bcbecbc4a`에서 필수 CI·self-review를 완료했고, 원 기여 이력을 보존하는 merge `f5f0fbee95b5fb0e32e99f8f639d228ae55389dd`로 수용했다. 작업지시자는 기존 eq-01 본문 굵기·수식 간격/분수 모양 및 picture-crop 가는 테두리 차이에 한정한 예외와 Draft #8646의 간접 MERGED 효과를 승인했다. 새 숫자 오류·회귀·필수 검사 실패는 예외에 포함하지 않았다. #536·#8701은 OPEN을 유지하며 #8337 완료·원 PR별 후속 댓글·정리는 정본 순서에 따라 이어 간다.

[최종 CI](../assets/seo-rii-20261107/remote/ci-success.json) · [self-review](../assets/seo-rii-20261107/remote/self-review.json).
