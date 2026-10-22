# 온전한 중첩 표의 행 높이와 clip 소유 복원

- 부모 HEAD: `6c977c63c92ba9444415ab405046333c8e99c7bb`.
- 변경 patch SHA-256: `03ed493c7219b9ff08189eca5e0123a72abef7ed77db9743252414179e8d5fcb`.
- Native CLI SHA-256: `bb51351cc555854f630b5d12c37fc8b4e0702ed249eb785d7bbb5b9623d22af4`.
- Native lib SHA-256: `f073c7eebbade4eb3af7867c41e24ac084c8606979eee38f03305b4453eb6ae9`.
- fixed base: `2e087a5b4bc1515ad877e8020b30114639a59392`, CLI SHA-256 `39beec91d5bdfcc3a356071f3c497efdf8bd3f4cf402df6fa72de7c47a211913`.

## 실제 조건과 전체 비교

Native Print96DPI / PDFium152.0.7947.0 smooth RGB / raw2px90%, 같은713개 글꼴 hash, capture8. HWP는compat2024와2024 Print, HWPX는compat2022와2020 Print를 사용했다. 기준 PDF·테스트·문턱은 변경하지 않았다. 파일별 base/수정 전/후115쪽씩 총690쪽을 새로 산출했다.

- giant-hwp: `issue1949_giant_cell_nested_tables_perf.hwp` SHA-256 `ef10261cd29325116028e4f4f3e6be1a72c675eb771bddfd8484e7fe5aa94b4e` / `issue1949_giant_cell_nested_tables_perf-hwp-2024.pdf` SHA-256 `9fb08dea604813eed90a0449f29682d400be54dacd413352cc4f680f37cd4a44`.
- giant-hwpx: `issue1949_giant_cell_nested_tables_perf.hwpx` SHA-256 `fc6e5f156de470dfbb14aab392389491720ee7fb1bf6f03fe9a018e93b420c65` / `issue1949_giant_cell_nested_tables_perf-hwpx-2020.pdf` SHA-256 `ba065afdd3ace7fd6b87fe82848e6cae3d01915a1606e4d8a70e4c2153d57d73`.

두 형식 모두 후보 최저97.01264%(102쪽), 미달·쪽수 불일치0이다. 4쪽은97.05212→99.07270%. 두 형식 모두4쪽만 PNG가 바뀌었으며, 전115쪽의 TextRun 순서·소유(pi)·개수7919개가 수정 전과 같다. base 및 수정 전 대비0.1%p 이상 하락은0이다.

4쪽 review/overlay를 직접 열어 내용에 맞춘 두 머리행, 마지막 행의 세 셀·세 줄·하단 괘선 보존을 확인했다. 본문의 일부 글자 가장자리와 괘선 굵기 차이는 남는다. 완전 일치나 fresh WASM 수용으로 확대하지 않는다.

## 원본 규칙과 실제 소비

원 보정 `3fe47ee64facea31823eb1cbd02035f86093fac6`과 리베이스 전5e23738의 no-cache 행 계산·clip 소비를 대조했다. 기존 순수 `fit_stored_declared_tail_row_heights`를 현재 행 계산이 사용하지 않아 앞 머리행 각각1282HU와 끝 행5331HU가 총7635HU를 초과했다. 동일 helper가 내용 하한 위의 끝 빈 여유260HU만 회수한다. 앞 두 행17.093333px는 그대로, 끝 행71.08→67.613333px, 총105.266667→101.8px다. 현재 upstream의 행 span 해결 순서·내용 하한과 편집/재조판 가드를 유지했다.

원본 시작·전체 행/내용 창을 소유한 Exclusion child가 공유 float frame의 occupied_bottom까지 조상 cell clip을 연다. 부분 컷은 제외하며 pagination 예산·흐름 cursor를 늘리지 않는다. 실제SDK의 원본 선언7635HU와 최종bbox101.8px가 일치하고, 독립 Print 좌표·모든 조상 clip·내용 소유 기존4검사가 PASS다.

집중20건은16 PASS/4 FAIL(3.973초). 나머지4건은 #4138 편집 분할본 그림·자식 프레임 소유로 이번 보정 전에도 실패했다. 기존 검사/px 기대값을 수정하지 않았다. 두 추가 파일의 변경은 rustfmt 정리만이다. 새 full nextest·fresh WASM 전쪽·실제 소비 경로와 다른 보류는 아직 필요하다.

## ignored 원장

- `/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-7752-20261022/repair-focused-v17/source.json` — SHA-256 `bf1521c94b54d1bd9a08746a1fedcfee62ac24db26d79315ef7686b1204dc715`.
- `/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-7752-20261022/repair-focused-v17/result.json` — SHA-256 `8c7b4ad9ad5309008cab75bab9e856ed5a72db00a3b989ffe5d58e335b7a56c8`.
- `/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-7752-20261022/repair-focused-v17/nextest.log` — SHA-256 `48c3dbff1350712929562b5eae67459b57455f3f18b4458f06d1a4aa05056aaa`.
- `/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-7752-20261022/repair-focused-v17/artifacts.json` — SHA-256 `5cd1fc22f48524c830c300837941ee88cdd3a63221e120f5529aaecba320c161`.
- `/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-7752-20261022/giant-originals-native-v17/producer.json` — SHA-256 `162c1b456f853f423f9f7ba0cfbc0a7f7915b27835ca808c7b00132f49cd306e`.
- `/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-7752-20261022/giant-originals-native-v17/results.json` — SHA-256 `2573e068ce1d96e94caf29978dc218151213353c1aa8a570fd80176072f95327`.
- `/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-7752-20261022/v17-before-after-comparison.json` — SHA-256 `8adb5d0636beab6dedd55d05fc06027ac1a448b21dbdccd779f20ca5e543a0f7`.
- `/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-7752-20261022/giant-header-source-cause/after-v17.log` — SHA-256 `c0e6fb89a191a3d44ab02e206c5c10c36f16487f3c11cd1b09e6422afe7c7edb`.

## 같은 소스의 기존 실패 재검사

56건24 PASS/32 FAIL(47.878초). 수정 전 대비 새 실패0, 온전한 자식3건 복구.
- `/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-7752-20261022/retry-remaining-native-v17/result.json` — SHA-256 `c12b0bc23074a8dbe03747051c503156686e72dc2ea3d85b1b054fbf2ce385e0`.
- `/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-7752-20261022/retry-remaining-native-v17/failure-index.json` — SHA-256 `9883b7582c944167991bda0d7cefc1bac52dd4edb95c800929bd1956a159b64a`.
- `/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-7752-20261022/retry-remaining-native-v17/nextest.log` — SHA-256 `cc71cbceadd3372147703b57158a22f1c34140b27f6d15af23b5f81147407d74`.
