# PR #1527 code review 계획서

- PR: https://github.com/edwardkim/rhwp/pull/1527
- 제목: `Task #1525 #1526: HWPX 직렬화 충실도 - 글머리표 + borderFill 이미지 채움`
- 작성자: `planet6897` (Jaeuk Ryu)
- 관련 이슈: #1525, #1526
- 작성일: 2026-06-25
- 처리 경로: 외부 contributor PR 일반 검토. 본 계획서 작성 당시 요청 범위는 로컬 review 계획서 작성이었으며 PR head push는 수행하지 않았다.
- base/head: `edwardkim/rhwp:devel` `0ec4b976` <- `planet6897/rhwp:pr-task1525` `cc7c60c8`
- 작성 시점 참고값: `MERGEABLE` / `BEHIND`, `maintainerCanModify=true`
- 규모: 3 files, +125 / -21

`draft`, `mergeable`, `head SHA`, `CI 상태`는 변하는 값이므로 최종 판단 전 최신 상태를 다시 확인한다.

## 1. 목적

PR #1527은 HWPX roundtrip serializer에서 누락되던 두 리소스를 보존하는 변경이다.

1. #1525: `doc_info.bullets`를 `<hh:bullets>`로 직렬화해 글머리표 마커 글리프 소실을 막는다.
2. #1526: `borderFill`의 image fill을 `<hc:imgBrush><hc:img binaryItemIDRef=.../></hc:imgBrush>`로 직렬화해 셀/쪽 배경 이미지 소실을 막는다.

이 계획서는 최종 review 문서가 아니라, 코드 리뷰에서 확인할 축과 검증 순서를 정리한다. 계획서 작성 단계에서는 문서 push, PR head 수정, GitHub review 제출을 하지 않았다.

## 2. 현재 PR 메타

| 항목 | 내용 |
|---|---|
| base | `devel` |
| head branch | `planet6897:pr-task1525` |
| head SHA | `cc7c60c8c4939cb7dfec8316ff309752cc538187` |
| commit 수 | 2 |
| 변경 파일 | `src/serializer/hwpx/header.rs`, `src/serializer/hwpx/shape.rs`, `tests/visual_roundtrip_baseline.rs` |
| GitHub CI | 작성 시점 head 기준 Build & Test, CodeQL 계열 통과. WASM Build skipped |
| merge 상태 | `MERGEABLE`이나 `BEHIND` |
| 연결 이슈 | #1525, #1526 둘 다 open. PR body는 `closes`를 쓰지만 API `closingIssuesReferences`는 비어 있어 merge 후 수동 확인 필요 |

현재 `upstream/devel`은 `87959e46`까지 진행되어 PR base `0ec4b976` 이후 #1524 변경이 추가돼 있다. 최종 review 전 최신 devel과의 merge simulation이 필요하다.

## 3. 커밋별 검토 범위

| 커밋 | 내용 | 주요 검토 축 |
|---|---|---|
| `bfb2bf8b` | `write_bullets` / `write_bullet` 추가, `write_header`에서 `numberings -> bullets -> paraProperties` 순서로 방출 | header schema 순서, bullet id/char/useImage/paraHead 값, 기존 parser 보존 범위와 일치 여부 |
| `cc7c60c8` | `write_fill_brush`에 `SerializeContext` 전달, `FillType::Image`에서 등록된 binData 참조 방출 | `borderFill` image fill의 3-way 정합, body shape fill call site 영향, 미등록 binData fallback |

`tests/visual_roundtrip_baseline.rs`는 위 두 변경으로 PASS 승격된 샘플을 `VISUAL_XFAIL`에서 제거한다. baseline 목록 변경은 코드 변경의 결과로 타당한지 별도 검증한다.

## 4. 코드 리뷰 체크리스트

### 4.1 HWPX header 순서와 schema 정합

- `write_header()`의 refList 방출 순서가 OWPML schema의 `numberings -> bullets -> paraProperties` 순서와 일치하는지 확인한다.
- `<hh:bullets itemCnt=...>`가 `doc_info.bullets.len()`과 일치하는지 확인한다.
- `<hh:bullet id=...>`가 실제 HWPX 관찰값과 같은 1-based id를 쓰는지 확인한다.
- `char` 속성이 `Bullet.bullet_char`를 정확히 보존하는지 확인한다.
- `useImage`가 schema boolean이지만 원본 샘플처럼 `"0"` / `"1"` 형식으로 허용되는지 확인한다.
- `paraHead level="0"`은 schema의 positiveInteger와는 어긋나지만, `hy-002`, `footnote-01`, `2026_oss_rst` 원본 HWPX도 bullet paraHead에서 `level="0"`을 사용한다. 따라서 blocker로 단정하지 말고 원본 호환값으로 기록한다.

### 4.2 Bullet serializer의 보존 범위

- 현재 HWPX parser `parse_bullet_hwpx`는 `char`와 `useImage`만 읽고 자식 `paraHead`, `img`, `checkedChar`는 skip한다.
- PR writer도 현재 IR이 보존하는 `char`와 `useImage` 중심으로 방출한다.
- `checkedChar`와 image bullet의 실제 이미지 참조는 현재 parser/IR이 보존하지 않으므로 이번 PR 범위 밖으로 분류한다.
- 다만 `useImage=1`이 들어왔을 때 `<hh:bullet useImage="1">`만 쓰고 자식 `<hh:img>`를 쓰지 않는 구조가 한컴/HWPX 소비자에게 어떤 의미인지 리스크로 기록한다.
- 기존 #1058 parser 회귀 테스트와 충돌하지 않는지 확인한다.

### 4.3 `imgBrush` / `binaryItemIDRef` 정합

- `write_border_fills()`가 `SerializeContext`를 `write_border_fill()`로 전달하고, `write_border_fill()`이 `write_fill_brush()`에 전달하는 흐름을 확인한다.
- `FillType::Image`에서 `ctx.resolve_bin_id(img.bin_data_id)`가 성공할 때만 `<hc:img>`를 방출하는지 확인한다.
- 방출되는 `binaryItemIDRef`가 `content.hpf` manifest id와 ZIP `BinData/imageN.ext` entry에 대응하는지 확인한다.
- `bright`, `contrast`, `effect`가 parser의 `ImageFill` 값과 역매핑되는지 확인한다.
- 미등록 `bin_data_id`일 때 기존처럼 빈 `<hc:imgBrush mode=.../>`를 유지해 잘못된 참조를 만들지 않는지 확인한다.

### 4.4 `write_fill_brush` 공용 함수의 blast radius

- `write_fill_brush` call site는 `header.rs`의 `borderFill`과 `shape.rs`의 `write_rect`다.
- `write_rect`도 `ctx`를 전달받게 되었으므로 body shape fill의 `FillType::Image` 동작이 바뀔 수 있다.
- 현재 `parse_shape_fill_brush`는 `imgBrush`의 mode만 읽고 내부 `<hc:img>`의 `binaryItemIDRef`를 `Fill.image.bin_data_id`로 캡처하지 않는다. 따라서 body shape fill은 대체로 `bin_data_id=0`이고 `ctx.resolve_bin_id(0)` 실패로 기존 빈 `imgBrush` fallback을 유지할 가능성이 높다.
- 이 전제를 실샘플 또는 unit test로 재확인한다. 만약 body shape image fill에서 `bin_data_id`가 들어오는 경로가 있으면 PR 설명 범위보다 동작 변화가 넓어지므로 별도 검토한다.

### 4.5 Visual baseline 승격의 타당성

- `VISUAL_XFAIL`에서 제거된 샘플 5개가 실제로 head에서 PASS하는지 확인한다.
- 제거 사유가 두 변경과 직접 연결되는지 확인한다.
  - `hy-002`, `footnote-01`, `2026_oss_rst`: bullet definition 누락 해소.
  - `el-school-001`, `aift`: borderFill image fill 누락 해소.
- 잔여 `VISUAL_XFAIL`이 `143E433F503322BD33.hwpx`, `k-water-rfp.hwpx` 두 건으로 남는 것이 현재 기준과 맞는지 확인한다.
- baseline 목록을 줄이는 변경이므로 반드시 head에서 `visual_xfail_entries_still_fail`까지 같이 확인한다.

## 5. 로컬 검증 계획

검증은 현재 작업트리를 직접 오염시키지 않도록 `/private/tmp` 분리 worktree에서 수행한다.

권장 worktree:

| 구분 | 위치 | 커밋 |
|---|---|---|
| base | `/private/tmp/rhwp-pr1527-base` | `0ec4b976` |
| head | `/private/tmp/rhwp-pr1527-head` | `cc7c60c8` |
| latest merge simulation | `/private/tmp/rhwp-pr1527-merge` | `refs/remotes/codex/pr-1527` + `upstream/devel` |

필수 명령:

```bash
cargo fmt --check
git diff --check 0ec4b976..HEAD
cargo test --test visual_roundtrip_baseline
cargo test --test hwpx_roundtrip_baseline
cargo test --test issue_1058_textbox_list_header
cargo test --lib serializer::hwpx::header
cargo test --lib serializer::hwpx::shape
cargo clippy --all-targets -- -D warnings
```

상황에 따라 추가:

```bash
cargo test --lib serializer::hwpx::package_check
cargo test --lib serializer::hwpx::picture
cargo test --release --test visual_roundtrip_baseline
```

대형 샘플 포함 테스트는 시간이 걸릴 수 있으므로, 첫 검증은 debug profile로 수행하고 최종 merge 전에는 release 또는 CI 결과와 함께 판단한다.

## 6. 직접 산출물 검사 계획

대표 샘플을 roundtrip serialize한 뒤 `Contents/header.xml`, `content.hpf`, `BinData/`를 직접 확인한다.

### Bullet 대표 샘플

- `samples/hwpx/hy-002.hwpx`
- `samples/hwpx/footnote-01.hwpx`
- `samples/hwpx/2026_oss_rst.hwpx`

확인 항목:

- roundtrip output header에 `<hh:bullets itemCnt=...>`가 존재한다.
- bullet chars가 원본과 동일하다. 예: `❏`, `※`, soft-hyphen, `❍`.
- `ParaShape`의 `heading type="BULLET" idRef=...`가 참조하는 bullet id가 존재한다.
- reparse 후 `doc_info.bullets.len()`과 `bullet_char`가 보존된다.
- render geometry diff에서 구조 불일치가 사라진다.

### imgBrush 대표 샘플

- `samples/hwpx/el-school-001.hwpx`
- `samples/hwpx/aift.hwpx`

확인 항목:

- roundtrip output header의 image borderFill에 `<hc:imgBrush mode=...>` 내부 `<hc:img binaryItemIDRef="imageN">`가 존재한다.
- `imageN`이 `content.hpf` manifest에 존재한다.
- `BinData/imageN.ext` ZIP entry가 존재한다.
- `bright`, `contrast`, `effect`가 원본/IR과 일치한다.
- reparse 후 `BorderFill.fill.image.bin_data_id`가 0으로 떨어지지 않는다.

## 7. 최신 devel 반영 계획

PR은 작성 시점 `BEHIND`다. 현재 `upstream/devel`에는 #1524 merge가 포함되어 있다.

검토 단계:

1. `git merge-tree 0ec4b976 refs/remotes/codex/pr-1527 upstream/devel`로 충돌 여부를 확인한다.
2. 충돌이 없으면 review 문서에는 "최신 devel과 충돌 가능성 낮음"으로 기록하되, 실제 PR head update는 작업지시자 승인 전 수행하지 않는다.
3. 최종 merge 준비 단계에서 `BEHIND`를 해소할지, admin merge로 처리할지 별도 승인받는다.

사전 관찰: merge-tree는 충돌을 보고하지 않았고 #1524 변경과 #1527 변경 영역이 분리되어 있다. 단, 최종 판단 전 최신 upstream을 다시 fetch해야 한다.

## 8. 예상 리스크 분류

### Blocking 후보

- roundtrip output에 `<hh:bullets>`가 나오지만 referenced bullet id와 실제 bullet id가 어긋나는 경우.
- `<hc:img binaryItemIDRef>`가 manifest 또는 ZIP entry와 불일치하는 경우.
- `visual_roundtrip_baseline`에서 새 PASS 승격 샘플이 여전히 실패하는 경우.
- `write_fill_brush` signature 변경으로 다른 serializer call site가 누락되어 컴파일 또는 동작이 깨지는 경우.
- 최신 devel merge에서 serializer/header/baseline 영역 충돌이 발생하는 경우.

### Non-blocking 후보

- `checkedChar`, image bullet의 자식 `<hh:img>` 미보존. 현재 parser/IR이 보존하지 않는 범위다.
- bullet `paraHead`의 세부 속성은 원본 그대로가 아니라 최소 뼈대다. parser가 현재 무시하는 범위다.
- body shape fill의 image reference는 여전히 미캡처로 빈 `imgBrush` fallback을 유지할 가능성이 높다. 이번 PR 목적은 borderFill image fill이다.
- `imgBrush`의 `alpha`를 항상 `"0"`으로 쓰는 점. 기존 parser가 header borderFill image alpha를 별도 보존하지 않는다면 현재 범위에서는 허용 가능하지만 확인 필요하다.
- PR body는 `closes #1525`, `closes #1526`를 쓰지만 GitHub API `closingIssuesReferences`가 비어 있다. merge 후 issue state 확인이 필요하다.

## 9. review 문서 작성 계획

최종 code review 문서(`mydocs/pr/pr_1527_review.md`)에는 다음을 포함한다.

1. PR 메타와 변경 범위.
2. #1525/#1526 원인과 PR 수정 지점의 대응 관계.
3. 코드 리뷰 결과.
4. 로컬 검증 결과.
5. 최신 devel 반영 필요 여부.
6. blocking findings 유무.
7. merge 전 조건.
8. merge 후 #1525/#1526 close 확인 계획.
9. contributor contribution 보존 관점의 merge 방식 권고. 원 contributor 2 commits는 rewrite/squash하지 않고 유지하는 방향을 기본으로 한다.

문서 push는 이 계획서와 최종 review 문서 검토 후, 작업지시자 승인 전에는 수행하지 않는다.

## 10. 현재 결론

현재까지의 1차 코드 독해 기준으로 변경 방향은 이슈 원인과 직접 맞는다. 다만 최종 review 전에는 다음 네 가지를 반드시 확인해야 한다.

1. bullet id/char/useImage가 roundtrip 후 참조와 함께 보존되는지.
2. `imgBrush`의 `binaryItemIDRef`가 manifest와 ZIP entry에 3-way로 정합하는지.
3. visual baseline 승격 5건이 실제 head에서 PASS하는지.
4. 최신 `upstream/devel`과의 merge 결과가 충돌 없이 유지되는지.
