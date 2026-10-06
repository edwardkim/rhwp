# PR #1527 최종 code review

- PR: https://github.com/edwardkim/rhwp/pull/1527
- 제목: `Task #1525 #1526: HWPX 직렬화 충실도 - 글머리표 + borderFill 이미지 채움`
- 작성자: `planet6897` (Jaeuk Ryu)
- 관련 이슈: #1525, #1526
- 작성일: 2026-06-25
- base/head: `edwardkim/rhwp:devel` `0ec4b976` <- `planet6897/rhwp:pr-task1525` `cc7c60c8`
- 변경 규모: 3 files, +125 / -21
- 처리 범위: code review, 로컬 검증, 수동 시각 검증 follow-up 문서화.

## 1. 최종 판단

**수용 권고. Blocking finding 없음.**

PR #1527은 #1525와 #1526에서 확인된 HWPX serializer 누락을 좁은 범위에서 보완한다.

1. `doc_info.bullets`를 `<hh:bullets>`로 직렬화해 글머리표 정의와 마커 글리프 소실을 막는다.
2. `borderFill` image fill의 `<hc:img binaryItemIDRef=...>`를 직렬화해 셀/쪽 배경 이미지 소실을 막는다.
3. 해당 수정으로 visual baseline의 기존 XFAIL 5건을 PASS로 승격한다.

코드 변경은 parser/IR이 이미 보존하던 값의 serializer 누락을 메우는 형태라 방향이 타당하다. 로컬 검증과 GitHub CI 모두 PR head `cc7c60c8` 기준으로 통과했다.

단, PR은 최신 `devel` 기준 `BEHIND` 상태다. 최신 `devel`과 merge-tree 수준의 충돌은 관찰되지 않았지만, 실제 merge 전에는 PR head update 또는 maintainer merge 정책을 확정해야 한다.

수동 시각 검증 중 `2026_oss_rst.hwpx` 1페이지 제목 영역에 원본에는 없는 사각형 테두리가 roundtrip 후 생기는 별도 문제가 발견됐다. base/head roundtrip 모두에서 재현되므로 #1527 regression으로 보지 않고, 기존 roundtrip 잔존 버그로 #1531에 분리했다.

## 2. GitHub 상태

| 항목 | 상태 |
|---|---|
| PR state | `OPEN` |
| Draft | false |
| Mergeable | `MERGEABLE` |
| Merge state | `BEHIND` |
| Maintainer can modify | true |
| Review decision | 비어 있음 |
| Closing issues | GitHub API 기준 `closingIssuesReferences=[]` |
| GitHub checks | Build & Test 성공, CodeQL 계열 성공, WASM Build skipped |

`closingIssuesReferences`가 비어 있으므로, PR 본문에 이슈 닫힘 문구가 있더라도 merge 후 #1525/#1526 상태를 수동으로 확인하는 편이 안전하다.

## 3. 변경 범위 검토

| 파일 | 변경 | 판단 |
|---|---|---|
| `src/serializer/hwpx/header.rs` | `write_header()`에서 `write_bullets()` 호출 추가, `write_bullets()`/`write_bullet()` 구현, `borderFill` 직렬화에 `SerializeContext` 전달 | 타당 |
| `src/serializer/hwpx/shape.rs` | `write_fill_brush()`가 `SerializeContext`를 받아 image fill의 manifest id를 해석하고 `<hc:img>` 방출 | 타당 |
| `tests/visual_roundtrip_baseline.rs` | bullet 3건, imgBrush 2건을 `VISUAL_XFAIL`에서 제거 | head 검증에서 PASS하므로 타당 |

## 4. 코드 리뷰 결과

### 4.1 Bullet serializer

`write_header()`의 refList 순서는 `fontfaces -> borderFills -> charProperties -> tabProperties -> numberings -> bullets -> paraProperties -> styles`로 유지된다. PR이 추가한 `<hh:bullets>` 위치는 기존 numbering 뒤, paraProperties 앞이라 schema 관찰 순서와 맞다.

`write_bullets()`는 `doc_info.bullets.len()`을 `itemCnt`로 쓰고, 각 bullet을 1-based `id`로 직렬화한다. `char`는 `Bullet.bullet_char`, `useImage`는 `image_bullet > 0` 기준의 `"0"`/`"1"`로 방출한다. 현재 parser가 보존하는 핵심 값과 writer가 방출하는 값의 범위가 일치한다.

`hh:paraHead level="0"`은 schema의 `positiveInteger` 표현과는 긴장이 있지만, PR 대상 샘플(`hy-002`, `footnote-01`, `2026_oss_rst`) 원본도 bullet paraHead에서 `level="0"`을 사용한다. 한컴 호환 관찰값을 따른 것으로 보이며 blocker로 보지 않는다.

현재 parser/IR은 bullet의 자식 이미지나 `checkedChar` 전체를 보존하지 않는다. 따라서 image bullet의 실제 이미지 참조와 checked character까지 무손실로 다루는 것은 이번 PR 범위 밖이다.

### 4.2 `borderFill` image fill serializer

기존 `write_fill_brush()`의 `FillType::Image` 경로는 빈 `<hc:imgBrush>`만 쓰고 `binaryItemIDRef`를 누락했다. PR은 `SerializeContext::resolve_bin_id(img.bin_data_id)`가 성공할 때만 `<hc:img binaryItemIDRef=...>`를 방출한다.

이 방식은 세 가지 점에서 안전하다.

1. parser가 이미 `BorderFill.fill.image.bin_data_id`에 읽어 둔 값을 serializer가 역매핑한다.
2. `content.hpf` manifest id를 `SerializeContext`에서 해석하므로 ZIP `BinData`와 manifest 사이의 3-way 정합을 깨지 않는다.
3. 미등록 `bin_data_id`는 기존처럼 빈 `<hc:imgBrush mode=.../>`로 남겨 잘못된 `image0` 참조를 만들지 않는다.

`bright`, `contrast`, `effect`도 현재 `ImageFill` 모델 값에서 직렬화된다. `alpha`는 항상 `"0"`으로 쓰이는데, header borderFill image alpha를 별도로 보존하는 IR이 현재 없으므로 이번 PR의 목적에는 부합한다. 향후 alpha까지 보존하려면 parser/IR 확장이 먼저 필요하다.

### 4.3 공용 `write_fill_brush()` 영향

`write_fill_brush()`는 `header.rs`의 `borderFill`과 `shape.rs`의 `write_rect()`가 공유한다. PR로 `write_rect()`에도 `ctx`가 전달되지만, 현재 body shape fill parser는 내부 `<hc:img>`의 `binaryItemIDRef`를 image fill의 `bin_data_id`로 캡처하지 않는 경로가 주된 상태다. 이 경우 `resolve_bin_id()`가 실패해 기존 빈 `imgBrush` fallback이 유지된다.

따라서 이번 변경의 실질 효과는 PR 설명처럼 header `borderFill` image fill 보존에 집중된다. 공용 함수 변경으로 컴파일/테스트 회귀는 관찰되지 않았다.

### 4.4 Visual baseline 승격

`VISUAL_XFAIL`에서 제거된 5건은 head에서 `visual_roundtrip_baseline` 전체 테스트가 통과했다.

| 샘플 | 기존 실패 원인 | PR 대응 |
|---|---|---|
| `hy-002.hwpx` | bullet definition 누락 | `<hh:bullets>` 직렬화 |
| `footnote-01.hwpx` | bullet definition 누락 | `<hh:bullets>` 직렬화 |
| `2026_oss_rst.hwpx` | bullet definition 누락 | `<hh:bullets>` 직렬화 |
| `el-school-001.hwpx` | borderFill image fill 누락 | `<hc:img binaryItemIDRef>` 직렬화 |
| `aift.hwpx` | borderFill image fill 누락 | `<hc:img binaryItemIDRef>` 직렬화 |

잔여 XFAIL은 `143E433F503322BD33.hwpx`, `k-water-rfp.hwpx` 두 건으로 남아 있으며 이번 PR 범위와 분리된다.

### 4.5 수동 시각 검증에서 발견한 별도 이슈

PR #1527 수동 시각 검증 중 `2026_oss_rst.hwpx` 1페이지 상단 `< 결과보고서 작성 안내 >` 제목 영역에 원본에는 없는 사각형 테두리가 roundtrip 결과에 생기는 현상을 확인했다.

이 현상은 PR base `0ec4b976`에서 생성한 `2026_oss_rst.rt.hwpx`와 PR head `cc7c60c8`에서 생성한 `2026_oss_rst.rt.hwpx` 양쪽에서 동일하게 재현된다. 따라서 #1527이 새로 만든 regression이 아니라 기존 HWPX roundtrip의 border/style 보존 문제로 판단한다.

또한 기존 `render-diff`/`visual_roundtrip_baseline`은 이 1페이지 차이를 잡지 못했다. 해당 게이트가 주로 RenderNode 구조와 bbox 변위를 비교하기 때문에, 원본에 없던 선/테두리/stroke 스타일이 생겨도 bbox나 구조가 크게 변하지 않으면 통과할 수 있다. 이 한계와 기존 roundtrip 버그는 #1531로 분리 추적한다.

## 5. 로컬 검증 결과

검증 worktree: `/private/tmp/rhwp-pr1527-head`

검증 head: `cc7c60c8`

| 명령 | 결과 |
|---|---|
| `cargo fmt --check` | PASS |
| `git diff --check 0ec4b976..HEAD` | PASS |
| `cargo clippy --all-targets -- -D warnings` | PASS |
| `cargo test --lib serializer::hwpx::header` | PASS, 25 passed |
| `cargo test --lib serializer::hwpx::shape` | PASS, 14 passed |
| `cargo test --lib serializer::hwpx::picture` | PASS, 18 passed |
| `cargo test --lib serializer::hwpx::package_check` | PASS, 8 passed |
| `cargo test --test issue_1058_textbox_list_header` | PASS, 10 passed |
| `cargo test --test hwpx_roundtrip_baseline` | PASS, 4 passed |
| `cargo test --test visual_roundtrip_baseline` | PASS, 3 passed |

로컬 검증 중 `/private/tmp` worktree 생성 시 `pdf-large/hwpx/2026_oss_rst.pdf`에 대해 Git LFS pointer 경고가 한 번 출력됐다. PR 변경 파일이나 HWPX serializer 동작과 직접 관련 없는 기존 LFS 상태로 판단한다.

## 6. 최신 devel 반영

현재 `upstream/devel`은 `87959e46`까지 진행되어 PR base `0ec4b976` 이후 #1524가 추가로 merge되어 있다. GitHub도 PR #1527을 `BEHIND`로 표시한다.

사전 merge-tree 확인에서는 충돌이 관찰되지 않았고, #1524 변경과 #1527 변경 영역은 분리되어 있다. 그래도 최종 merge 전에는 다음 중 하나를 선택해야 한다.

1. PR head에 최신 `devel` merge commit을 올리고 CI를 다시 확인한다.
2. 저장소 정책상 허용된다면 `BEHIND` 상태를 유지한 채 maintainer merge를 진행한다.

외부 contributor의 contribution 보존 관점에서는 원 contributor의 두 commit을 rewrite/squash하지 않는 방식이 가장 명확하다. 리뷰 문서 커밋이나 최신화 커밋이 필요하다면 maintainer의 별도 commit으로 추가하고, 원 contributor commit의 author/commit message는 유지하는 편이 좋다.

## 7. GitHub review 권고

GitHub review는 **Approve 가능**으로 판단한다. Inline blocking comment는 남길 필요가 없다.

리뷰 코멘트에 남길 핵심은 다음 정도면 충분하다.

```text
Reviewed the serializer changes for #1525/#1526. The patch fills the missing HWPX round-trip pieces in a narrow way: bullets are emitted in the expected header order, and borderFill image fills now resolve binData through SerializeContext before writing hc:img.

Local checks passed on cc7c60c8: fmt, clippy, focused serializer tests, package_check, hwpx_roundtrip_baseline, and visual_roundtrip_baseline. No blocking findings.

While doing manual visual comparison, I noticed an existing roundtrip issue in `2026_oss_rst.hwpx`: page 1 gets an extra rectangle border around the title area in both base and head roundtrip outputs. Since it reproduces before this PR, I do not consider it a #1527 regression; I filed it separately in #1531. It also highlights a limitation of the current geometry-based visual gate for stroke/style-only differences.

The PR is still behind devel, so please update/merge with latest devel or confirm maintainer merge policy before final merge. I would preserve the contributor-authored commits rather than rewriting them.
```

## 8. Merge 전 체크리스트

- [ ] 최신 `devel` 반영 방식을 결정한다.
- [ ] 최신화 커밋을 올린다면 새 CI 완료 후 merge한다.
- [ ] contributor commit 보존을 위해 squash/rebase rewrite 여부를 명시적으로 피한다.
- [ ] merge 후 #1525/#1526이 자동 close되지 않으면 수동 comment/close 여부를 확인한다.
- [ ] #1531은 base/head 모두 재현되는 기존 roundtrip 버그이므로 #1527 blocker로 처리하지 않는다.

## 9. 결론

PR #1527은 변경 범위, 코드 구조, 테스트 결과 모두 이슈 #1525/#1526 해결 방향과 맞다. 남은 결정은 코드 품질 문제가 아니라 merge 운영 문제다. 최신 `devel` 반영과 CI 확인만 정리되면 merge 준비가 된 PR로 판단한다.
