# 수식 브랜치에서 #7445 이관 검사를 분리한 원문

사용자 지시에 따라 `fix/equation-font-metrics-pagination-20261007`에서 제거 전 head `592a7a889769d047d6ae035694923ad48249810b`의 검사 원문을 보존한다. `.rs.txt`는 정식 Rust suite가 자동 수집하지 않는 자료이며, 각 파일의 다른 정상 검사도 삭제 전 문맥을 보존하기 위해 함께 들어 있다.

- [정확한 함수·행·원본 SHA 원장](transfer-inventory.json)
- [반영 범위와 검증 결과](../../../../working/task_m100_7642_stage136.md)
- [대표 이슈 #7445](https://github.com/edwardkim/rhwp/issues/7445)의 최신 하위 이슈 기록을 기준으로 적용했다.

28개 회귀 함수, 혼합 입력 1개와 세로 assertion 1곳, 원장 29행을 활성 검증에서 분리했다. 제거 전 함수는 이 디렉터리의 이전 source 경로에 `.txt`를 붙인 파일과 위 commit에서 복원할 수 있다. 원본 HWP/HWPX·Print PDF와 다른 정상 계약은 기존 경로에 보존한다.

이관 철회·로컬 보정 완료와 과거 실패는 구분한다. 문서가 같은 하위 이슈에 등장한다는 이유로 parser·serializer·보안 계약이나 다른 정상 함수를 함께 폐기하지 않았다. 이관은 출력 개선·전체 검증 완료·PR 승인 판정이 아니다. 관련 전쪽 Native/fresh WASM 최저90% 이상 및 직접 판독 후 관계형 회귀를 새 이름으로 복원한다.
