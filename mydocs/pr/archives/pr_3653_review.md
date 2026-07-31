---
kind: pr-review
status: active
---

# PR #3653 검토 — 그림 바이트 키 조회와 base64 생략 opt-in

| 항목 | 값 |
| --- | --- |
| 작성자 / reviewer | `@lpaiu-cs` / `@jangster77` |
| 원 PR / 관련 이슈 | [#3653](https://github.com/edwardkim/rhwp/pull/3653) / [#3315](https://github.com/edwardkim/rhwp/issues/3315) Track 3 |
| 원 head 참고값 | `c8f0e99bceb66790f4f7b6c83c328cb91983b96a` |
| 통합 후보 | [#3661](https://github.com/edwardkim/rhwp/pull/3661) `52903c91bf132f7f3a977afc9cc265859b024c85` |
| 원 변경 규모 | 8 files, +549 / -55 |
| 권고 | #3661의 통합 범위로 수용. #3315는 umbrella로 open 유지 |

## 변경과 통합 판정

전체 `PageLayerTree`를 반복해서 JSON으로 넘기면 이미지 base64가 입력마다 WASM 경계를 다시 건넌다.
이 PR은 기존 기본 직렬화는 바꾸지 않고 `omitImageBytes=true`일 때만 바이트를 생략하며,
`getSourceImageBytes(key)`로 실제 방출 payload를 다시 받는 additive API를 둔다.

- `bin:{epoch}:{bin_data_id}:{variant}` 키는 snapshot 복원·새 문서에서 epoch를 달리하고, JPEG
  워터마크 bake와 원본/변환 payload를 variant로 구분한다.
- 키를 해석하는 단일 경로가 `emitted_image_bytes`를 사용하므로 BMP/PCX/TIFF·회색 JPEG 변환과
  watermark PNG bake가 기존 JSON 방출물과 조용히 갈라지지 않는다.
- PageLayerTree cache fingerprint에는 image-byte omission bit를 포함해 inline과 omission 결과가
  같은 cache entry를 공유하지 않는다.
- 기본 호출은 여전히 image data URL을 포함한다. 구형 WASM·해석 불가 키는 Studio가 전체 tree
  경로로 되돌아갈 수 있다.

원 기능 commit은 통합 branch의 `18631bce7`로 patch 동등하게 누적했다. source branch의 devel
병합은 포함하지 않았고, 뒤의 #3660은 이 API 위에 쌓이는 관계여서 한 번만 선행 적용했다.

## 검증

| 검증 | 결과 |
| --- | --- |
| source #3653 CI | full CI, CodeQL, Canvas visual diff success |
| 통합 code head CI | lint·WASM check, frontend package gates, Native Skia, archive, default-feature 8 shards, CodeQL, Canvas visual diff, `Build & Test` 모두 success |
| 체리픽 동등성 | `c8f0e99bc` = `18631bce7` patch-id, `git diff --check` 통과 |
| 회귀 계약 | image-key의 바이트·mime 동등성, omission 출력, 기본 inline 보존, cache variant 분리, stale/malformed key 거부를 integration test로 고정 |
| 로컬 WASM | `CARGO_TARGET_DIR=target/review-lpaiu-cs-20260731 CARGO_INCREMENTAL=0 wasm-pack build --target web --out-dir pkg` exit 0 |
| 추가 전체 Cargo | source 및 정확한 통합 head full CI와 중복되므로 작업지시에 따라 실행하지 않음. 성공 근거로 사용하지 않음 |

## 범위와 권고

이 PR은 #3315 Track 3의 바이트 전달 계약만 만든다. 전체 tree를 좁은 flow query로 대체하는
소비자 최적화는 #3660에서 검토한다. 따라서 통합 PR 본문에는 `Closes #3315`를 쓰지 않고
umbrella 이슈를 open으로 둔다.

**권고: 수용.** #3661의 같은 code head는 full CI와 `MERGEABLE`을 확인했다. review 문서만 추가한
head는 별도 fast-pass를 통과한 뒤 통합 PR 하나로 merge한다.
