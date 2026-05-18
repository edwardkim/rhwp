# Task m100 #949 Stage 31 - id/instid + text-box envelope 조합 검증

## 1. 목적

Stage 30 수동 판정은 Stage 28과 동일했다.

```text
파일손상과 중단된 페이지 위치가 동일하다.
```

따라서 `id/instid` mapping은 필요한 계약을 닫았지만, 단독으로는 `hwpx-h-03`
파일손상을 해소하지 못했다.

Stage 31의 목적은 따로 검증했던 두 축을 동시에 적용한 후보를 만드는 것이다.

```text
1. Stage 30: HWPX id/instid를 HWP5 CTRL_HEADER/SHAPE tail에 분리 매핑
2. Stage 23: drawText text-box의 LIST_HEADER/PARA_HEADER envelope materialization
```

이번 단계는 실패한 baseline을 다른 샘플에 다시 적용하는 단계가 아니다. 정답 HWP와 비교해
닫힌 record 조합을 확인한 뒤, 한컴 에디터에서 수동 판정하는 단계다.

## 2. 구현 내용

수정 파일:

```text
src/document_core/converters/hwpx_to_hwp.rs
```

주요 변경:

```text
1. TextBox 내부 paragraph가 그림 control을 포함하는 경우에만 HWP5 text-box envelope 후보로 본다.
2. 후보 TextBox의 raw_list_header_extra가 비어 있으면 13-byte tail을 materialize한다.
3. 후보 TextBox 내부 paragraph의 raw_header_extra가 부족하면 12-byte tail을 materialize한다.
4. char shape, range tag, line segment count를 PARA_HEADER tail에 반영한다.
5. Stage 30 id/instid 계약과 동시에 닫히는지 확인하는 단위 테스트를 추가했다.
```

추가 검증 테스트:

```text
hwpx_h_03_draw_text_envelope_materializes_with_id_instid_contract
```

## 3. 생성 결과

생성 디렉터리:

```text
output/poc/hwpx2hwp/task949/stage31_id_instid_textbox_envelope/
```

생성 파일:

| file | size | rhwp reload |
|---|---:|---|
| `hwpx-h-01.hwp` | 366 KB | ok, pages=9 |
| `hwpx-h-02.hwp` | 32 KB | ok, pages=10 |
| `hwpx-h-03.hwp` | 38 KB | ok, pages=9 |
| `hy-001.hwp` | 87 KB | ok, pages=2 |

생성 로그:

```text
output/poc/hwpx2hwp/task949/stage31_id_instid_textbox_envelope/generation.md
```

## 4. 한컴 시각 판정 요청

| file | 한컴 판정 유형 | 이미지 출력 | 표/셀 배치 | 셀 텍스트 클리핑 | 마지막 페이지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task949/stage31_id_instid_textbox_envelope/hwpx-h-01.hwp` | Stage 30과 동일 |  |  |  |  |  | guard, rhwp pages=9 |
| `output/poc/hwpx2hwp/task949/stage31_id_instid_textbox_envelope/hwpx-h-02.hwp` | Stage 30과 동일 |  |  |  |  |  | guard, rhwp pages=10 |
| `output/poc/hwpx2hwp/task949/stage31_id_instid_textbox_envelope/hwpx-h-03.hwp` | Stage 30과 동일 |  |  |  | Stage 30과 동일 | 1페이지 마지막 표 다음 페이지네이션 실패 | target, rhwp pages=9 |
| `output/poc/hwpx2hwp/task949/stage31_id_instid_textbox_envelope/hy-001.hwp` | Stage 30과 동일 |  |  |  |  | 정상 조판 | #974 guard, rhwp pages=2 |

## 5. 정답 HWP 비교 결과

진단 산출물:

```text
output/poc/hwpx2hwp/task949/stage31_id_instid_textbox_envelope/h03_stage31_section0.jsonl
output/poc/hwpx2hwp/task949/stage31_id_instid_textbox_envelope/h03_shape_bundles_w8.md
output/poc/hwpx2hwp/task949/stage31_id_instid_textbox_envelope/h03_contract_analyze/
```

`hwpx-h-03` 손상 후보 지점에서 정답 HWP와 일치한 record:

| record | 의미 | Stage 31 판정 |
|---:|---|---|
| `#824 CTRL_HEADER` | outer rect `id` -> common instance id | 정답 hash 일치 |
| `#826 LIST_HEADER` | drawText text-box list envelope | 정답 size/hash 일치 |
| `#827 PARA_HEADER` | drawText inner paragraph envelope | 정답 size/hash 일치 |
| `#831 CTRL_HEADER` | first inner picture `id` -> common instance id | 정답 hash 일치 |
| `#834 SHAPE_PICTURE` | first inner picture `instid` -> picture payload/tail | 정답 hash 일치 |
| `#835 CTRL_HEADER` | second inner picture `id` -> common instance id | 정답 hash 일치 |
| `#837 SHAPE_PICTURE` | second inner picture `instid` -> picture payload/tail | 정답 hash 일치 |

아직 정답 HWP와 다른 record:

| record | 남은 차이 |
|---:|---|
| `#825 SHAPE_COMPONENT` | outer rect component payload hash 다름 |
| `#832 SHAPE_COMPONENT` | first inner picture component payload hash 다름 |
| `#836 SHAPE_COMPONENT` | second inner picture component payload hash 다름 |

즉 Stage 31은 Stage 30의 `id/instid` 축과 Stage 23의 text-box envelope 축을 동시에 닫았다.
여기서도 한컴 파일손상이 같은 위치에 남는다면 다음 원인은 더 이상 `LIST_HEADER`,
`PARA_HEADER`, `id/instid`, `CTRL_DATA`가 아니라 `SHAPE_COMPONENT` payload 쪽이다.

## 6. 실행한 검증

```text
cargo fmt --check
cargo test hwpx_h_03_rect_draw_text_contract_from_source
cargo test hwpx_h_03_draw_text_envelope_materializes_with_id_instid_contract
cargo build
target/debug/rhwp dump-pages <generated hwp files>
target/debug/rhwp hwp5-inventory-diff <oracle h03> <generated h03> --focus shape --window 8
target/debug/rhwp hwp5-contract-analyze <h03 hwpx> <oracle h03> <generated h03>
```

검증 결과:

```text
모두 통과
```

## 7. 수동 판정 결과

작업지시자 판정:

```text
판정 결과는 30 stage와 동일합니다.
```

계속 관찰되는 특성:

```text
1. hy-001.hwp는 rhwp-studio에서 정상 조판된다.
2. hwpx-h-03.hwp는 rhwp-studio에서 1페이지 마지막 표 다음 페이지네이션이 실패한다.
```

## 8. 판정 해석

Stage 31은 실패로 판정한다.

중요한 점은 실패의 의미가 더 좁아졌다는 것이다.

```text
1. #824 CTRL_HEADER, #826 LIST_HEADER, #827 PARA_HEADER,
   #831 CTRL_HEADER, #834 SHAPE_PICTURE, #835 CTRL_HEADER, #837 SHAPE_PICTURE는
   정답 HWP와 hash가 일치한다.
2. 그런데 한컴 판정은 Stage 30과 동일하다.
3. 따라서 id/instid, drawText LIST_HEADER/PARA_HEADER envelope, CTRL_DATA,
   SHAPE_PICTURE payload/tail은 현재 손상 해소의 남은 후보에서 제외한다.
```

또 하나의 분리는 rhwp-studio 판정이다.

```text
hy-001:    rhwp-studio 정상 조판
hwpx-h-03: rhwp-studio 1페이지 마지막 표 다음 페이지네이션 실패
```

따라서 다음 단계는 한컴 에디터에서만 보이는 strict contract 문제가 아니라,
rhwp 재로드 조판에서도 관찰되는 `hwpx-h-03` 전용 구조/배치 계약 문제를 먼저 추적해야 한다.

## 9. 다음 작업

다음 stage는 `#825/#832/#836 SHAPE_COMPONENT` payload를 대상으로 한다.

다만 접근 순서는 다음과 같이 고정한다.

```text
1. 먼저 hwpx-h-03 generated HWP의 rhwp-studio 페이지네이션 실패 지점을 로컬에서 설명한다.
2. 그 실패 지점과 정답 HWP의 #825/#832/#836 SHAPE_COMPONENT payload 차이를 연결한다.
3. byte graft가 아니라 필드 의미를 해석한 뒤, HWPX source 또는 IR 필드에서 생성 가능한 값을 찾는다.
4. hwpx-h-01/hwpx-h-02/hy-001 guard를 유지한 상태에서만 HWP 생성 후보를 만든다.
```

Stage 31에서 이미 닫힌 축은 다음 단계에서 다시 후보로 되돌리지 않는다.

```text
LIST_HEADER/PARA_HEADER/id/instid/CTRL_DATA/SHAPE_PICTURE tail 재검증 금지
```

## 10. 판정 기준

성공:

```text
1. hwpx-h-01/hwpx-h-02 guard 성공 유지
2. hwpx-h-03 파일손상 해소 또는 중단 위치 이동
3. hy-001 #974 guard 성공 유지
```

실패:

```text
1. hwpx-h-03 결과가 Stage 28/30과 동일
2. hwpx-h-01/hwpx-h-02 guard 회귀
3. hy-001이 Stage 23처럼 파일손상 발생
```

실패 시 다음 작업:

```text
Stage 32는 #825/#832/#836 SHAPE_COMPONENT payload를 byte/field 단위로 닫는다.
Stage 31에서 이미 닫힌 LIST_HEADER/PARA_HEADER/id/instid/CTRL_DATA 축은 재검증 대상으로 되돌리지 않는다.
```
