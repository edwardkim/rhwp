# Task #975 최종 보고서 — PageBackground 이미지 fill_mode 및 색상 워터마크 보정

- 이슈: [#975](https://github.com/edwardkim/rhwp/issues/975)
- 브랜치: `task-975-page-background-fill-mode`
- 기준: `upstream/devel` 최신 동기화 후 `39d90d9d`
- 수행일: 2026-05-18

## 1. 결론

이슈의 핵심 결함은 PageBackground 이미지 `fill_mode=Center`가 SVG/Web Canvas에서 페이지 전체로 stretch되던 문제였다.
수정 후 `Center` 계열은 이미지 원본 크기 기준으로 배치되고, `FitToSize`는 기존처럼 bbox 전체 채우기를 유지한다.

추가 검증 과정에서 색상 워터마크 샘플 2종이 모두 다음 한컴 preset 값을 공유함을 확인했다.

```text
effect=0(RealPic), brightness=-50, contrast=70
```

이 preset은 기존 brightness/contrast 필터 대신 공통 색상 워터마크 보정을 적용한다.

```text
SVG: saturate(0.9165) + contrast(0.9313) + brightness(2.0972) + opacity(0.21729612)
Canvas: saturate(92%) contrast(93%) brightness(210%) + globalAlpha 0.21729612
```

## 2. 주요 변경

렌더 트리/스타일 전달:

```text
ResolvedImageFill: brightness, contrast, effect 보존
PageBackgroundImage: brightness, contrast, effect 보존
ImageNode: RealPic 워터마크 preset 판정 helper 추가
```

SVG/Web Canvas:

```text
PageBackground image fill_mode 적용
RealPic 색상 워터마크 preset 공통 필터 적용
기존 brightness/contrast 필터 rhwp-img-bc-b-50c70 미적용
```

배경 ImageFill 전달:

```text
table_layout: 셀 배경 ImageFill tone 속성 전달
shape_layout: 도형 배경 ImageFill tone 속성 전달
```

## 3. 검증 샘플

```text
/Users/melee/Downloads/143E433F503322BD33.hwp
/Users/melee/Downloads/253E164F57A1BC6934.hwp
```

SVG 산출물:

```text
/private/tmp/rhwp-task975/svg-watermark-fit-final-143/143E433F503322BD33.svg
/private/tmp/rhwp-task975/svg-watermark-fit-final-253/253E164F57A1BC6934_001.svg
/private/tmp/rhwp-task975/svg-watermark-fit-final-253/253E164F57A1BC6934_002.svg
```

세 SVG 모두 `rhwp-realpic-watermark-tone`과 `opacity="0.21729612"`가 적용됐고, 기존 `rhwp-img-bc-b-50c70` 필터는 생성되지 않았다.

색상 보정값은 두 HWP에서 추출한 실제 워터마크 이미지와 한컴 뷰어 watermark-only 스크린샷 crop을 비교해 산정했다.
최종값은 `143E433F503322BD33.hwp`와 `253E164F57A1BC6934.hwp`의 aligned crop 공통 최적값이며, 파일명 분기가 아니라 동일 RealPic 색상 워터마크 preset 전체에 적용한다.

```text
/private/tmp/rhwp-task975/extracted-bins/143E433F503322BD33_bin003_storage0003.png
/private/tmp/rhwp-task975/extracted-bins/253E164F57A1BC6934_bin001_storage0001.jpg
/private/tmp/rhwp-task975/extracted-bins/253E164F57A1BC6934_bin002_storage0002.jpg

공통 보정값:
  saturation = 0.91646104
  contrast   = 0.93125103
  brightness = 2.09719097
  opacity    = 0.21729612
```

비-RealPic 워터마크는 기존 brightness/contrast 필터와 opacity 0.17을 유지해 `복학원서.hwp` 계열 회귀를 피했다.

## 4. 검증 결과

```text
cargo fmt --all -- --check
  통과

cargo test --lib realpic_watermark
  ok. 2 passed

cargo test --lib
  ok. 1312 passed; 0 failed; 2 ignored

cargo test --test svg_snapshot
  ok. 8 passed

cargo check --target wasm32-unknown-unknown --lib
  통과

docker-compose --env-file .env.docker run --rm wasm
  통과, pkg/rhwp_bg.wasm 갱신
```

현재 `rhwp-studio` 확인 URL:

```text
http://127.0.0.1:7700/
```

작업지시자 시각 확인 결과, Fit aligned 4-param 보정값을 최종값으로 결정했다.

## 5. 잔존 리스크

한컴 내부의 워터마크 색 변환 로직은 공식적으로 확인되지 않았다.
따라서 이번 보정은 두 색상 워터마크 샘플과 macOS 한컴뷰어 시각 기준에 맞춘 근사값이다.

`253E164F57A1BC6934.hwp` export 중 표 overflow 경고 2건이 남아 있지만, 이는 기존 레이아웃 overflow이며 워터마크 색상 보정과는 무관하다.
