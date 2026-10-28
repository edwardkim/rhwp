# Task #1585 최종 보고서 — HWPX 캡션 내 플로팅 이미지 렌더링 지원

## 개요

- Issue: #1585
- Parent issue: #1270
- 선행 PR: #1551
- 브랜치: `local/task1585`
- Worktree: `/private/tmp/rhwp-task1585`
- 기준 커밋: `upstream/devel` `a94e2051`

#1585는 #1270에서 분리된 후속 작업이다. #1551은 캡션 내 인라인 이미지 스레딩을 복구했고, 이번 작업은 캡션 내부 `textWrap="TOP_AND_BOTTOM"` 그림이 화면에 방출되지 않는 문제를 처리한다.

## 기준 정렬

Stage 3 시작 전에 #1551 merge 상태를 확인했다.

```text
state=MERGED
mergedAt=2026-06-27T04:12:17Z
mergeCommit=a94e20518b486caabd82de824784bdf25a0e1c87
```

이후 `git fetch upstream devel` 및 `git rebase upstream/devel`을 수행했고, 현재 브랜치는 `a94e2051` 기준이다. 따라서 이번 변경은 #1551 변경을 중복 포함하지 않는다.

## 구현 내용

### 캡션 내부 `TopAndBottom` 그림 렌더링

`src/renderer/layout/picture_footnote.rs`의 `layout_caption()`에서 caption paragraph 조판 직후 `TopAndBottom` picture 전용 helper를 호출하도록 했다.

핵심 정책:

- `Control::Picture` 중 `pic.common.text_wrap == TextWrap::TopAndBottom`인 그림만 대상으로 한다.
- #1551 인라인 경로에서 이미 같은 control이 등록된 경우 중복 방출하지 않는다.
- caption content box와 paragraph line segment를 기준으로 좌표를 계산한다.
- 이미지 payload와 `ImageNode` 생성은 기존 `layout_picture()` 경로를 재사용한다.
- 샘플처럼 `treat_as_char=true`와 `TopAndBottom`이 함께 있는 경우에도, 플로팅 배치용 clone은 non-TAC로 정규화해 렌더링한다.

### depth 1 중첩 표 caption 보강

#1270 첨부 실샘플의 `image2`는 top-level 표 caption이 아니라 내부 표의 TOP caption 안에 있었다. 기존 `table_layout.rs`는 table caption 렌더링을 `depth == 0`으로 제한하고 있었기 때문에, caption picture helper만 추가해도 실샘플의 caption 자체가 렌더링되지 않았다.

이번 변경은 범위를 좁혀 다음 경우만 caption 렌더링을 허용했다.

- `depth == 0`: 기존 동작 유지
- `depth == 1`이고 caption 안에 `TopAndBottom` picture가 있는 경우: caption 렌더링

중첩 caption의 `CellContext`는 기존 caption 식별 센티널인 `cell_index=65534`를 유지하도록 조정했다.

## 테스트

신규 테스트:

```text
tests/issue_1585_caption_floating_image.rs
```

검증 내용:

- top-level table caption 내부 `TopAndBottom` picture가 payload 포함 `ImageNode`로 1회 방출된다.
- depth 1 nested table caption 내부 `TopAndBottom` picture가 caption context와 함께 1회 방출된다.
- caption image가 중복 방출되지 않는다.

## 실샘플 확인

대상:

```text
/private/tmp/issue1270/서울 문화예술단체 지원사업.hwpx
```

#1551 merge 직후, #1585 변경 전 page 0 SVG:

```text
<image> count: 1
```

#1585 변경 후 page 0 SVG:

```text
<image> count: 2
```

추가된 두 번째 image는 `image2` / `SEOUL MY SOUL` 로고에 해당한다.

```xml
<image x="499.47333333333336"
       y="238.9333333333333"
       width="214.66666666666666"
       height="35.266666666666666"
       .../>
```

기존 `image1`도 유지됐다.

## rhwp-studio 시각 검증

작업지시자가 로컬 `rhwp-studio` 웹서버에서 #1270 첨부 샘플을 직접 로드해 시각 검증을 완료했다.

검증 서버:

```text
http://127.0.0.1:7701/
```

검증 시각:

```text
2026-06-27 13:52 KST
```

## 검증 결과

다음 명령이 통과했다.

```text
cargo fmt --check
cargo test --test issue_1585_caption_floating_image
cargo test --test issue_1270_caption_inline_image
cargo test --test issue_530
cargo test --test issue_1459_topbottom_picture_reflow
cargo test --test issue_1139_inline_picture_duplicate
cargo test --test issue_1352_table_cell_tac_picture_text
cargo test --test issue_1486_hwpx_partial_tac_table
cargo test --lib
cargo clippy --lib -- -D warnings
git diff --check
```

`cargo test --lib` 결과:

```text
1959 passed; 0 failed; 6 ignored
```

## 변경 파일

소스:

```text
src/renderer/layout/picture_footnote.rs
src/renderer/layout/table_layout.rs
tests/issue_1585_caption_floating_image.rs
```

문서:

```text
mydocs/orders/20260627.md
mydocs/plans/task_m100_1585.md
mydocs/plans/task_m100_1585_impl.md
mydocs/working/task_m100_1585_stage1.md
mydocs/working/task_m100_1585_stage2.md
mydocs/working/task_m100_1585_stage3.md
mydocs/report/task_m100_1585_report.md
```

## PR 초안

제목:

```text
Task #1585: HWPX 캡션 내 플로팅 이미지 렌더링 지원
```

본문:

```markdown
## 요약

- 캡션 paragraph 내부 `TopAndBottom` picture를 caption 영역 기준으로 렌더링합니다.
- depth 1 중첩 표 caption 중 `TopAndBottom` picture가 있는 경우만 caption 렌더링을 허용합니다.
- top-level / nested table caption 플로팅 이미지 회귀 테스트를 추가했습니다.

## 배경

#1551은 #1270의 (a) 범위였던 캡션 내 인라인 이미지 스레딩을 복구했습니다.

이번 PR은 후속 이슈 #1585의 범위인 (b) 캡션 내 플로팅 이미지 렌더링을 다룹니다. #1270 첨부 샘플의 `image2` / `SEOUL MY SOUL` 로고는 내부 표의 TOP caption 안에 있는 `textWrap="TOP_AND_BOTTOM"` 그림이므로, #1551 merge 이후에도 별도 처리가 필요했습니다.

## 구현

- `layout_caption()`에서 caption paragraph 조판 후 `TopAndBottom` picture 전용 렌더링 helper를 호출합니다.
- #1551 인라인 경로에서 이미 등록된 control은 중복 방출하지 않습니다.
- caption 배치용 picture clone은 non-TAC로 정규화해 기존 `layout_picture()` 경로를 재사용합니다.
- nested table caption은 `depth == 1`이고 caption 안에 `TopAndBottom` picture가 있는 경우에만 렌더링합니다.
- 중첩 caption도 기존 caption 센티널인 `cell_index=65534`를 유지합니다.

## 검증

- `cargo fmt --check`
- `cargo test --test issue_1585_caption_floating_image`
- `cargo test --test issue_1270_caption_inline_image`
- `cargo test --test issue_530`
- `cargo test --test issue_1459_topbottom_picture_reflow`
- `cargo test --test issue_1139_inline_picture_duplicate`
- `cargo test --test issue_1352_table_cell_tac_picture_text`
- `cargo test --test issue_1486_hwpx_partial_tac_table`
- `cargo test --lib`
- `cargo clippy --lib -- -D warnings`

## 실샘플 확인

로컬 #1270 첨부 샘플의 page 0 SVG에서 `<image>` 개수가 #1551 merge 직후 1개에서 이번 변경 후 2개로 증가하는 것을 확인했습니다. 추가 방출된 image는 `image2` / `SEOUL MY SOUL` 로고입니다.

## 시각 검증

로컬 `rhwp-studio` 웹서버에서 작업지시자가 #1270 첨부 샘플을 직접 로드해 시각 검증을 완료했습니다.

## 범위

이번 PR은 #1585의 캡션 내부 `TopAndBottom` 플로팅 이미지 렌더링에 집중합니다. caption 내부 모든 floating object의 일반화나 caption 재귀 렌더링 확장은 포함하지 않습니다.
```

## 남은 작업

커밋, push, PR 생성은 작업지시자 승인 후 진행한다. 이슈 close는 별도 승인 전까지 수행하지 않는다.
