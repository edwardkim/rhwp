# Task M100-1090 완료 보고서

## 1. 이슈

GitHub Issue #1090:

```text
[hwpx2hwp] el-school-001.hwpx 표 셀 배경 이미지 HWP 저장 지원
```

부모 이슈:

```text
#1064
```

## 2. 목표

`samples/hwpx/el-school-001.hwpx`를 HWP로 저장할 때 표 셀의 배경 그림 채우기 유형을
한컴 정답 HWP와 동일하게 보존한다.

## 3. 문제

작업지시자 재현 결과:

```text
한컴 원본/정답:
  셀 테두리/배경 > 그림 > 채우기 유형 = 크기에 맞추어

기존 rhwp 저장 결과:
  셀 테두리/배경 > 그림 > 채우기 유형 = 바둑판식으로-세로/왼쪽
```

## 4. 원인

HWPX 파서는 `imgBrush@mode`를 `ImageFillMode::FitToSize`로 정상 매핑하고 있었다.

문제는 DocInfo `BORDER_FILL` 직렬화 경로였다.

```text
기존 저장:
  FitToSize -> 3

HWP5 parser contract:
  3 -> TileVertLeft
  5 -> FitToSize
```

즉 rhwp는 "크기에 맞추어"를 저장한다고 생각했지만, HWP5 파일에는 한컴이
`바둑판식으로-세로/왼쪽`으로 해석하는 값이 기록되고 있었다.

## 5. 수정

수정 파일:

```text
src/serializer/doc_info.rs
src/serializer/doc_info/tests.rs
```

수정 내용:

```text
1. DocInfo BORDER_FILL image fill mode 직렬화 값을 HWP5 parser contract와 일치시킴
2. FitToSize 저장값을 3에서 5로 수정
3. Tile/Center/Left/Right 계열 전체 0..15 매핑 보강
4. ImageFillMode 전체 값의 직렬화 회귀 테스트 추가
```

## 6. 산출물

판정용 HWP:

```text
output/poc/hwpx2hwp/task1090/stage1_cell_background_image_contract/el-school-001-fit-mode-fixed.hwp
```

단계 보고서:

```text
mydocs/working/task_m100_1090_stage1.md
```

## 7. 검증

실행한 검증:

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

`cargo test --test hwpx_to_hwp_adapter`:

```text
49 passed, 0 failed, 11 ignored
```

## 8. 시각 판정

작업지시자 판정:

```text
시각 판정 통과
```

확인 의미:

```text
1. 생성 HWP가 한컴 에디터에서 정상 확인됨
2. 표 셀 배경 그림 채우기 유형이 "크기에 맞추어"로 보존됨
3. 기존 "바둑판식으로-세로/왼쪽" 저장 오류가 해소됨
```

## 9. 결론

#1090은 DocInfo `BORDER_FILL` 이미지 채우기 mode 매핑 오류였다. HWPX 파싱층이 아니라 HWP5
직렬화층의 enum 값 불일치였으며, `ImageFillMode::FitToSize`를 HWP5 값 `5`로 저장하도록 수정해
해결했다.

완료 후 #1064의 후속/서브 이슈로 닫을 수 있다.

