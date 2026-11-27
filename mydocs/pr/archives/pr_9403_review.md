# PR #9403 리뷰 — 글상자 안쪽 여백 클릭과 커서 위치

## 최종 판정

**메인테이너 보완 후 devel 병합 완료.** 실제 Studio에서 둘째 글상자 끝을 한 번 누른 뒤 K를 입력하자 둘째 글상자만 바뀌었다. 글상자 사이의 빈 곳과 바깥 테두리는 묶음을 선택했고, 선택 상태에서 다른 글상자로 들어가 입력할 수도 있었다. 원 기여와 보완을 함께 담은 통합 PR의 CI가 통과했다. [최종 검증](../assets/pblsketch-20261126/README.md)

## 접수와 변경 범위

[원 PR #9403](https://github.com/edwardkim/rhwp/pull/9403)의 작성자는 pblsketch이고 대상은 `devel`이다. 연결 이슈는 [#9399](https://github.com/edwardkim/rhwp/issues/9399)다. 접수할 때 리뷰 요청이 비어 있어 2026-11-26에 `edwardkim`을 지정하고 다시 조회했다. 원 커밋은 `45a124f241269c0c20f3c1236ee71815acc58528`이며 [제품 CI](https://github.com/edwardkim/rhwp/actions/runs/45543473091)와 [정책 검사](https://github.com/edwardkim/rhwp/actions/runs/45546581858)가 성공했다.

검토 기준은 2026-11-26의 `upstream/devel` `48a678399072d0df11380e1ad6a3b6aa77dd677a`이다. 두 원 커밋을 번호순으로 병합한 로컬 소스는 `2647cad077ca350659b15863df30bad6e01eeca4`다. 원 PR 두 건은 서로의 커밋에 의존하지 않는다. 각 원 커밋과 현재 기준의 병합 시뮬레이션, 실제 누적 병합, 공백 검사와 Rust 테스트 자동 수집 검사를 통과했다.

## 검토 대상

`src/document_core/queries/cursor_rect.rs`에서 클릭 지점을 포함하는 칸과 글상자의 경로 깊이를 비교하고, 해당 글상자에서 가까운 줄과 글자 위치를 찾는다. 기존 `devel`의 글 끝·행간 11개 지점 중 7곳에서 기대한 경로나 글자 위치와 달랐다. 본문 글상자, 표 칸 안 형제 글상자, 글상자 안 표 칸에 대해 API 응답과 실제 클릭 후 글자가 입력되는 위치를 확인한다.

- `samples/issue6974/synth_textbox_vgrow.hwp`
- `tests/fixtures/group_textbox_cell/cell-group-textbox.hwp`
- `samples/issue8513/textbox_floating_table_offset.hwpx`

## 메인테이너 보완

원 기여는 글상자 끝·행간 등 11개 위치를 모두 올바르게 조회했다. 다만 같은 지점을 Studio에서 클릭하면 바깥 표 칸이 소유한 묶음이 선택되어 입력한 문자가 둘째 글상자에 들어가지 않았다. [원 기여에서의 실제 클릭](../assets/pblsketch-20261126/group-click.png)

`2323676b53`은 페이지 개체 목록에 묶음의 소유 칸 경로를 전달한다. Studio는 실제 누른 글상자가 해당 묶음에 속하고 포인터가 테두리 안쪽에 있을 때 글 편집으로 진입한다. 이미 묶음을 선택한 상태에서도 다른 자식 글상자로 들어갈 수 있다. `995c962841`은 글상자 사이 빈 곳을 누를 때 바깥 칸에서 자식 글상자의 글자를 찾던 문제를 수정했다. 칸의 전체 경로와 구역을 대조하면서 같은 칸의 다른 문단에는 계속 접근할 수 있다. [재현 절차](pblsketch_20261126_review_impl.md)

## 최종 병합

원 기여 `45a124f24126`를 포함한 제출 커밋 `98a060eb9d6e`을 병합했다. 최신 `devel`과의 실제 병합 트리에서 Native 집중 회귀 23건과 Studio 키보드·포인터 8사례를 재검증했다. [통합 PR #9436](https://github.com/edwardkim/rhwp/pull/9436) · [병합 커밋 `28541762d2e3`](https://github.com/edwardkim/rhwp/commit/28541762d2e3b8be09221c2c02359440b241d472) · [셀프 리뷰](https://github.com/edwardkim/rhwp/pull/9436#pullrequestreview-6080796097) · [제품 CI](https://github.com/edwardkim/rhwp/actions/runs/45896083472) · [CI 정책 감사](https://github.com/edwardkim/rhwp/actions/runs/45899991302) · [최신 devel 대조](../assets/pblsketch-20261126/README.md#11월-27일-제출-직전-devel-대조)

## Merge 후 contributor PR comment 계획

글 끝과 줄 사이에서 가까운 글상자 문단을 찾는 원 기여에 감사한다. Studio에서 둘째 글상자에 직접 입력한 결과와 빈 곳·테두리의 묶음 선택을 설명하고, 같은 위치를 누른 전후 화면을 원 PR에 포함한다. [보완 전](../assets/pblsketch-20261126/group-click.png) · [보완 후](../assets/pblsketch-20261126/group-click-corrected.png)
