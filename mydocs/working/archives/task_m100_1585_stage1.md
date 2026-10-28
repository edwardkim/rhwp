# Task #1585 Stage 1 완료보고서 — 기준 정렬과 재현 확정

## 범위

- 선행 PR #1551 상태 확인
- 최신 `upstream/devel` 기준 코드 상태 확인
- #1270 첨부 HWPX의 `image2` 위치/속성 재확인
- 현재 기준에서 1페이지 SVG `image2` 누락 재현
- Stage 2 전제 조건 정리

## 기준 상태

작업 브랜치:

```text
local/task1585...upstream/devel
```

기준 커밋:

```text
d8e792fe Merge pull request #1538 from planet6897/pr-task1537
```

선행 PR #1551 상태:

```text
state=OPEN
mergedAt=null
mergeCommit=null
base=devel
head=local/task1270
```

따라서 2026-06-27 현재 최신 `upstream/devel`에는 #1551 변경이 아직 포함되어 있지 않다.

## 선행 PR 미반영 확인

현재 기준의 `layout_caption()`은 여전히 #1551 이전 형태다.

확인 위치:

```text
src/renderer/layout/picture_footnote.rs:610-670
```

관찰:

- `layout_caption()` 시그니처에 `bin_data_content` 인자가 없다.
- `layout_composed_paragraph()` 호출의 마지막 인자들이 `None`으로 남아 있다.
- `tests/issue_1270_caption_inline_image.rs`도 현재 기준에 없다.

이는 #1585 구현을 바로 시작하면 #1551 인라인 스레딩 변경을 중복 포함할 수 있음을 의미한다.

## 첨부 HWPX 구조 확인

대상 파일:

```text
/private/tmp/issue1270/서울 문화예술단체 지원사업.hwpx
```

`image2` 바이너리:

```text
BinData/image2.png
size: 7272 bytes
PNG image data, 232 x 37, 8-bit/color RGBA, non-interlaced
```

`Contents/section0.xml`에서 `image2`는 내부 표의 TOP caption 안에 존재한다.

핵심 XML:

```xml
<hp:caption side="TOP" fullSz="0" width="8504" gap="850" lastWidth="47591">
...
<hp:pic id="1490368362" zOrder="12" numberingType="PICTURE"
        textWrap="TOP_AND_BOTTOM" textFlow="BOTH_SIDES" ...>
...
<hp:curSz width="16100" height="2645"/>
...
<hc:img binaryItemIDRef="image2" bright="0" contrast="0"
        effect="REAL_PIC" alpha="0"/>
...
<hp:pos treatAsChar="1" ... vertRelTo="PARA" horzRelTo="COLUMN"
        vertAlign="TOP" horzAlign="LEFT" vertOffset="0" horzOffset="0"/>
...
<hp:shapeComment>그림입니다.
원본 그림의 이름: 26_SMS_BKCOLOR_KOR.png
원본 그림의 크기: 가로 3192pixel, 세로 526pixel</hp:shapeComment>
```

주의:

- `hp:pic@textWrap="TOP_AND_BOTTOM"`이므로 이슈 #1585의 플로팅 배치 대상이다.
- `hp:pos@treatAsChar="1"`도 존재하지만, 메인테이너가 분류한 문제 범위는 `hp:pic@textWrap` 기준의 플로팅 캡션 이미지 후속이다.

## 현재 기준 렌더 재현

실행:

```bash
cargo run --quiet --bin rhwp -- export-svg \
  /private/tmp/issue1270/*.hwpx \
  -p 0 \
  -o /private/tmp/rhwp-task1585-stage1-svg
```

결과:

```text
문서 로드 완료: /private/tmp/issue1270/서울 문화예술단체 지원사업.hwpx (56페이지)
  → /private/tmp/rhwp-task1585-stage1-svg/서울 문화예술단체 지원사업_001.svg
내보내기 완료: 1개 SVG 파일 → /private/tmp/rhwp-task1585-stage1-svg/
```

SVG `<image>` 개수:

```text
1
```

SVG에 방출된 유일한 image:

```xml
<image x="69.90666666666667"
       y="109.79999999999998"
       width="200"
       height="118.4"
       .../>
```

이는 기존 `image1`에 해당한다. `image2`, `26_SMS`, `SEOUL`, `MY SOUL` 관련 문자열은 SVG에서 발견되지 않았다.

## 판정

Stage 1 재현은 성공이다.

- 첨부 HWPX에는 `image2`가 정상 PNG로 존재한다.
- `image2`는 caption 내부 `hp:pic@textWrap="TOP_AND_BOTTOM"` 그림이다.
- 현재 최신 `upstream/devel` 기준 SVG 1페이지에는 `image1`만 방출되고 `image2`는 방출되지 않는다.
- 동시에 #1551은 아직 merge되지 않아, 구현 단계에서는 기준 정렬을 먼저 해결해야 한다.

## Stage 2 전제 조건

Stage 2 구현계획서 작성 전에 다음을 먼저 결정해야 한다.

1. #1551이 merge되었는지 다시 확인한다.
2. merge 완료라면 최신 `upstream/devel`로 rebase한 뒤 #1585 설계를 작성한다.
3. merge 전이라면 다음 중 하나를 작업지시자가 승인해야 한다.
   - #1551 merge 대기
   - #1551 변경을 #1585 브랜치에 포함해 통합 설계
   - #1551 브랜치를 base로 임시 구현계획 작성

권장 방향은 #1551 merge 후 최신 `upstream/devel`로 재기준화한 다음 #1585 구현계획서를 작성하는 것이다. 그래야 (a) 인라인 스레딩과 (b) 플로팅 캡션 이미지 변경이 PR 단위로 분리된다.

## 다음 단계

작업지시자 승인 후 Stage 2 구현계획서를 작성한다. 단, Stage 2 시작 시 #1551 상태를 다시 확인하고 기준 정렬 방식을 먼저 확정한다.
