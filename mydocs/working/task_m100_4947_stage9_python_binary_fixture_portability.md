# task_m100_4947 stage 9: Python 바이너리 fixture 이식성 보정

## 목표

전체 로컬 검증이 Linux 전용 실행 파일 경로 때문에 macOS에서 실패하지 않도록 자동화 도구 계약
fixture를 이식 가능한 경로로 바꾼다.

## 원인

- `test_engagement_records_absolute_corpus_for_later_sws_reread`는 실제 CLI 실행을 전부 mock 처리한다.
- 그런데 `run_engagement`의 사전 조건을 통과하기 위해 `/bin/true`를 rhwp 바이너리 경로로 넣었다.
- Linux와 달리 현재 macOS에는 `/bin/true` 파일이 없어 제품 코드에 도달하기 전에 종료 코드 2가 됐다.
- `RHWP_BIN` 환경변수를 지정해도 test argument의 `bin` 값이 우선하므로 해결되지 않았다.

## 수정

- 테스트 프로세스에서 항상 존재하는 `sys.executable`을 fixture의 실행 파일 경로로 사용한다.
- capability 조회와 corpus 처리는 기존 mock을 그대로 사용하므로 테스트 의미는 바뀌지 않는다.

## 검증

```bash
python3 -m unittest \
  scripts.tests.test_automation_tool_contracts.CapabilityContracts.test_engagement_records_absolute_corpus_for_later_sws_reread
python3 -m unittest discover -s scripts/tests -p 'test_*.py'
```
