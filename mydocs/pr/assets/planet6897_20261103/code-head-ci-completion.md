# 통합 PR #8622의 code head CI 완료 기록

검증 code head는 `acbc0bfeb5660133b0732ce77eae16bd50b29f9f`, base는 `d4a72efd7c91d1e25092ed30b9d44e43cbebb9f0`다.
2026년 11월 5일 KST에 정확한 head의 다음 workflow 5개와 check 38개(33 success·5 정상 skipped)를 확인했다.
후속 CI Impact Policy status도 성공했다. 이 기록 뒤의 문서-only trailing head는 별도로 검사한다.

| 실행 | 실제 결과 |
| --- | --- |
| [CI](https://github.com/edwardkim/rhwp/actions/runs/41965861415) | success · Rust archive A/B/C/D/E·Native Skia·lint·frontend package |
| [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/41965861678) | success · 별도 신규 보안 검사도 success |
| [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/41965861401) | success |
| [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/41965861459) | success |
| [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/41965861426) | success |
| [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/41968966442) | success · 별도 status |

## CodeQL 오탐 10건의 승인된 처리

경고 #462·#466·#470·#472·#474·#475·#476·#497·#498·#499의 SARIF source→sink와 실제 format 인수를 대조했다.
`from_bytes_with_password`가 반환하는 DocumentCore 전체에 taint가 전파됐지만 지목된 opt-in 진단은
인덱스·기하 수치·bool/enum·숫자 컷·배치 구조체·포인터만 출력하며 암호·원문·복호화 바이트를 출력하지 않는다.
작업지시자가 이 10건만 승인한 뒤 `false positive`로 dismiss했고 state·reason·comment를 API로 재확인했다.
[신규 보안 검사](https://github.com/edwardkim/rhwp/runs/127044322863)는 success·새 경고 0건이다.
기존 별도 경고 #478·#483·#484는 변경하지 않았다. source·보안 규칙·테스트 기대값·CI 허용치를 바꾸지 않았다.

## 로컬·시각 완료 증거

[필수 41개 실행](rebase8609/rust-regression-validation.md), [입력 바이트](input-integrity.tsv),
[직접 판독](direct-reading-current.md), [새 base Native](rebase8609/native-base-preservation.md)·[fresh WASM](rebase8609/wasm-base-preservation.md),
[보호 PR Native](current-c183a1ac61/pr8389_8397/native-validation.md)·[fresh WASM](current-c183a1ac61/pr8389_8397/wasm-validation.md)를 연결한다.
CI 성공 뒤 문서만 바꾸므로 전체 회귀·WASM 빌드·전쪽 TSV 산출을 반복하지 않았다.
