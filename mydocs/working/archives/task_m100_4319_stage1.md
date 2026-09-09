# task_m100_4319 Stage 1 — HWPX Chart·OLE 캡션 파싱 유실

- **이슈**: [#4319](https://github.com/edwardkim/rhwp/issues/4319)
- **브랜치**: `fix/issue-4319-chart-ole-caption`
- **분기 기준**: `upstream/devel` `e48fe8694`
- **상태**: 로컬 검증 통과(타이밍 flake 1건 제외), PR 게시
- **기록일**: 2026-08-10 KST

## 1. 결함 — 파서가 캡션을 아예 안 읽는다

`parse_hp_chart_element`/`parse_hp_ole_element` 가 공유하는 `parse_common_shape_children` 에
`<hp:caption>` 처리가 **하나도 없었다.** `drawing.caption` 조차 채워진 적이 없다 — 렌더 쪽
`shape_layout.rs` 의 폴백이 암시하던 것과 달랐다.

## 2. 게이트가 왜 못 잡았나 — 비교기는 멀쩡했다

이게 이슈의 핵심 질문이었다. 답은 **비교기가 아니라 파서**다.

`roundtrip.rs::shape_caption` 은 이미 `Chart(x) => &x.caption, Ole(x) => &x.caption` 을 올바로
읽고, `diff_documents`/`IrDifference::ObjectCaption` 이 모든 `ShapeObject` 변형의 캡션을 균일하게
비교한다. **비교기는 처음부터 정상이었다.**

진짜 원인은 파서가 그 필드를 처음 파싱부터 `None` 으로 남겨, 왕복 전후 비교가 `None == None` 으로
항상 같았던 것이다. 비교기 코드를 건드리지 않고 IR 수준 테스트 2건
(`issue4319_ole/chart_caption_loss_in_gate`)으로 이를 증명했다 — #1403 이 남긴 커버리지 공백을
메운다.

## 3. 두 번째 결함 — 독립적으로 격리했다

파서를 고쳐도 **저장에서 다시 사라졌다.** `write_chart_element`(`<hp:chart>` 재방출 경로, #3546)가
`write_caption` 을 아예 부르지 않았다 — `write_ole` 는 이미 부른다.

직렬화기 수정만 되돌려 `issue4319_chart_caption_roundtrips` 는 실패하고
`issue4319_ole_caption_roundtrips` 는 통과하는 것을 확인해 **파서 공백과 별개 결함**임을 격리했다.

## 4. 구현

- `parse_common_shape_children` 에 `caption_out` 파라미터 + `caption` match arm 추가.
- 두 호출부가 `ole.caption = ole.drawing.caption.take()` 로 정규화 — HWP5 파서
  (`parser/control/shape.rs:213`, `:222`)와 정확히 같은 형태다.
- `write_chart_element` 에 `ctx` 를 넘기고 `write_caption` 호출 추가.

## 5. 재현 — 합성 XML

`~/hwpdocs_10k/` 3,418개 `.hwpx` 를 스캔했으나 `<hp:chart>` 1건, `<hp:ole>` 8건,
`<hp:caption>` 109건이 있는데 **동시 출현이 0건**이었다. 실물 재현이 불가능해, OWPML
`AbstractShapeObjectType`(caption 이 정식 자식, `OLEType` 이 상속)과 실제 코퍼스의 `<hp:pic>`
캡션 블록을 구조 템플릿으로 써서 스키마 유효 XML 을 만들고 실제 `parse_hwpx_section()` 에
통과시켰다 — 조작이 아니라 진짜 코드 경로 재현이다.

## 6. 검증 (완료)

- 회귀 테스트 4건. `git diff`/`checkout --` 방식으로 **수정 전 실패를 각각 확인**했다
  (`git stash` 는 워크트리 간 공유 사고 때문에 쓰지 않았다).
- `cargo test --profile release-test --tests` — 3379 passed, 1 failed.
- `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings` 통과.

**flake 1건**: `document_core::text_security::tests::scan_cost_stays_linear_as_input_grows` —
이 변경과 무관한 파일의 벽시계 타이밍 비율 단언이다. 단독 재실행에서 통과했다. 실행 당시 이 머신
load average 가 120~130(16코어, 다수 에이전트 동시 빌드)이었다.

남은 미래 조건은 GitHub Actions 와 작업지시자 승인, merge 다.
