# #8109 저장·재열기 시각 검증 입력

공개 `pdf/issue7998/body-list.pdf`, `pdf/issue8109/` resource 대조군 및 명시적 Arial 대체
`standard-substitute.hwpx`에서 생성한 HWP/HWPX다. 새로운 한컴 원본으로 분류하지 않는다.
2단계에서 직접 생성·편집·저장하고 독립 한컴2020 Print와 대조한 바이트를 보존했다.

- 목록: 무편집, X 삽입, 직접 삭제·복원, PDF/HWP/HWPX 재열기 후 Snapshot undo/redo와 저장.
- resource 대조군: absent·inline/indirect absent, 네 스타일, Form별 같은 이름의 다른 프로그램 상태.
- 독립 명시적 대체: HWPX → HWP → HWPX 재열기·교차 저장.

동일한 SHA-256의 저장본은 한 파일로 보존하고 각 동작을 manifest의 별도 행으로 연결한다.
원본·각 저장 동작·Print 생성 절차와 판독은 `mydocs/working/task_m100_8109_stage2.md`,
최종 검증과 입력/Print별 SHA-256·TSV 경로는
`mydocs/pr/assets/issue_8109/stage3/inputs.tsv` 및 `mydocs/working/task_m100_8109_stage3.md`에 있다.
각 Print는 `pdf/issue8109/print/`에 있으며 실제 검증한 것과 바이트가 같다.
상용 font 프로그램은 저장하지 않았다.

`only-source-font.hwp`는 `pdf/issue8109/standard-substitute.hwpx`를 Native DocumentCore로
읽고, 각 언어의 Helvetica/Arial 대체 선언 한 항목만 남긴 뒤 글자모양의 font ID를0으로
연결해 정식 HWP serializer로 저장했다. 대체 이름이 별도 face entry 없이도 유효한 경계다.
독립 한컴2020 Print·Native·fresh WASM 모두1쪽/100%를 확인한 뒤 보존했다.
브라우저의 늦은 Local Font Access 갱신 검사에서는 공개 Noto Serif 프로그램에 제어된 Arial
별칭을 주어 실제 자원 소비와 문서 보존을 확인한다. 그 합성 자원 계약을 Arial 모양의 시각
일치로 해석하지 않는다.

이 폴더는 이미 독립 출력으로 검증한 저장 결과의 재현 자료다. `samples/`의 corpus 기준값이나
새 렌더링 golden을 등록하지 않으며 기존 정식 `issue_8109_pdf_font_substitution` 검사를 대신하지 않는다.
