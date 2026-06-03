# Task M100-1201 Stage 4 완료 보고 — 대상 샘플 구조/시각 검증

## 작업 범위

구현계획서 Stage 4 범위에 따라 #1201 대상 HWPX/PDF 샘플에서 바탕쪽 masterpage가 section에 연결되고, EVEN/ODD가 반대로 적용되지 않는지 확인했다.

대상 파일:

```text
/Users/melee/Downloads/[2027] 온새미로 1 본교재.hwpx
/Users/melee/Downloads/[2027] 온새미로 1 본교재.pdf
```

## 구조 검증

명령:

```text
cargo run --bin rhwp -- dump "/Users/melee/Downloads/[2027] 온새미로 1 본교재.hwpx" | rg "=== 구역|바탕쪽:|\[[0-9]+\] (Even|Odd|Both)|p\[[0-9]+\]: cc=.*text="
```

확인 결과:

- HWPX는 rhwp 기준 47쪽으로 파싱된다.
- section 0부터 section 4까지 모두 `바탕쪽: 2개`가 채워진다.
- section 0:
  - `[0] Even` 텍스트: `2027 온새미로 II`
  - `[1] Odd` 텍스트: `독서 · 문학`
- section 1부터 section 4:
  - `[0] Even` 텍스트: `2027학년도 수능 대비`
  - `[1] Odd` 텍스트: `독서 · 문학`

해석:

- `content.hpf` manifest item id/href와 section XML의 `masterPage@idRef` 연결이 동작한다.
- `SectionDef.master_pages`가 비어 있지 않고, 각 section에 EVEN/ODD 바탕쪽이 들어간다.
- XML root의 `type="EVEN"`/`type="ODD"`가 각각 `HeaderFooterApply::Even`/`Odd`로 매핑된다.

## 시각 검증

생성물:

```text
output/poc/task1201_masterpage/stage4/[2027] 온새미로 1 본교재_004.svg
output/poc/task1201_masterpage/stage4/[2027] 온새미로 1 본교재_005.svg
output/poc/task1201_masterpage/stage4/[2027] 온새미로 1 본교재_006.svg
output/poc/task1201_masterpage/stage4/[2027] 온새미로 1 본교재_007.svg
output/poc/task1201_masterpage/stage4/png/[2027] 온새미로 1 본교재_004.png
output/poc/task1201_masterpage/stage4/png/[2027] 온새미로 1 본교재_005.png
output/poc/task1201_masterpage/stage4/png/[2027] 온새미로 1 본교재_006.png
output/poc/task1201_masterpage/stage4/png/[2027] 온새미로 1 본교재_007.png
tmp/pdfs/task1201/pdf_page_004.png
tmp/pdfs/task1201/pdf_page_005.png
tmp/pdfs/task1201/pdf_page_006.png
tmp/pdfs/task1201/pdf_page_007.png
```

PDF 기준 확인:

- PDF 4쪽: 짝수쪽 바탕쪽. 좌측 하단 `4 2027학년도 수능 대비`.
- PDF 5쪽: 홀수쪽 바탕쪽. 우측 상단 회색 bar, 우측 하단 `독서 · 문학 5`.
- PDF 6쪽: 짝수쪽 바탕쪽. 좌측 계열 머리말/꼬리말.
- PDF 7쪽: 홀수쪽 바탕쪽. 우측 계열 머리말/꼬리말.

rhwp SVG/PNG 확인:

- rhwp 5쪽은 홀수쪽 바탕쪽 형태인 우측 상단 회색 bar가 보인다.
- rhwp 6쪽은 `dump-pages` 기준 `page_num=6`이며 짝수쪽 바탕쪽 형태인 좌측 계열 머리말이 적용된다.
- rhwp 7쪽은 `dump-pages` 기준 `page_num=7`이며 홀수쪽 바탕쪽 형태인 우측 상단 회색 bar가 적용된다.
- SVG DOM에는 6쪽 하단 `6 2027학년도 수능 대비`, 7쪽 하단 `독서 7 문학` 텍스트가 존재한다.

주의:

- rhwp 파싱 결과는 47쪽, 원본 PDF는 49쪽이라 전체 쪽 번호의 1:1 비교는 아직 불안정하다. 이는 #1201의 masterpage 연결 문제와 별개인 기존 pagination/layout 차이로 본다.
- rhwp SVG의 footer 텍스트는 DOM에는 있으나 일부 PNG 렌더에서 매우 작거나 보이지 않는다. 해당 텍스트는 `font-size`가 극소값으로 출력되는 경향이 있어, #1201의 idRef 연결 문제보다는 별도 텍스트 metric/textbox 렌더링 이슈로 분리하는 것이 타당하다.

## 결론

Stage 4 범위에서 #1201의 핵심 완료 조건은 충족했다.

- section XML의 `masterPage@idRef`가 실제 샘플에서 수집된다.
- manifest id/href와 idRef가 연결되어 section별 `master_pages`가 채워진다.
- EVEN/ODD type은 XML 명시값 기준으로 매핑된다.
- 대상 페이지에서 홀짝 바탕쪽이 구조적으로 반전되지 않는다.

## 다음 단계

Stage 5에서는 회귀 검증과 최종 보고서를 작성한다.

예정 검증:

```text
cargo fmt --all --check
cargo test --lib hwpx
cargo test --test issue_1100_exam_social_hwpx_header
cargo test --test issue_1113_header_autonum_placeholder
```

Stage 5 착수 전 작업지시자 승인이 필요하다.
