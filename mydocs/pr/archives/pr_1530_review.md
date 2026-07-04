# PR #1530 검토 기록 — HWPX OLE 개체 데이터 참조 보존

- PR: https://github.com/edwardkim/rhwp/pull/1530
- 제목: `Task #1529: HWPX 직렬화 — OLE 개체(<hp:ole>) 데이터 참조 보존`
- 작성자: `planet6897` (Jaeuk Ryu)
- 관련 이슈: #1529
- 작성일: 2026-06-26
- 처리 경로: collaborator-mediated 외부 PR 처리 경로. `maintainerCanModify=true`이므로 review 문서를
  PR head의 `mydocs/pr/archives/`에 직접 포함한다.
- base/head: `edwardkim/rhwp:devel` `48e9670` <- `planet6897/rhwp:pr-task-ole` `d52a165c`
- 규모: 3 files, +85 / -10

`draft`, `mergeable`, `head SHA`, `CI 상태`는 변하는 값이므로 최종 판단 전 최신 상태를 다시 확인한다.

## 1. 목적

PR #1530은 HWPX roundtrip serializer에서 OLE 개체가 데이터 참조를 잃고 placeholder로 강등되는
문제를 다룬다.

대상 증상:

- `samples/hwpx/143E433F503322BD33.hwpx`
- roundtrip 후 1페이지 차트/OLE가 `RawSvg`에서 `Placeholder`로 바뀌는 구조 불일치
- 연결 이슈: #1529 `HWPX 직렬화: OLE 개체(<hp:ole>) 데이터 참조 미보존 — placeholder 강등`

이 문서는 코드 리뷰에서 확인할 축, 검증 순서, 최종 확인 결과를 함께 기록한다.

## 2. 현재 PR 메타

| 항목 | 내용 |
|---|---|
| state | open |
| draft | false |
| mergeable | `MERGEABLE` |
| base | `devel` |
| head branch | `planet6897:pr-task-ole` |
| head SHA | `d52a165c4e3d0c899b88de47ac38efecb7119a99` |
| commit 수 | 3 (`e6598e6a`, devel merge 2건) |
| 변경 파일 | `src/serializer/hwpx/section.rs`, `src/serializer/hwpx/shape.rs`, `tests/visual_roundtrip_baseline.rs` |
| GitHub comments/review threads | 작성 시점 없음 |
| 연결 이슈 | PR body는 `closes #1529`를 언급하지만 GitHub API `closingIssuesReferences=[]`; merge 후 이슈 상태 수동 확인 필요 |

PR body는 원래 "#1527 위에 쌓인 PR"이라고 설명한다. 최신 확인 기준 #1527은
2026-06-25 04:02 UTC에 merge되었으므로, #1530 검토에서는 다음을 확인한다.

1. #1527 변경이 base에 이미 들어온 뒤에도 #1530 고유 diff만 남는지.
2. `VISUAL_XFAIL` 잔여 목록이 `k-water-rfp.hwpx`만 남는 방향이 맞는지.
3. `143E433F503322BD33.hwpx` 승격이 #1530 OLE serializer 변경과 직접 연결되는지.

## 3. 커밋별 검토 범위

| 커밋 | 내용 | 주요 검토 축 |
|---|---|---|
| `e6598e6a` | `write_ole` 신설, OLE dispatch 연결, visual baseline 승격 | `<hp:ole>` XML 구조, binData 참조, extent/shape_attr 보존, 기존 shape serializer 영향 |
| `639a75a` | `devel` merge | #1527 merge 이후 diff 오염 여부 |
| `d52a165` | `devel` merge | 최신 base 정합, CI 재실행 상태 |

## 4. 코드 리뷰 체크리스트

### 4.1 OLE serializer 구조

- `section.rs`가 `ShapeObject::Ole`를 기존 공용 shape writer가 아니라 `write_ole()`로 보내는지 확인한다.
- `<hp:ole>`에 필요한 공통 속성과 OLE 전용 속성이 원본/IR과 맞게 직렬화되는지 확인한다.
- `objectType`, `drawAspect`, `binaryItemIDRef`, `<hc:extent>`, shape attr 블록, lineShape,
  size/position/outMargin/caption 방출 순서가 HWPX 관찰값과 호환되는지 확인한다.
- picture writer와 공통 helper를 공유하는 경우 OLE에 맞지 않는 picture 전용 속성이 섞이지 않는지 확인한다.
- caption이나 lineShape가 없는 OLE에서 빈 XML 또는 잘못된 기본값을 만들지 않는지 확인한다.

### 4.2 binData 3-way 정합

- `OleShape.bin_data_id`가 `SerializeContext::resolve_bin_id()`를 통해 `content.hpf` manifest id로 변환되는지 확인한다.
- roundtrip 결과의 `binaryItemIDRef`가 `Contents/content.hpf`와 `BinData/*` ZIP entry에 모두 대응하는지 확인한다.
- `bin_data_id`가 0이거나 미등록일 때 잘못된 `image0`/빈 참조가 생기지 않는지 확인한다.
- parser가 roundtrip 결과를 다시 읽었을 때 `OleShape.bin_data_id`가 0으로 떨어지지 않는지 확인한다.
- OLE 데이터 보존이 chart RawSvg 렌더 경로까지 이어져 `RawSvg -> Placeholder` 구조 손실이 사라지는지 확인한다.

### 4.3 serializer 영향 범위

- 변경이 HWPX serializer의 OLE 경로에만 국한되는지 확인한다.
- picture/shape/group/table writer의 공통 helper 변경으로 다른 객체 직렬화가 바뀌지 않는지 확인한다.
- `write_ole()`가 기존 `render_common_shape_xml` 경로에서 얻던 size/position/outMargin/caption 정보를 빠뜨리지 않는지 확인한다.
- XML namespace와 element 이름이 `hp`, `hc` 기대 위치와 일치하는지 확인한다.

### 4.4 visual baseline 승격

- `tests/visual_roundtrip_baseline.rs`에서 `143E433F503322BD33.hwpx`가 `VISUAL_XFAIL`에서 제거되는지 확인한다.
- 제거 후 `cargo test --test visual_roundtrip_baseline` 전체가 통과하는지 확인한다.
- `visual_xfail_entries_still_fail` 기준으로 잔여 `k-water-rfp.hwpx`는 여전히 실패해야 한다.
- 승격 이유가 PR body의 `RawSvg -> Placeholder` 해소와 직접 대응하는지 `render-diff` 구조 delta로 확인한다.

### 4.5 이슈/운영

- #1529가 open 상태인지 최종 merge 전 다시 확인한다.
- PR body의 `closes #1529`가 자동 close로 잡히지 않는 경우 merge 후 수동 확인 절차를 남긴다.
- contributor commit은 rewrite/squash하지 않고 보존하는 방향을 기본으로 검토한다.
- PR head에 maintainer merge commit 2건이 있으므로 최종 review 문서에서 author/merge 이력을 명확히 기록한다.

## 5. 로컬 검증 계획

검증은 현재 작업트리를 직접 오염시키지 않도록 `/private/tmp` 분리 worktree에서 수행한다.
현재 로컬 작업트리에는 별도 수정이 있으므로 #1530 검증과 섞지 않는다.

권장 worktree:

| 구분 | 위치 | 커밋 |
|---|---|---|
| head | `/private/tmp/rhwp-pr1530-head` | `d52a165c` |
| merge simulation | `/private/tmp/rhwp-pr1530-merge` | 최신 `upstream/devel` + PR head |

준비 명령:

```bash
git fetch upstream pull/1530/head:codex/pr-1530
git worktree add -B pr1530-head /private/tmp/rhwp-pr1530-head codex/pr-1530
```

필수 검증:

```bash
cargo fmt --check
git diff --check upstream/devel..HEAD
cargo test --lib serializer::hwpx::shape
cargo test --test hwpx_roundtrip_baseline
cargo test --test visual_roundtrip_baseline
cargo test --test issue_1156_chart_column_flow
cargo clippy --all-targets -- -D warnings
```

OLE 대상 샘플 집중 검증:

```bash
cargo run --bin rhwp -- render-diff samples/hwpx/143E433F503322BD33.hwpx --via hwpx --max-disp 0.5
cargo run --bin rhwp -- hwpx-roundtrip samples/hwpx/143E433F503322BD33.hwpx -o output/poc/pr1530-roundtrip
```

roundtrip 산출물 직접 검사:

```bash
unzip -p output/poc/pr1530-roundtrip/143E433F503322BD33.rt.hwpx Contents/section0.xml | rg '<hp:ole|binaryItemIDRef|hc:extent'
unzip -p output/poc/pr1530-roundtrip/143E433F503322BD33.rt.hwpx Contents/content.hpf | rg 'BinData|binaryItem'
unzip -l output/poc/pr1530-roundtrip/143E433F503322BD33.rt.hwpx | rg 'BinData/'
```

## 6. 작업지시자 시각 검증 방법

검증 산출물은 승인 후 다음 경로에 만든다.

- `output/poc/pr1530-ole-visual/original/`
- `output/poc/pr1530-ole-visual/roundtrip/`
- `output/poc/pr1530-ole-visual/debug-original/`
- `output/poc/pr1530-ole-visual/debug-roundtrip/`

생성 명령:

```bash
cargo run --bin rhwp -- export-svg samples/hwpx/143E433F503322BD33.hwpx \
  -o output/poc/pr1530-ole-visual/original

cargo run --bin rhwp -- export-svg output/poc/pr1530-roundtrip/143E433F503322BD33.rt.hwpx \
  -o output/poc/pr1530-ole-visual/roundtrip

cargo run --bin rhwp -- export-svg samples/hwpx/143E433F503322BD33.hwpx \
  -p 0 --debug-overlay -o output/poc/pr1530-ole-visual/debug-original

cargo run --bin rhwp -- export-svg output/poc/pr1530-roundtrip/143E433F503322BD33.rt.hwpx \
  -p 0 --debug-overlay -o output/poc/pr1530-ole-visual/debug-roundtrip
```

작업지시자 확인 포인트:

1. `original`과 `roundtrip`의 1페이지 SVG를 나란히 연다.
2. 차트/OLE 영역이 roundtrip 후 회색 placeholder 또는 `OLE 개체`/`차트` 텍스트로 바뀌지 않는지 확인한다.
3. 차트 위치, 크기, 본문 흐름, 페이지 수가 원본과 눈에 띄게 달라지지 않는지 확인한다.
4. debug overlay에서 OLE 주변 문단/표 경계가 원본과 roundtrip 사이에 크게 이동하지 않는지 확인한다.
5. 가능하면 한컴 2022 편집기에서 원본 HWPX와 roundtrip HWPX를 열어 1페이지 차트 영역을 비교한다.

주의: `render-diff`와 SVG 비교는 rhwp 내부 roundtrip 회귀 게이트다. 한컴 편집기 직접 출력이 가능하면
그 결과가 시각 판정의 1차 권위 자료다.

## 7. 예상 리스크 분류

### Blocking 후보

- `binaryItemIDRef`가 `content.hpf` manifest 또는 ZIP `BinData` entry와 불일치하는 경우.
- roundtrip 후 `OleShape.bin_data_id`가 0으로 떨어져 placeholder가 계속 발생하는 경우.
- `visual_roundtrip_baseline`에서 `143E433F503322BD33.hwpx` 승격 후 실패하는 경우.
- `write_ole()`가 기존 공통 writer의 size/position/outMargin/caption을 누락해 배치가 바뀌는 경우.
- picture/shape serializer가 함께 회귀하는 경우.

### Non-blocking 후보

- OLE 내부 데이터 자체의 완전한 의미 해석은 이번 PR 범위 밖이다.
- 한컴 정답지 대비 차트 내부 그래픽 충실도 문제는 기존 OLE chart renderer 범위와 분리한다.
- `k-water-rfp.hwpx` visual xfail은 대형 복합 잔여 이슈로 이번 PR blocker로 보지 않는다.
- GitHub API가 #1529 자동 close 연결을 잡지 못하면 merge 후 수동 확인 대상으로 둔다.

## 8. review 문서 작성 계획

최종 code review 문서에는 다음을 포함한다.

1. PR 메타와 #1527 스택 해소 상태.
2. #1529 원인과 PR 수정 지점의 대응 관계.
3. `write_ole()` 코드 리뷰 결과.
4. binData/content.hpf/BinData ZIP entry 3-way 정합 확인.
5. 로컬 테스트와 `render-diff` 결과.
6. 작업지시자 시각 검증 결과.
7. blocking findings 유무.
8. merge 전 조건과 merge 후 #1529 close 확인 계획.

문서 커밋 push, GitHub Actions 확인, merge, PR merge comment, #1529 close comment는 2026-06-26
작업지시자 요청에 따라 진행한다.

## 9. 현재 결론

현재까지의 PR 메타와 검증 결과 기준으로 변경 방향은 #1529 원인과 직접 맞는다.
최종 review 전 확인 항목은 다음과 같이 완료했다.

1. `<hp:ole>`의 `binaryItemIDRef`가 PR head roundtrip 결과에서 보존됨을 확인했다.
2. `content.hpf`와 ZIP `BinData` entry가 참조와 3-way로 정합함을 확인했다.
3. `143E433F503322BD33.hwpx`가 `visual_roundtrip_baseline`에서 PASS로 승격됨을 확인했다.
4. 작업지시자 시각 검증에서 PR head roundtrip 후 OLE 차트가 placeholder로 강등되지 않음을 확인했다.
5. PR 반영 전 `upstream/devel` roundtrip 파일은 `RawSvg -> Placeholder` 구조 불일치가 발생함을 확인했다.
