# Task M100 #3930 Stage 4 - CharShape sentinel 실제 Hancom PDF 판정

- 이슈: [#3930](https://github.com/edwardkim/rhwp/issues/3930)
- 브랜치: `fix/issue-3930-save-layout-input`
- 시작 기준: `928cb282e` (`feat(diagnostic): CharShape sentinel probe 추가`)
- 기록일: 2026-08-05 KST
- 상태: 완료

## 목표

Stage 3의 raw CharShape sentinel probe가 원본 HWPX와 rhwp 저장 HWP의 Hancom 2020
`PrintToPDFEx` 시각 차이를 실제로 줄이는지 판정한다. production serializer와 MCP
server/client는 이 단계에서 수정하지 않는다.

## 시작 상태

- 기준 PDF: 원본 HWPX를 Hancom 2020 MCP로 출력한 383쪽 PDF.
- 기존 rhwp 후보: 383쪽이며 기준과 273쪽 raster byte-identical, 109쪽 pixel-changed.
- p53의 `<참고 자료>` title style은 HWPX logical `charPr`와 한컴 직접 HWP가 HWP5 inactive
  underline/strikeout/shadow sentinel에서 다르다.
- Stage 3 diagnostic은 semantic field가 같은 CharShape에 한해 oracle의 attr/shadow color만
  이식하며 ambiguous mapping을 fail-closed로 건너뛴다.

## 계획

1. `--only-char-shape-defaults`로 target HWP 하나만 생성하고 generation report의 matched,
   unmatched, ambiguous 수와 rhwp reload 결과를 확인한다.
2. p53 title이 참조하는 generated CharShape raw payload가 oracle sentinel으로 바뀌었는지
   structured CFB 검사로 확인한다.
3. 표준 MCP async lifecycle로 target HWP를 PDF로 변환한다. terminal status, validation,
   `PrintToPDFEx`/`PrintMethod=0`, page count, output bytes와 checksum을 기록한다.
4. download checksum을 확인한 뒤 기준·기존 후보·sentinel 후보의 96dpi 383쪽 raster를
   pixelmatch로 비교한다.
5. 효과가 없으면 sentinel 가설을 기각하고 production serializer 변경 없이 다음 raw axis로
   넘어간다. 효과가 있을 때만 별도 Stage에서 canonicalization 구현 계획을 세운다.

## 성공 기준

- probe가 semantic ambiguity를 넘어 raw record를 무차별 이식하지 않는다.
- PDF 성공만으로 결론내리지 않고 전수 raster 수치로 효과/무효를 판정한다.
- MCP server/client가 아닌 rhwp HWP5 저장 계약 문제인지 분리해 기록한다.

## 테스트 결과

각 실행 직후 명령, exit code, 생성 경로와 해석을 추가한다.

### CharShape-only probe 생성

실행 명령:

```bash
target/debug/rhwp hwp5-contract-probe \
  'output/task3930-stage1/mcp/2025-행정업무운영-편람-mcp.hwp' \
  'output/task3930-stage2/hancom-single-odd-picture/2025-행정업무운영-편람-rhwp-single-odd-picture.hwp' \
  --out-dir output/task3930-stage4/char-shape-sentinel-only \
  --only-char-shape-defaults
```

exit code `0`. 생성 파일은
`output/task3930-stage4/char-shape-sentinel-only/08_char_shape_defaults_only.hwp`다.

generation report 결과:

- `CHAR_SHAPE defaults`: `733`
- unmatched: `22`
- ambiguous: `182`
- DocInfo만 변경했고 BodyText section은 `0`개 변경했다.
- rhwp reload는 성공했고 내부 page count `387`을 보고했다. 이는 기존 Hancom PDF의 383쪽과
  다르므로, rhwp 자체 page count를 성공 판정에 사용하지 않는다. 실제 Hancom PDF page count와
  raster 전수 비교가 필요하다.

다음 raw trace에서 p53 title의 expected sentinel 교체를 먼저 확인한다.

`target/debug/rhwp hwp5-anchor-trace --help`는 exit code `0`으로
`<파일.hwp> --needle <텍스트> [--section N] [--window N] [--out <path>]` 계약을 확인했다.
다음 trace는 section `2`, 같은 anchor와 window를 명시해 generated 후보와 sentinel 후보를
비교한다.

두 `hwp5-anchor-trace`는 모두 exit code `0`, hit `1`을 반환했다. diff는 source path만 다르고
section body record 및 `PARA_CHAR_SHAPE` reference는 동일했다. 이는 probe가 DocInfo만 바꾼
계약과 일치한다. 다음 structured CFB 검사는 해당 reference ID의 `CHAR_SHAPE` payload가 실제로
oracle attr/shadow color로 치환됐는지 확인한다.

### p53 CharShape raw sentinel 치환 확인

`olefile`과 raw deflate DocInfo parser로
`char-shape-sentinel-check.tsv`를 생성했다. exit code `0`.

- oracle `0x0228`과 sentinel `0x0231`은 attr `fa00043c`, shadow color `c0c0c000`을 포함한
  74 byte payload 전체가 같다.
- oracle `0x0229`와 sentinel `0x0232`도 attr `f800043c`, shadow color `c0c0c000`을 포함한
  payload 전체가 같다.
- baseline `0x0231`/`0x0232`의 attr은 각각 `02000000`/`00000000`이었고, 후자는 shadow color
  `b2b2b200`도 달랐다. 따라서 probe가 p53 target의 oracle sentinel을 실제로 정확히 이식했다.

다음 HWP 2020 MCP async PDF 변환은 이 raw change 하나가 383쪽 출력에 미치는 효과를 판정한다.

### 실제 HWP 2020 MCP PDF 완료 및 byte integrity

MCP async job `112a68aa-93b4-49f6-ba0e-0a51e50ce88e`는 556초에 성공했다.

- `run_status=0`, validation `ok`, timeout false
- `PrintToPDFEx`, `PrintMethod=0`
- HWP editor/PDF pages: `383/383`, page match `ok`
- server output: `20,569,627` bytes, SHA-256
  `94d47c68d1f57e0ad672c8f01481b96da455cbacc4d73ac8e7c7b29aa7227edb`

client `download`도 성공했다. local `sha256sum`은 server SHA-256과 같고, `pdfinfo`는 383쪽,
`556 x 754 pts`, `20,569,627` bytes를 보고했다. 따라서 이 Stage의 MCP PDF는 server 변환과
client 전달 모두 정상이며 다음 96dpi raster 비교의 유효한 입력이다.

다음 테스트는 sentinel PDF 383쪽을 rasterize하고 기존 Stage 2 기준·baseline raster와
`pixelmatch(threshold=0.1, includeAA=false)`로 전수 비교한다.

raster 생성 전 `df -h /`와 기존 raster 디렉터리를 확인했다. root free space는 약 `832MB`이고,
기존 Stage 2 기준+baseline 96dpi raster는 `97MB`다. 새 sentinel raster 한 세트와 JSON report를
생성할 공간은 충분하며, 기존 공유 raster나 target 디렉터리는 삭제하지 않는다.

`pdftoppm -r 96 -png`는 exit code `0`으로 sentinel PDF raster `383`장을
`output/task3930-stage4/char-shape-sentinel-only/mcp/raster-96dpi/candidate/`에 생성했다.
PDF page count와 raster page count가 모두 383이므로, 다음 전수 비교에서 페이지 누락으로 인한
거짓 동일/차이 판정은 없다.

local Node resolution으로 `pixelmatch`와 `pngjs`를 확인했으나 exit code `1`로
`pixelmatch` module을 찾지 못했다. 비교 대상 PDF/raster에는 이상이 없고 workspace에 test
dependency를 추가하지 않기 위해, 다음 비교는 cache에만 설치되는 `npx --package` 실행으로
동일한 `pixelmatch(threshold=0.1, includeAA=false)` 알고리즘을 사용한다.

`npx --package` 재시도도 exit code `1`로 같은 module resolution 실패를 보였다. npm exec는
패키지 binary path만 노출하고 current working directory의 Node resolver에는 package root를
추가하지 않는다. 다음에는 repository 밖 cache prefix에 `pixelmatch@5.3.0`과 `pngjs@7.0.0`을
설치해 absolute `require`로 실행한다. source, `package.json`, lockfile은 변경하지 않는다.

`npm install --prefix /home/tsjang/.cache/rhwp-pixelmatch pixelmatch@5.3.0 pngjs@7.0.0`는
exit code `0`으로 3개 package를 cache prefix에만 설치했다. repository 상태와 dependency
manifest에는 변경이 없다. 다음 Node 비교는 이 cache의 CommonJS module을 absolute require해
기준/source, 기존 baseline, sentinel candidate를 같은 threshold로 전수 비교한다.

### 383쪽 raster 전수 비교

cache-prefix `pixelmatch@5.3.0`/`pngjs@7.0.0` 비교는 exit code `0`으로
`raster_compare.json`을 만들었다. 설정은 `threshold=0.1`, `includeAA=false`다.

| 비교 | byte-identical | pixel changed pages | pixel total | p4 | p53 | p304 | p383 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 기준 vs 기존 baseline | 273 | 109 | 392,833 | 718 | 10,774 | 3,485 | 132 |
| 기준 vs CharShape sentinel | 277 | 105 | 392,203 | 718 | 10,774 | 3,485 | 132 |
| 기존 baseline vs sentinel | 376 | 7 | 631 | 0 | 0 | 0 | 0 |

CharShape inactive sentinel은 실제 Hancom PDF에 영향이 있으나, 이 fixture에서는 7쪽/631픽셀의
작은 효과만 보였다. p53 title/reference box와 p304 반복 표 등 대표적인 큰 불일치는 변하지
않았다. 따라서 이것만으로 #3930을 해결했다고 볼 수 없으며, 다음에는 7쪽의 page-level delta를
확인해 production canonicalization을 독립 보정으로 채택할 가치가 있는지 판정한다.

### Sentinel 영향 쪽 식별

`page_deltas.tsv`는 baseline과 sentinel 사이에 바뀐 7쪽을 분리했다.

| 쪽 | 기준 vs baseline | 기준 vs sentinel | sentinel로 줄어든 픽셀 |
| --- | ---: | ---: | ---: |
| 79 | 200 | 0 | 200 |
| 82 | 13 | 0 | 13 |
| 149 | 75 | 0 | 75 |
| 222 | 1,631 | 1,491 | 140 |
| 223 | 754 | 676 | 78 |
| 231 | 2,390 | 2,293 | 97 |
| 369 | 27 | 0 | 27 |

7쪽 모두 기준 대비 pixel diff가 감소했고, 4쪽(79, 82, 149, 369)은 PNG까지 기준과 같아졌다.
sentinel이 새로운 시각 불일치를 만들지 않고 Hancom direct HWP 저장값 방향으로 단조 개선한다는
증거다. 다만 p53/p304의 대형 불일치는 독립 raw axis이므로, 다음 Stage는 이 canonical
CharShape serialization만 production code로 일반화하고 같은 full-document MCP PDF로 회귀를
검증한다.

## Stage 종료

Stage 4는 fail-closed raw probe가 실제 Hancom 2020 PDF에서 작지만 단조로운 개선을 만든다는
사실까지 검증했다. 이 결과는 production serializer가 모든 logical inactive style을 일괄
canonicalize해도 안전하다는 증명은 아니므로, Stage 5에서 production 저장 경로와 동일한 383쪽
MCP 검증을 별도로 수행한다.
