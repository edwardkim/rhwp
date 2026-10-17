# CGMP 회귀 원본의 독립 Print 재검토

- 원본: `samples/issue6035/cgmp_evaluation_table.hwpx`, SHA256 `0f03119d6ac3e19f74fc9fcec5315ac6ed436bab5abffd0804e9cf3350dd747a`. 고정 upstream base `21ed64071f81f35f37618717693bd30f727392c9`에도 같은 바이트다.
- 기준: `pdf/semanticist21-20261008/cgmp-original-v97/cgmp_evaluation_table-0f03119d6ac3-2020.pdf`, SHA256 `08ae67334ed2cb690dc25f5447d9c7204044a5b313e026fb20f4147de0182d4a`,48쪽.
- 실제 출력: Hancom2020 `11.0.0.9136`, managed direct DLL, PrintMethod0, 한 쪽씩 인쇄, 원본 용지/방향 유지, 전처리 없음. job `7d4c4ecb-a9c6-409e-b21b-a59a0c0313ae`.
- 독립 Print에서 ‘다. 물 공급 설비는 다음을 만족하는가?’는35쪽, ‘물의 정체와 오염’은36쪽에 있다. 기존 두 회귀의 같은 쪽/제목만 있으면 빈 쪽이라는 기대와 다르다.
- Native base/current 전48쪽 모두90% 미달이며, 현재 후보35/38쪽의 별도 악화도 남는다. 이 자료는 진단 증거이며 새 regression/golden 또는 출력 개선 완료의 근거가 아니다. 검사 기대값은 아직 변경하지 않았다.
- 전체 명령·TSV·MCP 실행 정보·대표 review는 ignored `output/pr-review/semanticist21-20261008/`의 `cgmp-original-actual-print-v97.json`, `cgmp-original-base-current-native-minima-v97.json`, `native-current-cgmp6035-water-direct-review-v97/`와 통합 리뷰의 진행83–84에 연결한다.
