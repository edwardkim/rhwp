# #4669 HWPX `hp:ole` shape-component 보존 (M05-9 / #5450)

날짜: 2026-08-18
축: HWPX 저장 — OLE / shape-component 직렬화만
비범위: 쪽수(#3737), `char_shapes`, `hp:pic` offset(#4668), gym

## 1. 문제

`rhwp export-hwpx samples/한셀OLE.hwpx out.hwpx` 한 번으로

- `hp:ole id` 가 원문과 다른 값(관측: `id="0"`)으로 재부여되고
- `<hp:curSz width="0" height="0"/>`(한컴 원산 관례)가 재계산 크기
  (한셀 표본 기준 29999×4051)로 바뀌며
- 같은 이유로 `offset` / `flip` / `renderingInfo` / `lineShape` 도
  기본값으로 재유도됐다.

한글 화면 표시에는 영향이 없을 수 있다(#3543 과 같은 값 보존 부류).
그러나 offset·행렬을 읽는 다른 소비자(다운스트림 변환기, 외부 배치 도구)는
오배치를 얻는다. `BinData` 파트 `ole1.ole` → `image1.OLE` 개명은
`image{N}` 불변식(#1891) 의도 설계이므로 결함이 아니다.

## 2. 원인

| 지점 | 간극 |
| --- | --- |
| `parse_common_shape_children` | chart·OLE 공용 자식 파서에 `offset`/`orgSz`/`curSz`/`flip`/`renderingInfo`/`lineShape` arm 이 없어 IR 에 원문이 안 실렸다. `hwpx_to_hwp.rs` 주석이 이 간극을 자인. |
| `apply_hwpx_ole_shape_component_contract` | `curSz=0` 을 orgSz 로 materialize 할 때 `current_*_was_zero` 센티널(#2017)을 안 세웠다. pic·일반 도형은 이미 해결, OLE 만 커버리지 밖. |
| `parse_hp_ole_element` | `id` arm 부재. 차트는 #3546 에서 `id` 를 받았으나 OLE 는 누락. `instid` 만 파싱. |
| `write_ole` | 원문 `id` 필드가 없어 `instance_id`(또는 0)를 `id` 와 `instid` 에 겸용 방출. |

`write_cur_sz` 와 `write_shape_component_block` 자체는 이미 센티널·원문
방출 로직을 갖고 있었다. 빠진 것은 OLE 파서가 그 필드를 IR 에 싣는 일과
`hwpx_ole_id` 를 되쓰는 일이다.

## 3. 수정

코드 경로 (devel 에 착수된 보존 계약을 이 PR 이 코퍼스로 고정·고도화):

1. `parse_common_shape_children` 이 `offset`/`orgSz`/`curSz` 를
   `parse_object_layout_child` 에 위임하고, `flip`/`renderingInfo`/`lineShape`
   를 도형·그림 파서와 동형으로 읽는다.
2. `parse_hp_ole_element` 가 `id` 와 `instid` 를 분리한다. `id` 폴백은
   **instid 속성 부재** 시에만 적용한다. 명시적 `instid="0"`(#4099 차트
   fallback OLE)은 덮지 않는다. 명시적 `id="0"` 도 원문 0 으로 남긴다.
3. `OleShape.hwpx_ole_id: Option<u32>` — HWP5 출신은 `None` → 종전대로
   `id=instid=instance_id`.
4. `apply_hwpx_ole_shape_component_contract` 가
   `materialize_shape_current_size_from_original` 을 호출해 was_zero
   센티널을 세운다. writer `write_cur_sz` 가 0 을 복원한다.
5. `write_ole` 가 `hwpx_ole_id.unwrap_or(instance_id)` 를 `id` 에,
   `instance_id` 를 `instid` 에 쓴다. 자식은
   `write_shape_component_block` + `write_line_shape`.

이 PR 은 위 계약을 **실물 샘플 + 137개 픽스처·봉투 전사 + 왕복 시험**으로
고도화한다. 쪽수·char_shapes·pic offset writer 는 손대지 않는다.

## 4. 한셀OLE 실측 정답

`samples/한셀OLE.hwpx` `Contents/section0.xml` 의 `hp:ole` (축약):

```xml
<hp:ole id="2141242094" instid="1067500271" objectType="EMBEDDED"
        binaryItemIDRef="ole1" drawAspect="CONTENT" numberingType="PICTURE">
  <hp:offset x="0" y="0"/>
  <hp:orgSz width="42001" height="13501"/>
  <hp:curSz width="29999" height="4051"/>
  <hp:flip horizontal="0" vertical="0"/>
  <hp:rotationInfo angle="0" centerX="14999" centerY="2025" rotateimage="1"/>
  <hp:renderingInfo>
    <hc:transMatrix e1="1" e2="0" e3="0" e4="0" e5="1" e6="0"/>
    <hc:scaMatrix e1="0.714245" e2="0" e3="0" e4="0" e5="0.300052" e6="0"/>
    <hc:rotMatrix e1="1" e2="0" e3="0" e4="0" e5="1" e6="0"/>
  </hp:renderingInfo>
  <hc:extent x="29999" y="4051"/>
  <hp:lineShape color="#000000" width="0" style="NONE" endCap="ROUND"/>
  ...
</hp:ole>
```

이슈 본문이 지적한 `curSz=0` 은 한컴 원산 파일에 흔한 형태이며, 이 표본
자체는 비-0 curSz 를 가진다. 코퍼스 `021_cursz_zero_zero` 와
`152_combo_issue_body_xml` 이 0 센티널 축을 담당한다.

## 5. 저장 계약

저장 후 각 `hp:ole` 에 대해:

| 필드 | 보존 |
| --- | --- |
| `id` | `hwpx_ole_id` 원문. 없으면 `instance_id`. 원문이 0 이 아니면 `id="0"` 재부여 금지. |
| `instid` | `instance_id`. 명시적 0 유지. |
| `hp:offset` | `offset_x/y` 를 u32 wraparound 십진수로 방출 (#3544 와 동형). pos 유래 재작성 금지. |
| `hp:orgSz` | `original_width/height` |
| `hp:curSz` | was_zero 면 `0`, 아니면 `current_width/height` |
| `hp:flip` | `horz_flip` / `vert_flip` |
| `hp:rotationInfo` | angle, centerX/Y, rotateimage |
| `hp:renderingInfo` | `raw_rendering` 행렬 (trans/sca/rot) |
| `hp:lineShape` | style/width/color/endCap |

비계약 (이 축에서 단언하지 않음):

- `objectType` (한셀은 `EMBEDDED`, writer 는 역사적으로 `UNKNOWN` 을 쓸 수 있음)
- `binaryItemIDRef` 개명 (`ole1` → `image1`, #1891)
- `dropcapstyle` / `href` / `hasMoniker` / `eqBaseLine` 기본값
- 페이지 수, 글자 모양, pic offset

## 6. 코퍼스

경로: `tests/fixtures/issue_4669_ole_shape_component/`

| 산출 | 역할 |
| --- | --- |
| `xml/*.xml` | 픽스처 섹션. 각 파일이 한 계약 사례. |
| `envelopes/*.json` | 파싱→저장 기대 봉투 전사 (`schema=rhwp.issue4669.ole-shape-component.v1`). |
| `catalog.tsv` | 색인. |
| `README.md` | 생성 방법. |

가족:

- `oracle` — 한셀 원문
- `id-instid` — 분리·부재·0·u32 최대·i32 경계
- `cursz-orgsz` — 0 센티널, 비대칭 0, 7200 함정, 소수 크기
- `offset` — 양수, wraparound, i32::MIN, 병진 일치
- `flip` / `rotation` / `rendering` / `lineshape`
- `hansel-mutant` — 한셀 원문에서 한 필드만 변경
- `ole-attr` — drawAspect·lock·textWrap 과 자식 동시
- `multi-ole` — 한 문단 여러 OLE
- `combo` — 이슈 본문 재현 포함

생성: `python scripts/generate_issue_4669_ole_fixtures.py`

봉투 한 건의 핵심 키:

- `save_id` / `instance_id` / `was_zero` / `offset_u32`
- `expect_xml` — 저장 XML 에 있어야 하는 조각
- `forbid_xml` — 재유도 산출물 (예: curSz=0 이 orgSz 로 바뀐 태그)
- `forbid_id_zero` — 원문 id 가 0 이 아닐 때 시작 태그 `id="0"` 금지

시험은 픽스처의 `hp:ole` 를 `한셀OLE.hwpx` 의 section0 에 끼워 넣은 뒤
`DocumentCore::export_hwpx_native` 로 저장하고 조각을 대조한다.
템플릿의 `secPr`·BinData·header 는 그대로 두어 파서가 실패키지로 열리게 한다.

## 7. 시험

| 시험 | 무엇을 고정하는가 |
| --- | --- |
| `src/parser/hwpx/section.rs` `issue4669_parse_*` | IR 적재 (id/instid 분리, 자식, was_zero) |
| `src/serializer/hwpx/shape.rs` `issue4669_write_*` | writer 방출 (id, curSz=0, wraparound offset, flip, 행렬, lineShape) |
| `tests/cases/issue_4669_ole_shape_component.rs` | 실샘플 + 코퍼스 왕복 + 이슈 본문 curSz=0 + catalog↔envelope 짝 |

1커맨드 스모크:

```
cargo test --test regression_suite_019 issue_4669
```

(`tests/cases/issue_4669_ole_shape_component.rs` 는 suite 019 로 배정된다.)

## 8. 하지 않은 것

- 페이지 수 / gym / M08 / M05-4(#3737)
- `hp:pic` offset writer (#4668, M05-8)
- `char_shapes`
- `objectType` 원문 보존, BinData 파일명 원문 보존
- `write_chart_element` 의 chart `id` (#3546 축)

## 9. 재현

```
rhwp export-hwpx samples/한셀OLE.hwpx out.hwpx
```

저장본 `Contents/section0.xml` 의 `hp:ole` 에서 `id="2141242094"` 와
`instid="1067500271"`, `orgSz`/`curSz`/`offset`/`scaMatrix e1="0.714245"` 가
살아 있으면 이 축은 통과다.
