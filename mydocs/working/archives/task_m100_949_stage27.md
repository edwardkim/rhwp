# Task M100 #949 Stage 27: TextBox sequence contract 진단

## 1. 목적

Stage 26 이후에도 `hwpx-h-03`과 `hy-001`에서 한컴 파일손상 판정이 유지되었다.

이번 단계는 구현을 하지 않고, 정답 HWP와 Stage 26 생성 HWP를 비교하여 글상자/그림 주변의
HWP5 record contract가 어디서 깨지는지 확인한다.

## 2. 비교 대상

```text
samples/hwpx/hancom-hwp/hwpx-h-03.hwp
output/poc/hwpx2hwp/task949/stage26_genshape_common_attr_candidate/hwpx-h-03.hwp

samples/hwpx/hancom-hwp/hy-001.hwp
output/poc/hwpx2hwp/task949/stage26_genshape_common_attr_candidate/hy-001.hwp
```

## 3. 산출물

```text
output/poc/hwpx2hwp/task949/stage27_textbox_sequence_contract/h03_oracle_section0.jsonl
output/poc/hwpx2hwp/task949/stage27_textbox_sequence_contract/h03_stage26_section0.jsonl
output/poc/hwpx2hwp/task949/stage27_textbox_sequence_contract/hy_oracle_section0.jsonl
output/poc/hwpx2hwp/task949/stage27_textbox_sequence_contract/hy_stage26_section0.jsonl
output/poc/hwpx2hwp/task949/stage27_textbox_sequence_contract/h03_shape_bundles_w12.md
output/poc/hwpx2hwp/task949/stage27_textbox_sequence_contract/hy_shape_bundles_w12.md
output/poc/hwpx2hwp/task949/stage27_textbox_sequence_contract/sequence_findings.md
```

## 4. `hwpx-h-03` 문제 지점 비교

`hwpx-h-03`의 2페이지 글상자 주변 record sequence는 통째로 누락된 상태가 아니다.

| idx | 정답 tag | 정답 size | 생성 tag | 생성 size | 판정 |
|---:|---|---:|---|---:|---|
| 824 | `CTRL_HEADER GenShape` | 60 | `CTRL_HEADER GenShape` | 60 | size 동일, payload 다름 |
| 825 | `SHAPE_COMPONENT` | 252 | `SHAPE_COMPONENT` | 252 | size 동일, payload 다름 |
| 826 | `LIST_HEADER` | 33 | `LIST_HEADER` | 20 | size 다름 |
| 827 | `PARA_HEADER` | 24 | `PARA_HEADER` | 22 | size 다름 |
| 831 | `CTRL_HEADER GenShape` | 176 | `CTRL_HEADER GenShape` | 176 | size 동일, payload 다름 |
| 832 | `SHAPE_COMPONENT` | 196 | `SHAPE_COMPONENT` | 196 | size 동일, payload 다름 |
| 833 | `CTRL_DATA` | 76 | `CTRL_DATA` | 76 | payload 일치 |
| 834 | `SHAPE_PICTURE` | 91 | `SHAPE_PICTURE` | 91 | size 동일, payload 다름 |
| 835 | `CTRL_HEADER GenShape` | 46 | `CTRL_HEADER GenShape` | 46 | size 동일, payload 다름 |
| 836 | `SHAPE_COMPONENT` | 196 | `SHAPE_COMPONENT` | 196 | size 동일, payload 다름 |
| 837 | `SHAPE_PICTURE` | 91 | `SHAPE_PICTURE` | 91 | size 동일, payload 다름 |
| 838 | `SHAPE_RECTANGLE` | 33 | `SHAPE_RECTANGLE` | 33 | payload 일치 |

해석:

```text
1. h03 파일손상은 "컨트롤이 없다"보다 "record envelope/payload가 한컴 계약과 다르다"에 가깝다.
2. hp:pic@href -> CTRL_DATA는 정답지와 일치한다.
3. Stage 23에서 LIST_HEADER/PARA_HEADER size를 맞췄지만 파일손상이 해소되지 않았으므로,
   header tail만 단독 보정하는 접근은 충분하지 않다.
```

## 5. `hy-001` 문제 지점 비교

`hy-001`은 같은 글상자/그림 계열이지만 더 강한 단서를 준다.

| idx | 정답 tag | 정답 size | 생성 tag | 생성 size | 판정 |
|---:|---|---:|---|---:|---|
| 20 | `CTRL_HEADER GenShape` | 46 | `CTRL_HEADER GenShape` | 46 | size 동일, payload 다름 |
| 21 | `SHAPE_COMPONENT` | 292 | `SHAPE_COMPONENT` | 196 | size 다름 |
| 22 | `SHAPE_PICTURE` | 91 | `SHAPE_PICTURE` | 91 | size 동일, payload 다름 |
| 23 | `LIST_HEADER` | 47 | `LIST_HEADER` | 34 | size 다름 |
| 24 | `PARA_HEADER` | 24 | `PARA_HEADER` | 22 | size 다름 |

특히 `SHAPE_COMPONENT`가 `292 -> 196`으로 줄어든다. 현재 writer 구조와 대조하면 이 96바이트
차이는 임의의 tail 누락보다 rendering matrix count 차이로 보는 것이 더 정확하다.

```text
top-level 기본 생성:
ctrl_id 8 + ShapeComponentAttr 42 + rendering(cnt=1) 146 = 196 bytes

cnt=2 rendering 생성:
ctrl_id 8 + ShapeComponentAttr 42 + rendering(cnt=2) 242 = 292 bytes
```

즉 `hy-001` 정답지는 `group_level=0`처럼 보이는 그림에서도 `cnt=2` rendering 정보를 기록한다.
현재 writer는 `group_level > 0`일 때만 `cnt=2`를 쓰므로, 한컴 내부 구조체의 rendering contract를
`group_level` 하나로 판단하는 현재 조건이 틀렸을 가능성이 높다.

## 6. 현재 결론

Stage 27의 판정은 다음이다.

```text
record sequence 누락: 아님
level/order 대규모 오류: 아님
payload/envelope materialization 오류: 유력
1차 후보: SHAPE_COMPONENT rendering matrix count/materialization
```

한컴은 HWPX XML 요소를 HWP5 record로 직접 복사하지 않고, 내부 구조체/조판 객체에 매핑한 뒤
그 구조체를 HWP5 record contract에 맞게 다시 serialize하는 것으로 보는 편이 맞다.

따라서 rhwp의 다음 구현도 XML 속성 하나씩 덧붙이는 방식이 아니라, HWPX source를
`SHAPE_COMPONENT`/`SHAPE_PICTURE`/TextBox 내부 paragraph 구조체로 정확히 materialize하는 방향으로
가야 한다.

## 7. 다음 작업

다음 Stage 28의 선행 작업은 다음이다.

```text
1. 정답 HWP의 SHAPE_COMPONENT rendering count를 field 단위로 확인한다.
2. generated SHAPE_COMPONENT와 byte offset별 차이를 분리한다.
3. `hy-001`의 292-byte SHAPE_COMPONENT와 generated 196-byte SHAPE_COMPONENT의 96-byte 차이를
   `cnt=2` rendering contract로 검증한다.
4. h03의 252-byte outer shape도 같은 rendering count 계열인지 확인한다.
5. 그 뒤에만 source 수정 계획서를 작성한다.
```

이번 단계에서는 소스 수정은 하지 않았다.
