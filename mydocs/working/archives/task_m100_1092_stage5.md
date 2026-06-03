# Task M100-1092 Stage 5 작업 기록

## 1. 단계 목표

Stage 4의 `fieldBegin/fieldEnd` range 복원 후보는 한컴 시각 판정에서 메모 스타일 개선으로
이어지지 않았다. Stage 5에서는 정답지 `samples/aift.hwp`와 생성 HWP의 메모 anchor 문단
`PARA_CHAR_SHAPE` 차이를 비교해, HWPX `charPr`에서 누락된 글자 그림자 offset 보존을 확인한다.

## 2. 핵심 비교

대상 문단:

```text
section=2, para=482
텍스트: "    - 공동기관1 : 두아즈"
```

정답지 `samples/aift.hwp`:

```text
[CS] id=287
bold=true
spacing=-10
ratio=100
base=1200
attr=0x00000002
text=#000000
shade=#FFFFFFFF
shadow=#C0C0C0
border_fill_id=2
shadow_type=0
shadow_off=(10, 10)
```

Stage 4 생성본:

```text
[CS] id=271
bold=true
spacing=-10
ratio=100
base=1200
attr=0x00000002
text=#000000
shade=#FFFFFFFF
shadow=#C0C0C0
border_fill_id=2
shadow_type=0
shadow_off=(0, 0)
```

HWPX 원본의 `charPr id="271"`에는 다음 정보가 존재한다.

```xml
<hh:shadow type="NONE" color="#C0C0C0" offsetX="10" offsetY="10"/>
```

따라서 생성본의 `shadow_off=(0, 0)`은 HWPX parser가 `offsetX/offsetY`를 누락한 결과로 본다.

## 3. 구현 내용

수정 소스:

```text
src/parser/hwpx/header.rs
```

적용 내용:

```text
1. `<hh:shadow offsetX="...">`를 CharShape.shadow_offset_x로 보존한다.
2. `<hh:shadow offsetY="...">`를 CharShape.shadow_offset_y로 보존한다.
3. shadow type이 `NONE`이어도 offset은 버리지 않는다.
```

추가 검증:

```text
test_parse_char_pr_preserves_shadow_offsets_even_when_shadow_is_none
```

## 4. 생성 후보

출력 파일:

```text
output/poc/hwpx2hwp/task1092/stage5_memo_anchor_shadow_offset/aift-shadow-offset.hwp
```

파일 정보:

```text
size = 4,605,952 bytes
rhwp info = ok, sections=3, pages=76
```

Stage 5 생성본의 메모 anchor char shape:

```text
[CS] id=271
bold=true
spacing=-10
ratio=100
base=1200
attr=0x00000002
text=#000000
shade=#FFFFFFFF
shadow=#C0C0C0
border_fill_id=2
shadow_type=0
shadow_off=(10, 10)
```

정답지와 논리 속성은 일치한다. ID는 `271`과 `287`로 다르지만, HWPX 원본에는 `287`이 없고
한컴 저장 HWP가 내부적으로 char shape table을 확장한 결과로 보인다.

## 5. 실행한 검증

```text
cargo fmt --check
cargo test -q test_parse_char_pr_preserves_shadow_offsets_even_when_shadow_is_none
cargo test -q test_parse_field_begin_end_materializes_field_range
cargo check
cargo run --quiet --bin rhwp -- info output/poc/hwpx2hwp/task1092/stage5_memo_anchor_shadow_offset/aift-shadow-offset.hwp
```

결과:

```text
success
```

## 6. 판정 요청

다음 파일을 한컴 에디터에서 판정한다.

| file | 한컴 판정 유형 | 메모 표시 스타일 | 표/페이지 배치 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task1092/stage5_memo_anchor_shadow_offset/aift-shadow-offset.hwp` |  |  |  |  | shadow offset 보존 후보 |

판정 포인트:

```text
1. 메모 anchor 왼쪽 텍스트의 초록 배경/표시가 정답지와 가까워졌는지 확인한다.
2. 오른쪽 메모 박스/연결선 스타일이 정답지와 가까워졌는지 확인한다.
3. 표 높이/페이지 배치 차이는 이미 별도 이슈로 분리했으므로, 이번 판정에서는 메모 표시 스타일만 본다.
```
