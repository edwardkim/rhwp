# Task m100 #949 Stage 30 - HWPX id/instid contract 구현 후보

## 1. 목적

Stage 29에서 확인한 `hwpx-h-03` 손상 지점의 강한 차이는 HWPX `id`와 `instid`가
HWP5의 서로 다른 필드로 내려가지 못하는 문제였다.

Stage 30에서는 이 축을 실제 parser 구현에 반영했다.

```text
HWPX hp:*@id
  -> HWP5 CTRL_HEADER CommonObjAttr.instance_id

HWPX hp:*@instid
  -> HWP5 SHAPE_COMPONENT tail DrawingObjAttr.inst_id
  -> HWP5 SHAPE_PICTURE extra tail Picture.instance_id
```

이번 단계는 "컨트롤 추가"가 아니라, 이미 존재하는 HWPX source 속성을 IR에서 분리 보존해
serializer가 준비된 HWP5 필드에 정확히 기록할 수 있게 하는 작업이다.

## 2. 구현 내용

수정 파일:

```text
src/parser/hwpx/section.rs
src/document_core/converters/hwpx_to_hwp.rs
```

주요 변경:

```text
1. hp:pic@id를 CommonObjAttr.instance_id로 파싱한다.
2. hp:pic@instid를 Picture.instance_id로 파싱한다.
3. hp:rect 등 shape object의 @id를 CommonObjAttr.instance_id로 파싱한다.
4. hp:rect 등 shape object의 @instid를 DrawingObjAttr.inst_id로 파싱한다.
5. id가 없고 instid만 있는 경우에는 기존 호환성을 위해 CommonObjAttr.instance_id에 instid를 fallback한다.
6. hwpx-h-03 글상자 내부 rect/pic 계약을 검증하는 단위 테스트를 보강했다.
```

테스트에서 확인한 HWPX source 값:

| source | IR field | expected |
|---|---|---:|
| `hp:rect@id` | `shape.common().instance_id` | 1875692958 |
| `hp:rect@instid` | `shape.drawing().inst_id` | 801951135 |
| `hp:pic[0]@id` | `Picture.common.instance_id` | 1875692960 |
| `hp:pic[0]@instid` | `Picture.instance_id` | 801951137 |
| `hp:pic[1]@id` | `Picture.common.instance_id` | 1875692962 |
| `hp:pic[1]@instid` | `Picture.instance_id` | 801951139 |

## 3. 생성 결과

생성 디렉터리:

```text
output/poc/hwpx2hwp/task949/stage30_id_instid_contract/
```

생성 파일:

```text
output/poc/hwpx2hwp/task949/stage30_id_instid_contract/hwpx-h-01.hwp
output/poc/hwpx2hwp/task949/stage30_id_instid_contract/hwpx-h-02.hwp
output/poc/hwpx2hwp/task949/stage30_id_instid_contract/hwpx-h-03.hwp
output/poc/hwpx2hwp/task949/stage30_id_instid_contract/hy-001.hwp
```

파일 크기와 rhwp 재로드 결과:

| file | size | rhwp reload |
|---|---:|---|
| `hwpx-h-01.hwp` | 366 KB | ok, pages=9 |
| `hwpx-h-02.hwp` | 31 KB | ok, pages=10 |
| `hwpx-h-03.hwp` | 37 KB | ok, pages=9 |
| `hy-001.hwp` | 86 KB | ok, pages=2 |

생성 로그:

```text
output/poc/hwpx2hwp/task949/stage30_id_instid_contract/generation.md
```

## 4. 한컴 시각 판정 요청

| file | 한컴 판정 유형 | 이미지 출력 | 표/셀 배치 | 셀 텍스트 클리핑 | 마지막 페이지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task949/stage30_id_instid_contract/hwpx-h-01.hwp` | Stage 28과 동일 |  |  |  |  |  | guard |
| `output/poc/hwpx2hwp/task949/stage30_id_instid_contract/hwpx-h-02.hwp` | Stage 28과 동일 |  |  |  |  |  | guard, rhwp pages=10 |
| `output/poc/hwpx2hwp/task949/stage30_id_instid_contract/hwpx-h-03.hwp` | 파일손상 |  |  |  | Stage 28과 같은 위치에서 중단 |  | target, rhwp pages=9 |
| `output/poc/hwpx2hwp/task949/stage30_id_instid_contract/hy-001.hwp` | 파일손상 |  |  |  | Stage 28과 같은 위치에서 중단 |  | #974 guard, rhwp pages=2 |

작업지시자 판정:

```text
28 stage와 동일한 결과입니다. 파일 손상과 중단된 페이지 위치가 동일합니다.
```

## 5. 정답 HWP 비교 결과

`hwpx-h-03`의 손상 후보 지점에 대해 정답 HWP와 Stage 30 생성 HWP를 비교했다.

진단 산출물:

```text
output/poc/hwpx2hwp/task949/stage30_id_instid_contract/h03_stage30_section0.jsonl
output/poc/hwpx2hwp/task949/stage30_id_instid_contract/h03_shape_bundles_w8.md
output/poc/hwpx2hwp/task949/stage30_id_instid_contract/h03_contract_analyze/
```

확인된 개선:

| record | 의미 | Stage 30 판정 |
|---:|---|---|
| `#824 CTRL_HEADER` | outer rect `id` -> common instance id | 정답 hash 일치 |
| `#831 CTRL_HEADER` | first inner picture `id` -> common instance id | 정답 hash 일치 |
| `#835 CTRL_HEADER` | second inner picture `id` -> common instance id | 정답 hash 일치 |
| `#834 SHAPE_PICTURE` | first inner picture `instid` -> picture extra tail | 정답 hash 일치 |
| `#837 SHAPE_PICTURE` | second inner picture `instid` -> picture extra tail | 정답 hash 일치 |

즉 Stage 29에서 지목한 `id/instid` 축은 실제 HWP5 record에서 정답지와 맞아졌다.

남은 차이:

| record | 남은 차이 |
|---:|---|
| `#825 SHAPE_COMPONENT` | payload hash 다름 |
| `#826 LIST_HEADER` | text-box list tail size/hash 다름 |
| `#827 PARA_HEADER` | text-box paragraph tail size/hash 다름 |
| `#832 SHAPE_COMPONENT` | first inner picture component payload hash 다름 |
| `#836 SHAPE_COMPONENT` | second inner picture component payload hash 다름 |

따라서 Stage 30 이후에도 `hwpx-h-03` 파일손상이 유지된다면, `id/instid` 단독 보정은 충분조건이
아니다. 다음 후보는 다음 둘 중 하나다.

```text
1. Stage 23에서 맞췄던 text-box LIST_HEADER/PARA_HEADER tail과 Stage 30 id/instid를 동시에 적용한다.
2. 그래도 동일하면 #825/#832/#836 SHAPE_COMPONENT payload contract를 field 단위로 닫는다.
```

## 6. 실행한 검증

```text
cargo fmt --check
cargo test hwpx_h_03_rect_draw_text_contract_from_source
cargo build
```

검증 결과:

```text
모두 통과
```

rhwp 재로드:

```text
hwpx-h-01: ok, pages=9
hwpx-h-02: ok, pages=10
hwpx-h-03: ok, pages=9
hy-001:    ok, pages=2
```

## 7. 현재 결론

Stage 30은 한컴 파일손상 문제의 한 축을 해결한 구현 후보다.

확정된 것:

```text
1. HWPX id와 instid는 같은 값이 아니다.
2. id는 HWP5 CTRL_HEADER의 common object instance id로 내려가야 한다.
3. instid는 HWP5 SHAPE_COMPONENT/SHAPE_PICTURE의 별도 tail instance id로 내려가야 한다.
4. 이 축을 반영하면 hwpx-h-03 손상 지점의 CTRL_HEADER와 SHAPE_PICTURE extra tail은 정답지와 일치한다.
```

아직 확정되지 않은 것:

```text
1. 이 변경만으로는 hwpx-h-03 한컴 파일손상이 해소되지 않았다.
2. hy-001도 Stage 28과 같은 위치에서 중단되므로 #974 guard 역시 통과하지 못했다.
3. 다음 단계는 Stage 23 text-box tail 계약과 Stage 30 id/instid 계약을 조합해 검증해야 한다.
4. 조합 후에도 동일하면 #825/#832/#836 SHAPE_COMPONENT payload가 다음 원인이다.
```
