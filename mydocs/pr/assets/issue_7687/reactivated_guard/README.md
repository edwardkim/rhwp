# #7687: 이미 이관한 원본의 부적합한 검사 재활성화 보관

## 원문과 선행 처리 추적

원 PR #8005의 `839af8c92427d834d7042ec540c5e0fbab01473c`를
`fe46e0698603ea22d2fcbed3e801921091cd7e86`으로 적용하며 추가된
`hwpspec_empty_host_lines_fill_gap_and_title_clears_second_table`만 활성 경로에서 제외했다.
고정 기준 `upstream/devel` `2e3b4f880bd11d35398e6ea0100846611f13a72d`에는
이 테스트 파일 자체가 없다. base에 등록된 검사를 새로 이관했다고 보고하지 않는다.

같은 원본은 이미 이전 검토에서 보관·이관됐다. 선행 원장은
`mydocs/pr/assets/issue7445/format_spec13_test_removal_validation.json`이며, 원본은
`mydocs/pr/assets/issue7445/한글문서파일형식_5.0_revision1.3.hwp`에 있다.
현재 `samples/`의 입력과 보관본의 SHA-256은 모두
`f21edf2138e134702366f2fb6a2ab082b05c6dcb42216ec4fa2575ed292efd1d`다.
이 경로는 과거 처리의 출처이며 새 공통 이관 대상 지정이 아니다.
같은 미해결 증상의 기존 이슈는 #7687이며 중복 이슈를 만들지 않는다.

원 PR 자체도 Native 72쪽 / 공식 PDF 71쪽과 목표 쪽 raw Native 81.2% /
WASM 81.8%를 기록했다. 합성 H1~H6의 100%로 원본의 등록 조건을 대신할 수 없다.
고정 base도 72쪽, 현재 후보는 74쪽이고 공식 PDF는 71쪽이다. 쪽수 차이로 전체
비교를 중단한 실행에서 raw 최저값을 만들어 기록하지 않는다. 이 제외는 출력 개선이나
원본 전체의 해결을 뜻하지 않으며 현재 구현의 다른 회귀를 면제하지 않는다.

## 보존과 제거 범위

`issue_7687_empty_host_multi_float_gap.rs.txt`는 제거 전 파일 **전체 원문**이며 공용
helper를 포함한다. 원본 Git blob은 `51f03fb7336f58762925361384abeadb266b5362`,
파일 SHA-256은 `86c029d77872bcb111379f5c482816111b2fe3eeea0561a48cc269f6e23beed7`이다.
`preservation.json`에 입력·독립 PDF·고정 base·적용 커밋·제거 함수·유지 범위를 적었다.

제거한 것은 해당 함수 한 개다. 모듈 자동 수집, 정상 합성 검사
`synthetic_empty_host_cases_follow_hancom_gap_rule`, 공용 helper, 다른 parser·IR·
roundtrip 검사는 유지했다. 다른 정상 검사도 참조하는 `samples/` 원본과 공식 PDF는
이동하거나 삭제하지 않았다. 새 baseline 완화나 문서 전체 corpus 제외를 추가하지 않았다.
정상 H1~H6는 같은 후보 Native/fresh WASM의 전체 6쪽 모두 raw 100%이며
각 backend 하위 폴더에 TSV 출력 원문을 보존했다.

## 복원 조건

같은 원본의 전체 쪽수가 독립 Print와 일치하고, 전체 Native/fresh WASM raw 최저값이
각각 90% 이상이며 직접 확대 판독을 마친 후 복원한다. 내용 소유로 대상 쪽을 찾아
순서·누락·중복·표 사이 포함·겹침 관계를 검사한다. 기존 19쪽 핀만 20쪽으로 바꾸는
진단 사본의 통과를 복원 근거로 삼지 않는다. ±2px 고정은 해당 쪽 양쪽 raw 98% 이상과
직접 판독을 충족할 때만 추가하며, 같은 핵심 경계의 100% 쪽을 먼저 고른다.
