# 22 — LOAD_FAIL

비교 대상 바이트를 파싱하지 못했다. **측정 실패**이지 회귀 검출이
아니다.

- 단건 없는 파일: `오류: 파일 읽기 실패`, exit 1
- 배치 한 행: status `LOAD_FAIL`, TSV `error` 컬럼에 이유, 다른 행은 계속
- 배치 `--json`: 그 줄에 `error` 키. 전건 중 하나라도 있으면 전체 exit 1
- 배치 폴더 자체 없음: 비교 시작 전 exit 2

처방: `rhwp info <파일> --json` 으로 그 파일만 연다. 암호·손상·확장자
사칭을 가른다. 이웃 스킬 `rhwp-doc-triage` 로 넘길 수 있다.

LOAD_FAIL 을 exit 3 으로 접지 않는다. 3 은 "재봤다, 차이가 있다"이다.

JSON 배치 행에 `error` 키가 있으면 `regression` 은 false 다. 측정하지
못했으므로 회귀를 검출했다고 말할 수 없다. TSV 의 `error` 컬럼과
같은 축이다.
