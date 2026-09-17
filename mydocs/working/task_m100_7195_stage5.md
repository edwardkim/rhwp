---
kind: investigation
status: active
---

# #7195 5단계 — 최신 devel 통합 검증

## 승인과 입력

2026-09-17 작업지시자가 기록 커밋 → 충돌 통합 → 검증 순서를 승인했다.
4단계 기록은 `6533d50e6`으로 커밋했다. 통합 대상은 fetch한
`upstream/devel fcbd00e0fa`이며 PR #7200까지 포함한다.
원격 push·PR·이슈 종료는 수행하지 않는다. 별건 실패의 추가 수정·ignore 확대도 하지 않는다.

## 충돌 해소 근거

유일한 텍스트 충돌은 `typeset.rs`의 행 컷 재시도다.

- devel: 최초 컷에 `advance_row_cut_with_mixed_nested_reserve`를 적용하여
  실제 선택한 중첩 뷰포트의 추가 공간을 예약한다.
- #7195: 넘치는 조각을 재시도할 때 `advance_row_cut_within_capacity`를 사용하여
  줄 소유는 유지하되 저장 프레임 끝까지 예산 밖으로 재확장하지 않는다.
- 통합: 최초 예약 경로는 devel대로 유지한다. 재시도는 측정한
  `painted_tail = split_total - consumed_height - padding`을 예약한 엄격한 컷을 사용한다.
  이 꼬리는 중첩 예약 공간도 포함하므로 이미 예약을 뺀 `budget` 대신
  원래 `content_budget`에서 빼서 같은 공간을 두 번 차감하지 않는다.
  재시도 결과는 `row_cut_content_height`로 다시 측정하여 후보의 물리 점유를 검사한다.
  일반 텍스트 행은 중첩 예약이 없으므로 #7195의 동작을 유지한다.

같은 helper라는 명칭만으로 동등성을 주장하지 않는다. 최초 컷과 재시도의 예약 계층을
구분하고 #7195 저장 프레임/본문 경계/소유 보존과 #7140 중첩 예약 계약을 함께 실행한다.
양쪽에 기존부터 있던 강제 전진 및 후속 이슈 대상의 예외는 이번 충돌 해소에서 확대하지 않는다.

## 검증 계획과 상태

1. 별도 review worktree에서 suite 준비 후 집중 계약 실행.
2. 영향 페이지 출력 및 기존 승인 출력과의 비교.
3. 통합본 전체 회귀와 lint 3종/workspace build/policy/Native Skia/Docker WASM.
4. 4단계 28건 실패와 통합 후 결과를 이름·원인별로 대조.

## 집중 검증 결과

- 통합 커밋: `cb151503146af73df18e10017f0038ca66ecd530`.
- 별도 worktree: `/home/edward/mygithub/rhwp-review-7195-integration`.
- root의 오래된 파생 suite 때문에 첫 `cargo fmt --all -- --check`는 누락된
  `issue_7090_sibling_table_occupancy.rs` 참조로 실패했다. root 파생물을 임의 정리하지 않고
  새 review worktree에서 suite 준비 후 실행한 fmt 검사는 PASS였다.
- suite 정책: 1350 sources / 5826 static attrs / 48 targets PASS.
- unit-tier 정책: 4205 tests / 298 modules PASS.
- 집중 계약: **36 PASS / 1 FAIL** (37건). #7195/#2279/#2308/#3798,
  #7150 및 PR #7200 관련 계약과 #7140의 본문 경계 검사는 통과했다.
- 실패: `issue_7140_nested_row_unit_paint_height::hwpx_page_count_matches_the_oracle`,
  `samples/table_giant_cell_overfill.hwpx` 실제 **49쪽**, 기대 **48쪽**.
  테스트 기준은 유지했다. 아래 동일 입력의 최신 devel 실행 대조로 새 통합 차이를 확인했다.
- 통합 전 승인된 86712의 4–8·11·12쪽은 같은 원본으로 재출력한 SVG와 render tree가
  7/7 byte-identical이다. 11·12쪽 PNG를 열어 마지막 줄/다음 쪽 이어받기와 표 경계를 확인했다.
  이 결과를 새 한컴 시각 판정이나 다른 페이지의 무회귀로 확대하지 않는다.

증적: `output/7195/stage5/{focused,fmt,suite_policy,unit_policy}.{json,log}`,
`approved-pages/comparison.json`, `approved-pages/p11.png`, `approved-pages/p12.png`.

## 전체 회귀와 Docker WASM

- 동일 제품 SHA `cb151503146af73df18e10017f0038ca66ecd530`에서
  `cargo nextest run --locked --cargo-profile release-test --target-dir
  /home/edward/mygithub/rhwp/target/pr-review --tests --test-threads 4 --no-fail-fast` 실행.
- **9961건 실행: 9931 PASS / 30 FAIL / 47 skipped**, exit 100.
  이전 28개 실패 이름은 모두 남았고 2개가 추가되었다. 추가 2개는 같은 giant HWPX의
  49쪽/48쪽 차이를 검출한 #7140 계약과 oracle partition 5다.
  증적: `output/7195/stage5/regression.{json,log}`, `regression-comparison.json`.
- Docker WASM 빌드 PASS:
  `docker compose --env-file .env.docker run --rm -e CARGO_BUILD_JOBS=4 wasm`.
  빌드 약 8분 18초, 11,221,576 bytes,
  SHA256 `9a274ae5d735868841928890353cd49495c0b433e20420d092331bd6311b91e4`.
  회귀와 별도 checkout/target에서 병행했으며 성능 측정 결과로 사용하지 않는다.
- Studio `http://127.0.0.1:7700/`의 WASM 응답 HTTP 200 및 산출물 hash 일치 확인.
  86712는 64쪽이며 승인된 4–8·11·12쪽의 WASM tree/SVG도 이전 승인 출력과 7/7 동일하다.
  giant HWPX는 WASM에서도 49쪽이다.
  증적: `wasm-docker.log`, `wasm-verification.json`, `approved-pages/`.

## 새 통합 차이: giant HWPX 40쪽

같은 `samples/table_giant_cell_overfill.hwpx`
(SHA256 `5d7eb4a21e46d9ad01a0f631eea2b1f2ec8a71750b4d448868e944e1b95042f4`)를
별도 base worktree와 통합 worktree에서 빌드한 CLI로 직접 비교했다.

| 항목 | 최신 devel `fcbd00e0fa` | 통합 `cb1515031` |
|---|---|---|
| 총 물리 페이지 | 48 | 49 |
| 최초 ledger 차이 | 40쪽 | 40쪽 |
| 동일 표 조각 | s0 pi=0 ci=3, row 4–5, startCut 1182 | 동일 |
| endCut | 1227 | 1210 |
| 본문 높이 | 1009.1467 px | 1009.1467 px |
| usedHeight | 1495.5067 px | 1002.44 px |

40쪽 PNG를 직접 열어 비교했다. base는 더 많은 본문을 배치하며 표 프레임이 페이지 아래로
이어진다. 통합본은 계산한 점유 높이가 본문에 들어오지만 보이는 텍스트 아래에 큰 여백이
남고, 41쪽에서 해당 문단이 이어진다. **물리 높이 검사 통과만으로 올바른 분할을
입증하지 못한다.** 측정과 실제 paint의 차이에 대한 정밀 판정이 필요하며,
이를 기존 실패로만 분류하거나 한컴 피델리티 통과로 선언하지 않는다.
기존 #7140과 관련된 새 통합 차이로 구분하고 추가 제품 수정·ignore 확대·기준값 변경은 하지 않았다.

증적: `output/7195/stage5/giant-base-comparison.json`,
`giant-{base,integrated}-pages.json`, `giant-base/p40.png`,
`giant-integrated/p40.png`, `giant-integrated/p41.png`.

## 다음 게이트

추가 차이에 대한 메인테이너 시각 트리아지가 먼저다. 승인된 86712 페이지 보존과 전체 회귀
성공은 별개이며 현재 전체 회귀는 실패 상태다.
통합 SHA의 native/WASM/workspace-all-targets Clippy 3종, workspace build 및
Native Skia 3종 검증은 아직 미실행이다. 이후 제출 게이트로 남긴다.
원격 push·PR·이슈 종료는 수행하지 않았다.
