# PR #3305 검토 기록

## 라우팅

```text
base route: collaborator_external_pr
modifiers: intake_and_review, local_validation, rework_and_exceptions
loaded documents: pr_review_workflow.md, pr_review/README.md,
  collaborator_external_pr.md, intake_and_review.md, local_validation.md,
  rework_and_exceptions.md
current head: 작성 시점 참고값 3009a461ce3944d534aa2061c40f25bde80acb16
```

## PR metadata

| 항목 | 작성 시점 참고값 |
| --- | --- |
| PR | [#3305](https://github.com/edwardkim/rhwp/pull/3305) |
| 제목 | `fix(parser): WMF/CFB 메모리 안전 버그 3건` |
| 작성자 / base | `kevin9327` / `devel` |
| contributor 원 commit / 현재 head | `1b02247` / `pr/task-static-bugs-bundle` · `3009a461ce3944d534aa2061c40f25bde80acb16` |
| 관련 이슈 | [#3301](https://github.com/edwardkim/rhwp/issues/3301) |
| 규모 | 4 files, +139/-6, 1 commit |
| 원 PR 상태 | `MERGEABLE`, `CLEAN`; maintainer 수정 허용, reviewer `jangster77` 요청 완료 |
| 검토 branch | `review/kevin9327-parser-safety-20260726`, 최신 `upstream/devel` `0324cd9` 기준 |

초기 조사 head `1b02247` 뒤 contributor가 `devel`을 merge한 `3009a46`을 push했다. 최신 head는
`0324cd9`를 포함하며 CI·CodeQL·Render Diff가 모두 성공했다. 이 상태와 SHA도 merge 전 다시 확인한다.

## 변경 검토와 메인터너 보정

- 원 contributor commit `1b02247`은 검토 branch에 `d307bac`으로 충돌 없이 cherry-pick했다.
- PolyPolygon의 총점 누산을 `u32`로 올린 변경은 유효하다. 기존 `u16` 누산은 65,536 이상에서
  debug panic 또는 release wrap으로 이어져 `aPoints`를 덜 읽을 수 있었다.
- Lenient CFB reader의 `visited_fat_sids`는 header DIFAT과 추가 DIFAT 양쪽에서 같은 FAT SID를
  한 번만 읽게 하므로, 손상 CFB가 같은 물리 FAT sector를 반복 기재해 `fat` 벡터를 부풀리는 것을 막는다.
- 원 PR의 음수 `BitmapInfoHeader::width()` 절대값 변경은 제거했다. 이미 PR base의
  `BitmapInfoHeaderInfo::parse()`가 `width <= 0`을 오류로 거부하므로, 직접 struct를 만드는 단위
  테스트만 통과하는 중복 변경이었다. MS-WMF도 `Width`가 양수여야 한다고 규정한다.
- maintainer 보정 commit `e7a34e0`은 위 중복 코드·테스트·보고서 주장을 제거하고, 두 실제 결함에
  최소 회귀 테스트를 추가했다. contributor 원 commit은 rebase·amend·force-push하지 않는다.

## 검증

- `CARGO_TARGET_DIR=target/review-kevin9327-parser-safety-20260726 CARGO_INCREMENTAL=0`
  `cargo test --profile release-test --lib`: **2921 passed, 0 failed, 7 ignored**
- 같은 target의 `cargo test --profile release-test --tests`: 통과
- `total_point_count_above_u16_does_not_wrap`: 65,536개 `PointS`를 실제로 전부 읽는지 통과
- `lenient_open_deduplicates_fat_sector_id_from_difat`: header와 추가 DIFAT의 중복 FAT SID를 한 번만
  반영하는지 통과
- `cargo fmt --check`, `git diff --check`, `cargo clippy --all-targets -- -D warnings`: 통과
- Rust parser 변경만 있으며 renderer/layout, fixture, PDF는 바뀌지 않아 visual sweep·IR field sweep
  baseline 갱신·WASM build는 적용 대상이 아니다.

## 최종 권고

**메인터너 보정 후 수용 가능**이다. 보정 commit과 아래 review 기록을 최신 contributor source head
`3009a46` 위에 별도 commit으로 적용한다. 그 새 head의 full CI와 작업지시자 push·merge 승인이 최종
조건이다.

이슈 [#3301](https://github.com/edwardkim/rhwp/issues/3301)의 제목·본문은 3건을 주장하므로, 실제 push
후 2건으로 정정하는 maintainer comment 또는 편집이 필요하다.
