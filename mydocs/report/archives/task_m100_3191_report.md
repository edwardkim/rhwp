---
kind: report
status: active
last_verified: 2026-07-23
---

# #3191 처리 결과 — 숨은설명 안 그림 BinData 순서 수집/remap 누락

## 문제

`src/document_core/converters/hwpx_to_hwp.rs` 의 `collect_bin_order_from_control()` 과
`remap_bin_refs_in_control()` 은 각주(Footnote)·미주(Endnote)·머리말(Header)·꼬리말(Footer)·표(Table,
캡션 포함) 안의 그림은 모두 순회하지만 숨은설명(`Control::HiddenComment`) 안의 그림만 두 함수 모두에서 빠져
있었다. 같은 파일의 `collect_object_border_fill_refs_from_paragraph()` 는 이미 `Control::HiddenComment`
를 처리하고 있어(#2467/05af4fe7) 컨테이너 순회 목록이 함수마다 어긋나 있었다.

## 영향

OLE Storage 를 포함한 HWPX 문서에서 `materialize_hwp5_bin_data_order()` 가 BinData 순서를 재배치할 때,
숨은설명 안 그림의 `bin_data_id` 는 remap 되지 않고 옛 순번을 그대로 가리켜 재배치 후 다른 그림을 참조하거나
존재하지 않는 BinData 를 참조하게 된다.

## 재현 (red)

`table_caption_picture_bin_ref_is_remapped` 와 동형의 테스트를 숨은설명 컨테이너로 작성:

- `hidden_comment_picture_bin_ref_is_remapped`: `remap_bin_refs_in_control()` 호출 후 숨은설명 안 그림의
  `bin_data_id` 가 remap 되지 않음을 확인 (FAIL: left=1, right=2 기대).
- `hidden_comment_picture_is_collected_into_bin_order`: `collect_bin_order_from_control()` 호출 후 순서
  목록에 숨은설명 안 그림의 `bin_data_id` 가 없음을 확인 (FAIL: left=[], right=[2] 기대).

## 수정

`collect_bin_order_from_control()` 과 `remap_bin_refs_in_control()` 의 match 문에 다른 컨테이너와 동일한
패턴으로 `Control::HiddenComment(comment) => ...comment.paragraphs...` 분기를 추가했다.

## 검증

```
cargo test --lib document_core::converters::hwpx_to_hwp
```

수정 전 신규 테스트 2건 FAIL 확인 → 최소 수정 적용 → 43 tests passed (기존 41건 + 신규 2건), 0 failed.
