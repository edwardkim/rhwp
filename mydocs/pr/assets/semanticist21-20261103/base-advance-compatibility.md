# 보정 제출 직전의 base 전진과 호환성

본 작업의 검증 code는 `5e374d01f641745d20bc9424ce233021a327ed16`, 로컬 증적 head는 `d31c4bd3bb00f8ed7f304899e521ec4969bf3bbe`다. push 직전 devel이 `1924d371d6`에서 `96827b3accd98ad0b80f76260b42fba24eded504`로 전진해 [#8203](https://github.com/edwardkim/rhwp/pull/8203)과 [#8583](https://github.com/edwardkim/rhwp/pull/8583)을 포함했다.

[검토 중 base 전진 A.0](../../../manual/pr_review/review_only_fast_pass.md#a0-검토-중-base-전진과-오늘할일-갱신)에 따라 source branch를 반복 merge/rebase하지 않았다. merge-tree `695ff80982982d1485ab3368b5e9377f18f0a2d2`는 충돌 없이 성립했다. 이 tree를 branch와 분리된 임시 worktree에 materialize해 실제 컴파일로 확인했다.

- 최신 base가 바꾼 제품·검사·도구·글꼴 라이선스 경로 중 겹치지 않는 35개는 merge-tree와 base의 Git blob이 모두 같다.
- 겹치는 제품 파일은 hwpctrl_sets.rs 하나다. #8203의 scan_paragraph 자리표 소비와 본 작업의 control_identity Form 식별은 서로 다른 함수이며 두 보정이 tree에 모두 있다. 원 contributor head #8203을 이 배치에 추가로 수용한 것으로 세지 않는다.
- #8583의 WASM 메트릭·프로그램·JPEG painter와 검증 파일은 해당 base 내용 그대로다. 새 base 자체의 [검토와 전 383쪽 증거](https://github.com/edwardkim/rhwp/blob/96827b3accd98ad0b80f76260b42fba24eded504/mydocs/pr/archives/pr_8583_review.md)를 확인했다. 해당 383쪽을 본 작업에서 재실행한 것으로 표시하지 않는다.
- 최신 base를 명시한 manifest/unit-tier 검사: PASS.
- merged workspace all-target `cargo check --locked`: exit 0, 106.24초.
- merged wasm32 lib `cargo check --locked`: exit 0, 75.45초.

이것은 현재 base와의 정책·컴파일 호환성과 양쪽 변경의 보존 확인이다. 이 임시 tree의 새 전체 회귀나 새 Visual Sweep으로 과장하지 않는다. 본 작업 head에서 완료한 전체 회귀·정확한 Native CI 22개 목록·전쪽 시각 검증의 producer는 그대로 기록한다. 새 제출 head의 실제 GitHub Full CI·CodeQL과 최종 current-base merge gate를 모두 확인한 뒤 병합한다. source/test/workflow 변경은 추가하지 않았다.
