# Task M100-1092 Stage 3 작업 기록

## 1. 단계 목표

Stage 2의 `06_memo_field_id_fix.hwp`는 다음 두 축을 정답지와 일치시켰다.

```text
1. DocInfo MEMO_SHAPE payload
2. MEMO field marker CTRL_HEADER payload
```

하지만 한컴 에디터에서는 여전히 Runtime Error가 발생했다.

```text
R6025 pure virtual function call
```

따라서 Stage 3에서는 한컴 공식 HWP5 스펙과 `samples/aift.hwp` 정답지를 기준으로 마지막 구역 끝
`MEMO_LIST` 컨테이너를 materialize했다.

## 2. 한컴 공식 스펙 확인

참조 문서:

```text
mydocs/tech/한글문서파일형식_5.0_revision1.3.md
```

확인한 근거:

```text
1. HWPTAG_MEMO_SHAPE: 메모 모양
2. HWPTAG_MEMO_LIST: 메모 리스트 헤더
3. 각 구역의 가장 끝 위치에는 확장 바탕쪽 관련 정보가 저장된다.
4. 마지막 구역의 가장 끝 위치에는 메모 관련 정보가 저장된다.
5. FIELD_MEMO 자체는 정의되어 있지만, aift.hwp 정답지의 MEMO field marker는 %unk + 0x8001 형태다.
```

해석:

```text
HWPX의 fieldBegin type="MEMO"를 HWP5로 저장할 때는 본문 field marker만으로 끝나지 않는다.
마지막 구역 끝에 MEMO_LIST record와 메모 본문 paragraph list를 추가로 저장해야 한다.
```

## 3. 구현 내용

수정한 소스:

```text
src/model/control.rs
src/parser/hwpx/section.rs
src/serializer/body_text.rs
```

적용 내용:

```text
1. Field 모델에 memo_paragraphs를 추가했다.
2. HWPX fieldBegin type="MEMO" 내부 subList 문단을 memo_paragraphs로 파싱한다.
3. BodyText 마지막 구역 직렬화 시 MEMO field 목록을 모아 문서 끝에 MEMO_LIST 컨테이너를 생성한다.
4. 메모 root 문단은 마지막 본문 문단의 문단 모양/글자 모양/line segment를 기반으로 생성한다.
5. 메모 root line segment의 vertical_pos는 직전 문단에서 line_height + line_spacing만큼 내려간 값으로 보정한다.
6. MEMO_LIST 하위 메모 본문 문단에는 PARA_LINE_SEG를 쓰지 않는다.
```

## 4. 생성 후보

출력 위치:

```text
output/poc/hwpx2hwp/task1092/stage3_memo_list_candidate/
```

생성 파일:

```text
output/poc/hwpx2hwp/task1092/stage3_memo_list_candidate/07_memo_list_materialized.hwp
output/poc/hwpx2hwp/task1092/stage3_memo_list_candidate/08_memo_list_no_lineseg.hwp
output/poc/hwpx2hwp/task1092/stage3_memo_list_candidate/09_memo_list_root_lineseg.hwp
```

최종 판정 후보:

```text
output/poc/hwpx2hwp/task1092/stage3_memo_list_candidate/09_memo_list_root_lineseg.hwp
```

파일 정보:

```text
size = 4,605,440 bytes
rhwp info = ok, sections=3, pages=76
```

## 5. 정답지 비교

정답지:

```text
samples/aift.hwp
```

생성한 비교 산출물:

```text
output/poc/hwpx2hwp/task1092/stage3_memo_list_candidate/09_section2_inventory.md
output/poc/hwpx2hwp/task1092/stage3_memo_list_candidate/09_inventory_diff.md
```

`09` 후보의 마지막 구역 끝에는 `MEMO_LIST` 2개가 생성된다.

```text
BodyText.Section2#25863 MEMO_LIST payload=01 00 00 00
BodyText.Section2#25868 MEMO_LIST payload=02 00 00 00
```

정답지의 대응 record:

```text
BodyText.Section2#25924 MEMO_LIST payload=01 00 00 00
BodyText.Section2#25929 MEMO_LIST payload=02 00 00 00
```

메모 본문 컨테이너 구조도 정답지와 같은 축으로 맞췄다.

```text
MEMO_LIST
LIST_HEADER
PARA_HEADER
PARA_TEXT
PARA_CHAR_SHAPE
```

중요한 세부 차이:

```text
07 후보:
  메모 본문 문단에 불필요한 PARA_LINE_SEG가 생성됨

08 후보:
  메모 본문 PARA_LINE_SEG 제거

09 후보:
  08 후보에 더해 메모 root line segment vertical_pos를 정답지 규칙에 맞게 보정
```

## 6. 검증

실행한 명령:

```text
cargo fmt --check
cargo test -q test_parse_memo_field_begin_uses_id_as_hwp5_field_id
cargo test -q test_parse_memo_field_parameters_preserves_number_as_memo_index
cargo check
cargo run --quiet --bin rhwp -- info output/poc/hwpx2hwp/task1092/stage3_memo_list_candidate/09_memo_list_root_lineseg.hwp
```

결과:

```text
success
```

기존 경고는 있었지만 이번 수정으로 새 실패는 발생하지 않았다.

## 7. 판정 요청

다음 파일을 한컴 에디터에서 판정한다.

| file | 한컴 판정 유형 | 메모 표시 | 파일손상 여부 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task1092/stage3_memo_list_candidate/09_memo_list_root_lineseg.hwp` |  |  |  |  | MEMO_SHAPE + field marker + MEMO_LIST |

판정 기준:

```text
1. 한컴 에디터 Runtime Error R6025가 사라지는지 확인
2. 한컴 메모 컨트롤이 유지되는지 확인
3. rhwp-studio는 현재 메모 렌더링을 하지 않으므로, 이번 단계의 1차 성공 기준은 한컴 정상 로딩이다.
```
