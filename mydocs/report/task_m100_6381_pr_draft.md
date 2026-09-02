# Task M100 #6381 PR 초안

## 제목

```text
[CLI] test-caption의 검증 없는 성공을 차단
```

## 본문

```markdown
## 변경 요약

내부 진단 명령 `test-caption`이 고정 fixture의 캡션 변경을 하나도 검증하지 못해도 SVG와 `완료`를
남기고 exit 0으로 종료하던 false-pass를 제거합니다.

- 네 caption mutation 결과를 개별 추적하고 실패 원인을 stderr에 기록합니다.
- mutation 성공 대상도 Picture 종류와 caption 방향·세로 정렬·폭·간격을 정확히 확인합니다.
- mutation 또는 verification이 하나라도 실패하면 SVG를 만들기 전에 exit 1로 종료합니다.
- 네 대상이 모두 통과한 경우에만 기존 성공 stdout, SVG 파일명과 `완료`를 유지합니다.

## 회귀 계약

- 고정 대상이 없는 임의 실문서: exit 1, stderr 진단, `완료`·SVG 없음, panic 없음
- 일부 대상만 있는 합성 HWP: exit 1, 일부 mutation 성공 뒤에도 `완료`·SVG 없음
- 네 대상이 모두 있는 합성 HWP: exit 0, 기존 성공 stdout, SVG 1개 이상

합성 fixture는 공개 `HwpDocument` API와 저장소의 작은 PNG asset으로 만들며 별도 binary fixture는
추가하지 않습니다.

## 범위

고정 좌표를 자동 그림 탐색으로 일반화하지 않습니다. caption setter, renderer, layout, document model,
Render Diff workflow와 공개 CLI schema도 변경하지 않습니다.

## 검증

- [x] focused nextest: 3/3 pass
- [x] 전체 integration nextest: 8,660/8,660 pass, 43 skipped
- [x] `cargo clippy --locked --all-targets --target-dir target/pr-review -- -D warnings`
- [x] `cargo fmt --all -- --check`
- [x] integration manifest: 1,032 sources / 4,533 attrs / 48/48 targets
- [x] source-side unit tier: 4,221 tests / 299 modules
- [x] Markdown link check
- [x] `git diff --check`

Closes #6381
```

## 게시 경계

- base: `edwardkim/rhwp:devel`
- head: `postmelee:task_m100_6381-test-caption-false-pass`
- PR: [#6391](https://github.com/edwardkim/rhwp/pull/6391)
- code candidate: `988b9c85f021a082c96713ce16c51c97ba7f4864`
- 상태: Draft 생성 완료
- 작업지시자가 remote push와 Draft PR 생성을 승인했다.
- 번호 기반 [self-review 문서](../pr/archives/pr_6391_review.md)는 trailing docs-only commit으로 반영한다.
