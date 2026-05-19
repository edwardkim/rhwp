# Task M100-993 Stage 5 작업 기록

## 1. 목적

`mel-001`에서 분리한 축소 샘플 `tb-org-02`로 조직도 표 셀 내부 텍스트 배치 문제를
먼저 해결한다.

입력 샘플:

```text
samples/hwpx/tb-org-02.hwpx
samples/hwpx/hancom-hwp/tb-org-02.hwp
pdf-large/hwpx/tb-org-02.pdf
```

회귀 guard:

```text
samples/hwpx/mel-001.hwpx
samples/hwpx/hancom-hwp/mel-001.hwp
```

## 2. 진단 요약

처음 의심한 축은 “조직도 셀 텍스트가 한컴에서는 8pt인데 rhwp-studio에서는 10pt로
처리된다”는 것이었다.

그러나 `tb-org-02.hwpx`와 정답 HWP를 비교한 결과, 조직도 본문 텍스트의 주요 글자
크기는 10pt 계약이 맞다.

```text
charPr 14: height=1000 -> 10pt
charPr 15: height=800  -> 8pt, 주로 괄호 숫자 같은 보조 텍스트
paraPr 24: lineSpacing=PERCENT 80, margin prev=500(HwpUnitChar case)
```

따라서 이번 문제의 핵심은 글꼴 크기 자체가 아니라 병합 셀의 세로 방향 배치 계약이다.
문제가 되는 조직도 셀들은 HWPX와 정답 HWP 모두 셀 세로 방향 정렬이 `Top`으로
해석된다.

대표 셀:

```text
셀[151] r=4,c=24 rs=4,cs=2 text="한국|산업|안전|보건|공단"
HWPX dump: valign=Top
HWP  dump: valign=Top
```

즉 파서가 `Top`을 `Center`로 잘못 읽은 것이 아니라, 렌더러가 셀 내부 문단의 y 위치를
계산할 때 `Top` 셀의 line segment 위치 계약을 충분히 반영하지 못한 것이 핵심이다.

정리하면 다음 계약을 만족해야 한다.

```text
1. 병합 셀의 vertical_align=Top은 셀 내용 전체를 가운데로 밀어서는 안 된다.
2. HWP/HWPX가 각 문단에 LINE_SEG.vertical_pos를 제공하면, 그 값은 셀 상단 기준
   문단 위치로 해석해야 한다.
3. 셀 내부 여러 문단의 y 위치를 단순 누적하면 한컴 PDF보다 아래로 밀리고,
   결과적으로 Top 셀이 Center에 가까운 배치처럼 보인다.
```

대표 셀 dump:

```text
셀[151] text="한국|산업|안전|보건|공단"
p[0] ps_id=24 ls[0] vpos=500  lh=1000 ls=-200
p[1] ps_id=24 ls[0] vpos=1800 lh=1000 ls=-200
p[2] ps_id=24 ls[0] vpos=3100 lh=1000 ls=-200
p[3] ps_id=24 ls[0] vpos=4400 lh=1000 ls=-200
p[4] ps_id=24 ls[0] vpos=5700 lh=1000 ls=-200
```

이 패턴은 한컴이 셀 내부 문단을 `spacing_before + lineSpacing` 누적 결과로만 놓는 것이
아니라, `LINE_SEG.vpos`로 각 문단 top을 고정해 둔 사례로 해석한다.

정답 PDF와 현재 SVG 좌표를 비교하면 이 방향이 맞다. 예를 들어 `한국|산업|안전|보건|공단`
셀의 정답 PDF 첫 줄 bbox y는 약 `147.289pt`이고, 96dpi SVG 좌표계로 환산하면
약 `196.39px`이다. 현재 SVG의 텍스트 baseline은 `208.48px`이며 10pt 글꼴의 ascent를
감안하면 실제 glyph top은 이 값과 근접한다.

## 3. 구현 변경

수정 파일:

```text
src/renderer/layout/table_layout.rs
```

변경 내용:

```text
셀 내부 문단에 LINE_SEG.vertical_pos가 있으면, nested table이 아닌 경우 para_y를
cell_y + pad_top + vpos 위치로 재앵커링한다.
```

주의점:

```text
layout_composed_paragraph()가 paraPr.spacing_before를 다시 더하므로,
호출 전 para_y에서 spacing_before를 빼서 최종 line top이 vpos에 맞도록 보정했다.
```

이 변경은 조직도형 표처럼 하나의 병합 셀 안에 여러 짧은 문단이 있고,
`vertical_align=Top`, `lineSpacing=80%`, 음수 `line_spacing`, 문단별 `LINE_SEG.vpos`가
함께 들어오는 케이스를 대상으로 한다.

## 4. 생성 산출물

산출물 위치:

```text
output/poc/hwpx2hwp/task993/stage5_org_cell_font_size_probe/
```

생성 파일:

```text
output/poc/hwpx2hwp/task993/stage5_org_cell_font_size_probe/tb_org_02_after/tb-org-02.svg
output/poc/hwpx2hwp/task993/stage5_org_cell_font_size_probe/tb_org_02_hwp_after/tb-org-02.svg
output/poc/hwpx2hwp/task993/stage5_org_cell_font_size_probe/mel_001_after/mel-001_002.svg
```

대표 텍스트 위치 확인:

```text
tb-org-02.hwpx export와 tb-org-02.hwp export에서 대표 조직도 셀 텍스트의 y 좌표가 동일하게
나왔다.
```

예:

```text
청: y=267.1599999999999, font-size=13.333333333333334(px)
(:  y=282.22666666666663, font-size=10.666666666666666(px)
```

## 5. 실행한 검증

```text
cargo fmt --check
cargo test table_axis_materializes_hancom_record_contract --lib
cargo test captioned_table_materializes_hancom_caption_common_attr_bit --lib
cargo build
target/debug/rhwp export-svg samples/hwpx/tb-org-02.hwpx -o output/poc/hwpx2hwp/task993/stage5_org_cell_font_size_probe/tb_org_02_after --debug-overlay
target/debug/rhwp export-svg samples/hwpx/hancom-hwp/tb-org-02.hwp -o output/poc/hwpx2hwp/task993/stage5_org_cell_font_size_probe/tb_org_02_hwp_after --debug-overlay
target/debug/rhwp export-svg samples/hwpx/mel-001.hwpx -o output/poc/hwpx2hwp/task993/stage5_org_cell_font_size_probe/mel_001_after --debug-overlay
```

결과:

```text
cargo fmt --check: 성공
관련 단위 테스트 2개: 성공
cargo build: 성공
SVG export: 성공
```

`mel-001.hwpx` export에서는 기존부터 존재하던 `LAYOUT_OVERFLOW` 경고가 여러 페이지에서
출력되었다. 이번 단계에서는 조직도 셀 내부 문단 위치 개선 여부만 우선 판정한다.

## 6. 시각 판정 결과

작업지시자는 다음 파일을 정답 PDF와 비교해 시각 판정한다.

```text
output/poc/hwpx2hwp/task993/stage5_org_cell_font_size_probe/tb_org_02_after/tb-org-02.svg
pdf-large/hwpx/tb-org-02.pdf
```

보조로 `mel-001` 2페이지도 확인한다.

```text
output/poc/hwpx2hwp/task993/stage5_org_cell_font_size_probe/mel_001_after/mel-001_002.svg
```

판정 결과:

```text
셀 내부 세로 방향 윗쪽 맞춤 정상화.
조직도 병합 셀에서 텍스트가 가운데 정렬처럼 내려가던 문제가 해소됨.
```

이번 단계의 선결 조건은 통과했다. 다음 단계에서는 `mel-001`의 남은 문제를 다시 분리한다.

```text
1. 2페이지 첫 1x1 표 배경이 검정/비정상으로 저장되는 문제
2. 1페이지 처음 부분에 나타나는 정체 모를 선
```
