# #7998 최종 P5 증거

고정 검증 집합158입력169쪽/각 backend의 원본 TSV와 대표 비교 자료다.
`inputs.tsv`는 실제 실행 입력·독립 Print PDF와 저장소 파일의 바이트 대응을 기록한다.
기존 동일 파일은 중복하지 않는다. 비공개 견적서 원본·이미지는 로컬에만 보존한다.

`evidence-inputs/`는 실제 편집 저장본과 대응하는 독립 한컴 Print의 진단 증거다.
자동 수집 fixture/golden이나 모든 입력의 정상 출력 선언이 아니다. Native Helvetica 목록의
raw 미달은 정식 글꼴 예외 계약과 #8109로 구분하며 신규 정상 렌더링 회귀로 등록하지 않는다.
`silhouette.tsv` 원문을 수정하거나 점수·허용치를 완화하지 않는다.

검증 제품·명령·판정은 [최종 보고서](../../../../report/task_m100_7998_report.md)를 따른다.

## TSV 원문 다운로드

각 ZIP은 같은 폴더의 개별 TSV를 바이트 그대로 묶었다. 압축 해제 후
`tsv-files.sha256`으로 원문 일치를 확인할 수 있다. 후보158입력169쪽/각 backend,
기준11입력22쪽/각 backend이며 각 문서의 전체 쪽을 포함한다.

- [후보 Native 원문 전체](candidate-native-tsv.zip) · [개별 파일](candidate-native/)
- [후보 fresh WASM 원문 전체](candidate-wasm-tsv.zip) · [개별 파일](candidate-wasm/)
- [devel Native 원문 전체](base-native-tsv.zip) · [개별 파일](base-native/)
- [devel fresh WASM 원문 전체](base-wasm-tsv.zip) · [개별 파일](base-wasm/)

글꼴 예외 JSON의 실행 당시 절대 경로는 원 증거 위치다. `font-evidence/`의 같은 파일명은
해시가 같은 공개 사본이며 `font-exception-gates.json`이 그 해시를 고정한다.
