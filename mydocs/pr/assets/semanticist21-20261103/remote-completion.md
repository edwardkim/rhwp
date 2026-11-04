# semanticist21 열 건 통합 — 확정된 원격 결과

## 통합과 판정

- 통합 PR: [#8548](https://github.com/edwardkim/rhwp/pull/8548). 제출·검토 head `ad4232749d450c109c33b5b0ffc570aee652b66c`, 제품 보정 source `5e374d01f641745d20bc9424ce233021a327ed16`.
- merge commit: [`28b58695cb01a20e980459e77a35f964817c7459`](https://github.com/edwardkim/rhwp/commit/28b58695cb01a20e980459e77a35f964817c7459), 병합 시각 `2026-11-04T11:02:24Z`. squash하지 않고 열 개 원 head와 기여 commit을 보존했다.
- [self-review](https://github.com/edwardkim/rhwp/pull/8548#pullrequestreview-5767237915)는 위 제출 commit을 지정한 COMMENT review다. 자기 PR에 APPROVE를 게시한 것으로 세지 않는다.
- Native/fresh WASM 각각 32개 입력·72쪽의 최저 raw는 92.89641%·92.88059%이며 쪽수 불일치·누락·90% 미만은 없다. 직접 확대 판독·수정 전후·실제 API·저장 재열기를 함께 확인했다. Native 편람 383쪽 최저는 99.96465%, 기준 대비 하락은 0쪽이다.
- 시각 producer와 source 수정 시점을 [최신 검증](rebase-validation.md), [CI 보정](ci-head223-followup.md), [base 전진 호환성](base-advance-compatibility.md)에 구분했다. 진단 출력 보정 뒤 새 raster를 실행했다고 보고하지 않는다.

## 실제 CI

- [CI](https://github.com/edwardkim/rhwp/actions/runs/41811036602): 최신 head의 Lint·전체 Archive A–E·Native Skia·Frontend와 최종 Build & Test 성공. Lint 첫 실행은 runner ENOSPC였고 사용자 승인으로 해당 작업과 종속 집계만 1회 재실행했다. 성공한 다른 작업은 재실행하지 않았다.
- [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/41811036496): 네 언어와 GHAS check 성공. [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/41811036427), [Adapter](https://github.com/edwardkim/rhwp/actions/runs/41811036938), [Proptest](https://github.com/edwardkim/rhwp/actions/runs/41811036943), [Skill router](https://github.com/edwardkim/rhwp/actions/runs/41811036454)도 성공했다.
- 최종 CI Impact Policy status의 성공을 확인했다. 최초 controller 발행 성공이나 pending을 최종 감사 성공으로 대신하지 않았다.
- Rust 분석 `2093550961`의 결과는 129→126개, PR merge-ref open alert는 15→12개다. #493·#494·#496의 PR instance가 fixed이며 새 alert ID는 없다. 기존 12개를 이번 PR에서 모두 해결한 것으로 보고하지 않고 query·분류·보안 설정은 바꾸지 않았다.
- 실제 GitHub tested merge `1dfe95166431dac7fd944934024341fc2cad3e3f`는 CI 시점 devel `96827b3accd98ad0b80f76260b42fba24eded504`와 위 제출 head를 부모로 포함했다. PR API의 이전 base 표시와 실제 merge ref를 구분했다.

## 로컬 검증과 한계

보정 source에서 stock nextest 12,475 PASS / 45 skip, CI의 Native 22개 명령 192 PASS, 별도 Native 전체 lib 4,040 PASS / 12 ignored, Native 출처 계약 11 PASS, fmt·root/workspace/WASM Clippy·workspace build를 완료했다. Studio 2,717 PASS / 2 skip과 운영 계약 275 PASS는 앞선 동일 기능 producer의 실행으로 구분한다. 전체 시각·실제 API·최소 글꼴 환경·해시·저장 결과는 [기존 정식 증적](rebase-validation.md)에 연결한다.

기존 암호 fixture는 새 세 CodeQL sink를 수정 전에도 실행하지 않았다. 따라서 새 세 sink의 runtime before-FAIL 증거로 주장하지 않으며 CodeQL의 독립 before/after를 사용한다. 앞선 H4 진단 채널의 실제 CLI 음성/양성 대조는 별개다.

## 병합 후 duration 게시

[duration 갱신 41818191710](https://github.com/edwardkim/rhwp/actions/runs/41818191710)은 성공했다. 최종 승인/측정 run `41811036602` attempt 2와 같은 head를 검증했고 B/C/D/E의 실제 측정 attempt는 모두 1이다. artifact `13314100219·13313531412·13312998571·13313656810`과 tested merge `1dfe95166431dac7fd944934024341fc2cad3e3f`를 수집했다. guarded Git Data API가 `published:true`, commit `a02867ac45235a0767878a7669c455ab505c3e7a`, blob `fc91d880a617912e888df81e45ef01ab3d04f213`으로 게시한 것을 확인했다. 이 메타데이터 작업은 병합 후 제품 검증 CI 실행이 아니다.

## 후속 처리 순서

CI 변경 PR의 옵션 2에 따라 이 별도 records-only PR에서 개별 검토 10개를 archive로 이동하고 오늘할일을 반영한다. 코드·test·workflow·기존 시각 asset을 다시 수정하지 않는다. 실제 review-only preflight·required check와 mergeability를 확인한 뒤 기록 PR을 병합한다. 그다음 devel 동기화, #8223 완료 확인, 원 PR 10건의 수용 완료·감사 댓글과 이미지 API/화면 검증, 소유 branch/worktree·output/debug 정리를 진행한다. 이 문서 작성 시 아직 하지 않은 조치를 완료로 보고하지 않는다.
