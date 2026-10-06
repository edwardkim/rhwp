# #2767 처리 결과 — HWPX→HWP 그림 캡션 attr 비트 + BinData remap 잔여 수정

## 사전 확인 — devel 은 이미 일부를 포함하고 있었다

작업 시작 시 `origin/devel`을 다시 fetch해 확인한 결과, 이슈 본문이 결함 B로
지목한 것 중 **`Control::Picture`의 캡션 remap과 `Control::Table`의 캡션 remap은
이미 devel에 존재**했다(주석에 `[#2736]` 태그, `table_caption_picture_bin_ref_is_remapped`·
`picture_caption_picture_bin_ref_is_remapped` 테스트도 이미 있음). 이슈가 미리
고지한 병행 PR #2749(같은 파일)는 `gh pr view 2749`로 **CLOSED, mergedAt: null**
(미머지)임을 확인했으므로, 이 부분은 #2749가 아닌 다른 경로로 이미 devel에
반영된 것으로 보인다.

실제로 남아 있던 것만 수정 범위로 좁혔다:

- **결함 A** (전부 미해결) — 그림 CTRL_HEADER의 캡션 attr 비트(bit 29)
- **결함 B 잔여** — `remap_bin_refs_in_control`의 `Control::HiddenComment` arm 부재,
  `remap_bin_refs_in_shape`의 `ShapeObject::Picture`(그룹 내부 그림) 캡션 미재귀
  (`ShapeObject::Picture`는 `drawing_mut()`이 항상 `None`이라 공통 caption remap
  경로를 타지 않음 — 모델상 `DrawingObjAttr`를 갖지 않고 `Picture` 자신의
  `caption` 필드를 직접 가짐)

## 수정

`src/document_core/converters/hwpx_to_hwp.rs` 한 파일:

1. **결함 A**: `materialize_picture_caption_common_attr()` 신설 — 그림에 캡션이
   있고 bit29가 아직 꺼져 있으면 **OR**(recompute 아님)한다. 표처럼
   `pack_common_attr_bits(...)`로 재계산하면 표 전용 비트를 그림에 강제로 얹는
   회귀가 되므로, HWPX 파서가 이미 채운 `common.attr`에 비트만 얹는다.
   `adapt_paragraph_with_context`의 `Control::Picture` arm에서 캡션 문단 보강
   직후 호출. `AdapterReport`에 `picture_caption_common_attr_materialized`
   카운터 추가 + `changed_anything()` 합산.
2. **결함 B 잔여**: `remap_bin_refs_in_control`에 `Control::HiddenComment` arm
   추가(adapt 워크·border-fill 워크는 이미 #2467 근거로 처리 중이던 것과 동형).
   `remap_bin_refs_in_shape`의 `ShapeObject::Picture` arm에 `pic.caption` 재귀
   추가(그룹 내부 그림 캡션).

이슈 §A-6에 따라 도형(`$rec`)·OLE·연결선 캡션의 attr 비트는 설정하지 않았다 —
`serializer/control.rs`가 도형 캡션 레코드 자체를 아직 출력하지 않아 자기모순
레코드가 되기 때문(별도 후속 과제).

## 테스트 (red → green)

기존 `mod tests`에 4건 추가:

- `picture_caption_common_attr_bit_is_or_ed_in_when_caption_present` — 캡션이 있는
  그림의 `common.attr`가 `0x042A_2211` → `0x242A_2211`로 OR됨을 단언 + 멱등성 확인
- `picture_caption_common_attr_bit_untouched_without_caption` — 캡션이 없으면
  비트를 건드리지 않음(거짓양성 방지)
- `hidden_comment_picture_bin_ref_is_remapped` — 숨은설명 안 그림의 `bin_data_id`가
  remap을 반영함
- `grouped_picture_caption_bin_ref_is_remapped` — 그룹 내부 그림(`ShapeObject::Picture`)
  캡션 안 그림의 `bin_data_id`가 remap을 반영함

수정 전 코드(4곳의 fix 부분만 되돌린 상태)로 실행한 결과:

```
failures:
    document_core::converters::hwpx_to_hwp::tests::grouped_picture_caption_bin_ref_is_remapped
    document_core::converters::hwpx_to_hwp::tests::hidden_comment_picture_bin_ref_is_remapped
    document_core::converters::hwpx_to_hwp::tests::picture_caption_common_attr_bit_is_or_ed_in_when_caption_present

test result: FAILED. 42 passed; 3 failed; 0 ignored; 0 measured; 2481 filtered out
```

(`picture_caption_common_attr_bit_untouched_without_caption`는 대조군이라 수정과
무관하게 항상 통과 — 의도된 결과.)

수정 적용 후 재실행:

```
running 45 tests
...
test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 2481 filtered out; finished in 2.84s
```

red → green을 로컬에서 직접 확인했다.

## 검증 (디스크 제약으로 경량 검증만 수행)

- `cargo check --lib` 통과
- 신설 테스트 4건 포함 `document_core::converters::hwpx_to_hwp::tests` 모듈 전체
  (45건) `cargo test --lib`로 실행, 전부 통과(red 확인 1회 + green 확인 2회)
- 전체 `cargo test`, `cargo build --lib`, `cargo clippy --profile release-test`는
  로컬 디스크 여유 공간 제약(빌드 중 완전 소진 1회 발생 — `target/debug/incremental`
  삭제로 복구)으로 스킵
- `rustfmt --edition 2021` 적용, `git diff --name-only`로 의도한 파일만 변경됨을
  확인
- 이슈 §A-5/§B-3이 제안한 실파일 회귀(`samples/hwp3-sample14-hwp5.hwpx` gso attr
  비교, "OLE storage 재정렬 + 숨은설명/그림캡션" 동시 보유 실파일)는 재확인하지
  않았다 — 이슈 §B-3이 그런 조합 문서가 샘플에 없다고 명시하고 있어, 단위 테스트가
  현재 유일하게 재현 가능한 방법이다.

## 범위 밖

- 도형(`$rec`)·OLE·연결선 캡션의 attr 비트 설정(§A-6) — `serializer/control.rs`
  소관의 별개 상위 결함
- collect 측(`collect_bin_order_from_control`)의 HiddenComment 미방문(§B-2) —
  fallback으로 이미 전단사가 보장되어 결함이 아님(이슈 본문 판정)

## 변경 파일

- `src/document_core/converters/hwpx_to_hwp.rs`
