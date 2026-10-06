# Stage 4 보고 — Task M100-1271

## 범위

- 이슈: [#1271](https://github.com/edwardkim/rhwp/issues/1271)
- 단계: 대상 샘플 구조/시각 검증
- 대상 HWPX: `samples/hwpx/[2027] 온새미로 1 본교재.hwpx`
- 기준 PDF: `pdf-large/hwpx/[2027] 온새미로 1 본교재.pdf`

## 산출물

```text
output/poc/task1271_stage4/pdf/page-01.png
output/poc/task1271_stage4/pdf/page-02.png
output/poc/task1271_stage4/pdf/page-03.png
output/poc/task1271_stage4/pdf/page-04.png
output/poc/task1271_stage4/pdf/page-05.png

output/poc/task1271_stage4/svg/[2027] 온새미로 1 본교재_001.svg
output/poc/task1271_stage4/svg/[2027] 온새미로 1 본교재_002.svg
output/poc/task1271_stage4/svg/[2027] 온새미로 1 본교재_003.svg
output/poc/task1271_stage4/svg/[2027] 온새미로 1 본교재_004.svg
output/poc/task1271_stage4/svg/[2027] 온새미로 1 본교재_005.svg
```

SVG 시각 확인용 PNG 는 `qlmanage` 로 생성했다. `export-png` 는 현재 빌드가 `native-skia`
feature 없이 동작 중이라 사용할 수 없었다.

## PDF 기준 확인

명령:

```text
/Users/melee/.cache/codex-runtimes/codex-primary-runtime/dependencies/bin/pdfinfo "pdf-large/hwpx/[2027] 온새미로 1 본교재.pdf"
/Users/melee/.cache/codex-runtimes/codex-primary-runtime/dependencies/bin/pdftoppm -png -r 96 -f 1 -l 4 "pdf-large/hwpx/[2027] 온새미로 1 본교재.pdf" output/poc/task1271_stage4/pdf/page
/Users/melee/.cache/codex-runtimes/codex-primary-runtime/dependencies/bin/pdftoppm -png -r 96 -f 5 -l 5 "pdf-large/hwpx/[2027] 온새미로 1 본교재.pdf" output/poc/task1271_stage4/pdf/page
```

결과:

- PDF 기준본은 46쪽이다.
- 1쪽: 표지
- 2쪽: MEMO
- 3쪽: 1주차 간지
- 4쪽: 본문 시작, 하단 쪽번호 4가 왼쪽에 위치
- 5쪽: 다음 본문, 하단 쪽번호 5가 오른쪽에 위치

참고: Poppler 렌더 중 `Adobe-Korea1` language pack 과 일부 폰트 관련 경고가 출력되었다.
그래도 이번 검증 대상인 페이지 흐름, MEMO/간지/본문 위치, 홀짝 바탕쪽 방향은 확인 가능했다.

## HWPX 구조 확인

명령:

```text
cargo run --quiet --bin rhwp -- dump-pages "samples/hwpx/[2027] 온새미로 1 본교재.hwpx"
cargo run --quiet --bin rhwp -- dump-pages "samples/hwpx/[2027] 온새미로 1 본교재.hwpx" | rg -n "문서 로드|=== 페이지 1 |=== 페이지 2 |=== 페이지 3 |=== 페이지 4 |=== 페이지 45 |=== 페이지 46 |PartialTable"
```

결과:

```text
문서 로드: samples/hwpx/[2027] 온새미로 1 본교재.hwpx (46페이지)
=== 페이지 1 (global_idx=0, section=0, page_num=1) ===
=== 페이지 2 (global_idx=1, section=0, page_num=2) ===
=== 페이지 3 (global_idx=2, section=0, page_num=3) ===
=== 페이지 4 (global_idx=3, section=1, page_num=4) ===
=== 페이지 45 (global_idx=44, section=3, page_num=45) ===
=== 페이지 46 (global_idx=45, section=4, page_num=46) ===
```

확인:

- HWPX 렌더 결과도 46쪽이다.
- 앞 4쪽에 `PartialTable` 조각 페이지가 끼지 않는다.
- 2쪽은 `MEMO` 문단이다.
- 3쪽은 1주차 간지 shape 페이지다.
- 4쪽은 `section=1, page_num=4` 이고 첫 본문 문단은 `강의 01.` 이다.

## HWPX 시각/바탕쪽 확인

명령:

```text
cargo run --quiet --bin rhwp -- export-svg "samples/hwpx/[2027] 온새미로 1 본교재.hwpx" -o output/poc/task1271_stage4/svg -p 0
cargo run --quiet --bin rhwp -- export-svg "samples/hwpx/[2027] 온새미로 1 본교재.hwpx" -o output/poc/task1271_stage4/svg -p 1
cargo run --quiet --bin rhwp -- export-svg "samples/hwpx/[2027] 온새미로 1 본교재.hwpx" -o output/poc/task1271_stage4/svg -p 2
cargo run --quiet --bin rhwp -- export-svg "samples/hwpx/[2027] 온새미로 1 본교재.hwpx" -o output/poc/task1271_stage4/svg -p 3
cargo run --quiet --bin rhwp -- export-svg "samples/hwpx/[2027] 온새미로 1 본교재.hwpx" -o output/poc/task1271_stage4/svg -p 4
```

확인:

- 1쪽 표지는 기준 PDF와 같은 내용과 하단 곡선 배치를 보인다.
- 2쪽은 MEMO 단독쪽으로 유지된다.
- 3쪽은 1주차 간지로 유지된다.
- 4쪽은 본문 시작쪽이며 상단 장식과 하단 쪽번호가 짝수쪽 방향이다.
- 5쪽은 상단 장식과 하단 쪽번호가 홀수쪽 방향이다.

쪽번호 위치는 SVG 좌표로도 확인했다.

```text
4쪽: textbox-clip-4 x=60.453333..., text "4" x=60.453333...
5쪽: textbox-clip-4 x=495.120000..., text "5" x=733.240021...
```

즉 4쪽은 왼쪽 하단, 5쪽은 오른쪽 하단으로 기준 PDF의 홀짝 방향과 일치한다.

참고: `qlmanage` 썸네일은 일부 페이지에서 하단이 잘려 보일 수 있어, 하단 쪽번호 위치는
SVG 텍스트 좌표를 함께 사용해 확인했다.

## 회귀 테스트

명령:

```text
cargo test --test issue_1271_hwpx_behind_text_table -- --nocapture
```

결과:

```text
test onsaemiro_front_matter_is_not_shifted_by_behind_text_table_fragment ... ok
test result: ok. 1 passed
```

## 판단

- #1271 대상 샘플에서 PDF 기준본과 HWPX 렌더 결과의 총 페이지 수가 46쪽으로 일치한다.
- 원래 문제였던 2-3쪽 앞부분 밀림은 해소되어, 2쪽 MEMO와 3쪽 1주차 간지가 기준 PDF와 같은 위치에 있다.
- 4쪽이 최종 `page_num=4` 로 본문을 시작하고, 4쪽/5쪽의 바탕쪽 방향도 기준 PDF의 짝수/홀수 방향과 일치한다.
- Stage 5 에서는 전체 회귀 검증과 최종 보고서를 진행한다.
