# 현재 실제 API 동작과 독립 검출

[실제 12그룹·저장 bytes 동일성·독립 CFB·필드 문맥](../runtime-checks.md)에 결과를 기록했다.
드라이버와 정식 fixture-map.json은 [기존 API 소스](../../validation-r5/api/claim-probe.mjs)를 재사용한다.
최적화 패키지는 현재 WASM SHA를 공급한다. 원시 실행 로그·중간 JSON은 제출하지 않는다.
수정 전에도 통과하는 PR #8602의 RHWP 재열기는 결함 검출 근거가 아니며 독립 원시 레코드 대조로 보완했다.
