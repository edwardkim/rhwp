# Task M100-1098 Stage 1

## 1. 목표

`exam_kor.hwpx`에서 출력되지 않는 요소를 HWP 원본과 비교해 확인하고, 우선순위가 높은 두 축을
구현한다.

```text
1. 바탕쪽: 짝수쪽, 홀수쪽, 마지막쪽
2. 글상자/사각형: 모서리 곡률
```

## 2. 입력

```text
samples/hwpx/exam_kor.hwpx
samples/exam_kor.hwp
```

## 3. 확인 결과

### 3.1 HWPX 바탕쪽

`exam_kor.hwpx`에는 `Contents/masterpage0.xml`부터 `Contents/masterpage8.xml`까지
9개의 바탕쪽 XML이 들어 있다.

`content.hpf` manifest의 관찰된 배치는 다음과 같다.

```text
masterpage0, masterpage1, masterpage2, section0
masterpage3, masterpage4, masterpage5, section1
masterpage6, masterpage7, masterpage8, section2
```

각 섹션마다 3개의 바탕쪽이 있고, root `masterPage@type`은 다음과 같이 대응된다.

```text
EVEN      -> 짝수쪽
ODD       -> 홀수쪽
LAST_PAGE -> 마지막쪽
```

기존 HWPX 파서는 `content.hpf`에서 section과 BinData만 수집하고 masterpage XML을 읽지 않았다.
따라서 렌더러가 이미 `SectionDef.master_pages`를 지원해도 HWPX 문서에서는 바탕쪽이 비어 있었다.

### 3.2 사각형 모서리 곡률

`exam_kor.hwpx`의 `hp:rect`에는 다음과 같은 `ratio` 값이 들어 있다.

```xml
<hp:rect ... ratio="10">
<hp:rect ... ratio="50">
```

기존 HWPX 파서는 모든 사각형을 `round_rate: 0`으로 생성했다. 렌더러는 이미
`RectangleShape.round_rate`를 사용하므로, 누락 지점은 파서였다.

## 4. 적용한 구현

### 4.1 HWPX masterpage 매핑

```text
content.hpf manifest 순서에서 section 앞에 위치한 masterpage 항목을 해당 section의 바탕쪽으로 묶는다.
masterpage*.xml을 MasterPage 모델로 파싱한다.
EVEN/ODD/LAST_PAGE를 기존 HeaderFooterApply + is_extension으로 매핑한다.
HWPX 출처 저장 시 raw child record가 없으면 MasterPage 모델에서 HWP5 LIST_HEADER + 문단 목록을 생성한다.
```

### 4.2 사각형 곡률 매핑

```text
hp:rect@ratio -> RectangleShape.round_rate
```

## 5. 생성 파일

```text
output/poc/hwpx/task1098/stage1_exam_kor_gap_trace/exam_kor-stage1.hwp
```

비교용 SVG:

```text
output/poc/hwpx/task1098/stage1_exam_kor_gap_trace/oracle_svg/exam_kor_001.svg
output/poc/hwpx/task1098/stage1_exam_kor_gap_trace/hwpx_svg/exam_kor_001.svg
output/poc/hwpx/task1098/stage1_exam_kor_gap_trace/generated_svg/exam_kor-stage1_001.svg
output/poc/hwpx/task1098/stage1_exam_kor_gap_trace/generated_svg/exam_kor-stage1_020.svg
```

## 6. 내부 검증

```text
cargo fmt --check
cargo check
cargo test parser::hwpx::content::tests::test_parse_content_hpf_master_pages_by_manifest_order
```

결과:

```text
success
```

## 18. PAGE 기준과 지문 문단 박스 렌더링 정정

작업지시자 시각 확인에서 `exam-kor-2p.hwpx` 마지막 페이지 바탕쪽의 우측 하단
`*확인사항` 박스가 정답 HWP보다 잘못된 위치에 출력되는 문제가 확인되었다.

원인:

```text
1. HWP/HWPX의 `PAGE` 위치 기준은 물리 용지 전체가 아니라 본문 영역 기준이다.
2. 바탕쪽은 본문보다 먼저 렌더링되므로, 기존 구현에서는 current_body_area가 아직 채워지지 않은
   상태에서 masterpage 내부 표 위치를 계산했다.
3. 그 결과 LAST_PAGE 바탕쪽 내부 표의 `horzRelTo=PAGE`, `vertRelTo=PAGE`가 종이 기준처럼 계산되었다.
```

수정:

```text
build_master_page()에서 masterpage 내부 객체를 배치하기 전에
현재 paper width와 body area context를 명시적으로 설정한다.

PAPER 기준:
  paper_area 사용

PAGE 기준:
  current_body_area를 통해 본문 영역 기준으로 계산
```

또한 4~9번 문제의 지문 박스가 HWPX 렌더링에서 누락되는 문제가 확인되었다.

정답 HWP와 HWPX source 비교 결과, 4~9번 지문 박스는 표가 아니라 `지문` 문단 스타일의 문단
테두리다.

```text
HWPX header.xml:
  paraPr id=36 styleName=지문
  borderFillIDRef=10

borderFill #10:
  left/right/top/bottom type=SOLID
  width=0.12 mm
  color=#000000
```

기존 렌더러는 border line type이 존재해도 내부 width index가 `0`이면 보이지 않는 테두리로
판정했다. 그러나 HWP/HWPX의 border width index `0`은 없음이 아니라 가장 얇은 선 폭 계열이다.
선 없음 여부는 width가 아니라 `line_type=None`으로 판단해야 한다.

수정:

```text
1. 문단 테두리 병합/출력 여부를 line_type 기준으로 판정한다.
2. 표 외곽 fallback border도 동일하게 line_type 기준으로 판정한다.
3. width index 0은 보이는 선으로 렌더링한다.
```

비교 산출물:

```text
output/poc/hwpx/task1098/stage15_hwpx_passage_box_compare/oracle_2p/exam-kor-2p_002.svg
output/poc/hwpx/task1098/stage15_hwpx_passage_box_compare/hwpx_2p/exam-kor-2p_002.svg
```

비교 결과:

```text
정답 HWP page 2 line count: 22
HWPX page 2 line count:     22
```

지문 박스 핵심 좌표도 정답 HWP와 HWPX 렌더링이 같은 위치에 출력된다.

```text
왼쪽 지문 박스:
  top:   x=117.1733..540.4533, y=242.4133
  left:  x=117.1733, y=242.4133..1412.8933
  right: x=540.4533, y=242.4133..1412.8933

오른쪽 지문 박스:
  top:   x=582.0533..1005.3333, y=1020.3733
  left:  x=582.0533, y=211.6533..1020.3733
  right: x=1005.3333, y=211.6533..1020.3733
```

남은 차이:

```text
정답 HWP의 해당 선 두께는 SVG 기준 stroke-width=0.5
HWPX 렌더링은 source의 width=0.12 mm를 width index 0으로 파싱하여 stroke-width=0.4
```

이번 단계에서는 박스 미출력 문제를 해결한 것으로 보고, 두께 매핑 차이는 별도 시각 판정 또는
후속 width contract 검증 대상으로 분리한다.

검증:

```text
cargo fmt --check
cargo test test_1098_hwpx_last_page_master_replaces_base_master
cargo build
```

결과:

```text
success
```

`dump` 확인:

```text
section0 바탕쪽: 3개
[0] Even
[1] Odd
[2] Both, is_ext=true, ext_flags=0x0002
사각형 round=10%
사각형 round=50%
```

## 7. 판정 요청

| file | 한컴 판정 유형 | 바탕쪽 짝/홀/마지막 | 사각형 모서리 곡률 | 마지막 페이지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|
| `output/poc/hwpx/task1098/stage1_exam_kor_gap_trace/exam_kor-stage1.hwp` |  |  |  |  |  |  |

## 8. Stage 2: 파일 손상 후보 보정

작업지시자 판정:

```text
Stage 1 생성 HWP는 한컴 에디터에서 파일 손상 판정.
```

Stage 1의 HWP5 record dump를 정답지와 비교한 결과, 단순 렌더링 누락이 아니라 HWPX 바탕쪽을
HWP5 `SECTION_DEFINE` 하위 record contract로 저장하는 과정에 누락이 있었다.

### 8.1 확인한 차이

정답지 `samples/exam_kor.hwp`:

```text
SectionDef flags = 0xC0000004
바탕쪽 3개
바탕쪽 감추기 = true
바탕쪽 내부 표:
  wrap=TopAndBottom
  vert=Paper/9921
  horz=Paper/8788
  size=66616x4820
마지막쪽 바탕쪽:
  is_ext=true
  overlap=true
  ext_flags=0x0003
```

Stage 1 생성본:

```text
SectionDef flags = 0x00000000
바탕쪽 내부 표:
  wrap=Square
  vert=Paper/0
  horz=Paper/0
  size=0x0
마지막쪽 바탕쪽:
  is_ext=true
  overlap=false
  ext_flags=0x0002
```

따라서 파일 손상 후보는 다음으로 좁혀졌다.

```text
1. hp:secPr@masterPageCnt, hp:visibility@hideFirstMasterPage가 HWP5 SectionDef flags에 반영되지 않음
2. section XML 파싱 뒤 package-level masterpage를 붙였지만, 첫 문단 SectionDef control 복사본이 갱신되지 않음
3. 바탕쪽 내부 문단의 표/도형이 HWPX to HWP adapter를 통과하지 않아 Table CTRL_HEADER가 비어 있음
4. LAST_PAGE 바탕쪽의 HWP5 확장/중복 플래그가 한컴 저장본과 다름
```

### 8.2 적용한 보정

```text
1. hp:secPr@masterPageCnt -> SectionDef flags bit 30-31
2. hp:visibility@hideFirstMasterPage -> SectionDef flags bit 2
3. SectionDef control이 이미 있어도 section.section_def 최신값으로 교체
4. SectionDef.master_pages 내부 문단도 HWPX to HWP adapter 적용
5. HWPX LAST_PAGE masterpage는 pageDuplicate="0"이어도 HWP5 ext_flags=0x0003으로 저장
```

### 8.3 생성 파일

```text
output/poc/hwpx/task1098/stage2_masterpage_list_header_contract/exam_kor-stage2.hwp
```

### 8.4 내부 검증

```text
cargo fmt --check
cargo check
cargo build
cargo test -q test_parse_master_page_last_page_extension
cargo test -q test_parse_content_hpf_master_pages_by_manifest_order
cargo test -q test_parse_rect_ratio_as_round_rate
```

결과:

```text
success
```

Stage 2 생성본 dump 요약:

```text
SectionDef flags = 0xC0000004
바탕쪽: 3개
바탕쪽 감추기: true
바탕쪽 내부 표:
  wrap=TopAndBottom
  vert=Paper/9921
  horz=Paper/8788
  size=66616x4820
마지막쪽 바탕쪽:
  is_ext=true
  overlap=true
  ext_flags=0x0003
사각형:
  round=10%
  round=50%
```

정답지와 여전히 다른 항목:

```text
1. HWP 자체 dump의 마지막 바탕쪽 apply label은 generated가 Even으로 표시된다.
   단, HWP5 바탕쪽 record에는 apply enum이 직접 저장되지 않고 순서와 ext_flags로 해석된다.
   이번 단계에서는 정답지와 같은 ext_flags=0x0003을 맞추는 것을 우선한다.

2. 일부 Shape common attr의 bit 26 차이:
   generated attr=0x044A2400, oracle attr=0x004A2400
   이 bit는 기존 HWPX GenShape 저장 후보 비트로, 한컴 판정 후 필요하면 별도 분리한다.
```

## 9. Stage 2 판정 요청

| file | 한컴 판정 유형 | 바탕쪽 짝/홀/마지막 | 사각형 모서리 곡률 | 마지막 페이지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|
| `output/poc/hwpx/task1098/stage2_masterpage_list_header_contract/exam_kor-stage2.hwp` | 파일손상 | - | - | - | - | Stage 1 파일손상 보정 후보 |

## 10. Stage 3: SectionDef CTRL_HEADER tail 후보

작업지시자 판정:

```text
Stage 2 생성 HWP도 한컴 에디터에서 파일손상 판정.
```

Stage 2는 바탕쪽 개수, 감추기 플래그, 내부 표/도형 배치, 마지막쪽 확장 플래그를 맞췄지만
정답지와 `SectionDef` CTRL_HEADER payload 길이가 여전히 달랐다.

정답지 `samples/exam_kor.hwp`:

```text
CTRL_HEADER SectionDef size = 47
ctrl_data size = 43
tail after first 24 bytes = 19 bytes
tail = 01 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
```

Stage 2 생성본:

```text
CTRL_HEADER SectionDef size = 38
ctrl_data size = 34
tail after first 24 bytes = 10 bytes
tail = 00 00 00 00 00 00 00 00 00 00
```

따라서 Stage 3 후보는 HWPX 출처 `SectionDef`에 바탕쪽이 있을 때 정답지와 같은
`대표Language=1 + 17 byte zero` 확장 tail을 materialize한다.

### 10.1 생성 파일

```text
output/poc/hwpx/task1098/stage3_sectiondef_tail_contract/exam_kor-stage3.hwp
```

### 10.2 내부 검증

```text
cargo fmt --check
cargo check
cargo build
cargo test -q test_parse_master_page_last_page_extension
```

결과:

```text
success
```

Stage 3 생성본의 `SectionDef` record 확인:

```text
CTRL_HEADER SectionDef size = 47
ctrl_data size = 43
tail after first 24 bytes = 19 bytes
```

### 10.3 판정 요청

| file | 한컴 판정 유형 | 바탕쪽 짝/홀/마지막 | 사각형 모서리 곡률 | 마지막 페이지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|
| `output/poc/hwpx/task1098/stage3_sectiondef_tail_contract/exam_kor-stage3.hwp` | 파일손상 | - | - | - | - | SectionDef tail 후보 |

## 11. Stage 4: SectionDef tail byte alignment

작업지시자 판정:

```text
Stage 3 생성 HWP도 한컴 에디터에서 파일손상 판정.
정답 HWP와 저장한 HWP의 파일 크기 차이도 큼.
```

파일 크기:

```text
samples/exam_kor.hwp                                      10M
output/poc/hwpx/task1098/stage3_sectiondef_tail_contract/exam_kor-stage3.hwp  6.7M
```

파일 크기 차이는 BinData/stream 저장 방식 차이도 포함할 수 있으므로, 우선 한컴 손상 판정에 직접
영향을 줄 수 있는 `SectionDef` record contract를 계속 좁힌다.

Stage 3는 `SectionDef` CTRL_HEADER 전체 크기를 정답지와 같은 47바이트로 맞췄지만, payload tail의
바이트 위치가 정답지와 달랐다.

정답지:

```text
head47 hash = 46a0d2d333cbe998
tail after first 24 bytes = 00 00 01 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
```

Stage 3:

```text
head47 hash = 3ce3b35abfa14f9e
tail after first 24 bytes = 01 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
```

따라서 Stage 4 후보는 바탕쪽이 있는 HWPX 출처 `SectionDef` tail을 다음처럼 materialize한다.

```text
대표Language = 0
observed marker = 1
zero padding = 15 bytes
total tail = 19 bytes
```

### 11.1 생성 파일

```text
output/poc/hwpx/task1098/stage4_sectiondef_tail_alignment/exam_kor-stage4.hwp
```

파일 크기:

```text
output/poc/hwpx/task1098/stage4_sectiondef_tail_alignment/exam_kor-stage4.hwp  6.7M
```

### 11.2 내부 검증

```text
cargo fmt --check
cargo check
cargo build
target/debug/rhwp hwp5-inventory samples/exam_kor.hwp --section 0 --out /tmp/exam_oracle_s0.md
target/debug/rhwp hwp5-inventory output/poc/hwpx/task1098/stage4_sectiondef_tail_alignment/exam_kor-stage4.hwp --section 0 --out /tmp/exam_stage4_s0.md
```

결과:

```text
success
```

Stage 4의 `SectionDef` CTRL_HEADER는 정답지와 바이트 단위 hash가 같아졌다.

```text
oracle  CTRL_HEADER#4 hash = 46a0d2d333cbe998
stage4  CTRL_HEADER#4 hash = 46a0d2d333cbe998
```

아직 남아 있는 section page-control 차이:

```text
PAGE_DEF bottom margin:
  oracle = 8504
  stage4 = 6904

PAGE_BORDER_FILL attr:
  oracle first/second = 0x00000001
  stage4 first/second = 0x00000041
```

### 11.3 판정 요청

| file | 한컴 판정 유형 | 바탕쪽 짝/홀/마지막 | 사각형 모서리 곡률 | 마지막 페이지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|
| `output/poc/hwpx/task1098/stage4_sectiondef_tail_alignment/exam_kor-stage4.hwp` |  |  |  |  |  | SectionDef tail byte alignment 후보 |

## 12. Stage 5: 1페이지 축소 샘플

작업지시자 요청에 따라 먼저 1페이지 샘플로 파일손상 원인을 좁힌다.

입력:

```text
samples/hwpx/exam-kor-1p.hwpx
samples/exam-kor-1p.hwp
```

생성 파일:

```text
output/poc/hwpx/task1098/stage5_1p_scope_probe/exam-kor-1p-stage5.hwp
```

파일 크기:

```text
samples/hwpx/exam-kor-1p.hwpx                                      865K
samples/exam-kor-1p.hwp                                            954K
output/poc/hwpx/task1098/stage5_1p_scope_probe/exam-kor-1p-stage5.hwp 667K
```

HWPX는 ZIP 컨테이너이며, 압축 전 내부 크기는 약 15.9MB다. `BinData/image2.bmp` 하나가
15.1MB지만 ZIP deflate로 약 638KB까지 압축된다. 정답 HWP와 생성 HWP 모두 `rhwp info` 기준
BinData 로드 크기는 같으므로, 파일 크기 차이는 주로 컨테이너/stream 압축률과 한컴 저장 부가 record
차이로 본다.

1페이지 샘플에서 확인된 내용:

```text
SectionDef CTRL_HEADER hash = 정답지와 동일
ColumnDef head16 = 정답지와 동일
PAGE_BORDER_FILL attr = generated first/second 0x00000041, oracle first/second 0x00000001
PAGE_DEF bottom margin = generated 6904, oracle 8504
```

따라서 다단 정보는 이 샘플의 파일손상 1차 원인으로 보지 않는다. 다음 후보는
`PAGE_BORDER_FILL` attr의 `0x40` bit다.

## 13. Stage 6: PAGE_BORDER_FILL attr 보정

HWPX `pageBorderFill@type`의 `BOTH/EVEN/ODD`는 적용 대상 구분이지만, 이 값이 HWP5
`PAGE_BORDER_FILL` attr bit `0x40`으로 저장된다는 근거는 1페이지 정답지에서 확인되지 않았다.

수정 후보:

```text
HWPX pageBorderFill apply type에서 PAGE_BORDER_FILL attr 0x40을 세우지 않는다.
```

생성 파일:

```text
output/poc/hwpx/task1098/stage6_page_border_fill_attr/exam-kor-1p-stage6.hwp
```

내부 검증:

```text
cargo fmt --check
cargo check
cargo build
target/debug/rhwp hwp5-inventory samples/exam-kor-1p.hwp --section 0 --out /tmp/exam_1p_oracle_stage6_s0.md
target/debug/rhwp hwp5-inventory output/poc/hwpx/task1098/stage6_page_border_fill_attr/exam-kor-1p-stage6.hwp --section 0 --out /tmp/exam_1p_stage6_s0.md
```

결과:

```text
success
```

`PAGE_BORDER_FILL` record 비교:

```text
oracle PAGE_BORDER_FILL#8  hash = aca95214524f33b2
stage6 PAGE_BORDER_FILL#8  hash = aca95214524f33b2

oracle PAGE_BORDER_FILL#9  hash = 9f4901fecea89ca9
stage6 PAGE_BORDER_FILL#9  hash = 9f4901fecea89ca9

oracle PAGE_BORDER_FILL#10 hash = 9f4901fecea89ca9
stage6 PAGE_BORDER_FILL#10 hash = 9f4901fecea89ca9
```

아직 남아 있는 section page-control 차이:

```text
PAGE_DEF bottom margin:
  oracle = 8504
  stage6 = 6904

FOOTNOTE_SHAPE second record:
  oracle  = ... 00 00 00 00 ...
  stage6  = ... 00 00 ff ff ...
```

판정 요청:

| file | 한컴 판정 유형 | 바탕쪽 짝/홀/마지막 | 사각형 모서리 곡률 | 마지막 페이지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|
| `output/poc/hwpx/task1098/stage6_page_border_fill_attr/exam-kor-1p-stage6.hwp` |  |  |  |  |  | 1페이지 PAGE_BORDER_FILL attr 후보 |

## 14. Stage 7: PAGE_DEF / FOOTNOTE_SHAPE 저장 contract 보정

작업지시자 판정:

```text
Stage 6 1페이지 후보도 한컴 에디터에서 파일손상 판정.
```

Stage 6에서 `PAGE_BORDER_FILL`는 정답지와 일치했으나 다음 차이가 남아 있었다.

```text
PAGE_DEF bottom margin:
  oracle = 8504
  stage6 = 6904

FOOTNOTE_SHAPE second record:
  oracle = ... 00 00 00 00 ...
  stage6 = ... 00 00 ff ff ...
```

원인:

```text
1. HWPX header version="1.4"를 HWP3-origin으로 식별하면서 page_def.margin_bottom 자체를
   1600 줄였다.
2. endNotePr noteLine type="NONE"에도 separator_margin_top 기본값 -1을 적용했다.
```

수정:

```text
1. HWPX->HWP 저장 contract에서는 PAGE_DEF margin_bottom 원본값을 보존한다.
   HWP3-origin pagination 보정은 page_def.pagination_bottom_tolerance로 분리한다.
2. noteLine type="NONE"인 미주 모양은 separator_margin_top=0을 유지한다.
```

생성 파일:

```text
output/poc/hwpx/task1098/stage7_page_control_contract/exam-kor-1p-stage7.hwp
```

내부 검증:

```text
cargo fmt --check
cargo check
cargo build
cargo test --test issue_554
target/debug/rhwp hwp5-inventory output/poc/hwpx/task1098/stage7_page_control_contract/exam-kor-1p-stage7.hwp --section 0 --out /tmp/exam_1p_stage7_s0.md
```

결과:

```text
success
issue_554: 12 passed
```

Stage 7에서 1페이지 정답지와 일치한 page-control record:

```text
PAGE_DEF#5          hash = 3c8f31ff3cecc3b7
FOOTNOTE_SHAPE#6    hash = d149bf91d63574cb
FOOTNOTE_SHAPE#7    hash = d61ff49ba1dd8317
PAGE_BORDER_FILL#8  hash = aca95214524f33b2
PAGE_BORDER_FILL#9  hash = 9f4901fecea89ca9
PAGE_BORDER_FILL#10 hash = 9f4901fecea89ca9
ColumnDef           hash = 2f7dc9b49adaf97c
```

판정 요청:

| file | 한컴 판정 유형 | 바탕쪽 짝/홀/마지막 | 사각형 모서리 곡률 | 마지막 페이지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|
| `output/poc/hwpx/task1098/stage7_page_control_contract/exam-kor-1p-stage7.hwp` |  |  |  |  |  | 1페이지 page-control contract 후보 |

## 15. Stage 12: 문서 진단 도구 재검토

작업지시자 요청:

```text
기존에 만들어 놓은 분석 도구 사용이 도움이 되는지도 확인
```

결론:

```text
도움이 된다. 이번 이슈는 "미구현 컨트롤이 빠졌다"보다
"같은 컨트롤을 HWP5 record tree의 어느 부모/레벨에 붙였는가" 문제로 좁혀졌다.
```

사용한 진단 흐름:

```text
1. hwp5-inventory로 oracle/generated BodyText record tree 확인
2. hwp5-inventory-diff로 1p~4p 고정 차이 확인
3. hwp5-contract-analyze로 HWPX / oracle HWP / generated HWP control graph 비교
```

산출물:

```text
output/poc/hwpx/task1098/stage12_contract_analyze_1p/
output/poc/hwpx/task1098/stage12_contract_analyze_2p/
output/poc/hwpx/task1098/stage12_contract_analyze_3p/
output/poc/hwpx/task1098/stage12_contract_analyze_4p/
```

### 15.1 컨트롤 개수는 맞는다

`exam-kor-1p` 기준 BodyText tag count는 정답지와 generated가 같다.

```text
CTRL_HEADER      39 / 39
LIST_HEADER      22 / 22
PARA_HEADER      60 / 60
PARA_CHAR_SHAPE  60 / 60
PARA_LINE_SEG    60 / 60
SHAPE_COMPONENT  21 / 21
SHAPE_PICTURE    11 / 11
SHAPE_POLYGON     4 / 4
SHAPE_RECTANGLE   6 / 6
TABLE             6 / 6
```

Control type count도 같다.

```text
AutoNumber  4 / 4
ColumnDef   4 / 4
Footer      1 / 1
GenShape   21 / 21
NewNumber   1 / 1
PageHide    1 / 1
SectionDef  1 / 1
Table       6 / 6
```

따라서 파일손상 원인을 단순한 컨트롤 누락으로 보면 안 된다.

### 15.2 문제는 SectionDef 하위 record tree다

`hwp5-contract-analyze`가 잡은 가장 큰 차이는 `SectionDef#0`의 하위 record 수다.

정답지:

```text
CTRL_HEADER:12
LIST_HEADER:10
PARA_HEADER:12
PARA_CHAR_SHAPE:12
PARA_LINE_SEG:12
PARA_TEXT:11
SHAPE_COMPONENT:6
SHAPE_PICTURE:2
SHAPE_POLYGON:2
SHAPE_RECTANGLE:2
TABLE:2
```

generated:

```text
CTRL_HEADER:19
LIST_HEADER:16
PARA_HEADER:21
PARA_CHAR_SHAPE:21
PARA_LINE_SEG:21
PARA_TEXT:19
SHAPE_COMPONENT:9
SHAPE_PICTURE:3
SHAPE_POLYGON:3
SHAPE_RECTANGLE:3
TABLE:4
```

같은 컨트롤들이 있지만 generated는 더 많은 문단/표/도형을 `SectionDef` 하위에 붙이고 있다.
이 차이는 1p, 2p, 3p, 4p에서 동일하게 반복된다.

### 15.3 LAST_PAGE 바탕쪽 직렬화가 의심 지점이다

HWPX 1페이지 축소 샘플에는 다음 masterpage가 있다.

```text
Contents/masterpage0.xml  type=EVEN
Contents/masterpage1.xml  type=ODD
Contents/masterpage2.xml  type=LAST_PAGE
```

현재 serializer는 HWPX 출처의 모든 `SectionDef.master_pages`를 `SectionDef` 하위 record로
materialize한다.

```text
src/serializer/control.rs
  serialize_section_def()
    for master_page in &sd.master_pages {
      serialize_master_page(master_page, level + 1, records);
    }
```

이 방식은 EVEN/ODD 바탕쪽에는 맞을 수 있지만, 정답지의 record tree와 비교하면 `LAST_PAGE`
바탕쪽까지 같은 위치에 붙이는 것은 한컴 저장본과 다르다.

정답지는 `idx=94`에서 `SectionDef` 하위 바탕쪽 record를 빠져나와 `ColumnDef`가 나온다.
반면 generated는 같은 지점에서 계속 `SectionDef` 하위에 `LIST_HEADER/PARA_HEADER/...`를 붙인다.

```text
oracle:
  idx 94  CTRL_HEADER ColumnDef  level=1 parent=PARA_HEADER#0

generated:
  idx 94  LIST_HEADER            level=2 parent=SectionDef CTRL_HEADER#4
  idx 95  PARA_HEADER            level=2 parent=SectionDef CTRL_HEADER#4
```

따라서 다음 구현 축은 `LAST_PAGE` masterpage를 HWP5에서 어떤 위치/레벨/부모로 저장해야 하는지
정답지 record tree에 맞춰 분리하는 것이다.

### 15.4 다음 작업 원칙

```text
1. 더 이상 무작위 후보 파일을 늘리지 않는다.
2. 1p sentinel에서 SectionDef 하위 record tree가 oracle과 같은지 먼저 맞춘다.
3. 그 뒤 2p/3p/4p로 확장한다.
4. 바탕쪽 출력 여부와 파일손상 여부를 분리해서 판정한다.
```

### 15.5 정정: 1p 실패 상태에서 2p/3p/4p 판정 요청은 잘못이다

작업지시자 판정:

```text
Stage 11의 1p/2p/3p/4p 생성 파일은 모두 한컴 에디터에서 파일손상 판정.
```

정정:

```text
1p sentinel이 파일손상인 상태에서 2p/3p/4p까지 한컴 판정을 요청한 것은 잘못된 진행이다.
```

이유:

```text
1. 1페이지짜리 축소 샘플도 파일손상이면, 같은 저장 contract를 적용한 2p/3p/4p도 실패하는 것이 정상이다.
2. 이 상태에서 페이지 수를 늘려 판정하면 정보량이 늘지 않는다.
3. 오히려 문제 축이 page growth인지 fixed contract인지 혼동하게 만든다.
4. 문서 진단 도구 매뉴얼의 sentinel 원칙에도 맞지 않는다.
```

따라서 이후 판정 요청은 다음 순서를 지킨다.

```text
1. 1p sentinel만 생성한다.
2. 한컴 에디터에서 1p가 파일손상 없이 열리는지 확인한다.
3. 1p가 통과한 뒤에만 2p/3p/4p로 확장한다.
```

현재 다음 단계의 유효한 목표는 2p/3p/4p 확장이 아니라, `exam-kor-1p`에서 확인된
`SectionDef` 하위 record tree 차이를 먼저 해소하는 것이다.

## 16. 전략 전환: HWP 저장보다 HWPX 바탕쪽 렌더링을 먼저 고정

작업지시자 판단에 따라 HWP 직렬화 후보 확장을 중단하고, 먼저 HWPX 자체를 rhwp-studio에서
정상 렌더링하는 쪽으로 순서를 바꾼다.

이유:

```text
1. HWPX 렌더링이 정확하지 않으면 HWP 저장 결과를 비교해도 원인 축이 섞인다.
2. exam 계열 문서는 바탕쪽(EVEN/ODD/LAST_PAGE)과 첫쪽 바탕쪽 감춤이 함께 쓰인다.
3. HWP 저장은 HWPX 렌더링 contract를 고정한 뒤 진행한다.
```

검증 산출물:

```text
output/poc/hwpx/task1098/stage13_hwpx_masterpage_rendering/exam-kor-1p.svg
output/poc/hwpx/task1098/stage13_hwpx_masterpage_rendering/oracle/exam-kor-1p.svg
output/poc/hwpx/task1098/stage13_hwpx_masterpage_rendering/hwpx_2p/exam-kor-2p_002.svg
output/poc/hwpx/task1098/stage13_hwpx_masterpage_rendering/oracle_2p/exam-kor-2p_002.svg
```

1페이지 단독 샘플은 `hideMasterPage=true` 때문에 바탕쪽이 출력되지 않는 것이 정답과 일관된다.
따라서 바탕쪽 렌더링 검증은 첫쪽 이후 페이지가 필요하다.

## 17. HWPX LAST_PAGE 바탕쪽 렌더링 규칙

`exam-kor-2p.hwpx`의 2쪽을 정답 HWP와 비교하면 기존 렌더링은 다음처럼 출력되었다.

```text
기존 HWPX 렌더링:
  홀홀수수형형
  22
  *확인사항 위치가 정답보다 위로 올라감

정답 HWP 렌더링:
  홀수형
  2
  *확인사항이 페이지 하단 바탕쪽 위치에 출력됨
```

원인:

```text
HWPX masterpage2.xml type=LAST_PAGE pageDuplicate="0"를
기본 ODD 바탕쪽 위에 추가 렌더링했다.
```

하지만 정답 동작은 마지막쪽 바탕쪽이 기본 홀/짝 바탕쪽을 대체하는 형태다.
`pageDuplicate="0"`은 렌더링 관점에서 기존 바탕쪽을 중복하지 않는 의미로 보아야 한다.

구현:

```text
1. MasterPage에 replace_base 플래그 추가
2. HWPX LAST_PAGE pageDuplicate="0" 파싱 시 replace_base=true
3. HWP5 저장 contract 보존을 위해 overlap/ext_flags는 기존처럼 보존
4. 렌더링 바탕쪽 선택 시 replace_base 확장 바탕쪽은 기본 바탕쪽을 대체
```

수정 후 검증:

```text
output/poc/hwpx/task1098/stage14_hwpx_masterpage_replace/exam-kor-2p_002.svg
```

수정 후 `exam-kor-2p.hwpx` 2쪽의 핵심 텍스트 그룹은 정답 HWP SVG와 일치했다.

```text
fixed HWPX:
  홀수형
  2
  *확인사항 y=1472.47

oracle HWP:
  홀수형
  2
  *확인사항 y=1472.47
```

회귀 방지 테스트:

```text
cargo test test_1098_hwpx_last_page_master_replaces_base_master
```

결과:

```text
success
```
