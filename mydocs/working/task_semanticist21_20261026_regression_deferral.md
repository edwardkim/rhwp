# 기존 미달 회귀의 범위 분리와 전체 검증 준비

## 판정 근거

현재 브랜치의 기존 보정 대상과 리베이스 기능 보존 범위만 처리한다. 다른 문서를 새로 추가하지
않으며, 삭제된 sample16·2025 편람의 코드·입력·파생 참조·보관본을 복원하지 않는다.

고정 base는 `238de6de41a208648f4ade6b494fe433a81c1fce`다. Native247과 fresh WASM070의
런타임 소스는 이 base와 같으며, 후보 Native248/fresh WASM071의 실제 측정 code는
`9504cf250a211077a2f6c7f9137a890f8f5db8eb`다. 이후 변경은 실행 불가능한 중복 분기 제거와
검사 보정으로 구분하며, 이전 측정을 새 head의 실행으로 표기하지 않는다.

| 대상 | base 근거 | 후보 근거 | 처리 |
| --- | --- | --- | --- |
| 규제샌드박스 그림 보존 원본 전 17쪽 | Native 최저 74.96795%, fresh WASM 75.12880% | Native 74.96784%, fresh WASM 75.13033% | 기존 미달로 #8236 등록 |
| 같은 문서의 스텁 최소본 전 17쪽 | Native 56.45794%, WASM JPEG decode 실패 | Native 56.45846%, WASM 같은 실패 | 실제 원본 그림의 결과와 구분 |
| ClickHere 80행 래퍼 | Native/fresh WASM 4쪽, 독립 Print 3쪽 | fresh WASM 3쪽이나 2쪽 raw 5.18157% | 기존 #8230에 연결 |

그림 보존 원본은 선행 #7163부터 이미 등록된 원본이며 최소본과 Contents XML 3개가 같다.
원본의 7·15쪽 Native/fresh WASM review와 standalone overlay를 직접 대조했다. 사진 뒤
본문의 줄 위치·굵기·간격과 스마트폰 그림의 크기·외곽에 기존 차이가 남는다. 최소본의 332바이트
JPEG 11개가 디코딩되지 않는 사실만으로 원본의 미달을 설명하지 않는다. 원본도 따로 확인했다.

#7158·#4764는 완료된 이슈이며, 열린 #8079는 다른 원본의 굵은 글꼴 전진 폭 문제다.
같은 미해결 파일명·증상의 열린 이슈가 없는 것을 조회한 뒤 [#8236](https://github.com/edwardkim/rhwp/issues/8236)을
등록했다. #8230은 같은 80행 입력·Print·실패 범위를 이미 기록하고 있어 중복 등록하지 않았다.
두 이슈 모두 이번 통합 PR의 Close 대상이 아니다.

## 보존과 활성 제거

- [#8236 재현 자산과 복원 원장](../pr/assets/issue_8236/regulatory_sandbox/README.md)에
  전체 5개 검사 모듈을 import·helper·assertion까지 `.rs.txt`로 바이트 보존했다. source commit·
  Git blob·SHA-256, 제거 9개 함수와 #6778의 활성 유지 6개 함수를 구분한다.
- 그림 보존 원본과 해당 독립 PDF·최소본 대조 PDF는 같은 자산 경로로 이동했다. 해시는 같다.
  최소본은 parser·serializer/글자 모양 계약이 공유하므로 원래 `samples/issue4090/` 경로에
  보존하며, renderer 축의 자동 수집만 정확한 경로로 제외한다. 이미 생성·인쇄한 편집 대조군도
  원래 바이트로 보존했으며 새 HWP나 새 Print를 만들지 않았다.
- 최소본의 `body_overflow_baseline.tsv`·`off_canvas_baseline.tsv` 행은 복원 원장에 보존하고
  활성 래칫에서 제거했다. 순수 PDF 쪽수 API·합성 비교 계약은 실제 문서를 읽지 않으므로 유지한다.
- [#8230 자산과 복원 원장](../pr/assets/issue_8230/clickhere_split_wrapper/README.md)에
  제거 전 전체 모듈·같은 HWPX·한컴 2024 Print·원문 TSV·대표 PNG를 보존했다.
  80행 분할 검사 1개와 전용 상수만 제거하고, 다른 논리 셀 주소·활성 누름틀 검사 3개는 유지한다.

합계 10개 렌더링 검사를 분리했다. 두 부분 모듈의 정상 검사 9개는 유지한다. 보관본만 추가했을
때 suite source 수는 1,771개로 같았고, 전용 원본 모듈 4개를 제거한 뒤 1,767개가 됐다.
exception 20개는 같다. 보관한 `.rs.txt`를 자동 수집하지 않는다. 삭제된 source를 되살리거나
`#[ignore]`·조용한 return으로 검사를 숨기지 않는다. 원인 보정 코드는 유지한다.

## 검증과 후속 처리

이관 자료는 기존 미달의 증거이며 정상 출력 승인 자료가 아니다. 기존 전체 페이지 TSV를 바이트
그대로 보존했고, raw 미달·디코딩 실패·쪽수 불일치를 구분했다. 실행 로그·중간 JSON은 ignored
`output/pr-review/semanticist21-20261019/`에만 둔다. 이슈의 게시 본문은 API로 재조회해 확인했다.
공개 source head가 생기면 새 보존 commit의 정확한 링크를 해당 이슈에 연결한다.

남은 검사는 같은 source의 검증 worktree에서 suite를 준비하고 정책·lint·전체 release-test
회귀를 실행한다. 전쪽 90% 이상·Print 쪽수·내용 보존·직접 판독을 마친 대상은 관계형 검사를
유지하며, 새 base 대비 raw 하락은 작업지시자의 0.1%p 기준을 따른다. 이관 이슈는 독립 Print와
Native/fresh WASM 전쪽 90% 이상 및 확대 판독 뒤 새 이름의 관계형 회귀로 복원한다.
완료된 해당 페이지가 양쪽 98% 이상일 때만 독립 좌표 ±2px를 추가한다.
