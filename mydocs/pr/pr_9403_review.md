# PR #9403 리뷰 — 글상자 안쪽 여백 클릭과 커서 위치

## 최종 판정

**머지 보류.** 실제 편집기와 최종 로컬 빌드의 검증이 진행 중이다.

## 접수와 변경 범위

[원 PR #9403](https://github.com/edwardkim/rhwp/pull/9403)의 작성자는 pblsketch이고 대상은 `devel`이다. 연결 이슈는 [#9399](https://github.com/edwardkim/rhwp/issues/9399)다. 접수할 때 리뷰 요청이 비어 있어 2026-11-26에 `edwardkim`을 지정하고 다시 조회했다. 원 커밋은 `45a124f241269c0c20f3c1236ee71815acc58528`이며 [제품 CI](https://github.com/edwardkim/rhwp/actions/runs/45543473091)와 [정책 검사](https://github.com/edwardkim/rhwp/actions/runs/45546581858)가 성공했다.

검토 기준은 2026-11-26의 `upstream/devel` `48a678399072d0df11380e1ad6a3b6aa77dd677a`이다. 두 원 커밋을 번호순으로 병합한 로컬 소스는 `2647cad077ca350659b15863df30bad6e01eeca4`다. 원 PR 두 건은 서로의 커밋에 의존하지 않는다. 각 원 커밋과 현재 기준의 병합 시뮬레이션, 실제 누적 병합, 공백 검사와 Rust 테스트 자동 수집 검사를 통과했다.

## 검토 대상

`src/document_core/queries/cursor_rect.rs`에서 클릭 지점을 포함하는 칸과 글상자의 경로 깊이를 비교하고, 해당 글상자에서 가까운 줄과 글자 위치를 찾는다. 기존 `devel`의 글 끝·행간 11개 지점 중 7곳에서 기대한 경로나 글자 위치와 달랐다. 본문 글상자, 표 칸 안 형제 글상자, 글상자 안 표 칸에 대해 API 응답과 실제 클릭 후 글자가 입력되는 위치를 확인한다.

- `samples/issue6974/synth_textbox_vgrow.hwp`
- `tests/fixtures/group_textbox_cell/cell-group-textbox.hwp`
- `samples/issue8513/textbox_floating_table_offset.hwpx`

