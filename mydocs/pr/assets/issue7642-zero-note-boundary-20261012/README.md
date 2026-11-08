# 높이 0 미주 경계의 Native 보정 증거

원본: `samples/3-11월_실전_통합_2024-구분선위0미주사이0구분선아래0.hwp` 10쪽.
독립 기준: `mydocs/pr/assets/pr_7600_control_print_20261006/input-11-3-11월_실전_통합_2024-구분선위0미주사이0구분선아래0-2024.pdf`.
같은 원본·Print의 일치율은 보정 전 85.44196%, 보정 후 98.67338%다.
보정 Native 바이너리 SHA-256: `e02987f4376305b4ba86f5e3f78981cf809cf6e0b05e85963cd6f03de2b4e953`.

Stage 143의 생산 코드와 같은 바이트로 빌드했다. 보정 전 source는 `68d8bf777f`다.
후 source·명령·입력/Print 해시·실제 exit와 미검증 범위는
[`task_m100_7642_stage143.md`](../../../working/task_m100_7642_stage143.md)에 연결한다.
전체 TSV·빌드 출처와 중간 JSON은 ignored output에 보존한다.

- review: Native/Print/overlay 전쪽 패널.
- overlay: 파랑=RHWP만, 빨강=Print만, 주황=잉크 차이, 회색=일치.
- detail: 동일 물리 영역의 보정 전/후/Print. 영상마다 따로 정렬·이동하지 않았다.

본문 마지막 빈 줄의 높이를 유지하면서 경계 줄간격과 구분선을 보정했다.
수식의 지수·괄호·글꼴 잉크 차이는 추가 판독이 필요하다. 선택 쪽의 Native 개선이며
전체 21쪽·fresh WASM·전체 회귀 완료를 뜻하지 않는다.
