# 실제 API와 독립 레코드 재현 소스

```bash
node mydocs/pr/assets/semanticist21_20261105/validation-r5/api/claim-probe.mjs <최적화 패키지 절대 경로> <output 결과 경로> <제품 SHA>
venv/bin/python mydocs/pr/assets/semanticist21_20261105/validation-r5/api/read-hwp-paragraph-records.py <기준 HWP> <후보 HWP>
```

fixture-map.json은 드라이버의 런타임 입력 계약이며 중간 실행 결과가 아니다.
현재 결과와 수정 전후 검출의 의미는 [현재 동작 기록](../../validation-r6/runtime-checks.md)에 있다.
원시 실행 로그·관측 JSON은 제출하지 않는다.
