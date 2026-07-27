# PR #3455 검토 기록 — 그림 신원 키와 prefetch 재사용 계약

| 항목 | 내용 |
| --- | --- |
| 원 PR | [#3455](https://github.com/edwardkim/rhwp/pull/3455) — `Task #3315: [성능] 그림 신원 키(sourceImageKey)와 서명 조회 API` |
| 작성자·검토자 | `@lpaiu-cs` (external contributor) · `@jangster77` (collaborator) |
| base / source head | `devel` / `991d85b038db336b89bf4934ee90774ed109943a` (`feat/3315-image-key`) |
| 원 변경 규모 | 18 files, +732 / -40, 4 commits(#3452 공유 1 + 고유 2 + devel merge 1) |
| 통합 검토 | `review/lpaiu-cs-20260727`; #3452 적용 뒤 고유 commit 누적 |
| 원 변경 적용 | 공유 `61c7bcdb`는 #3452에서 한 번만 적용; `b34611fb90d533f6260a63aa0a8c4ef8c08ab6b7`→`6d3dd3d05`; `18564b38cba04c5fec4c8bd5ae36845c6f91a2fe`→`f175aef8d`; devel merge 제외 |
| collaborator 보정 | `0b58a0d4497d2154b37e797ce49b8eca79357fd2`, browser 발견 후 `ceda586e72fbcaa18ab66a758d7356210be0836a` |
| 관련 이슈 | [#3315](https://github.com/edwardkim/rhwp/issues/3315) Track 3 선행의 부분 수용. umbrella 이슈는 닫지 않음 |
| 작성 시점 source 상태 | `MERGEABLE` / `CLEAN`, draft 아님; source Build & Test [성공](https://github.com/edwardkim/rhwp/actions/runs/30230590867/job/89870028492) |
| 라우팅 | base: `collaborator_external_pr`; modifiers: `intake_and_review`, `local_validation`, `visual_fixture_evidence`, `multi_pr_update_branch` |

Loaded documents: `pr_review_workflow.md`, `pr_review/README.md`,
`collaborator_external_pr.md`, `intake_and_review.md`, `local_validation.md`,
`visual_fixture_evidence.md`, `multi_pr_update_branch.md`.

## 원 변경 범위와 판정

원 PR은 그림 op에 `sourceImageKey = bin:{epoch}:{bin_data_id}:{variant}`를 싣고,
`getPageSourceImageKeys(page)`의 작은 응답을 Studio prefetch 서명으로 사용한다. 같은 페이지의 그림이
바뀌지 않으면 수 MB layer-tree JSON과 browser decode를 다시 수행하지 않으며, document digest를 서명에
넣어 서로 다른 문서의 같은 bin id가 충돌하지 않게 한다. #3452 commit은 source branch에 함께 있었지만
통합에서는 먼저 적용된 #3452와 중복하지 않았다.

신원 키 방향은 타당하지만 원 구현에는 cache correctness 차단점이 있었다. compact key에 없는 합성 그림,
현재 새로 생긴 rawSvg, 같은 바이트 파일 재열기, decode 실패·늦은 완료와 `PageLayerTree` 공개 struct 변경을
구분하지 못했고, key 조회가 full layer tree를 다시 만드는 부분도 작은 조회의 목적을 약화했다.

## Collaborator 보정

`0b58a0d44`에서 correctness와 공개 계약을 다음처럼 보정했다.

- 성공한 문서 load/new마다 `documentGeneration`을 증가시켜 같은 digest 파일을 다시 열어도 이전 서명을
  재사용하지 않는다.
- `bin_data_id == 0`이 하나라도 있으면 compact 응답을 `cacheable:false`로 만들어 page 전체를 cache하지
  않는다. 현재 rawSvg count가 0보다 커도 skip하지 않는다.
- decode가 모두 성공한 뒤에만 서명을 기록한다. 실패·빈 작업·document generation 변경·늦게 끝난 이전
  request는 기록하지 않으며 page별 request token으로 stale completion을 막는다.
- compact API는 cached RenderTree를 직접 pre-order 순회한다. full `PageLayerTree`를 다시 만들지 않고,
  JSON producer와 같은 image order를 쓴다.
- 공개 `PageLayerTree` struct literal을 깨지 않도록 epoch를 private `ResourceArena` 상태로 옮기고 schema
  minor를 19→20으로 올렸다. TypeScript에는 optional `sourceImageKey`를 추가했다.
- source key variant는 실제 base64가 달라지는 JPEG watermark bake만 `wmpng`, 나머지는 `src`로 둔다.
  public Vec API 호환은 #3411 review의 별도 회귀 test로 함께 고정했다.

실제 browser 검증에서 추가 결함을 발견했다. 기존 prefetch의
`/"type":"image"[^}]*?.../` 정규식은 image op 안의 중첩 `bbox` 첫 `}`에서 멈춰
`issue_241.hwpx`의 실제 raster image를 수집하지 못했다. `ceda586e7`에서 layer-tree JSON을 한 번 parse한
뒤 중첩 object/array를 재귀 순회해 image data URL을 모으도록 고쳤고 focused test를 추가했다.

## 실제 browser 검증과 시각 증적

Google Chrome `150.0.7871.186`, Node `v24.15.0`, Vite `127.0.0.1:7700`에서
`samples/hwpx/issue_241.hwpx`를 열었다. 실제 image op는
`root.children.2.child.children.0.children.9.ops.0`, key `bin:0:1:src`, MIME `image/png`, base64
길이 15816이었다.

- 첫 load: digest `blake3:c063e9d0f9160858588bceb0326663b6cc7784ce7ab71b4314b9e0bd19c17903`,
  generation 1, `{"cacheable":true,"keys":["bin:0:1:src"]}`.
- 같은 generation의 다음 render: layer-tree call 0회로 skip.
- 같은 바이트 파일 재열기: generation 2, layer-tree call 1회로 새 prefetch와 서명 기록.
- generation 2의 다음 render: layer-tree call 0회로 다시 skip.
- 세 번 반복 load에서도 서명이 안정적으로 기록됐고 console error·warning은 모두 0이었다.

아래 browser asset에서 page 1의 표·도장 raster image가 실제 화면에 남아 있음을 확인했다.

![PR #3455 동일 파일 재열기 후 image prefetch browser 검토](../assets/pr_3455_lpaiu-cs_image_prefetch_browser_review_p001.png)

browser asset은 `1139×961` PNG, SHA-256
`b74bf88646d3b8eaf6518e3e62ee004de739b65167f857ec19f8c6856bf255a8`다.

같은 fixture의 한글 2022 기준 PDF·rhwp·overlay 독립 sweep도 함께 보존했다.

![PR #3411·#3452·#3455 image pipeline page 1 검토](../assets/pr_3411_3452_3455_lpaiu-cs_image_pipeline_review_p001.png)

- fixture SHA-256: `757629ea3a84887ca1ccdc071aa61d2b7a414f5f8cd3bba5e8d86806edc6320e`.
- 기준 PDF SHA-256: `b408c3e858b99b6a6bda3cc82a9a871538c50b25c0ffc5f829e8d4be0f9b7a8a`.
- page 1 자동 후보 0/1, pixel match `96.02618%`, visual proxy `10.7953%`; 사람 확인에서 image
  누락·clipping 없음.
- 안정 pipeline asset SHA-256:
  `ad5ebf41c5e215004bdb84efaffdb95871a24310ce91fec02dbaa336876e2074`.
- 기준 devel과 후보 SVG는 byte-identical했고, 최종 `ceda586e7` OVR은 5개 preset 142 pages의 실제
  11개 개체에서 ±2px 회귀 0건이었다.
- 새 fixture가 없어 IR field sweep baseline 신규 등록 trigger는 없다.

최종 시각 판정 권위는 작업지시자에게 있다.

## 검증

- Rust image key focused: 6 passed; public Rust Vec compatibility: 2 passed.
- `cargo build --release`: 통과.
- `cargo test --release --lib`: 2949 passed / 0 failed / 7 ignored.
- `cargo test --profile release-test --tests`: 모든 target exit 0, IR field sweep 2/2 포함.
- Native Skia 공식 3종: 57/0, 2/0, 4/0.
- fmt·diff check·clippy·doc test와 fresh wasm-pack: 통과.
- 최종 `ceda586e7` 후보에서 TypeScript, Studio full 670 tests, production build: 통과.
- 최신 image-prefetch focused: 13 passed / 0 failed; 실제 Chrome 시나리오 통과.

마지막 correction 뒤 통합 PR 최신 full CI가 TypeScript·전체 Studio suite·WASM을 다시 검증해야 한다.
source head의 녹색 CI는 두 collaborator correction을 포함하지 않으므로 최종 근거를 대신하지 않는다.

## Risk와 최종 권고

cache 최적화는 실패나 문서 교체를 성공으로 오인하면 이미지가 빈 상태로 고착되는 correctness 위험이 있다.
보정 후보는 cache 불가·rawSvg·generation·request token·decode completion을 보수적으로 처리하고, 실제
browser에서 같은 generation skip과 동일 파일 재열기 재검증을 모두 확인했다. 중첩 image 탐색 결함도
browser에서 발견해 재귀 순회로 고쳤다. **두 단계 보정 후 기술적으로 수용 가능**하다.

#3315는 flow-image narrow query와 base64 생략 등 후속 Track이 남은 umbrella이므로 통합 PR에서 닫지 않는다.
최종 merge 조건은 최신 통합 head full CI, mergeable 상태와 작업지시자 승인이다.
