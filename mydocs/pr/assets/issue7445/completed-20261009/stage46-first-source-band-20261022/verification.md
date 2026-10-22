# 첫 저장 띠와 원본 프레임 검증 출처

- 부모 HEAD: `3fcf0c0ffd633fbdb3a2973cf97dd7d4b48e06f2`.
- Native CLI SHA-256: `339ca569b0f239b74626ed3d89f305238722d3fd0128361c232d8f5d859e404e`.
- Native lib SHA-256: `be32d6a378252e36248c765e2c6bc33478efcf628840c71f931b4c1e59c7553a`.
- 변경 파일 SHA-256: `59d63178065b46b5adae1cebca682fb6db48d04902587e31864227f59738fcd5`.
- patch SHA-256: `90fb30bb76628e8060cbf7659270975e439cdb2f0cd0f22ce726ce700d76d2ad`.

## 조건과 입력

Native Print96DPI / PDFium152.0.7947.0 smooth RGB / raw2px90%. 동일713개 글꼴 hash, capture8, Cargo 순차/jobs8/test threads8. fixed base는2e087a5b4bc1515ad877e8020b30114639a59392, Native SHA39beec91d5bdfcc3a356071f3c497efdf8bd3f4cf402df6fa72de7c47a211913.

- rowbreak-hwp: compat2024, `rowbreak-problem-pages.hwp` / `rowbreak-problem-pages-hwp-2024.pdf`.
  - 입력 SHA-256: `10b6ab6548610e18c82ba78a1c844a00107fedbb28c195cb05e6fd20626d33ed`.
  - PDF SHA-256: `2c49bda9cc21dc8b93b554d2607da241dd7e894657eb308ed823e7d44654e85c`.

- rowbreak-hwpx: compat2022, `rowbreak-problem-pages.hwpx` / `rowbreak-problem-pages-hwpx-2020.pdf`.
  - 입력 SHA-256: `4ccc321e7eb146f28b768654502a128ba1ae408238fd1ed37d51e963732eb5b0`.
  - PDF SHA-256: `e3dc7bb21707f92e42e83808023bf155f62475fc4eb0e0498709d03986c5afe5`.

- pii-original: compat2022, `75544_pii_bunseok.hwpx` / `75544_pii_bunseok-hwpx-2020.pdf`.
  - 입력 SHA-256: `f429768ec02e5a849d08c06678cbee029fdc8618ca9e2bdcdbba485087fe71f1`.
  - PDF SHA-256: `0fef38f44b3825f1c97869c2720112536504eaf30cec2a2d2af6d4399ffe8770`.

## 결과와 잔여

RowBreak 두 입력은 각18쪽·미달0,13쪽만 PNG 변경, 수정 전 대비0.1%p 하락0. 페이지별 TextRun 순서·개수 동일. 개인정보66쪽은 전후 PNG 모두 byte 동일,33쪽99.98985%. 개인정보5쪽 미달 및 devel 대비3/41쪽 하락은 그대로 보류다. Native22건17 PASS/5 FAIL, fresh WASM/최종 전체 회귀/실제 소비 미완료.

13쪽의 외곽선 하단·자형 가장자리 차이는 남는다. 새 px 기대값을 만들거나 부분 결과로 전체 완료를 주장하지 않는다.

## ignored 원장과 로그

아래 생성 JSON·로그·진단은 output에만 보존한다.

- `/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-7752-20261022/repair-focused-v16/result.json`
  - SHA-256: `b821ae9e8fa7150004a0eb7a180d36a58b71cef90e9c665cefc7a33cad6b78bd`.

- `/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-7752-20261022/repair-focused-v16/nextest.log`
  - SHA-256: `c7d48f75912e33bbe5d6d9ecd558120d45e3536f472ede2a4b89781398d98c26`.

- `/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-7752-20261022/first-source-band-native-v16/producer.json`
  - SHA-256: `882c9c14190d6328994ea66573d04bff9f99e4c1bd6fdaa15055db6fb583c1bc`.

- `/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-7752-20261022/first-source-band-native-v16/results.json`
  - SHA-256: `0dfbf5d46a72a0001e444a214e9079fe16e96b419c3ece7ebeca4ecf181b2489`.

- `/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-7752-20261022/first-source-band-native-v16/content-audit.json`
  - SHA-256: `acb114f103be3922eb0bbab5ce355101adee4d8ed0882c44d17659cb0f100e99`.

- `/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-7752-20261022/pii-center-control-native-v16/results.json`
  - SHA-256: `cf046f773ecf1b0378d180c166a5cf2a09e791462f25bff53be2e74c2f6a7089`.

- `/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-7752-20261022/v16-before-after-comparison.json`
  - SHA-256: `0651a57d5be61e3994b4c82b95797c36aae477363a453da260770def3e3f70c6`.

- `/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-7752-20261022/pii-fixed-base-vs-stage45.json`
  - SHA-256: `6fcd34dc3a1350fe776be73bc3d1205983e6a570352a06982268eb33ffa11cac`.

- `/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-7752-20261022/v16-source-consumers/rowbreak.log`
  - SHA-256: `01e9d0893919268f1ee4328e7ab22891be9baed0e707974ca7a7ab3f83c71c2b`.

- `/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-7752-20261022/v16-source-consumers/pii.log`
  - SHA-256: `7175fac4a898877a11716b0c5caad2427283c20d74b1b4e1fae3f7b04dbb9f96`.

## 같은 소스의 기존 실패 재검사

최종 v16에서56건21 PASS/35 FAIL(45.329초). 전체 회귀 수가 아니다.

- `/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-7752-20261022/retry-remaining-native-v16/result.json`
  - SHA-256: `3a6fdc7083ff9a0d01e5ceece79b9e039e25aaeaf0819611263c7ed834709a25`.

- `/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-7752-20261022/retry-remaining-native-v16/nextest.log`
  - SHA-256: `256cea6812c7e3dd7923d671adffe40622e79581566ccb71bb208430dc15b348`.

- `/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-7752-20261022/retry-remaining-native-v16/failure-index.json`
  - SHA-256: `eafa35750d87768a8b2d12041fd88f7cd39b3a425f3efd00973b21933b0d9ee3`.
