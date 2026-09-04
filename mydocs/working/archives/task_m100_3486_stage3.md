---
kind: investigation
status: active
canonical: mydocs/manual/codex/docs_and_git_workflow.md
last_verified: 2026-07-29
---

# Task #3486 Stage 3 — HWP5/HWPX 비교 계약과 쪽 배경 색조 원인 분리

- 이슈: [#3486](https://github.com/edwardkim/rhwp/issues/3486)
- 브랜치: `task_m100_3486_hwp3_render_fidelity_v2`
- 기준 오라클: `pdf/HWP3-password-123456.pdf`
- 범위: HWP3/HWP5/HWPX 공통 문서 계열의 parser 계약을 고정하고, 중앙 쪽 배경 그림이 기준 PDF보다
  옅게 표시되는 원인을 renderer 경계까지 좁힌다.

## 이번 중간 커밋의 입력·계약 보완

| 항목 | 확인 결과 | 목적 |
| --- | --- | --- |
| `samples/HWP5-nopassword-123456.hwp` | SHA-256 `a34ecb8cde85b6db49c64a954cb7fa5d23b5f49367bc4753c90bfe89a075b50d`, 24쪽·365문단 | HWP3과 같은 원본 계열의 평문 HWP5 비교 입력을 보존 |
| `samples/HWP5-nopassword-123456.hwpx` | 23쪽 | HWPX는 별도 PDF 기준선을 확보할 때까지 HWP3의 24쪽 오라클과 같은 쪽 번호로 비교하지 않음 |
| `samples/HWP5-password-123456.hwpx` | 정상 복호화 뒤 평문 HWPX와 parser shape 일치, 23쪽 | 암호 HWPX의 비교 입력 계약을 보호 |
| HwpUnitChar 문단 간격 | `left/right/indent`는 HWP3 계열 scale, `prev/next`는 raw HwpUnitChar 값 | HWPX의 흐름 차이를 색조 결함으로 오인하지 않도록 단위 계약을 고정 |
| `ir-diff --password-stdin` | 암호가 제공되면 password parser를 사용하고, 오류는 CLI exit 계약으로 분류 | 암호 HWPX에서도 parser/IR 비교가 가능하도록 보완 |

## 쪽 배경 raw 값과 화면 의미

HWPX `Contents/header.xml`의 실제 이미지 채우기는 다음과 같다.

```xml
<hc:imgBrush mode="CENTER">
  <hc:img binaryItemIDRef="image1" bright="50" contrast="-15" effect="REAL_PIC" alpha="0"/>
</hc:imgBrush>
```

HWP5 legacy `ImageFill`의 raw 저장 순서를 따르는 공통 IR에서는 이것을
`brightness=-15, contrast=50`으로 보존한다. HWP3 fixture도 같은 raw 값, `REAL_PIC`, `CENTER`, BMP
배경을 가진다. 이 저장 계약을 parser에서 화면 의미에 맞춘다는 이유로 뒤집으면 IR diff/round-trip
기준이 무너진다.

그러나 현 SVG와 Web Canvas renderer는 raw 필드를 그대로 화면 색조 인자로 사용한다. 독립 probe에서
HWP3와 HWPX 모두 다음 SVG filter와 opacity를 만들었다.

```text
rhwp-img-bc-b-15c50
opacity="0.17"
```

이는 XML의 화면 의미 `brightness=50, contrast=-15`와 반대 순서다. 기준 PDF의 중앙 그림이 더 선명한
원인을 문서별 상수나 암호 처리로 추측하지 않고, **legacy raw 순서를 display 의미로 투영하지 않은
renderer 경계 결함**으로 분리했다.

`alpha="0"`와 현재 `is_watermark()`의 `0.17` opacity가 기준 PDF에 맞는지는 아직 결론 내리지 않는다.
다음 단계는 먼저 색조 순서만 `(50, -15)`로 바로잡아 HWP3 1쪽을 기준 PDF와 다시 대조하고, opacity는
그 결과와 별도 회귀 계약으로 판정한다. RAW 값에 의존하는 REAL_PIC preset과 일반 `ImageNode`는 이
변경 범위에 넣지 않는다.

## 실행한 focused 검증

검토 전용 target에서 실행했다.

```text
CARGO_TARGET_DIR=target/task_3486_render_v2 CARGO_INCREMENTAL=0 \
  cargo test --profile release-test --test hwpx_password_fixture
3 passed; 0 failed

CARGO_TARGET_DIR=target/task_3486_render_v2 CARGO_INCREMENTAL=0 \
  cargo test --profile release-test --lib hwpunitchar_spacing_keeps_hwp3_lineage_storage_scale
1 passed; 0 failed

CARGO_TARGET_DIR=target/task_3486_render_v2 CARGO_INCREMENTAL=0 \
  cargo test --profile release-test --lib test_img_brush_total_keeps_total_mode
1 passed; 0 failed
```

이 결과는 parser/CLI 비교 계약의 검증이다. 기준 PDF 대비 색조 수용 판정이나 최종 시각 검증을
주장하지 않는다.

## 다음 단계

1. `PageBackgroundImage`에 raw→display 색조 변환을 명시하고 SVG·Web Canvas·Skia에 같은 변환을
   적용한다.
2. HWP3/HWPX fixture를 이용한 SVG 회귀 테스트와 HWP3 기준 PDF 1쪽 visual sweep을 실행한다.
3. 색조·opacity·중앙 그림 위치를 독립 항목으로 기록한 뒤, 남은 폰트/본문 기하 차이를 별도 분석한다.
