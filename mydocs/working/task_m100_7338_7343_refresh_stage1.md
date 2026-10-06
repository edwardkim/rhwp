# #7338-#7343 최신 head 재적용과 메인터너 보정 1단계

## 분석 기준

- 기준 base: `upstream/devel` `7a95e46e0`.
- #7338은 `c1ac0f987`로 이미 base에 병합되어 재적용하지 않는다.
- #7339~#7343은 이전 체리픽 뒤 contributor가 새 commit을 올렸으므로, 이전 SHA를
  재사용하지 않고 현재 head의 고유 code/doc commit을 base 위에 다시 적용했다.

## 최신 contributor 범위

| PR | 최신 코드 | 최신 증적 문서 |
| --- | --- | --- |
| #7339 | `cb20be6e1` | `ed3799fe2` |
| #7340 | `d327c2c2c` | `6e6a0b48c` |
| #7341 | `7dd7973f7`, `8b11a45da` | `0dd0c2a57` |
| #7342 | `195a3390e` | `cd0fbd0d3` |
| #7343 | `51606adcc` | `230477deb` |

## 남은 메인터너 보정

1. #7338 setter가 의미가 같은 `pageBreak`/`repeatHeader`를 다시 설정할 때도
   비표준 HWP5 raw bit를 정규화하지 않도록 serializer 소유 규칙을 복원한다.
2. #7339의 비-inline 개체 band 측정은 셀 내용 폭과 `HorzAlign`을 함께 적용해야
   Left/Right 표현만 다른 같은 물리 좌표를 서로 다른 band로 세로 합산하지 않는다.
3. #7340의 `0 -> 0` 저장 되감김은 행에 다른 후보가 있어도 현재 `end_cut`과 동일한
   unit에서 사다리 전진이 확인될 때만 고아 규칙 면제로 사용한다.

## 흡수된 이전 보정

이전 #7330 보정의 `treat_as_char`/다중 표 분기는 다시 적용하지 않는다. 최신 #7341의
`whitespace_only_host_line_already_placed`는 실제로 앞선 표가 host 줄을 배치한
`!is_first_table && pre_table_end_line > 0 && post_table_start == 0` 형상만 억제한다.
따라서 단일 비-TAC host의 저장 슬롯(#6925)은 보존하면서, 단일 TAC 중복(#1417/#2006)과
다중 표 중복(#7330)을 각각 기존 규칙과 새 규칙으로 처리한다.

## 다음 단계

위 세 보정을 현재 분리된 `row_step.rs` 호출 경로에 맞춰 구현하고, 추가 회귀 검사와
기존 대표 검사를 실행한 뒤 결과를 이 문서에 기록한다.

## 구현과 확인 결과

- `table_ops.rs`: API setter는 IR 의미만 변경하고, HWP5 serializer가 의미 차이를
  판단해 raw bit를 동기화하도록 원래 소유 경계를 복원했다.
- `height_measurer.rs`: 셀의 유효 좌우 패딩을 뺀 내용 폭과 `HorzAlign`으로 실제 x band를
  계산해, 같은 물리 위치의 Left/Right 개체가 같은 높이를 예약하게 했다.
- `table_layout.rs`와 `typeset/table/scan/runner/row_step.rs`: 확인된 저장 되감김 unit 목록을
  반환하고 현재 `end_cut`과 같은 unit만 고아 규칙 면제의 근거로 사용하게 했다.
- `cargo fmt --all -- --check` 및 `git diff --check`: 통과.
- 직접 회귀:

  ```text
  cargo nextest run --cargo-profile release-test --target-dir target/pr-review \
    --test regression_suite_008 --test regression_suite_016 \
    --test regression_suite_017 --no-fail-fast
  Summary [114.368s] 653 tests run: 653 passed (1 slow), 0 skipped
  ```

전체 regression, native/WASM Clippy, workspace build, Native Skia 및 fresh WASM 시각 검증은
다음 검증 단계에서 최신 통합 head를 기준으로 실행한다.
