---
kind: investigation
status: completed
canonical: mydocs/manual/bug_hunting_playbook.md
last_verified: 2026-08-02
---

# Task #3738 Stage 8 — HWP 그림 21 caption 셀 정렬과 전체 쪽수 최초 분기

- 선행 commit: `c92c3f6aa` (`fix: #3738 HWPX 이월 그림 offset 복원`)
- 기준: 개인정보 제거 원본 HWP/HWPX와 각각의 한컴오피스 2020 PDF
  ([경로·SHA-256·Git/LFS 판정](../../pdf/pr3740/README.md))
- 방법: [bug-hunter playbook](../manual/bug_hunting_playbook.md)의 한컴 PDF 페이지별 대조 여정

## 전체 문서 재현 상태

현재 release-test binary의 `info --json`과 `pdfinfo` 결과는 아래와 같다.

```text
HWP   rhwp info pageCount = 225    HWP PDF Pages = 215   (+10)
HWPX  rhwp info pageCount = 224    HWPX PDF Pages = 215  (+9)
```

따라서 그림 21 보정은 p23의 독립된 시각 결함을 고친 것이며, 전체 문서의 쪽수 정합 완료를 뜻하지 않는다.
전체 최초 흐름 분기는 p66–68에 있으므로 이 Stage의 p23 보정 커밋 뒤 Stage 9에서 별도 처리한다.

## p23 그림 21의 실제 원인과 보정

HWP p23의 outer table `pi=339`은 1×2이고 두 cell 모두 `vertAlign=CENTER`다. 그림 21/22는 각각
5줄 `Bottom` caption과 spacing을 가진 `TopAndBottom + Para` floating picture다. 보정 전 rhwp render
tree는 그림 21 본체를 `y=198.4px`, caption 첫 줄을 `y=544.7px`에 두었다. 한컴 PDF의 caption 첫 줄
text bbox는 `371.37pt = 495.16px`(96 DPI)다. 다음 본문 bullet은 rhwp와 PDF 모두 약 `y=617.9px`이므로,
문제는 본문 flow나 페이지 소유권이 아니라 그림 안쪽 cell의 수직 정렬이었다.

기존 cell-center 계산은 `pic_h`만 cell 중앙에 두었다. Bottom caption의 높이와 spacing을 제외했으므로
그림 본체와 caption으로 이루어진 시각 블록 전체는 아래로 약 50px 밀렸고, caption은 다음 본문과 겹쳤다.
`src/renderer/layout/table_layout.rs`의 기존 Issue #2071 특수 경로에서만, Bottom caption의
`calculate_caption_height + spacing`을 `pic_h`에 더한 `aligned_visual_h`로 Center/Bottom을 계산했다.
Top caption, 일반 picture, pagination/table flow 계산은 바꾸지 않았다.

보정 뒤 같은 render tree의 그림 21 본체는 `y=148.3px`, caption 첫 줄은 `y=494.7px`이며, PDF의
`495.16px`와 `0.46px` 차이다. 다음 body bullet도 `y=617.9px`으로 유지된다. 이 좌표는 실제 HWP를
읽는 focused regression으로 고정했다.

## 페이지 수 최초 분기: p728 RowBreak 표와 table footnote

PDF text층과 rhwp `export-text --json`의 페이지별 문자 멀티셋을 대조하면 5–65쪽은 같은 물리 페이지에서
대조되고, 처음 유의미한 흐름 분기는 66–68쪽이다.

- 한컴 PDF 66쪽은 p728(7×2 `RowBreak` 표)의 0–3행을 본문 하단과 각주 76–77 사이에 둔다.
- 한컴 PDF 67쪽은 p728의 4–6행 뒤 p730/p733/p736 항목과 77–85번 각주를 함께 둔다.
- rhwp는 p728 전체를 다음 쪽으로 미루고, 뒤 항목까지 밀어 사실상 빈 쪽을 만든다.

실제 `TypesetEngine` trace에서 p728은 본문 잔여 `622.2px`인데, table 전체 cell 각주를 미리 합산한
`fn=294.0px`를 예약해 첫 fragment의 가용 높이를 0으로 만든다. 그 결과 RowBreak first-row defer gate가
첫 행도 넣지 않고 table 전체를 다음 쪽으로 넘긴다. 현재 continuation 예외(#1937)는 fresh continuation
page에서만 이 전체 예약을 피한다.

또한 typeset 완료 뒤 table cell의 footnote를 전부 최종 fragment page에 등록한다. 기준 PDF처럼 첫
fragment의 anchor/각주는 p66에, 나머지는 p67에 귀속시키려면 fragment row 범위별 footnote 예약과
등록을 함께 바꿔야 한다. 전체 예약만 우회하면 각주가 없는 공간으로 그림/표를 밀어 넣는 잘못된 보정이
되므로 이 Stage에서는 구현하지 않았다.

구형 `Paginator::paginate_with_measured_opts`의 사전 전진 조건도 확인했지만, 이를 바꿔도 실제
`HwpDocument::page_count()`와 SVG는 달라지지 않았다. 사용자-visible 경로가 아닌 보정 후보는 되돌렸고,
Stage 9는 TypesetEngine의 fragment별 footnote ownership만 다룬다.

## 다음 경계

1. p728의 fragment row 범위를 기준으로 cell footnote를 예측·예약하고, 실제 fragment page에 한 번만
   등록하는 설계를 문서화한다.
2. focused regression과 p66–68 PDF visual sweep으로 첫 분기가 사라졌는지 확인한다.
3. 전체 pageCount가 HWP/HWPX 모두 215가 될 때까지 다음 최초 분기를 같은 방식으로 다시 분석한다.
