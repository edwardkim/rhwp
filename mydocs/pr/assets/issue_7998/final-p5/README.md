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

## 제출 제품 재산출 — 2026-10-21

최신 devel `2e3b4f880bd11d35398e6ea0100846611f13a72d`를 병합하고, 제품 source
`d1d3091c5`의 새 Native/fresh Docker WASM에서 같은158입력169쪽을 다시 산출했다.
이 폴더의 개별 TSV·대표 PNG는 새 실행과 바이트/픽셀이 같아 그대로 보존했다.
RGB 세 채널의 실제 픽셀을 비교했다. RGBA 이미지의 alpha 차이만으로 같음을 판정하지 않았다.
`producers.tsv`·`runtime-identity.json`은 새 producer를 기록한다. 기준5e2 이후 유입은
Qt/Native clipboard이며 비교 렌더링 경로가 같음을 실제 source diff로 확인했다.
기준 producer를 새 바이너리로 바꾸어 기록하지 않았다.

기존 Native Helvetica6건은 같은 raw·같은 geometry와 실제 요청/대체 글꼴의 정식 예외를
유지한다. 동작 불변 lint head `979188617`에 JSON을 고정한 실제 도구 판정은6건 모두
`font_mismatch_exception`이다. 후속 증거·문서만의 commit은 이 검증의 제품 코드를 바꾸지 않는다.
개인 PC 글꼴 경로나90% 허용치를 CI에 추가하지 않았다.
새 공백 저장 회귀·추가 원본 전체 TSV·실제162쪽 편집 복원/독립 Print와 CI 배포 글꼴
검증은 [추가 제출 증거](../submission-20261021/README.md)에 있다.

정식 예외 도구는 `979188617`의 코드 checkout에서 본 예외 JSON을 명시 공급해 실행했다.
이후 문서/증거만 커밋한 제출 head와 제품·테스트 소스가 같은지 별도로 확인한다.
재현할 때도 이 코드 SHA와 공개된 JSON을 함께 사용하며, 증거 commit SHA를
검증 실행 당시의 코드 SHA로 바꾸어 보고하지 않는다.
