# Task M100 #949 Stage 26: GenShape CommonObjAttr 계약 구현 후보

## 1. 목적

Stage 25에서 확인한 `hwpx-h-03` drawText/image group의 GenShape `CTRL_HEADER` field diff를
실제 HWPX -> HWP 저장 경로에 반영한다.

이번 단계는 다음 후보를 검증한다.

```text
hp:pos@flowWithText -> GenShape CTRL_HEADER attr bit 13
hp:pos@allowOverlap -> GenShape CTRL_HEADER attr bit 14
HWPX 출처 GenShape storage high-bit -> GenShape CTRL_HEADER attr bit 26
unsigned decimal offset -> signed i32 offset
```

## 2. 구현 변경

변경 파일:

```text
src/model/shape.rs
src/parser/control/shape.rs
src/parser/hwpx/section.rs
src/parser/hwpx/utils.rs
src/document_core/converters/common_obj_attr_writer.rs
```

핵심 변경:

```text
1. CommonObjAttr에 flow_with_text, allow_overlap, hwp5_gen_shape_attr_bit26 추가
2. HWP5 parser에서 attr bit 13/14/26을 모델에 보존
3. HWPX parser에서 hp:pos@flowWithText, allowOverlap, vertOffset, horzOffset 파싱
4. HWPX unsigned decimal negative offset을 i32 wrapping으로 처리
5. HWP5 writer에서 GenShape attr bit 13/14/26 합성
```

## 3. 생성 파일

| file | size | rhwp reload |
|---|---:|---|
| `output/poc/hwpx2hwp/task949/stage26_genshape_common_attr_candidate/hwpx-h-01.hwp` | 374,784 bytes | ok, sections=2, pages=9 |
| `output/poc/hwpx2hwp/task949/stage26_genshape_common_attr_candidate/hwpx-h-02.hwp` | 32,256 bytes | ok, sections=2, pages=10 |
| `output/poc/hwpx2hwp/task949/stage26_genshape_common_attr_candidate/hwpx-h-03.hwp` | 38,400 bytes | ok, sections=2, pages=9 |
| `output/poc/hwpx2hwp/task949/stage26_genshape_common_attr_candidate/hy-001.hwp` | 88,576 bytes | ok, sections=1, pages=2 |

추가 진단 산출물:

```text
output/poc/hwpx2hwp/task949/stage26_genshape_common_attr_candidate/h03_ctrl_header_after.md
output/poc/hwpx2hwp/task949/stage26_genshape_common_attr_candidate/h03_ctrl_hints.md
output/poc/hwpx2hwp/task949/stage26_genshape_common_attr_candidate/h03_shape_hints.md
output/poc/hwpx2hwp/task949/stage26_genshape_common_attr_candidate/generation.md
```

## 4. 한컴 시각 판정 요청

| file | 한컴 판정 유형 | 이미지 출력 | 표/셀 배치 | 셀 텍스트 클리핑 | 마지막 페이지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task949/stage26_genshape_common_attr_candidate/hwpx-h-01.hwp` | 성공 | 성공 | 성공 | 성공 | 성공 | 성공 | guard |
| `output/poc/hwpx2hwp/task949/stage26_genshape_common_attr_candidate/hwpx-h-02.hwp` | 성공 | 성공 | 성공 | 성공 | 성공 | 성공 | guard, rhwp reload pages=10 |
| `output/poc/hwpx2hwp/task949/stage26_genshape_common_attr_candidate/hwpx-h-03.hwp` | 파일손상 | 1페이지만 | 성공 | 성공 | 2페이지 글상자 전까지만 출력, 실패 | 1페이지 페이지네이션 실패 | target |
| `output/poc/hwpx2hwp/task949/stage26_genshape_common_attr_candidate/hy-001.hwp` | 파일손상 | 1페이지만 | 성공 | 성공 | 2페이지 마지막 표전까지만 출력, 실패 | 정상 | #974 guard |

## 5. `hwpx-h-03` CTRL_HEADER 판정

Stage 26 결과, 문제 후보였던 세 GenShape `CTRL_HEADER`의 첫 32바이트는 정답지와 일치한다.

```text
#824 outer rect CTRL_HEADER       head32 일치
#831 first child picture          head32 일치, vertical_offset=-2429 반영
#835 second child picture         head32 일치
```

하지만 전체 payload hash는 아직 다르다.

```text
#824 CTRL_HEADER payload hash differs
#831 CTRL_HEADER payload hash differs
#835 CTRL_HEADER payload hash differs
#825/#832/#836 SHAPE_COMPONENT payload hash differs
#834/#837 SHAPE_PICTURE payload hash differs
```

따라서 한컴 판정에서 `hwpx-h-03` 파일손상이 남으면 다음 단계는
`SHAPE_COMPONENT`/`SHAPE_PICTURE` payload 계약 디코드로 진행한다.

## 5.1 한컴 판정 해석

작업지시자 판정 결과, Stage 26은 Stage 23과 동일한 결과를 보였다.

```text
hwpx-h-01: 성공
hwpx-h-02: 성공
hwpx-h-03: 파일손상, 2페이지 글상자 전까지만 출력
hy-001   : 파일손상, 2페이지 마지막 표 전까지만 출력
```

이 결과로 확정할 수 있는 점:

```text
1. Stage 26에서 보정한 GenShape CTRL_HEADER attr bit 13/14/26 및 signed offset은
   h03/hy-001 파일손상 원인의 충분조건이 아니다.
2. h03의 #824/#831/#835 CTRL_HEADER head32가 정답지와 일치해도 파일손상 중단 위치가
   Stage 23 대비 전진하지 않았다.
3. 따라서 다음 후보를 CTRL_HEADER 공통 attr 쪽으로 더 미세하게 좁히는 것은 생산성이 낮다.
4. h01/h02가 성공했으므로 table-axis 계약은 유지되고 있다.
5. h03/hy-001이 공통으로 실패하는 지점은 글상자/drawText 내부 객체 계약이다.
```

Stage 26 판정은 기준표의 `C`에 해당한다. 다만 `hy-001` #974 guard가 파일손상을 보였으므로,
단순한 `hwpx-h-03` 특이 현상이 아니라 글상자/drawText 계열의 HWP5 record contract 누락으로
다루어야 한다.

다음 단계에서는 `SHAPE_COMPONENT`/`SHAPE_PICTURE` payload만 보지 않고,
문단 안에서 다음 구조가 HWP5 record sequence로 어떻게 materialize되는지 정답지와 비교한다.

```text
TABLE control
TEXT runs
DRAW_TEXT / TextBox
  inner paragraph
  TAC picture
  space text
  TAC picture
```

즉, 다음 단계의 질문은 “그림 개별 payload가 다른가”가 아니라
“글상자 내부 문단과 TAC 그림들이 한컴 HWP5 record contract에 맞는 순서, level, header, tail,
payload로 저장되는가”로 둔다.

## 6. 검증

```text
cargo build
cargo test
cargo fmt --check
git diff --check
target/debug/rhwp info output/poc/hwpx2hwp/task949/stage26_genshape_common_attr_candidate/hwpx-h-01.hwp
target/debug/rhwp info output/poc/hwpx2hwp/task949/stage26_genshape_common_attr_candidate/hwpx-h-02.hwp
target/debug/rhwp info output/poc/hwpx2hwp/task949/stage26_genshape_common_attr_candidate/hwpx-h-03.hwp
target/debug/rhwp info output/poc/hwpx2hwp/task949/stage26_genshape_common_attr_candidate/hy-001.hwp
```

결과:

```text
cargo build 통과
cargo test 통과
cargo fmt --check 통과
git diff --check 통과
rhwp reload 4개 파일 모두 성공
```

## 7. 다음 판정 기준

```text
A. h01/h02/hy-001 guard 유지 + h03 파일손상 해소
B. h01/h02/hy-001 guard 유지 + h03 중단 위치 전진
C. h03 변화 없음. SHAPE_COMPONENT/SHAPE_PICTURE payload 디코드 필요
D. guard 회귀. Stage 26 후보 적용 조건 축소 또는 폐기
```
