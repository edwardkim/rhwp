# Task M100 #3820 Stage 123 - 2025 행정업무운영 편람 HWP5 그림 계약 분석

- 이슈: [#3820](https://github.com/edwardkim/rhwp/issues/3820)
- 근거 댓글: [#3820 comment 5189062021](https://github.com/edwardkim/rhwp/issues/3820#issuecomment-5189062021)
- 기록일: 2026-08-11 KST
- 상태: 분석 진행 중 - 구현 보류

## 목적과 작업 규칙

원본 HWPX를 rhwp로 HWP5 저장한 뒤 Hancom Office 2020 `PrintToPDFEx`로 출력했을 때 남는
production fidelity 차이를 원인별로 분리한다. 이 Stage에서는 코드 보정을 확정하거나 커밋하지
않는다. 재현 가능한 관찰값, raw HWP5 계약, 가설의 적용 범위만 기록한 뒤 별도 구현 Stage를
연다.

이 문서 작성 전에 작업 트리에 실험적인 adapter/test 변경이 생겼다. 그 변경은 이 Stage의
수용된 구현이 아니며, 이 문서의 분석과 독립적인 재현 절차를 통과하기 전에는 커밋 또는 PR
근거로 사용하지 않는다.

## 재현 입력과 공정한 기준

| 역할 | 경로 | SHA-256 | 비고 |
| --- | --- | --- | --- |
| 원본 | `samples/2025 행정업무운영 편람(최종).hwpx` | `c6dd7e847a99f219681afc5a29c80a9665c04df9cda4d820a3350d739664fdf6` | HWPX 입력 |
| 한컴 직접 HWP | `samples/2025 행정업무운영 편람(최종).hwp` | `40d6d05eac4d55bdc4b0c62c42d93af104d5123b447581246f36fd15de7bd46f` | raw HWP5 비교 전용 |
| 저장소 PDF | `pdf/2025 행정업무운영 편람(최종)-2024.pdf` | `2cf19014c2835d3ca14014cc7f08c03850c2dc3b85c606bf4d70d864b1c568ef` | 383쪽, 555 x 752pt |
| Hancom 2020 HWPX 기준 | 임시 MCP 다운로드 | `52d8ac766f7021e0a2568e82f39e8859eca61f7a4b0ec37c7d1f79307ff82366` | 383쪽, 556 x 754pt, 19,657,247 bytes |
| Hancom 2020 rhwp baseline | 임시 MCP 다운로드 | `a7dea9b8ab45805cb31988b2dd949052d9ed5fc74fd3e0279429226a38b4da73` | 383쪽, 556 x 754pt, 20,569,644 bytes |

저장소의 `-2024.pdf`는 HWPX 기준 확인에는 보존하되, Hancom 2020 후보와 용지 크기가 달라
96dpi 픽셀 총량 비교에 직접 사용하지 않는다. production 저장 계약의 전수 수치는 원본 HWPX와
rhwp 저장 HWP를 **같은 Hancom 2020 `PrintToPDFEx`**로 출력한 556 x 754pt PDF끼리 계산한다.

## 현행 baseline

동일 엔진 PDF를 `pdftoppm -r 96 -png`으로 383쪽씩 rasterize하고
`pixelmatch(threshold=0.1, includeAA=false)`로 비교했다.

| 비교 | byte-identical | 픽셀 변경 쪽 | 변경 픽셀 |
| --- | ---: | ---: | ---: |
| 원본 HWPX Hancom 2020 vs rhwp baseline Hancom 2020 | 273 | 109 | 392,833 |

이는 이슈 댓글의 baseline과 정확히 같다. 따라서 현재 `upstream/devel`은 이 독립 입력 세트에서
기존 수치를 개선하거나 회귀시키지 않았다.

`pdf/2025 행정업무운영 편람(최종)-hwp-2020.pdf`는 2026-08-05의 별도 산출물이다. 현행 후보와
비교하면 370쪽, 9,704,073픽셀이 달라 현행 코드의 기준선으로 재사용할 수 없다.

## 우선 관찰 - p261 그림과 본문 레이어

변경 픽셀 상위 페이지는 p261(92,386), p239(13,655), p213(12,032), p211(11,584), p215(11,148),
p53(10,774)이다. p261의 PDF text layer는 기준/후보에서 동일하지만 후보 raster에서는 본문 위로
그림이 이동해 텍스트가 가려진다. 따라서 이는 텍스트 손실이나 페이지 owner 이동이 아니라
HWP5 그림 control의 paint/flow 계약 후보이다.

HWPX `Contents/section9.xml`의 p261 계열 그림은 다음 공통 조건을 가진다.

- `numberingType="PICTURE"`
- `textWrap="TOP_AND_BOTTOM"`
- `treatAsChar="1"`
- `vertRelTo="PARA"`, `horzRelTo="PARA"`
- `flowWithText="1"`, `allowOverlap="0"`

`※ 인계자는 인계함에서 인계신청 및 인계현황 확인` anchor를 section 9에서 HWP5 record로 대조한
결과는 다음과 같다.

| 항목 | 한컴 직접 HWP | rhwp baseline HWP |
| --- | --- | --- |
| GenShape CTRL_HEADER properties | `0x142a2311` | `0x042a2311` |
| 차이 | `0x10000000` (bit 28 set) | bit 28 clear |
| decoded common fields | `tac=true`, `TopAndBottom`, `Para/Para`, `flowWithText=true`, `allowOverlap=false` | 동일 |

이 차이는 semantic anchor fields가 같아도 HWPX-origin `PICTURE` control의 HWP5 bridge bit가
빠질 수 있음을 보여 준다. 다만 단일 raw 차이만으로 109쪽 전체의 원인 또는 안전한 보정 범위를
선언하지 않는다. Stage 2의 일반 `numberingType="PICTURE"` 보정이 문자처럼 취급되는 하위 집합에
어떻게 적용되는지, 같은 bit가 있는 다른 개체에 회귀가 없는지를 먼저 분석해야 한다.

## source-of-truth 및 범위 분석

HWP5 parser는 GenShape CTRL_HEADER의 bit 28을
`CommonObjAttr.hwp5_gen_shape_attr_bit28`으로 읽는다. HWPX generic object parser에는
`numberingType="PICTURE"`를 `ObjectNumberingType::Picture` 및 bit 28 boolean으로
materialize하는 경로가 있다. 반면 HWP5 serializer는 `common.attr != 0`이면 semantic field를
다시 pack하지 않고 기존 packed attr를 그대로 기록한다.

p261 baseline에서는 최종 HWP5 raw attr의 bit 28이 clear인 사실만 확정됐다. 그 그림의 `hp:pic`
parser가 generic materialization을 실제로 공유하는지, 아니면 PICTURE 전용 parse path가
boolean/packed attr를 누락하는지는 아직 미확정이다. 그러므로 "boolean=true와 stale attr의
동기화 누락"은 구현 가설이지 확정 원인이 아니다. object parser별 attr pack 시점과 adapter가
IR을 변경하는 모든 경로를 추적하기 전에는 이를 전역 serializer 정책으로 확장하지 않는다.

원본 HWPX XML을 section별 XPath로 집계한 결과는 다음과 같다.

| 집합 | 개수 |
| --- | ---: |
| `hp:pic` + `numberingType="PICTURE"` | 74 |
| 위 집합 + `treatAsChar="1"` | 61 |
| 위 집합 + `textWrap="TOP_AND_BOTTOM"` | 60 |
| section 9의 PICTURE 전체 / inline / inline TopAndBottom | 25 / 13 / 13 |

즉 p261은 단일 이미지 예외가 아니라 최소 60개의 inline TopAndBottom 그림이 있는 계약 군의
한 사례다. 보정 후보는 정확한 source predicate와 raw attr 동기화 범위를 문서화해야 하며,
단일 `instid`, 특정 section, 또는 시각 차이가 난 쪽 번호를 조건으로 삼지 않는다.

## 코드 변경 금지 상태

이 Stage의 현재 산출물은 문서와 임시 측정물뿐이다. HWPX adapter, common-object serializer,
회귀 test는 이 분석 단계에서 변경 대상으로 지정하지 않는다. 다음 implementation Stage를 열기
전에 raw `attr`가 stale해지는 호출 순서와 74개 후보 중 Hancom 직접 HWP bit 28 분포를 별도
원장으로 확정한다.

## 이미 기각된 축

이 Stage는 아래 기존 #3930 분석 결론을 다시 구현하지 않는다.

- table record bit 26 단독 교체: Hancom PDF 무변화
- cell `LIST_HEADER width_ref` 0/0x0400: Hancom PDF 무변화
- HWPX embedded BinData `NotAccessed` metadata: Hancom PDF 무변화
- 모든 inactive CharShape의 전역 sentinel canonicalization: baseline보다 개선 없음

CharShape fail-closed probe의 7쪽/631픽셀 단조 개선은 별도 provenance 문제이며, p261 그림
계약과 같은 원인으로 묶지 않는다.

## 구현 전 분석 게이트

다음 항목을 문서화한 뒤에만 별도 구현 Stage에서 코드를 수정한다.

1. HWPX parser, HWPX-to-HWP adapter, HWP5 common-object serializer 각각에서 bit 28의 source of truth와 raw `attr` 우선순위를 기록한다.
2. 원본 HWPX의 `numberingType="PICTURE"` 그림을 `treatAsChar`, wrap, group level, section별로 집계해 후보 범위를 정한다.
3. 한컴 직접 HWP와 rhwp baseline HWP에서 같은 control 집합의 bit 28 분포와 p261 외 상위 변경 페이지의 연결을 기록한다.
4. p53의 참고 상자처럼 그림이 아닌 도형/CharShape 축은 별도 후보로 유지한다. p261 결과를 전역 도형 보정의 근거로 일반화하지 않는다.
5. 구현 후보마다 focused Rust regression, HWP raw anchor trace, 동일 Hancom 2020 383쪽 raster 비교를 수용 게이트로 둔다. 개선 쪽뿐 아니라 새 pixel-changed page가 없는지 함께 기록한다.

## 분석 산출물

재생성 가능한 임시 산출물은 다음 아래에 분리했다. 저장소 추적 파일이나 기존 PDF는 변경하지 않는다.

```text
/tmp/rhwp-3820-production-fidelity-residual/
  baseline/
  reference-hwpx-2020/
  raster-96dpi/
  hancom2020-raster-compare.json
  anchor-p261/
```

## 다음 단계

이 문서의 분석 게이트 1~3을 완료해 bit 28 보정이 실제로 p261 계열에만 필요한지와 raw `attr`
동기화 책임 위치를 확정한다. 그 결과를 별도 implementation Stage에 적은 뒤에만 코드 변경을
재개한다.
## 런타임 모델 대조: p261 anchor

기존 `rhwp dump`만 사용해 원본 HWPX와 직접 저장된 HWP의 같은 논리 위치를 대조했다. 대상은 section 9의 `문단 9.101`이고, 다음 문단의 anchor 문구는 `※ 인계자는 인계함에서 인계신청 및 인계현황 확인`이다.

| 항목 | 원본 HWPX dump | 직접 HWP dump |
| --- | --- | --- |
| 그림 bin id | 422 | 422 |
| 현재 크기 | 38262 x 7421 HU | 38262 x 7421 HU |
| 글자처럼 | true | true |
| 줄바꿈 | TopAndBottom | TopAndBottom |
| 세로/가로 기준 | Para(0) / Para(0) | Para(0) / Para(0) |
| z-order | 111 | 111 |

두 dump가 보이는 모델 의미값은 일치한다. 직접 HWP에는 문단 keep attribute의 원시 보조 비트(`attr2=0x8`)가 있고 HWPX dump에는 없다. 다만 이 값은 그림의 `CTRL_HEADER`가 아니므로, p261 그림의 paint/flow 차이를 설명하는 근거로 사용하지 않는다.

이 결과와 직접 HWP 대 rhwp 저장 HWP의 `CTRL_HEADER` 차이(`0x142A2311` 대 `0x042A2311`, bit 28)를 함께 보면, 현재 가장 강한 가설은 **같은 의미 모델을 HWP5 공통 개체 속성으로 직렬화할 때의 호환 비트 보존**이다. 그러나 `dump`는 `CommonObj.attr`의 원시 값을 노출하지 않으므로, 현 단계에서는 HWPX 파서와 serializer 중 어느 단계가 bit 28을 누락시키는지 단정할 수 없다.

### 구현 전 결론 및 경계

- p261은 텍스트 손실이나 페이지 소유권 문제가 아니다. 두 PDF의 text layer가 동일하고, 두 입력 dump의 그림 의미값도 동일하다.
- `numberingType=PICTURE` 전체에 bit 28을 일괄 설정하는 방식은 아직 승인할 수 없다. 이 문서의 74개 picture / 61개 inline / 60개 inline `TOP_AND_BOTTOM` 분포는 영향 범위를 보여 줄 뿐, 모든 원시 HWP5 개체가 같은 값을 가져야 함을 증명하지 않는다.
- p53 등 다른 변경 페이지에는 CharShape/도형 paint 계열 잔차가 포함되어 있어, p261의 picture 계약 보정 효과와 합산해 판단하지 않는다.

### 다음 분석 게이트

1. 기존 진단 도구만으로 p261와 최소 한 개의 동일 클래스 picture에 대한 HWP5 `CTRL_HEADER` 원시 attr을 수집한다.
2. HWPX `hp:pic` 실제 변환 경로에서 `CommonObj.attr`와 bit 28 의미 필드가 언제 materialize되는지 코드 변경 없이 추적한다.
3. 위 두 결과가 일치할 때만, 영향 범위를 `treatAsChar=true` + `TOP_AND_BOTTOM`라는 관찰값이 아니라 HWPX/HWP5 형식 계약으로 정의한 별도 구현 단계를 연다.

## 원시 레코드 확장 대조: 그림 단일 특례가 아님

**분석 게이트 1 충족:** p261 대상과 같은 클래스의 두 번째 picture, 그리고 인접 table까지 실제 HWP5 `CTRL_HEADER`의 bit 28 누락을 확인했다. 이 결과는 단일 fixture 우연이나 그림 전용 serializer 결함이라는 가설을 배제한다.

동일 `hwp5-anchor-trace`의 32-record window로 원본 HWP와 rhwp 저장 HWP를 비교했다. p261의 대상 그림뿐 아니라 다음 GenShape picture와 인접 표에서도 같은 상위 비트가 일관되게 누락된다.

| 개체 | 원본 HWP `properties` | rhwp 저장 HWP `properties` | 관찰 |
| --- | ---: | ---: | --- |
| p261 첫 picture (`CTRL_HEADER` 933) | `0x142A2311` | `0x042A2311` | bit 28만 누락, semantic common fields 동일 |
| 다음 inline picture (`CTRL_HEADER` 955) | `0x142A2311` | `0x042A2311` | 같은 누락, semantic common fields 동일 |
| 인접 inline table (`CTRL_HEADER` 909) | `0x182A2211` | `0x082A2211` | bit 28만 누락, semantic common fields 동일 |

두 picture의 `CTRL_HEADER`는 rhwp 저장본에서 각각 2 byte 짧다(200 -> 198, 198 -> 196). 이는 bit 28 누락과 별개로 picture description tail 직렬화 차이가 남아 있음을 뜻한다. 이번 분석에서는 해당 tail이 visual mismatch의 원인이라고 결론 내리지 않으며, bit 28 보정 검증 시 분리해서 관찰한다.

### 갱신된 설계 제약

- 영향 범위는 `hp:pic`만으로 한정할 수 없다. 같은 공통 개체 계약을 쓰는 표에도 동일한 `CommonObj.attr` bit 28 손실이 있다.
- 반대로 문서 안의 모든 공통 개체에 bit 28을 강제할 근거도 아직 없다. 원본 HWP의 해당 raw bit가 어떤 HWPX semantic/numbering 계약에서 요구되는지 확인해야 한다.
- 이후 구현은 raw `attr`의 우선 직렬화 경로와 의미 필드의 동기화 책임을 함께 다뤄야 한다. 어느 한 그림 serializer에서 상수를 OR하는 방식은 표 및 기존 HWP 입력의 보존 규칙을 깨뜨릴 위험이 있다.

## Stage 123 종료 결정

분석 게이트를 종료한다. 구현 대상은 HWPX parser나 picture 전용 converter가 아니라, `CommonObj.attr != 0`일 때 raw 값을 그대로 우선하는 HWP5 공통 개체 serializer 경로다. `hwp5_gen_shape_attr_bit28=true`라는 이미 materialize된 의미값이 raw attr에 반영되지 않는 경우에만 bit 28을 추가해야 한다. false일 때 기존 raw bit를 제거하지 않아 HWP 원본 보존 계약을 유지한다.

이 결론은 p261 두 picture와 인접 table에서의 동일 raw diff로 뒷받침한다. description tail의 2 byte 차이와 p53 CharShape 잔차는 이번 보정의 수용 기준에서 분리한다.
