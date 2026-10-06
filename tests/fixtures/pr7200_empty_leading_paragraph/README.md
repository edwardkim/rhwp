# PR #7200 빈 선행 문단 경계

최종 `empty-first-{top,center,bottom}.hwpx`는 빈 선행 문단과 뒤 중첩 표를 한컴에서
정상 저장한 입력이다. 원래 합성 입력은 `cell-*-collapsed.hwpx`의 첫 `Start`만 지웠지만,
텍스트·줄간격 스타일과 저장 줄 메트릭이 모순됐다. 과거 입력과 실패 PDF/PNG는
`78fd17cf8`의 동일 경로에 보존한다.
이는 원본이 원래 PDF와 일치했다는 주장이 아니다.

## 생성과 독립 기준

1. 이전 HWPX의 `Contents/section0.xml`에서 `linesegarray` 5개만 제거했다.
   텍스트, 문단/글자 스타일, 표/셀 크기·정렬·여백은 유지했다.
2. engine 2020으로 HWP 저장하여 줄 캐시를 생성하고, 다시 HWPX로 저장했다.
3. 최종 HWPX 자체를 같은 엔진으로 PDF 출력했다. 비동기 status의 succeeded/terminal=true를
   확인하고 다운로드했다. 기존 파일 경로를 갱신하며 이름만 다른 원문 복사본은 추가하지 않는다.

정상 첫 빈 줄은 `textheight=1056, spacing=632 HU`, 다음 호스트 줄의 `vertpos=1688 HU`다.
이 값을 잃어버리고 중첩 표를 셀 상단으로 배치하던 오류를 검출한다. PDF 형식 버전·Creator
표기 때문에 재변환한 것이 아니라 입력 저장 정보를 고쳤으므로 대응 PDF를 새로 확보했다.

| 정렬 | HWP 저장 job | HWPX 저장 job | 최종 PDF job |
| --- | --- | --- | --- |
| top | `15de15f8-dda4-44b9-94a5-bae3d7b27a37` | `1d7e7b08-e452-4c20-9e30-32f78bcccce1` | `3cde7a31-073d-4589-b6c5-0e1de2f78c65` |
| center | `a3b6264c-d04f-42dc-8ec7-aa6e32610257` | `b846113a-c8bd-43ae-8a64-3df9574440e9` | `c371efa1-68b5-41bf-8f59-03d25ffb55c8` |
| bottom | `1392b41c-667e-41ab-a980-965cecbc151b` | `04c7472d-8c29-4242-ae9c-6e781eb40be0` | `5fbb341f-51e3-4473-8c76-128d034b1bfc` |

이번 9개 저장/출력 job이 보고한 실제 한컴 버전은 `11.0.0.9136`, 선택 engine은 `2020`이다.
앞선 width 대조군의 `12.0.0.4605`와 같은 설치 버전이라고 쓰지 않는다.

## 검사 경로

- `saved_empty_leading_paragraph_preserves_pdf_table_and_border_positions`: 이 최종 HWPX의
  표 상단·End 글자 위치·후행 문단 테두리를 독립 PDF 좌표와 대조한다. 수정 전 FAIL, 보정 후 PASS.
- `empty_leading_paragraph_keeps_its_line_space_in_nested_table_alignment`: 기존 `cell-*-collapsed.hwpx`를
  메모리에서 빈 문단으로 바꿔 1000+200HU의 16px 점유와 정렬 불변식을 별도로 검사한다.
  이 합성 계약을 최종 저장본의 PDF 기대값으로 사용하지 않는다.
- 기존 `check_empty_leading_paragraph.py`는 보정 전 16px 합성 입력의 역사적 재현 스크립트다.
  최종 정상 저장본에는 위 정식 Rust 검사를 사용한다.

기준 PDF 경로: `pdf/pr7200/pr7200-empty-first-{top,center,bottom}-2020.pdf`.

| 정렬 | HWPX SHA256 | PDF SHA256 |
| --- | --- | --- |
| top | `968414dda9eebe70a75457b22cfaf7726d91e8de264505a9c325b84af0c9dfac` | `d753c7471d1ed5e74b2c312795c49655d39479a9d09880227e2bde9fb30171ca` |
| center | `48abd07f9fb7dd8fc035b1876496123bd0b80a35355afc4cdda5bd4d7e79867d` | `5a45bd57cbfaa80624874adb1aa7d9c40309433fce5e21ee2bbf65fa70491e78` |
| bottom | `2fd4a7561717c895030c69a2bc7e5d823e5929785acf40b5bfcd36f669ed258e` | `610111c74e3abbbb01b7af91df9b605c34fb28741afe040928d3dcbed5a3bae0` |
