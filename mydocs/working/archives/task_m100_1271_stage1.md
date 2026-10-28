# Stage 1 보고 — Task M100-1271

## 범위

- 이슈: [#1271](https://github.com/edwardkim/rhwp/issues/1271)
- 단계: 회귀 재현 테스트와 진단 고정
- 대상 샘플: `samples/hwpx/[2027] 온새미로 1 본교재.hwpx`

## 변경

- `tests/issue_1271_hwpx_behind_text_table.rs` 추가
  - 한컴 PDF 기준 앞쪽 페이지 대응을 테스트로 고정한다.
  - 기대 배치:
    - page 2: `MEMO`
    - page 3: 1주차 표지 도형
    - page 4: section 1 본문 시작, `page_num=4`
  - 회귀 원인으로 관찰된 `PartialTable pi=3 ci=0` 이 page 2 에 나타나지 않아야 함을 검증한다.

## 재현 결과

명령:

```text
cargo test --test issue_1271_hwpx_behind_text_table -- --nocapture
```

결과:

```text
test onsaemiro_front_matter_is_not_shifted_by_behind_text_table_fragment ... FAILED
```

핵심 실패 출력:

```text
PDF 기준 page 2 는 MEMO 쪽이어야 한다.

=== 페이지 2 (global_idx=1, section=0, page_num=2) ===
  단 0 (items=2, used=116.0px)
    PartialTable   pi=3 ci=0  rows=0..2  cont=true  2x1  vpos=5868  start_cut=[32] end_cut=[]
    Shape          pi=3 ci=1    vpos=5868
```

## 판단

- 현재 기본 TypesetEngine 은 section 0 paragraph 3 의 글뒤로 표를 본문 흐름의 `PartialTable` 로 분할한다.
- 이 때문에 PDF 기준 page 2 에 있어야 할 `MEMO`가 page 3 으로 밀리고, page 4 에 있어야 할 section 1 본문 시작도 뒤로 밀린다.
- Stage 2 에서는 `src/renderer/typeset.rs` 의 paper-anchored `BehindText`/`InFrontOfText` 표 out-of-flow 판정을 보정한다.

## 검증

```text
cargo fmt
cargo test --test issue_1271_hwpx_behind_text_table -- --nocapture
```

- `cargo fmt`: 통과
- `cargo test --test issue_1271_hwpx_behind_text_table -- --nocapture`: 의도한 재현 실패 확인
