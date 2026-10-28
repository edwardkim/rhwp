# Task #1692 Stage 2 - SO-SUEOP PDF 기준 시각 검증 재분석

## 배경

Stage 1에서는 #1692의 직접 원인인 HWP3 글자색 손실만 확인했다. 그러나 `pdf/SO-SUEOP-2024.pdf`를
정답 기준으로 두고 페이지 단위 시각 검증을 다시 수행한 결과, 색상 복구만으로는 완료 판정을 할 수
없음이 확인됐다.

## 기준 파일

- PDF 기준: `pdf/SO-SUEOP-2024.pdf`
- HWP3 원본: `samples/SO-SUEOP.hwp`
- HWPX 기준본: `samples/SO-SUEOP.hwpx`

## 페이지 수

- PDF 기준: 46쪽
- HWP3 rhwp 렌더: 48쪽
- HWPX rhwp 렌더: 48쪽

## 재현 명령

```bash
pdftoppm -r 96 -png pdf/SO-SUEOP-2024.pdf tmp/visual-1692-full/pdf/pdf
./target/debug/rhwp export-svg samples/SO-SUEOP.hwp -o tmp/visual-1692-full/hwp3_svg
./target/debug/rhwp export-svg samples/SO-SUEOP.hwpx -o tmp/visual-1692-full/hwpx_svg
```

SVG는 `rsvg-convert`로 PNG 변환 후 페이지별 contact sheet와 이미지 차이 점수를 확인했다.

## 주요 관찰

- HWPX는 PDF 1~3쪽과 HWPX 1~3쪽이 대응한다.
- HWPX 4쪽은 `pi=90` 빈 문단 하나만 포함한 사실상 빈 페이지다.
- PDF 4쪽 내용은 HWPX 5쪽과 가장 잘 대응한다. 즉 HWPX 4쪽의 불필요한 단독 빈 페이지가
  이후 페이지를 한 장씩 밀고 있다.
- `dump-pages samples/SO-SUEOP.hwpx -p 3` 결과:
  - 페이지 4에는 `FullParagraph pi=90`, 빈 문단, `vpos=63012`만 존재한다.
- `dump samples/SO-SUEOP.hwpx -s 0 -p 90` 결과:
  - `text_len=0`, controls 없음
  - `line_seg vpos=63012`, `lh=1000`, `ls=400`
- 다음 문단 `pi=91`은 `vpos=0`으로 재시작한다.

## 1차 원인 후보

한컴 PDF에서는 이 빈 문단만 별도 페이지를 만들지 않는다. 따라서 페이지 끝의 빈 문단이 overflow를
유발하고 다음 보이는 문단이 `vpos=0`으로 재시작하는 경우, 해당 빈 문단은 페이지 높이를 소비하는
본문으로 취급하지 않아야 한다.

## 남은 확인

- HWPX 빈 문단 단독 페이지 제거 후 전체 페이지 수와 PDF 매칭을 재확인한다.
- 같은 보정이 HWP3에도 적용되는지 확인한다.
- HWP3는 HWPX와 달리 초반 4~5쪽에서 내용 밀도 차이가 있어 별도 원인이 남을 수 있다.
