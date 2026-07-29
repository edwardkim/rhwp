# task_m100_3047: 본문 최상위 미주 재번호 매기기 누락 수정

## 이슈
#3047

## 원인
`src/document_core/commands/footnote_ops.rs`의 `renumber_footnotes_in_section`은
각주 삭제 등 이벤트 후 구역 내 각주/미주 번호를 문서 순서대로 재계산한다.

표 셀 내부, 글상자 텍스트박스 내부 순회에서는 `Control::Footnote`와 `Control::Endnote`를
모두 처리하지만, 문단 최상위 컨트롤을 순회하는 바깥쪽 match에는 `Control::Footnote`
분기만 있고 `Control::Endnote` 분기가 없었다. 그 결과 표/글상자 밖에 직접 삽입된
미주는 각주 삭제 후에도 번호가 갱신되지 않았다.

## 수정
바깥쪽 match에 `Control::Endnote` 분기를 추가해 표/글상자 내부 처리와 동일하게
번호를 매기도록 했다.

## 테스트
`footnote_ops.rs`에 회귀 테스트
`delete_footnote_renumbers_top_level_endnote_after_it` 추가.
본문에 각주 1개 + 미주 1개를 삽입한 뒤 각주를 삭제하면, 남은 미주 번호가
1로 재계산되는지 검증한다.

`cargo test --lib footnote_ops::tests` 통과 확인.
