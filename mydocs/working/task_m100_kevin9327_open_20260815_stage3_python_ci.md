# Task M100 Kevin9327 공개 PR 통합 검토 Stage 3 - Python CI 경고 정리

## 목적

통합본의 새 `gym/tools/fuzz_corpus.py`와 그 순수 로직 계약이 기능상 통과하더라도
테스트 실행에서 `ResourceWarning`을 내지 않도록 파일 읽기·쓰기를 명시적으로 닫는
고수준 API로 정리한다.

## 실행 계획

1. 도구의 `open(...).read()`와 `open(...).write(...)`를 `Path.read_bytes`·`Path.write_bytes`로
   바꾼다.
2. 계약 테스트 fixture도 같은 방식으로 바꾼다.
3. CI와 같은 `unittest` 실행으로 테스트 통과와 경고 제거를 확인한다.

## 실행 결과

파일 열기 경로를 `Path.read_bytes`·`Path.write_bytes`로 바꾸고 fixture도 같은 API로
정리했다. 다음 검증은 2026-08-16에 통과했다.

```text
PYTHONWARNINGS=error::ResourceWarning python3 -m unittest scripts/tests/test_gym_fuzz_corpus.py
Ran 5 tests
OK

python3 -m unittest scripts/tests/test_gym_competitive_bench.py tools/agent_onboarding/test_rhwp_doctor.py tools/test_harness_proofs.py
Ran 46 tests
OK
```

따라서 이전 실행의 대량 `ResourceWarning`은 제거됐고, CI가 호출하는 새 fuzz·경쟁 벤치
계약은 기능과 자원 정리 양쪽에서 통과한다.
