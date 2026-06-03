# Task M100-1090 Stage 1 완료 보고서

## 1. 목표

`el-school-001.hwpx`를 HWP로 저장할 때 표 셀 배경 그림의 채우기 유형이 한컴 정답과 다르게
저장되는 문제를 수정했다.

작업지시자 확인:

```text
한컴 정답:
  셀 테두리/배경 > 그림 > 채우기 유형 = 크기에 맞추어

기존 rhwp 저장본:
  셀 테두리/배경 > 그림 > 채우기 유형 = 바둑판식으로-세로/왼쪽
```

## 2. 원인

HWPX 파서는 `FIT`, `FIT_TO_SIZE`, `STRETCH`, `TOTAL`을 `ImageFillMode::FitToSize`로
정상 매핑하고 있었다.

문제는 HWP5 DocInfo `BORDER_FILL` 직렬화였다.

```rust
ImageFillMode::FitToSize => 3
```

하지만 HWP5 파서의 이미지 채우기 mode 해석은 다음과 같다.

```text
3 => TileVertLeft
5 => FitToSize
```

따라서 `FitToSize`를 저장한다고 생각했지만, 실제 HWP5 파일에는 `3`이 기록되어 한컴이
`바둑판식으로-세로/왼쪽`으로 해석했다.

## 3. 수정

`src/serializer/doc_info.rs`의 `image_fill_mode_to_u8`를 HWP5 파서와 동일한 0..15 매핑으로
수정했다.

핵심 변경:

```text
FitToSize: 3 -> 5
TileVertLeft: 2 -> 3
TileVertRight: 4
나머지 Center/Left/Right 계열도 HWP5 parser contract와 동일하게 보강
```

회귀 테스트:

```text
test_serialize_border_fill_image_fill_mode_uses_hwp5_values
```

이 테스트는 `ImageFillMode` 전체 enum 값이 DocInfo `BORDER_FILL` 안에서 HWP5 값으로 저장되는지
검증한다.

## 4. 생성 산출물

```text
output/poc/hwpx2hwp/task1090/stage1_cell_background_image_contract/el-school-001-fit-mode-fixed.hwp
```

비교 대상:

```text
saved/111el-school-001.hwp
samples/el-school-001.hwp
```

파일 크기:

```text
generated: 71K
saved/111: 71K
oracle: 78K
```

## 5. rhwp 덤프 확인

기존 저장본:

```text
saved/111el-school-001.hwp
border_fill[7] fill_type=Image image(bin_id=1, mode=TileVertLeft)
```

수정 산출물:

```text
output/poc/hwpx2hwp/task1090/stage1_cell_background_image_contract/el-school-001-fit-mode-fixed.hwp
border_fill[7] fill_type=Image image(bin_id=1, mode=FitToSize)
```

따라서 작업지시자가 보고한 `크기에 맞추어 -> 바둑판식으로-세로/왼쪽` 저장 오류는
DocInfo image fill mode 매핑 오류로 확정한다.

## 6. 검증

```text
cargo fmt --check
cargo check
cargo test --lib test_serialize_border_fill_image_fill_mode_uses_hwp5_values
cargo test --test hwpx_to_hwp_adapter
```

결과:

```text
success
```

`cargo test --test hwpx_to_hwp_adapter` 결과:

```text
49 passed, 0 failed, 11 ignored
```

## 7. 작업지시자 시각 판정

다음 파일을 한컴 에디터에서 확인했다.

```text
output/poc/hwpx2hwp/task1090/stage1_cell_background_image_contract/el-school-001-fit-mode-fixed.hwp
```

확인 항목:

```text
1. 파일 정상 로딩 여부
2. 대상 셀 배경 그림 출력 여부
3. 셀 테두리/배경 > 그림 > 채우기 유형이 "크기에 맞추어"로 표시되는지
4. 표/셀 배치 회귀가 없는지
```

판정 결과:

```text
시각 판정 통과
```

따라서 #1090의 목표였던 표 셀 배경 이미지 채우기 유형 저장 오류는 Stage 1 수정으로 해결된
것으로 본다.
