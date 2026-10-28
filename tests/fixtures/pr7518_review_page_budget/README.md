# PR #7518 reviewer 페이지 예산 대조군

이 파일들은 정상 한컴 저장본이나 기준 출력이 있는 fixture가 아니라 **합성 경계 진단 입력**입니다.
원본은 `samples/issue7500/picture_band_cell_spaces.hwp`이며, 저장 정보 수용 조건을 완화하는 근거로 사용하지 않습니다.

생성 절차: `parse_hwp`로 원본을 읽고 복제한 뒤 각 section의 `section_def.page_def.height`와
그 section의 `Control::SectionDef`에 있는 같은 값을 `파일명 숫자 × 75 HU`로 바꿨습니다.
section의 `raw_stream`과 `raw_provenance`를 `None`으로 두고 `serialize_document`로 HWP5를 생성했습니다.
너비·여백·본문·그림 속성은 별도로 편집하지 않았습니다. 직렬화된 파일을 실제 `DocumentCore::from_bytes`와
`build_page_render_tree`로 실행했습니다. 원래 그림 3장이 다음 중 어느 한 쪽에 함께 존재하는지 확인하는 진단이며,
줄 메트릭이나 행 높이의 한컴 일치를 입증하지 않습니다.

| 파일의 페이지 높이(px) | 관측 쪽 수 | 그림 3장의 소속 쪽(1-based) |
| ---: | ---: | ---: |
| 500 | 4 | 3 |
| 600 | 3 | 2 |
| 700 | 3 | 2 |
| 800 | 3 | 2 |
| 900 | 3 | 2 |
| 1000 | 2 | 2 |
| 1100 | 2 | 1 |

코드 통합 후보 `55a2800aadc32fc85ed4aa3e8e17f6b169f3b0b0`에서 얻은 관측입니다.
수정 전 대조·정식 `tests/cases` 회귀·한컴 PDF가 없으므로 새로운 결함 검출이나 완전한 분할 계약 충족으로 판정하지 않습니다.
자세한 검토는 [PR #7518 review](../../../mydocs/pr/archives/pr_7518_review.md)를 참조합니다.

## 메인터너 보정의 실제 호출 경계 대조군

- `unsplit.hwp`: 같은 원본을 위 방식으로 직렬화하되 용지 높이를 225000 HU(3000px)로
  바꿨습니다. 행 5를 분할하지 않는 전체 셀 배치 경로의 높이·표 원점을 검사합니다.
- `nested-split.hwp`: 용지 높이를 37500 HU(500px)로 바꾸고, 바깥 행 5 / 열 1 / 마지막
  문단의 내부 표에서 각 셀의 선언 높이만 9000 HU(120px)로 바꿨습니다. 글자·그림·표 너비·
  오프셋은 유지했습니다. 내부 표의 행 컷 전후에 세로 리드가 한 번만 예약·배치되는지,
  4행 5열의 셀 소유가 빠지거나 중복되지 않는지 검사합니다.

- `terminal-tail.hwp`: 원본 용지 크기를 유지하고 바깥 표의 행 6·7을 제거해 `row_count=6`으로
  바꿨습니다. 마지막 행 5의 두 셀 높이를 85147 HU로 지정했습니다. 앞 조각 뒤의 마지막
  물리 밴드가 약 20px인 경계를 선택한 합성 입력이며, 구현에서 이 수치를 분기 조건으로
  사용하지 않습니다. 마지막 내용 유닛 뒤의 빈 물리 상자를 생략해 빈 쪽을 만드는지 검사합니다.

- `terminal-follower.hwp`: `terminal-tail.hwp`와 같은 조작을 적용하고 바깥 표 뒤에
  `PHYSICAL TAIL FOLLOWER` 문단을 추가했습니다. 문단 1의 서식을 복제하고 텍스트·문자 수를
  바꾼 뒤 controls와 LineSeg를 비웠습니다. 마지막 물리 상자와 후속 문단의 순서·보존을 검사합니다.

생성 API는 `parse_hwp → 모델 값 수정 → raw_stream/raw_provenance=None → serialize_document`이며,
정식 검사 위치는 `tests/cases/issue_7518_reflow_row_physical_frame.rs`입니다.
이 네 파일에도 독립 한컴 PDF가 없습니다. 재조판 계약 검증을 원본의 한컴 출력 일치와 구분합니다.

- `nested-auto-row.hwp`: 원본의 용지 높이를 37500 HU로 바꾸고 행 5 / 열 1의 마지막
  중첩 표를 2행 1열로 구성했습니다. 첫 행은 선언 높이 0인 자동 높이 셀에 `UNIT 000`부터
  `UNIT 064`까지 65개 문단을 넣고, 둘째 행은 `FINAL CHILD ROW` 하나를 넣었습니다.
  각 셀 너비는 중첩 표 전체 너비, span은 1입니다. 원 문단 서식을 복제하되 controls·LineSeg·
  char_offsets를 비우고 char_shapes는 첫 항목만 남겼습니다. row_sizes와 grid도 새 차원으로
  갱신했습니다. 원본의 앞 timeline 문단·내부 표 오프셋·여백·뒤 행은 유지했습니다.
- `nested-mixed-cell.hwp`: 같은 생성 절차로 내부 표를 1행 1열, 선언 높이 0, 65개 `UNIT`
  문단으로 구성했습니다. 자동 행 안 분할과 1×1 mixed fragment의 재귀 호출 모두 첫
  Para 리드·바깥 위 여백을 한 번만 소유하고, 65개 문단과 뒤 행이 정확히 한 번 보존되는지
  확인합니다. 두 입력 모두 명시적 쪽나눔이나 수동 저장 LineSeg를 추가하지 않았습니다.

이 두 파일은 실제 본문 예산과 임시 900px 측정 예산에서 서로 다른 child unit 원장이
선택되던 경계도 검사합니다. 정상 한컴 생성본 또는 기준 PDF로 취급하지 않습니다.

재귀 child의 요구 높이는 같은 RowCut의 셀 프레임·매 조각의 안 여백으로 검사합니다.
표 paint bbox는 테두리의 반 두께를 프레임 밖에 포함하므로 부모 안 여백 거리는 셀 프레임에서
측정하고, stroke를 포함한 실제 표 bbox도 부모 셀 안에 남는지 별도로 검사합니다.

## 추가 독립 출력과 저장 주변 프레임

위의 '독립 PDF 없음' 표기는 처음 경계 입력을 만든 시점의 기록이다. 이후 원본 합성
13개와 방향을 바로잡은 `valid_orientation` 여섯 입력을 공식 비동기 한컴 2020 엔진으로
출력해 `pdf/pr7518_review/` 및 `valid_orientation/`에 보존했다. 합성 입력의 계약과
그 입력의 실제 한컴 출력을 구분하며, 원 기관 문서 전체의 충실도로 확대하지 않는다.

`valid_hancom/{nested-split,nested-auto-row,nested-mixed-cell}.hwp`는 방향 대조군을
한컴에서 HWP로 다시 저장한 주변 프레임이다. 저장 줄이 정상 생성됐고 바깥 object
높이와 스타일 ID도 재계산됐다. 원래 NO_LS 입력과 구분해 모두 보존한다.
`valid_generated/`의 같은 세 파일은 이 저장본을 parse한 후 바깥 행 5의 양쪽 셀과
그 재귀 자식 문단의 LineSeg만 제거하고 section raw stream/provenance를 비워
다시 직렬화했다. 다른 행은 한컴 저장 프레임을 유지한다. 이 변경으로 텍스트나
문서별 분기를 새로 추가하지 않았다. 같은 새 입력을 다시 한컴 2020 PDF로 출력했다.

`valid_generated/terminal-follower.hwp`는 합성 follower의 문자 참조/오프셋 메타데이터를
유효하게 고치고, 표가 후속 문단을 밀도록 TopAndBottom 및 세로 오프셋 0을 직렬화했다.
오프셋 7018 HU는 행 5의 선언 높이에 옮겨 92165 HU로 지정했으며 raw control cache를
비우고 다시 읽어 실제 속성을 확인했다. 후속 문단은 실제 한컴 PDF p3에 존재한다.
기존 입력과 PDF를 바꿔치기하지 않았다. 입력별 HWP/PDF 해시와 변환 job ID는
`valid_generated/provenance.json`을 참조한다. endpoint·토큰·글꼴 원본은 공개 자료에 없다.

2/3쪽 그림 소유·빈 opening 및 마지막 빈 문단의 경계 검사는
`tests/cases/issue_7518_reflow_row_physical_frame.rs`에 포함한다. 진단 개선만으로
다른 input이나 Native/fresh WASM 전쪽 일치를 주장하지 않는다.
