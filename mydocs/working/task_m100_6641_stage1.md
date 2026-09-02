# Task M100 #6641 Stage 1 — 필드 setter 지원 행렬과 red 계약

- **실행일**: 2026-09-03 KST
- **작업 head**: `task_m100_6641@a62ce2f5e`
- **제품 기준**: 보존 WIP `ce2fb30b8` + `upstream/devel@2edbe62e5` 정상 merge
- **범위**: 공개 sample만 사용한 setter·저장 왕복 집중 계측

## 1. 결론

보존 WIP는 본문 HWP5, 1단계 표 셀 안 HWP5 ClickHere, 표 셀 안 글상자 HWP5 ClickHere와
2단계 중첩 HWPX ClickHere를 각 소유 폭으로 재조판한다. 지원 표면에 남은 제품 결함은 하나다.

1. HWPX 표 셀 가상 필드는 조회 결과에 합성 `fieldId`가 공개되지만
   `set_field_value_by_id`가 실제 ClickHere의 `field_ranges`로 처리해
   `InvalidField("field_range 인덱스 초과")`를 반환한다.
깊은 HWPX ClickHere에는 별도 관측이 있다. 편집 직후 소유 문단은 bit 31 합성 LineSeg를 가지며,
HWPX 저장기는 #5847 안전 규약에 따라 이를 파일에서 생략한다. owner reflow와 재적재 가능한 텍스트·
문자모양 축은 정상이지만 strict `edit_verify_report`는 저장 전 합성 LineSeg와 저장 뒤 부재를 차이로
센다. 이는 필드 전용 재조판 결함이 아니라 모든 HWPX 편집에 적용될 수 있는 일반 저장 정규화 경계다.
#6641에서 diff 예외나 serializer 변경을 섞지 않고 별도 후속 후보로 분리한다.

## 2. 지원 행렬

`set_field_value_by_name`은 occurrence 0으로 `set_field_value_by_name_at`에 위임한다.
실제 ClickHere는 세 setter가 같은 `FieldLocation` 기반 mutation에 수렴하고, 가상 셀 필드만
별도의 cell text mutation이 필요하다.

| 소유자·형식 | 표면 | Stage 1 결과 | disposition |
| --- | --- | --- | --- |
| 본문 HWP5 ClickHere | by-id / by-name / by-name-at | 기존 #838와 batch 계약에서 정상 | 보존 |
| 표 셀 HWPX 가상 필드 | by-name / by-name-at | CLI 정상 채움·verify diff 0 | 보존 |
| 표 셀 HWPX 가상 필드 | by-id | **실패**: 공개 합성 ID를 실제 range처럼 해석 | 기존 공개 조회/쓰기 일관성 복원 |
| 표 셀 HWP5 ClickHere | by-id / by-name / by-name-at | 대표 by-id 저장 왕복 정상 | 보존 |
| 표 셀 안 글상자 HWP5 ClickHere | by-id / by-name / by-name-at | 대표 by-name 저장 왕복 정상 | 보존 |
| 깊이 2 표 셀 HWPX ClickHere | by-id / by-name / by-name-at | 대표 by-name-at owner reflow·HWPX 합성 생략 정상 | 보존; strict verify 경계는 별도 후보 |

가상 셀 by-id는 신규 필드 종류를 추가하는 기능 확장이 아니다. `collect_all_fields`와 WASM 공개 문서가
이미 제공한 ID를 같은 공개 setter가 쓰지 못하는 비대칭을 복원하는 것이다.

## 3. 공개 fixture와 관측값

| fixture | field | 경로 | 결과 |
| --- | --- | --- | --- |
| `samples/field-01.hwp` | `회사명`, `작성자` | 본문 | 저장 왕복 LineSeg·값 일치 |
| `samples/issue6102/36444579_traffic_fine_exemption.hwpx` | `기관명` | 표 셀 가상 필드 | by-id range 초과 |
| `samples/76076_regulatory_analysis.hwp` | `안건명` | 표 셀 | 저장 왕복 LineSeg·값 일치 |
| `samples/basic/BlogForm_BookReview.hwp` | `이곳에 책 표지 그림을 넣으세요.` | 표 셀 → 글상자 | 저장 왕복 LineSeg·값 일치 |
| `samples/issue1893_clickhere_field_roundtrip.hwpx` | 조직명 XPath형 필드명 | 표 셀 → 표 셀 | 편집 직후 합성 LineSeg 유효, 파일에서는 생략·재적재 시 조건별 재합성 |

깊은 HWPX fixture의 무편집 `export-hwpx --verify`는 diff 0이다. 필드 mutation 뒤 strict verify는
`ParagraphLinesegs expected=1 actual=0` 한 건을 보고한다. 메모리 쪽 한 줄은 모두
`TAG_IMPLEMENTATION_PROPERTY`이고 serializer가 의도적으로 생략하므로, 이 차이는 원본 저장 정보의
소실이 아니다. 이를 #6641에서 허용 목록으로 숨기면 보호 불변식 1을 어기므로 별도 설계 대상으로 둔다.

## 4. red 실행 증적

파생 suite 준비·정합성 검사는 다음 집계를 통과했다.

```text
1120 sources / 4808 static test attrs
28 suites + 20 exceptions = 48/48 integration targets
nextest 최소 6559 cases
```

집중 실행:

```bash
node scripts/run-rust-test.mjs issue_838_field_set_value -- \
  --cargo-profile release-test --target-dir target/pr-review
```

결과:

```text
6 tests run: 5 passed, 1 failed
FAIL set_field_value_by_id_updates_virtual_hwpx_cell_and_roundtrips_layout
  InvalidField("field_range 인덱스 초과")
```

초기 red 계약은 HWPX 재적재 뒤에도 편집 직후 LineSeg가 그대로 남아야 한다고 잘못 요구했다.
그러나 reflow 합성 LineSeg(bit 31)를 파일에서 생략해 한컴이 안전하게 재계산하게 하는 것은
`#5847`의 기존 보호 규약이다. 이 계약을 HWP5 저장 LineSeg 보존과 HWPX 합성 LineSeg 생략으로
분리한 뒤 제품 red를 가상 셀 by-id 한 건으로 확정했다. 같은 HWPX의 strict verify 차이는 별도
저장 정규화 경계로 기록했다.

처음 실행의 0-test 결과는 원본 test 편집 뒤 파생 suite를 갱신하지 않은 실행 절차 오류였다.
`rust-test-suite-manifest --prepare --check` 후 위 6건을 다시 실행해 제품 실패 한 건을 확정했다.

## 5. Stage 2 입력 불변식

- 가상 셀 필드의 합성 ID는 name setter와 같은 cell mutation으로 수렴해야 한다.
- 실제 ClickHere의 ID 경로는 현재 range mutation을 유지해야 한다.
- 편집 성공으로 반환하는 모든 지원 경로는 소유 문단에 유효한 LineSeg를 가져야 한다.
- HWPX의 파생 LineSeg 생략을 결함으로 오인하지 않고, bit 31 표식과 재계산 가능한 텍스트 축을 보존한다.
- strict verify에서 합성 LineSeg의 저장 정규형을 비교하는 일반 정책은 #6641의 field query 수정에
  끼워 넣지 않는다.
- Gym reference·oracle, verify exit code, diff 허용 목록은 변경하지 않는다.
