# #2833 처리 결과 — HML 파서 into_table() row_sizes 계산 이차시간 수정

## 문제

`src/parser/hml/adapter.rs`의 `into_table()`이 `row_sizes`를 계산할 때
`table.row_count`(파일 `RowCount` 속성을 검증 없이 받은 `u16`)만큼 바깥 루프를 돌며
매 행마다 `cells` 전체를 선형 스캔했다(`O(row_count × cells.len())`). `RowCount`만
크게 부풀리고 `CELL`은 정상 개수인 입력이 **파싱 자체**를 느리게 만든다 — #2751이
지적한 export 경로(`serializer/hml/body.rs`)와는 별개로, import 경로에 남은 동형
잔여 결함이다.

## 수정

`src/parser/hml/adapter.rs`의 `into_table()` 한 곳:

```rust
let mut row_sizes = vec![0i16; source.row_count as usize];
for cell in &cells {
    if let Some(slot) = row_sizes.get_mut(cell.row as usize) {
        *slot = slot.saturating_add(1);
    }
}
```

`cells`를 한 번만 순회해 행별 카운트를 누적하는 방식으로 교체 —
`O(row_count + cells.len())`. `cell.row >= row_count`인 셀은 종전 코드에서도
`0..row_count` 범위 밖이라 어떤 행에도 카운트되지 않았으므로, `get_mut`이 `None`을
반환하는 경우 그대로 무시해 동일한 동작을 유지한다. 정상 문서(`cell.row <
row_count`)에서는 결과가 원본과 값 단위로 동일하다.

## 테스트 (red → green)

`tests/issue_2833_hml_adapter_row_sizes.rs` 신설. 이슈 실측(`RowCount=60000`,
`CELL=3000`)을 재현하는 최소 HML을 구성해 `parse_hml()`을 호출하고:

1. 파싱 소요시간이 500ms 미만임을 단언(느슨한 회귀 상한)
2. `row_sizes` 값 자체가 정확함을 단언 — `row_sizes.len() == 60000`,
   `row_sizes[0] == 3000`(모든 셀이 row=0), 나머지 행은 전부 0.

수정 전 코드(`(0..source.row_count).map(|row| cells.iter().filter(...))`)로
되돌려 실행한 결과:

```
thread 'inflated_row_count_does_not_slow_down_parsing' panicked:
row_sizes 계산이 O(row_count × cells.len()) 로 되돌아가면 이 상한을 넘음 (실제 1.5931682s)
```

수정 적용 후 재실행:

```
running 1 test
test inflated_row_count_does_not_slow_down_parsing ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
```

1.59초 → 0.2초(테스트 전체, 파싱 자체는 그중 일부)로 확인, red → green을 로컬에서
직접 확인했다.

## 검증 (디스크 제약으로 경량 검증만 수행)

- `cargo check --lib` 통과
- 위 테스트 1건 `cargo test --test issue_2833_hml_adapter_row_sizes`로 개별 실행
  통과(red 확인 1회 + green 확인 2회)
- 전체 `cargo test`, `cargo build --lib`, `cargo clippy --profile release-test`는
  디스크 여유 공간(~19GB, 거의 100% 사용) 제약으로 스킵
- `rustfmt --edition 2021` 적용, `git diff --name-only`로 의도한 파일(`adapter.rs`,
  신규 테스트 파일)만 변경됨을 확인

## 범위 밖

이슈 본문에서 언급된 동형 루프(`model/table.rs::rebuild_row_sizes` 등)는 이 이슈의
범위(`adapter.rs::into_table()` 한 곳)에 포함되지 않아 손대지 않았다.

## 변경 파일

- `src/parser/hml/adapter.rs`
- `tests/issue_2833_hml_adapter_row_sizes.rs` (신규)
