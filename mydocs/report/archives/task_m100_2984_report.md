# task_m100_2984 처리 결과 보고

## 이슈

edwardkim/rhwp#2984 — HWP3 그림 밝기/명암/그림효과(offset 339~341) 미반영으로 워터마크·그레이스케일·흑백 그림이 원본 그대로 렌더됨.

## 배경 / 근거

`char_shape.shade_color` HWP3 IR 변환 누락 수정(#2958, PR #2968)과 동일한 방법을 `src/parser/hwp3/mod.rs` 의 그림(HWP3 특수문자 코드 11) 변환 경로에 적용했다. 이 함수가 파싱된 HWP3 그림 정보 레코드의 모든 의미 있는 필드를 IR (`model::image::Picture`) 로 옮기는지 점검하기 위해, 로컬 canonical 스펙 문서 `mydocs/tech/한글문서파일구조3.0.md` "10.7. 그림 (11)" 절 표 43 "그림 식별 정보" 를 기준으로 실제 코드가 읽는 `info_buf` 오프셋들과 대조했다.

표 43 은 다음을 명시한다.

```
| 339 | byte | 밝기   | 워터마크: 그림의 밝기 (-100~100) |
| 340 | byte | 명암   | 워터마크: 그림의 명암 (-100~100) |
| 341 | byte | 그림효과 | 워터마크: 0=원래 그림으로, 1=그레이 스케일, 2=흑백으로 |
```

같은 절 설명: "밝기와 명암이 동시에 0 이 아닌경우 워터마크 판정으로 한다" — 즉 이 3바이트는 단순 워터마크 전용이 아니라 한글97/HWP3 문서가 그림에 흑백/그레이스케일 변환·밝기/명암 보정을 적용했을 때 저장되는 일반 이미지 효과 필드다.

`src/parser/hwp3/mod.rs` 의 그림(ch==11) 분기는 `info_buf` 를 348바이트 전부(offset 339~342 포함) 읽지만, 실제로 소비하는 오프셋은 0-4, 8, 9, 10-14, 18-34, 42-48, 58-64, 70-72, 74, 83-339(그림 이름) 뿐이었다. 339~341 은 한 번도 읽히지 않아 `pic.image_attr.brightness`/`contrast`/`effect` 가 항상 기본값(0, 0, `RealPic`)으로 고정되어 있었다.

같은 IR 필드는 HWP5 경로(`src/parser/control/shape.rs::parse_picture`, 890~923행)에서는 정상적으로 채워지므로, HWP3 → IR 변환 경로에서만 구조적으로 값이 소실되는 케이스임을 확인했다(#2958 과 동일 패턴).

## 수정 내용 (red → green)

- `src/parser/hwp3/mod.rs`
  - 새 순수 함수 `hwp3_picture_image_effect(info_buf: &[u8]) -> (i8, i8, ImageEffect)` 추가: offset 339=밝기, 340=명암, 341=그림효과(1=GrayScale, 2=BlackWhite, 그 외 RealPic) 를 읽어 반환한다.
  - 그림(ch==11) 파싱 분기에서 `pic.common.attr = build_common_obj_attr(&pic.common);` 직후 위 함수를 호출해 `pic.image_attr.brightness`/`contrast`/`effect` 에 대입.
  - 단위 테스트 `task2984_hwp3_picture_image_effect_reads_brightness_contrast_effect` 추가: 합성 348바이트 `info_buf` 에 밝기=-40, 명암=25, 그림효과=1(그레이스케일) 을 심어 반환값을 검증.
    - red 확인: `hwp3_picture_image_effect` 본문을 원래 버그와 동일하게 항상 `(0, 0, RealPic)` 을 반환하는 자리표시로 임시 교체한 뒤 `cargo test --lib task2984_hwp3_picture_image_effect` 실행 → `assertion left == right failed / left: 0 / right: -40` 로 실패(FAILED, 0 passed; 1 failed) 확인.
    - green 확인: 실제 오프셋 읽기 로직으로 되돌린 뒤 동일 명령 재실행 → `test result: ok. 1 passed; 0 failed`.

## 영향 범위

- HWP3(.hwp, 한글97) 문서에서 그림에 그레이스케일·흑백 변환이나 밝기/명암 보정이 적용돼 있어도 이제 IR(`Picture.image_attr`) 에 값이 반영되어, SVG/PDF 렌더링 및 HWPX 재저장(`bright`/`contrast`/`effect` 속성) 시 원본과 일치한다.
- 그 외 경로(HWP5, HWPX)는 변경 없음.

## 검증

- `cargo check --lib` — 통과.
- `cargo test --lib task2984_hwp3_picture_image_effect` — 통과 (1 passed).
- `rustfmt --edition 2021 src/parser/hwp3/mod.rs` — 적용, 추가 diff 없음.
- `CommonObjAttr`(`src/model/shape.rs`) 는 손대지 않음.

## 남은 이슈

없음. 이번 수정은 그림 밝기/명암/그림효과 3개 필드에 한정된 최소 diff다. 같은 그림 정보 레코드 안의 다른 미사용 바이트 범위(예: offset 342 그림보호 플래그)는 이번 스코프 밖으로 남겨 두었으며, 필요 시 별도 이슈로 분리할 수 있다.
