# PR #7121 검토

- 검토일: 2026-09-14
- PR: https://github.com/edwardkim/rhwp/pull/7121
- 작성자: dependabot[bot], base devel. reviewer edwardkim 추가(기존 jangster77 유지).
- 경로: maintainer_general + intake_and_review + local_validation + multi_pr_update_branch + rework_and_exceptions; 렌더 의존성에는 visual_fixture_evidence 참고.
- 원 head: `cd00c2b6316ba340b71886045920ab0431e1b42a`.
- 검토 base: `037e4906a93e99896daa145a5ee5517824bfeaf4`.
- 각 PR의 current-base merge-tree 및 여섯 건 누적 cherry-pick 모두 충돌 없음.
- 여섯 건 검토 순서: #7121 → #7122 → #7125 → #7126 → #7127 → #7128. #7121 실패 분리 후 나머지 다섯 건을 같은 순서로 재구성했다. 각 원 commit을 -x로 보존했고 원 PR branch는 변경하지 않았다. 기능상 선행 PR 의존성은 없으나 Cargo.lock 공통 변경을 누적으로 확인했다.
- 다섯 건 검증 후보: `4ff101bee6f187b20bda044d3182d64a6f2e95a0`.

## 검토·검증

GPU 선택 기능 빌드에서 E0308 두 건을 재현했다.

- `src/renderer/gpu.rs:191`: resvg-gpu의 usvg 0.46 Tree를 vello_svg의 usvg 0.48.1 Tree 인자로 넘길 수 없다.
- `src/renderer/gpu.rs:193`: vello_svg가 반환하는 vello 0.10 Scene을 rhwp의 vello 0.9 Scene에 append할 수 없다.
- Cargo.toml은 vello 0.9 / resvg-gpu 0.46을 유지하고 vello_svg만 변경했다. 같은 usvg Tree를 CPU/GPU에 공유하는 기존 계약이 깨졌다.
- 여섯 건 누적 `5912e5e767818be39fcb7438bf3993875b7b3fb9`에서 실패했고, 이 PR만 제외한 나머지 다섯 건 `4ff101bee6f187b20bda044d3182d64a6f2e95a0`에서 같은 명령이 성공했다. 다른 다섯 PR과 구분한 대조다.
- 명령: `cargo check --locked -p rhwp --lib --features gpu --target-dir /home/edward/mygithub/rhwp/target/pr-review`.
- 로그: `output/dependabot-20260914/gpu-check.log`, `gpu-without-7121.log`.
- 현재 CI YAML에서 Rust gpu feature 검사가 발견되지 않았다. Native Skia/Canvas Render Diff 성공은 별도 GPU Rust 경로의 빌드 보증이 아니다.
- 해제 조건: vello / vello_svg / resvg-gpu 버전과 공용 Tree·Scene 계약을 함께 정합화하고 gpu 빌드 및 해당 경로 검증을 제출한다. 이번 batch review에서 제품 보정이나 새 이슈를 만들지 않았다.
- 조판 원칙: 문서 특정 조건·기준값 변경 없음. 렌더링 입력 타입 계약의 호환성 문제이며 시각 출력 이전 컴파일 단계에서 실패했다.
- 검증 입력: sample 불필요(컴파일 재현). Cargo.toml·Cargo.lock·gpu.rs는 후보 commit에 포함되고 수정하지 않았다.

## 원격 CI 및 검증 경계

- [Canvas visual diff](https://github.com/edwardkim/rhwp/actions/runs/34800345883/job/103841622564): SUCCESS.
- [Lint (fmt, clippy, WASM check)](https://github.com/edwardkim/rhwp/actions/runs/34800350025/job/103841658562): SUCCESS.
- [Native Skia tests](https://github.com/edwardkim/rhwp/actions/runs/34800350025/job/103841658606): SUCCESS.
- [Frontend package gates](https://github.com/edwardkim/rhwp/actions/runs/34800350025/job/103841658609): SUCCESS.
- [Build & Test](https://github.com/edwardkim/rhwp/actions/runs/34800350025/job/103844275852): SUCCESS.

- 원 head의 진행/실패 check 없음. source·test·fixture 보정 없이 current-base merge clean이므로 원격 전체 CI를 재사용하고 로컬 집중 검증만 추가한다. SKIPPED를 실행 성공으로 세지 않는다.
- frontend 테스트에서 사용한 pkg는 기존 로컬 WASM SHA256 `61465d69c436028b5f5c8db93297a92fb7283cb58450ce0525b5b342943c1b9a`를 복사한 것이다. Rust 의존성 갱신을 반영해 새로 빌드한 WASM이라는 주장은 하지 않는다.
- primary devel·다른 작업의 worktree·원격 source 변경 없음. 원격 push·GitHub review/comment·merge는 하지 않았다.

## 판정

**머지 보류**. 위 GPU 호환성 실패를 해결하기 전에는 병합하지 않는다.

병합 직전에 원 head·CI·current-base 충돌을 재확인한다. 나머지 다섯 건은 한 번의 일괄 병합 승인 대상으로 보고하고 #7121은 분리한다.

## 보류 후 기록

- 작업지시자가 보류 사유 댓글 게시를 승인했고 [댓글](https://github.com/edwardkim/rhwp/pull/7121#issuecomment-5660572145)을 게시했다. API 재조회로 한글과 본문을 확인했다.
- 이번 검토 회차의 기록만 archive에 보존하며 PR은 OPEN으로 유지한다. 버전 정합 보정·재검토는 하지 않았다.
