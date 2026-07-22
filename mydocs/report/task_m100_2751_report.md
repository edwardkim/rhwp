# #2751 처리 결과 — HML 직렬화기 무검증 row_count → ROW 팽창/지연 수정

## 문제

`src/serializer/hml/body.rs`의 `write_table()`이 파일에서 온 무검증 `u16`
`table.row_count`만큼 `<ROW>`를 방출했다(`for row in 0..table.row_count`). 셀이 하나도
없는 행까지 `<ROW></ROW>`로 찍히고, 각 행마다 `table.cells` 전체를 선형 스캔해
`O(row_count × cells.len())`이 됐다. `RowCount="65535"`(`ColCount="1"`이라 #2731의
그리드 상한을 우회)만으로 정상 29,500 B 입력이 736,433 B(25배)로 팽창하고, CELL 2만 개
조합에서는 export가 4.35초 걸린다(#2722의 잔여).

## 수정

`src/serializer/hml/body.rs`의 `write_table()` 한 곳:

1. `serialized_row_count()` 신설 — 방출할 `<ROW>` 개수를 `table.row_count`와
   "셀이 실제로 자리한 최대 행+1" 중 작은 쪽으로 clamp. `row_span`은 사용하지
   않음(셀 하나의 `RowSpan=65535` 선언만으로 clamp가 무력화되는 함정을 피함).
2. 행마다 `cells` 전체를 다시 스캔하던 것을 `cells`를 한 번만 순회해 행별로
   버킷팅(`Vec<Vec<(usize, &Cell)>>`)하는 방식으로 교체 — `O(cells.len() + row_count)`.
   버킷은 원래 순회 순서를 보존해 `cell_index`(진단 경로 `CELL[i]`)도 그대로 유지.

이슈 본문 5.3에서 검토 후 기각한 대안(`RowCount` 속성 자체 축소, 직렬화기 고정 상한,
파서 측 변경)은 그대로 채택하지 않았다 — 속성값 변형은 계약 변경이고, 고정 상한은
실제 셀 데이터를 소리 없이 잃을 위험이 있다.

## 테스트 (red → green)

`tests/issue_2751_hml_table_row_bound.rs` 신설:

- `malicious_row_count_does_not_inflate_output_with_empty_rows` —
  `samples/hml/formatting_table.hml`의 `RowCount="1"`을 `"65535"`로 치환한 입력을
  파싱·export 후, 출력에 `<ROW>`가 1개, 빈 `<ROW></ROW>`가 0개임을 단언.
- `malicious_row_count_reparse_preserves_table_ir` — export 산출물을 재파싱해도
  표의 셀 개수가 그대로 보존됨을 확인(빈 `<ROW>` 제거가 무손실임의 증거).

수정 전 코드(`for row in 0..table.row_count { ... }`)로 되돌려 실행한 결과:

```
running 2 tests
test malicious_row_count_does_not_inflate_output_with_empty_rows ... FAILED
test malicious_row_count_reparse_preserves_table_ir ... ok

thread 'malicious_row_count_does_not_inflate_output_with_empty_rows' panicked ...
  row_open, 1 assertion 실패 (실제 65535)
```

수정 적용 후 재실행:

```
running 2 tests
test malicious_row_count_does_not_inflate_output_with_empty_rows ... ok
test malicious_row_count_reparse_preserves_table_ir ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
```

red → green을 로컬에서 직접 확인했다.

## 검증 (디스크 제약으로 경량 검증만 수행)

- `cargo check --lib` 통과
- 위 테스트 2건 `cargo test --test issue_2751_hml_table_row_bound`로 개별 실행 통과
  (red 확인 1회 + green 확인 2회)
- 전체 `cargo test`, `cargo build --lib`, `cargo clippy --profile release-test`는
  로컬 디스크 여유 공간(~19~20GB, 100% 사용) 제약으로 스킵
- `rustfmt --edition 2021` 적용 후 `git diff --name-only`로 의도한 파일만 변경됨을
  확인(`body.rs`, 신규 테스트 파일)
- 6.2절의 정상 문서 바이트 동일성(fnv1a64 지문 비교)은 디스크·시간 제약으로 직접
  재현하지 않았다 — 대신 위 IR 동등성 테스트로 대체 확인. 필요시 팔로업에서 지문
  비교를 추가할 수 있음.

## 범위 밖

이슈 7절에 기록된 동형 결함 3곳(`src/serializer/hwpx/table.rs:112`,
`src/parser/hml/adapter.rs:244`, `src/parser/hwpx/section.rs:1843`)은 이번 PR에서
고치지 않았다. 그중 `adapter.rs:244`는 이번 큐의 #2833과 겹치는 영역이라 별도 PR로
처리 예정이다.

## 변경 파일

- `src/serializer/hml/body.rs`
- `tests/issue_2751_hml_table_row_bound.rs` (신규)
