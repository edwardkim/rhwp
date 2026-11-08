# 민원서식의 닫힌 사다리 상자 원점

기존 원본 `samples/issue6032/2912695_civil_petition_form.hwp`와 독립 한컴 Print
`pdf/2912695_civil_petition_form-2020.pdf` 전 2쪽의 같은 backend 전후 증거다.
96 DPI / compat 2022 / PDFium 153.0.7999.0 / raw 2px 90%를 유지했다.

Native와 fresh WASM은 보정 전후 각 2쪽 모두 100%이며 전후 TSV·PNG는 byte 동일하다.
새 원본이 아니라 앞 표의 위 여백을 바꾼 기존 관계 검사에서만 발생하던 후속 표 이동을
같은 저장 원점으로 복구했다. 기대값을 수정하지 않았다. 보정 후 양쪽의 1/2쪽 review와
overlay를 직접 판독했다. raw 100%를 RGB 완전 일치로 해석하지 않는다.

원본·독립 PDF SHA와 생산 source/Native/JS/WASM SHA 및 원인·기존 해결 이력은
`mydocs/working/task_m100_7445_rebase_repair_stage65.md`에 있다.
전후 before/after는 직전 후보와의 비교이며 고정 devel 대비 최종 비교를 대신하지 않는다.

집중 55건은 46 PASS / 9 FAIL, 기존 #4068 계약은 FAIL→PASS다. 기존 RowBreak HWP/HWPX의
Native 렌더 트리 전 36쪽도 변하지 않았다. 전체 nextest·전체 범위 시각/소비 검증과
PR 준비 완료를 주장하지 않는다. 로그와 중간 JSON은 ignored output에 보존한다.
