# PR #3452 검토 기록 — base64 JSON 직접 기록 최적화

| 항목 | 내용 |
| --- | --- |
| 원 PR | [#3452](https://github.com/edwardkim/rhwp/pull/3452) — `Task #3315: [성능] 그림 base64를 이스케이프 스캔 없이 JSON 버퍼로 직접 인코딩` |
| 작성자·검토자 | `@lpaiu-cs` (external contributor) · `@jangster77` (collaborator) |
| base / source head | `devel` / `a2a836778f3d0d825d28975dfa90ca01d559f066` (`perf/3315-track2-base64-escape`) |
| 원 변경 규모 | 4 files, +192 / -30, 2 commits(기능 1 + devel merge 1) |
| 통합 검토 | `review/lpaiu-cs-20260727`; `upstream/devel` `7779e737ac5c5df3428d1a06f1099be16375be49` 기준 |
| 원 변경 적용 | `61c7bcdb9f6f8b0efb853764d4f62b7d2094034e`→`9c468dace12f1e28f7ad200ab4bd57d13c6774f2`; devel merge 제외. 선행 #3411의 바이트 타입 누적에 맞춰 두 borrow를 `data.as_slice()`→`&data[..]`로 의미 동등 적응 |
| collaborator 보정 | `0b58a0d4497d2154b37e797ce49b8eca79357fd2` 중 base64 무패딩 길이 test |
| 관련 이슈 | [#3315](https://github.com/edwardkim/rhwp/issues/3315) Track 2의 부분 수용. umbrella 이슈는 닫지 않음 |
| 작성 시점 source 상태 | `MERGEABLE` / `CLEAN`, draft 아님; source Build & Test [성공](https://github.com/edwardkim/rhwp/actions/runs/30230525928/job/89869777908) |
| 라우팅 | base: `collaborator_external_pr`; modifiers: `intake_and_review`, `local_validation`, `visual_fixture_evidence`, `multi_pr_update_branch` |

Loaded documents: `pr_review_workflow.md`, `pr_review/README.md`,
`collaborator_external_pr.md`, `intake_and_review.md`, `local_validation.md`,
`visual_fixture_evidence.md`, `multi_pr_update_branch.md`.

## 원 변경 범위와 판정

원 PR은 표준 base64 알파벳에 JSON escaping 대상 문자가 없다는 점을 이용해, 그림 바이트를 중간 `String`에
인코딩한 뒤 다시 escape scan하는 대신 최종 JSON buffer로 직접 기록한다. page background, image op의
resolved/original payload, resource image table과 overlay image 등 다섯 생산 경로를 공용
`write_json_base64`로 모은다.

성능 최적화에서 가장 중요한 수용 조건은 기존 JSON과 byte-for-byte 동일한 것이다. contributor test는
0..=255 전 바이트와 padding 길이를 비교했고 실제 layer tree·overlay 생산자도 확인한다. 통합 검토에서는
길이 `% 3 == 0`인 255-byte case를 `0b58a0d44`에 추가해 padding 없음, `== 1`, `== 2` 세 경우가 모두
명시적으로 지나도록 했다.

체리픽 때 #3411이 먼저 바이트 소유 타입을 보정한 상태여서 `queries/rendering.rs`와 `paint/json.rs`의
`data.as_slice()` 두 곳은 현재 타입에 맞는 `&data[..]`로 바꿔 적용했다. 빌려 읽는 동일 slice 계약이며
base64 출력 의미는 바꾸지 않는다. 이 적응 때문에 source와 통합 commit의 patch-id는 다르다.

PR 제목의 편집당 `58.8→15.2 ms`는 contributor 환경 측정값이며 통합 후보에서 독립 benchmark를 반복하지
않았다. merge 근거는 특정 배속이 아니라 동일 바이트 계약과 전체 회귀 게이트다.

## Renderer·fixture·시각 검증

- 재현 fixture: `samples/hwpx/issue_241.hwpx`
  (`SHA-256 757629ea3a84887ca1ccdc071aa61d2b7a414f5f8cd3bba5e8d86806edc6320e`).
- 기준 PDF: `pdf/hwpx/issue_241-2022.pdf`, 한글 2022, 1 page
  (`SHA-256 b408c3e858b99b6a6bda3cc82a9a871538c50b25c0ffc5f829e8d4be0f9b7a8a`).
- 기준 devel과 보정 후보의 `export-svg`는 SHA-256
  `d0247006516f54a8cb4b30ea734887883a57b8a2008ad60f75ca79bd92df88ef`로 byte-identical했다.
- visual sweep 임시 경로:
  `output/pr3411-3452-3455-image-pipeline-p1-20260727/pr3411-3452-3455-issue241-p1/`.
  문서의 유일한 page 1 한 쪽을 검토했고 자동 후보는 0/1, pixel match `96.02618%`,
  `visual_accuracy_proxy_percent` `10.7953%`였다. 사람 확인에서 그림·표·본문 누락이나 clipping은 없었다.
- 최종 `ceda586e7` OVR은 5개 preset 142 pages를 비교했고 실제 개체가 있는 3개 sample의 11개
  개체에서 ±2px 회귀 0건이었다. 0개 개체 행은 근거로 쓰지 않았다.
- 새 HWP/HWPX fixture가 없어 IR field sweep baseline 신규 등록 trigger는 없다.

![PR #3411·#3452·#3455 image pipeline page 1 검토](../assets/pr_3411_3452_3455_lpaiu-cs_image_pipeline_review_p001.png)

안정 asset은 `2416×1211` PNG, SHA-256
`ad5ebf41c5e215004bdb84efaffdb95871a24310ce91fec02dbaa336876e2074`다. 최종 시각 판정 권위는
작업지시자에게 있다.

## 검증

- `issue_3315_image_base64_round_trip` 및 helper focused: 통과. 253·254·255-byte 길이와 0..=255 전
  바이트를 기존 경로와 대조했다.
- `cargo build --release`: 통과.
- `cargo test --release --lib`: 2949 passed / 0 failed / 7 ignored.
- `cargo test --profile release-test --tests`: 모든 target exit 0, IR field sweep 2/2 포함.
- Native Skia 공식 3종: 57/0, 2/0, 4/0.
- fmt·diff check·clippy·doc test, fresh wasm-pack, TypeScript, 최종 Studio 670 tests와 production build: 통과.

source head의 녹색 CI는 contributor commit을 검증했지만 padding 보강과 다른 PR을 누적한 통합 후보를
대신하지 않는다. 최신 통합 head full CI가 필수다.

## Risk와 최종 권고

직접 buffer 기록은 escaping 전제를 잘못 세우면 큰 JSON을 조용히 손상시킬 수 있다. 표준 base64 alphabet,
세 padding 형태, 실제 producer와 전체 serializer 회귀가 같은 출력을 고정한다. **test 보정 후 기술적으로
수용 가능**하다. #3315는 후속 Track이 남은 umbrella이므로 닫지 않는다. 최종 merge 조건은 최신 통합
head full CI, mergeable 상태와 작업지시자 승인이다.
