# #8733 한컴 Print 관찰 자료

[합성 입력과 검증 집계](README.md)의 저장 결과를 로컬 한글 2020 11.0.0.9166에서 열고 Hancom PDF 1.3.0.550으로 인쇄한 자료다. 저장/내보내기 변환이 아닌 Print 경로이며, 입력·출력의 SHA-256과 대응 근거는 [인쇄 목록](hancom-print-manifest.json)에 기록했다.

## 관찰 결과

- cell-new-solid-hwp-2020.pdf, table-new-solid-hwp-2020.pdf: 새 셀·표 단색 HWP. 표 파일은 07:34:16 UTC 재인쇄 보존본이다.
- cell-new-solid-hwpx-2020.pdf, table-new-solid-hwpx-2020.pdf: 새 셀·표 단색 HWPX. 두 원본 확장자는 출력자가 직접 확인했다.
- imported-raw-zero-hwp-2020.pdf: 기존 무늬 0 대조군. 가로 무늬를 유지한다.
- cell-cleared-zero-hwp-2020.pdf, table-cleared-zero-hwp-2020.pdf: 무늬 없음을 명시적으로 선택한 HWP. 가로 무늬가 사라지고 단색은 유지된다.

모든 PDF는 A4 1쪽이며, 배경은 RGB(170,187,204)다. 새 단색과 명시적 제거 결과에는 가로 무늬가 없다. 셀의 HWP와 HWPX 인쇄본은 240 DPI 래스터 이미지가 바이트 단위로 동일하다. 다른 출력의 전체 이미지가 동일하다는 주장은 하지 않는다.

## 제한

- 초기 도구 관찰에서는 가로·세로 100%를 직접 확인하지 못했다. 후속 확인에서 출력자가 이 목록의 7개 모두를 인쇄 당시 가로·세로 100%, 원래 용지, 전체 페이지, 한 쪽씩으로 설정했다고 명시적으로 확인했다. 원본 확장자와 고유 PDF 대응은 그대로 유지한다.
- 설정 증거의 수준은 **출력자 확인**이다. UI 직접 관찰, PDF metadata에서의 설정 검출, 현재 기본값으로의 추정이라고 쓰지 않는다. 추가 재인쇄 없이 보존된 동일 PDF를 사용한다.
- 이 자료는 이번 저장·채움 검토의 독립 Print 증거다. 기존 raw0 페이지 렌더링 차이까지 시각 통과로 승인하지 않았으며, `oracle_page_count_baseline.tsv`에 추가하지 않았다. 원본은 `samples/` 코퍼스가 아닌 PR 합성 입력이다.
- 최초 table-new-solid.hwp의 Print PDF는 뒤의 HWPX 인쇄 중 덮어쓰기되었다. 최초 PDF의 해시·쪽수 기록과 [120 DPI 원본 래스터](hancom-table-new-solid-hwp-120dpi.png)는 남아 있다. 이후 실제 HWP를 다시 열어 인쇄한 table-new-solid-hwp-2020.pdf를 별도 보존했다. 두 HWP/HWPX 파일을 혼동하거나 서로 대체하지 않는다.
- Native/fresh WASM 페이지별 비교는 별도 증적으로 구분한다. 이 자료만으로 정량 시각 gate 통과를 선언하지 않는다.
