# 닫힌 빈 호스트 프레임 검증 출처

- 부모 HEAD: `8ae8e82197aaac0e40e1833bec9e5cffaa7501a3`.
- Native CLI SHA-256: `dc4212304848af6e7031abae6ba38fd56acf209c7057fd30b3eec866c2e54d05`.
- Native lib SHA-256: `f6ea95e58498b55bdfebac4e24a8faef83094e7d916780e10b07096fb72df3c6`.
- 변경 파일 SHA-256: `e59e27eab0917c7ed9f505ae9bc63f594782fc7f615aaab5b05732feaf66945e`.
- patch SHA-256: `bcabb74729b3e61c63c4f6b0a1e50172e4d3e7b0f162969a5c58b8d7f966718f`.

## 같은 입력과 조건

Native Print / 96DPI / PDFium152.0.7947.0 smooth RGB / raw2px90%. `--font-path ttfs/opensource --font-path /Users/tsjang/Library/Fonts`,713개 파일 hash 검증. 캡처 총8, Cargo 순차/jobs8/테스트 threads8.

- rowbreak-hwp: compat2024, `rowbreak-problem-pages.hwp` / `rowbreak-problem-pages-hwp-2024.pdf`.
  - 입력 SHA-256: `10b6ab6548610e18c82ba78a1c844a00107fedbb28c195cb05e6fd20626d33ed`.
  - PDF SHA-256: `2c49bda9cc21dc8b93b554d2607da241dd7e894657eb308ed823e7d44654e85c`.

- rowbreak-hwpx: compat2022, `rowbreak-problem-pages.hwpx` / `rowbreak-problem-pages-hwpx-2020.pdf`.
  - 입력 SHA-256: `4ccc321e7eb146f28b768654502a128ba1ae408238fd1ed37d51e963732eb5b0`.
  - PDF SHA-256: `e3dc7bb21707f92e42e83808023bf155f62475fc4eb0e0498709d03986c5afe5`.

## 결과와 보류

전18쪽×2입력×3역할=108쪽. 후보 각각18/18·미달0. HWP 최저93.06876%, HWPX92.71355%. 수정 전 대비0.1%p 하락0. fixed base 대비 HWPX8쪽의0.11821%p 하락은 기존 후보와 같으며 보류 상태다. Native 집중46건36 PASS/10 FAIL, 별도 보존3 PASS(1건 중복). fresh WASM/최종 전체 회귀/실제 소비는 미완료다.

11/12쪽 대표 이미지의 원본은 각 입력의 Native PNG와 독립 PDF 래스터다. 보정된 source 좌표로 이미지를 이동·변형하거나 마스킹하지 않았다.

## 원장과 로그

아래 파일은 ignored output의 생성 결과이며 PR에 포함할 임시 파일이 아니다.

- `/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-7752-20261022/empty-float-native-v13/producer.json`
  - SHA-256: `ad10487bcac3a4d91c87484f3e45c6041608d1beb1192d4e029f5942687d8355`.

- `/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-7752-20261022/empty-float-native-v13/results.json`
  - SHA-256: `0adc53e17e41fdcdf3de16e8cff4d5583f898d135c694d2fa826f515931e7983`.

- `/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-7752-20261022/empty-float-native-v13/comparison.json`
  - SHA-256: `45839c826f5688beed029a82c75a5644916d355ff9dff3c73ee28ba12dd462ef`.

- `/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-7752-20261022/empty-float-native-v13/content-audit.json`
  - SHA-256: `34e1287529ef9b8a88398720f58a3ca45c4a889c94e142873313b5c1a3d3f60a`.

- `/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-7752-20261022/repair-focused-v13/result.json`
  - SHA-256: `02fe87eb17dca727ed5dd6b7128c900a4e1a69ff2dd03a6c626240d2d904e87c`.

- `/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-7752-20261022/repair-focused-v13/nextest.log`
  - SHA-256: `0c87de7a4b42904f723ee647bc18da710a6d92f6eb00444d3ca86dc24710c452`.

- `/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-7752-20261022/preserved-contracts-native-v13/result.json`
  - SHA-256: `e11716e28a10a86b1bf3af2818a15e9d622b49950f0c7374a24817900fe5cfd7`.

- `/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-7752-20261022/rowbreak12-cut-cause/v13-actual/hwpx.log`
  - SHA-256: `22a116c997bef2b714ee04ce04e97dd3be8a5555e85297465beeccfb864490bc`.

- `/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-7752-20261022/v12-caption-fallback-control.json`
  - SHA-256: `0ab6391e240ed5849c0ca788188485b039dafc982974cf69370fa67986f501b1`.
