# Task #975 Stage 4 — 검증 결과

- 이슈: [#975](https://github.com/edwardkim/rhwp/issues/975)
- 브랜치: `task-975-page-background-fill-mode`
- 수행일: 2026-05-18

## 1. 대상 샘플

이번 검증은 색상 워터마크가 들어 있는 두 샘플을 기준으로 수행했다.

```text
/Users/melee/Downloads/143E433F503322BD33.hwp
/Users/melee/Downloads/253E164F57A1BC6934.hwp
```

`143E433F503322BD33.hwp`는 PageBackground `BorderFill` 이미지다.

```text
border_fill[2] image(bin_id=3, mode=Center, brightness=-50, contrast=70, effect=0)
```

`253E164F57A1BC6934.hwp`는 표 셀 배경 `BorderFill` 이미지다.

```text
border_fill[3] image(bin_id=1, mode=FitToSize, brightness=-50, contrast=70, effect=0)
border_fill[4] image(bin_id=2, mode=FitToSize, brightness=-50, contrast=70, effect=0)
```

두 샘플 모두 `effect=0(RealPic) + brightness=-50 + contrast=70` 조합이므로 동일한 색상 워터마크 preset으로 처리한다.

## 2. 색상 데이터 비교와 보정값

두 HWP에서 워터마크에 해당하는 원본 이미지 데이터를 추출했다.

```text
/private/tmp/rhwp-task975/extracted-bins/143E433F503322BD33_bin003_storage0003.png
/private/tmp/rhwp-task975/extracted-bins/253E164F57A1BC6934_bin001_storage0001.jpg
/private/tmp/rhwp-task975/extracted-bins/253E164F57A1BC6934_bin002_storage0002.jpg
```

추출 이미지에 rhwp가 적용하는 SVG/Web Canvas 공통 필터를 단독 적용한 결과와 한컴뷰어 화면의 워터마크 영역을 비교했다.
`143E433F503322BD33.hwp`는 파란/청록 영역이 워터마크 인상의 대부분을 결정하므로, 본문 글자, 표 선, 안티앨리어싱, 스크린샷 스케일 차이가 섞이지 않도록 해당 색상 픽셀 중심으로 마스크를 잡아 공통 보정값을 재산정했다.

```text
RealPic 색상 워터마크 preset:
  effect=0(RealPic), brightness=-50, contrast=70

최종 공통 보정값:
  saturation = 0.91646104
  contrast   = 0.93125103
  brightness = 2.09719097
  opacity    = 0.21729612
```

이 값은 파일명 분기가 아니라 동일 RealPic 색상 워터마크 preset 전체에 적용한다.
따라서 `253E164F57A1BC6934.hwp`의 표 셀 배경 워터마크도 같은 보정 경로를 탄다.

브라우저 SVG 렌더와 같은 수식으로 만든 isolated PNG는 rhwp 산출 SVG의 필터 결과와 채널별 평균 오차가 1px 이하로 일치했다.
따라서 남는 차이는 보정 수식 자체의 불일치가 아니라 전체 문서 렌더링에서 겹쳐지는 텍스트/선/스케일링 차이에 가깝다.

비-RealPic 워터마크(`issue-677/복학원서` 계열)는 이번 색상 보정 대상이 아니므로 기존 brightness/contrast 필터와 opacity 0.17을 유지한다.

## 3. export-svg 검증

산출물:

```text
/private/tmp/rhwp-task975/svg-watermark-fit-final-143/143E433F503322BD33.svg
/private/tmp/rhwp-task975/svg-watermark-fit-final-253/253E164F57A1BC6934_001.svg
/private/tmp/rhwp-task975/svg-watermark-fit-final-253/253E164F57A1BC6934_002.svg
```

명령:

```text
target/debug/rhwp export-svg /Users/melee/Downloads/143E433F503322BD33.hwp -o /private/tmp/rhwp-task975/svg-watermark-fit-final-143
target/debug/rhwp export-svg /Users/melee/Downloads/253E164F57A1BC6934.hwp -o /private/tmp/rhwp-task975/svg-watermark-fit-final-253
```

검증 결과:

```text
/private/tmp/rhwp-task975/svg-watermark-fit-final-143/143E433F503322BD33.svg
  rhwp-realpic-watermark-tone filter 1개
  opacity="0.21729612" 1개
  rhwp-img-bc-b-50c70 매칭 없음

/private/tmp/rhwp-task975/svg-watermark-fit-final-253/253E164F57A1BC6934_001.svg
  rhwp-realpic-watermark-tone filter 1개
  opacity="0.21729612" 1개
  rhwp-img-bc-b-50c70 매칭 없음

/private/tmp/rhwp-task975/svg-watermark-fit-final-253/253E164F57A1BC6934_002.svg
  rhwp-realpic-watermark-tone filter 1개
  opacity="0.21729612" 1개
  rhwp-img-bc-b-50c70 매칭 없음
```

공통 SVG 필터:

```xml
<filter id="rhwp-realpic-watermark-tone" color-interpolation-filters="sRGB">
  <feColorMatrix type="saturate" values="0.9165"/>
  <feComponentTransfer>
    <feFuncR type="linear" slope="0.9313" intercept="0.0344"/>
    <feFuncG type="linear" slope="0.9313" intercept="0.0344"/>
    <feFuncB type="linear" slope="0.9313" intercept="0.0344"/>
    <feFuncA type="identity"/>
  </feComponentTransfer>
  <feComponentTransfer>
    <feFuncR type="linear" slope="2.0972" intercept="0.0000"/>
    <feFuncG type="linear" slope="2.0972" intercept="0.0000"/>
    <feFuncB type="linear" slope="2.0972" intercept="0.0000"/>
    <feFuncA type="identity"/>
  </feComponentTransfer>
</filter>
```

판정:

```text
143 샘플: PageBackground Center 배치 유지 + 색상 워터마크 톤 보정 적용
253 샘플: 표 셀 배경 FitToSize 유지 + 색상 워터마크 톤 보정 적용
```

`253E164F57A1BC6934.hwp` export 중 발생한 `LAYOUT_OVERFLOW` 2건은 기존 표 크기/페이지 overflow 경고이며 워터마크 색상 보정과 무관하다.

## 4. 자동 검증

포맷:

```text
cargo fmt --all -- --check
```

결과: 통과.

좁은 회귀 테스트:

```text
cargo test --lib realpic_watermark
```

결과:

```text
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1302 filtered out
```

lib 전체 테스트:

```text
cargo test --lib
```

결과:

```text
test result: ok. 1312 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 42.10s
```

SVG snapshot 통합 테스트:

```text
cargo test --test svg_snapshot
```

결과:

```text
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.79s
```

wasm target check:

```text
cargo check --target wasm32-unknown-unknown --lib
```

결과:

```text
Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.60s
```

기존 warning 6건은 이번 변경과 무관한 기존 경고다.

## 5. rhwp-studio 확인 환경

수정된 Web Canvas 경로가 반영되도록 WASM 산출물을 재빌드했다.

```text
docker-compose --env-file .env.docker run --rm wasm
```

결과:

```text
Finished `release` profile [optimized] target(s) in 39.64s
[INFO]: :-) Done in 1m 03s
[INFO]: :-) Your wasm pkg is ready to publish at /app/pkg.
```

갱신된 WASM:

```text
/Users/melee/Documents/projects/forks/rhwp/pkg/rhwp_bg.wasm
Content-Length: 4625686
Last-Modified: Mon, 18 May 2026 09:00:31 GMT
```

현재 확인 URL:

```text
http://127.0.0.1:7700/
```

7700 서버는 갱신된 루트 `pkg/` WASM을 서빙한다.

## 6. 미실행 항목

`native-skia` 기반 `export-png`는 실행하지 않았다.

사유:

```text
1. Stage 1에서 확인한 Skia 경로는 이미 Some(image.fill_mode)를 전달한다.
2. 이번 시각 확인 대상은 export-svg와 rhwp-studio(Web Canvas/WASM)다.
3. 현재 기본 target/debug/rhwp는 native-skia feature 없이 빌드되어 export-png를 실행할 수 없다.
```

## 7. 결론

Stage 4 기준으로 #975의 핵심 결함과 색상 워터마크 보정은 해소됐다.

```text
PageBackground Center 이미지
  before: page bbox 전체 stretch
  after: 원본 512×512 중앙 배치

RealPic 색상 워터마크 preset
  before: PageBackground 또는 셀 배경 경로별 처리 불일치
  after: saturate(0.9165) + contrast(0.9313) + brightness(2.0972) + opacity(0.21729612) 공통 적용
```

다음 단계는 최종 보고서 작성이다.
