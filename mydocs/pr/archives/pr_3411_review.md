# PR #3411 검토 기록 — 그림 바이트 공유와 공개 Rust API 호환

| 항목 | 내용 |
| --- | --- |
| 원 PR | [#3411](https://github.com/edwardkim/rhwp/pull/3411) — `Task #3315: [성능] 그림 바이트 소유를 Arc 로 — 스냅샷 31배·load 1만배` |
| 작성자·검토자 | `@lpaiu-cs` (external contributor) · `@jangster77` (collaborator) |
| base / source head | `devel` / `64fe53bae600b82ed47fd18b4dee106ff94fffa7` (`perf/3315-track1-image-arc`) |
| 원 변경 규모 | 21 files, +136 / -119, 6 commits(기능 3 + devel merge 3) |
| 통합 검토 | `review/lpaiu-cs-20260727`; `upstream/devel` `7779e737ac5c5df3428d1a06f1099be16375be49` 기준 |
| 원 변경 적용 | `073378a8`→`5baefb899`, `06a2b685`→`4066fb8cf`, `374066a3`→`dfae7f30d`; devel merge commit 3개 제외 |
| collaborator 보정 | `0b58a0d4497d2154b37e797ce49b8eca79357fd2` 중 공개 API·공유 ingestion 범위 |
| 관련 이슈 | [#3315](https://github.com/edwardkim/rhwp/issues/3315) Track 1의 부분 수용. umbrella 이슈는 닫지 않음 |
| 작성 시점 source 상태 | `MERGEABLE` / `CLEAN`, draft 아님; source Build & Test [성공](https://github.com/edwardkim/rhwp/actions/runs/30231211661/job/89871788870) |
| 라우팅 | base: `collaborator_external_pr`; modifiers: `intake_and_review`, `local_validation`, `visual_fixture_evidence`, `multi_pr_update_branch` |

Loaded documents: `pr_review_workflow.md`, `pr_review/README.md`,
`collaborator_external_pr.md`, `intake_and_review.md`, `local_validation.md`,
`visual_fixture_evidence.md`, `multi_pr_update_branch.md`.

## 원 변경과 수용 범위

원 PR은 세션 중 삽입한 그림 바이트가 `Document::clone`과 레이어 생성에서 반복 복제되는 비용을 줄이기 위해
`BinDataBytes::Loaded`, `load()` 및 여러 renderer 공개 타입의 바이트 소유를 `Arc<[u8]>`로 바꿨다.
공유 소유 자체는 #3315의 방향과 맞고 cfg 뒤의 WASM·Native Skia 사용처까지 후속 commit에서 복구했다.

다만 `BinDataBytes::Loaded(Vec<u8>)`, `load() -> Vec<u8>`, `ImageNode.data`,
`PageBackgroundImage.data`, `ResolvedImagePayload.data`는 외부 Rust 소비자가 직접 생성·매칭할 수 있는 공개
계약이다. 이를 `Arc`로 일괄 교체하면 semver minor에서 source compatibility를 깨므로 원 변경 그대로는
수용할 수 없었다. 기여자 측 31배·1만배·7% 수치도 원 설계 전체를 전제로 한 측정이라, 공개 계약을 복원한
통합 후보의 성능 보증값으로 재사용하지 않는다.

## Collaborator 보정

`0b58a0d44`에서 공개 `Vec` 계약을 복원하면서 내부 공유 경로를 additive하게 남겼다.

- `BinDataBytes`의 기존 두 variant와 `Loaded(Vec<u8>)` exhaustive match를 보존했다.
- `load()`·`load_limited()`와 공개 renderer 구조체 필드는 다시 소유 `Vec<u8>`를 반환·보관한다.
- `from_shared`, `load_shared`, `load_limited_shared`와 in-memory resolver를 추가해 저장소 내부에서만 같은
  allocation을 공유할 수 있게 했다.
- 외부 이미지 주입, embedded bin-data 등록, HTML data-image import의 세 ingestion 경로는
  `from_shared`를 사용한다. 이 경로로 삽입한 그림은 문서 snapshot clone 뒤에도 같은 `Arc`를 공유한다.
- `tests/public_rust_api_vec_compat.rs` 두 건으로 기존 constructor·exhaustive match·struct literal과
  snapshot 포인터 공유를 함께 고정했다.

따라서 통합 후보는 "모든 renderer payload를 Arc로 바꾼다"가 아니라 **공개 호환을 유지한 채 신규
in-memory ingestion의 snapshot deep-copy를 제거한다**는 범위로 수용한다. `Lazy` 압축 해제 캐시와 전체
renderer payload 전파는 #3315의 후속 판단으로 남는다.

## Renderer·fixture·시각 검증

- 재현 fixture: `samples/hwpx/issue_241.hwpx`
  (`SHA-256 757629ea3a84887ca1ccdc071aa61d2b7a414f5f8cd3bba5e8d86806edc6320e`).
- 기준 PDF: `pdf/hwpx/issue_241-2022.pdf`, 한글 2022, 1 page
  (`SHA-256 b408c3e858b99b6a6bda3cc82a9a871538c50b25c0ffc5f829e8d4be0f9b7a8a`).
- 변경 전·후 `export-svg`는 SHA-256
  `d0247006516f54a8cb4b30ea734887883a57b8a2008ad60f75ca79bd92df88ef`로 byte-identical했다.
- visual sweep 임시 경로:
  `output/pr3411-3452-3455-image-pipeline-p1-20260727/pr3411-3452-3455-issue241-p1/`.
  실제 검토는 문서의 유일한 page 1 한 쪽이며 자동 후보는 0/1이었다. pixel match는 `96.02618%`,
  `visual_accuracy_proxy_percent`는 `10.7953%`다. 낮은 ink proxy는 후보·기준의 글꼴 raster 차이를 포함하며,
  사람 확인에서 표·도장 그림·본문이 모두 존재하고 clipping이나 누락은 없었다.
- 최종 `ceda586e7` OVR은 5개 preset 142 pages를 비교했고, 실제 개체가 있는 3개 sample의 11개
  개체에서 ±2px 회귀 0건을 확인했다. 개체가 0개인 두 행은 근거로 사용하지 않았다.
- 새 HWP/HWPX fixture를 추가·교체·이동하지 않아 IR field sweep baseline 신규 등록 trigger는 없다.

![PR #3411·#3452·#3455 image pipeline page 1 검토](../assets/pr_3411_3452_3455_lpaiu-cs_image_pipeline_review_p001.png)

안정 asset은 `2416×1211` PNG이며 SHA-256은
`ad5ebf41c5e215004bdb84efaffdb95871a24310ce91fec02dbaa336876e2074`다. 최종 시각 판정 권위는
작업지시자에게 있다.

## 검증

모든 Cargo 명령은 `CARGO_INCREMENTAL=0`, 검토 전용
`CARGO_TARGET_DIR=target/review-lpaiu-cs-20260727`로 순차 실행했다.

- 공개 Rust API 호환 test: 2 passed; image base64 focused 1 passed; image key focused 6 passed.
- `cargo build --release`: 통과.
- `cargo test --release --lib`: 2949 passed / 0 failed / 7 ignored.
- `cargo test --profile release-test --tests`: 모든 target exit 0, IR field sweep 2/2 포함.
- Native Skia 공식 3종: 57/0, 2/0, 4/0.
- `cargo fmt --check`, `git diff --check`, `cargo clippy --all-targets -- -D warnings`: 통과.
- doc test: 4 passed / 0 failed / 2 ignored.
- fresh `wasm-pack build --target web`, TypeScript 검사, 최종 Studio 670 tests와 production build: 통과.

source head의 녹색 CI는 contributor 원 설계만 검증한다. 공개 계약 복원과 다른 PR이 함께 있는 통합
후보는 review-only fast-pass가 아니며 최신 통합 head의 full CI를 별도로 통과해야 한다.

## Risk와 최종 권고

보정 뒤 남은 핵심 위험은 공유 최적화의 범위가 원 PR 설명보다 좁다는 점이다. 이를 과장하지 않고 공개
호환 test와 실제 shared-ingestion snapshot test를 수용 계약으로 삼는다. **Collaborator 보정 후 기술적으로
수용 가능**하다.

#3315는 Track 1–4의 umbrella이며 이번 후보는 snapshot 공유의 일부만 해결한다. 통합 PR 본문에서
`Closes #3315`를 쓰지 않고 open 상태를 유지한다. 최종 merge 조건은 최신 통합 PR head의 full CI,
mergeable 상태와 작업지시자 승인이다.
