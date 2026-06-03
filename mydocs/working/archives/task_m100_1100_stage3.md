# Task M100-1100 Stage 3 작업 기록

## 1. 목적

Stage 2에서 `exam_social.hwpx`의 본문 표 배치와 page count는 HWP 정답지와 맞췄지만,
작업지시자 확인 결과 웹 캔바스에서 바탕쪽이 보이지 않는 문제가 남아 있었다.

정정 사항:

```text
Stage 2 완료 판단은 성급했다.
HWP 화면을 기준으로 HWPX까지 해결된 것으로 오판한 상태였고,
HWPX 바탕쪽 도형 렌더링을 별도로 검증해야 했다.
```

## 2. 비교 결과

정답 HWP와 HWPX의 바탕쪽 구조는 동일하게 파싱된다.

```text
바탕쪽: 1개
적용: Both
문단: 2개
첫 문단 control:
  1. 세로선 도형
  2. 페이지 번호 표
```

하지만 Stage 2 산출물의 HWPX SVG에서 바탕쪽 세로선은 점처럼 렌더링되었다.

```text
HWP oracle:
  <line x1="514.006..." y1="132.16" x2="514.02" y2="1364.28" .../>

HWPX before Stage 3:
  <line x1="514.006..." y1="132.16" x2="514.006..." y2="132.16" .../>
```

## 3. 원인

원인은 두 축이었다.

```text
1. HWPX shape common attr가 HWP5 CTRL_HEADER CommonObjAttr bitfield로 materialize되지 않았다.
2. HWPX hp:line의 hc:startPt / hc:endPt를 LineShape.start/end로 파싱하지 않았다.
```

`exam_social.hwpx` 바탕쪽의 세로선 XML:

```xml
<hp:line textWrap="BEHIND_TEXT">
  <hp:curSz width="1" height="92409"/>
  <hp:sz width="1" widthRelTo="ABSOLUTE" height="92409" heightRelTo="ABSOLUTE"/>
  <hp:pos treatAsChar="0" flowWithText="0" allowOverlap="1"
          vertRelTo="PAPER" horzRelTo="PARA" vertAlign="TOP" horzAlign="CENTER"
          vertOffset="9912" horzOffset="0"/>
  <hc:startPt x="0" y="0"/>
  <hc:endPt x="100" y="100"/>
</hp:line>
```

`startPt/endPt`가 누락되면 `LineShape.start/end`가 모두 0으로 남는다. 렌더러는 `curSz`와
`orgSz` scale을 적용하더라도 시작점과 끝점이 같으므로 길이가 0인 선을 출력한다.

## 4. 수정 내용

수정 파일:

```text
src/parser/hwpx/section.rs
```

구현:

```text
1. hp:sz@widthRelTo / heightRelTo를 shape CommonObjAttr의 size criterion으로 파싱한다.
2. HWPX shape parsing 종료 시 CommonObjAttr bitfield를 materialize한다.
3. hc:startPt / hc:endPt를 LineShape.start/end로 파싱한다.
```

대표 검증 값:

```text
HWPX after Stage 3:
  masterpage line common.attr = 0x044a4700
  width_criterion = Absolute
  height_criterion = Absolute
  start = (0, 0)
  end = (100, 100)
```

## 5. 생성한 판정 파일

```text
output/poc/hwpx/task1100/stage4_hwpx_line_points_svg/exam_social_001.svg
```

Stage 3 수정 후 HWPX SVG:

```text
<line x1="514.0066666666667" y1="132.16"
      x2="514.02" y2="1364.28"
      stroke="#000000" stroke-width="1.5066666666666666"/>
```

이는 정답 HWP SVG의 바탕쪽 세로선 좌표와 일치한다.

## 6. 실행한 검증

```text
cargo fmt --check
cargo test parser::hwpx::section::tests::test_parse_hwpx_masterpage_line_materializes_shape_common_attr
cargo check
target/debug/rhwp dump samples/hwpx/exam_social.hwpx
target/debug/rhwp export-svg samples/hwpx/exam_social.hwpx -o output/poc/hwpx/task1100/stage4_hwpx_line_points_svg -p 0
docker compose --env-file .env.docker run --rm wasm
```

결과:

```text
성공
```

`pkg/`, `rhwp-studio/public/`, `web/`의 wasm 산출물도 새 빌드로 갱신했다.

## 7. 다음 단계

작업지시자가 웹 캔바스에서 `samples/hwpx/exam_social.hwpx`를 다시 시각 판정했다.

판정 기준:

```text
1. 바탕쪽 세로선이 HWP 정답지처럼 출력된다.
2. 바탕쪽 페이지 번호 표가 유지된다.
3. Stage 2에서 맞춘 4페이지 조판과 표 배치가 유지된다.
```

판정 결과:

```text
통과
```
