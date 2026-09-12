# #7015 / PR #7037 메인터너 검증 보완 Stage 1

## 분석

- 원 기여 head `a3a6cc87a2d1af5315f25dac985d76a50f4a636d`, devel 정렬 head `1077cc6b58d06f9a045cf516684314a1f8c6905e`.
- 현재 제품의 세로 자르기 실물 회귀는 통과했다. 같은 정책의 가로 자르기 및 무효 imgDim이 적응 fallback으로 내려가는 조합은 기존 8개 단위 테스트에서 명시적으로 보호되지 않았다.
- 기존 `test_compute_image_crop_src_offset_top_left`에 축을 바꾼 입력과 폭/높이 0인 기준 크기 입력을 추가한다. 실제 production 함수 호출 결과를 검사하며 source-side test 개수와 제품 코드·baseline은 바꾸지 않는다.
- 기존 head의 전체 nextest 9,489개 통과·46개 skip, crop unit 8개와 실물 회귀 1개, Native Skia lib 4,112개·13개 ignore, PNG 2개·직접 PDF 4개, fmt·Clippy 3종·workspace·WASM build·manifest 검사는 완료했다. 이 결과를 새 보완 테스트가 실행됐다는 증거로 쓰지 않는다.
- 기존 head의 3쪽 visual sweep에서 국민권익위원회 로고가 전체 표시됨을 직접 확인했다. 한컴 PDF 17쪽, rhwp 17쪽. 제목 글꼴·선 색상 차이는 별도 잔여로 구분한다.

## CI 검증과의 관계

- [정렬 후 CI](https://github.com/edwardkim/rhwp/actions/runs/34606501825)는 `current-base-update-merge-tree-green`으로 이전 source CI를 재사용했다. PR 단계 정상 skip은 확인했지만 새 B/C/D duration을 발행하지 않았다.
- 이번 테스트 보완은 기능 범위의 누락된 검증을 보강하는 변경이다. workflow·classifier·보안 조건을 강제로 완화하거나 기존 성공 run을 삭제하지 않는다.
- 원 PR에 이 검증 보완을 반영한 뒤 새 full CI, 정확한 run/attempt의 B/C/D duration 3개, 문서 trailing과 post-merge 재사용·refresh를 확인한다. #6901 종료는 실제 결과 확인 뒤 판단한다.

## 검증 계획

- 변경된 crop unit 8개와 source-side tier, fmt·Clippy 3종·workspace build를 실행한다.
- 기존 실행에 대한 결과를 보존하고 테스트 보완 head의 실제 실행 결과를 아래 별도로 기록한다. 제품 코드는 동일하다.

## 보완 결과

- 반대 축 자르기와 기준 폭/높이 0 입력의 production 함수 결과를 기존 테스트에 추가했다. 변경 후 crop unit 8개, fmt, tier(4205개 불변), native/WASM/workspace all-target Clippy, workspace build, manifest 검사가 모두 통과했다.
- 제품 코드는 정렬 head와 동일하다. 전체 회귀 9489개·Native Skia·WASM·3쪽 시각 대조는 위 정렬 head의 실행 결과이며, 새 test-only 보완의 전체 원격 CI는 push 뒤 확인한다.
- 한컴 기준 PDF: `pdf/30442-acrc-recommendation-business-burden-2020.pdf`, Hwp 2022 / Hancom PDF 1.3.0.550 / PDF 1.6 / 17쪽. MCP 비동기 job `fe1d1209-b3cf-401a-8620-77433213be75`, engine 2020, 455325 bytes, SHA-256 `07e129d80d0f289acfc7b02e2712160e248efff12230c1ed415df29d6e6b064b`.
- 실물 원본 SHA-256 `abcb65b90aae8af209d69df8a4eb658b569559ff05a81e7370620d848bd708e4`.
- 대표 PNG: `mydocs/pr/assets/pr_7037_20260911/logo-p003.png`, SHA-256 `0f0c56a3ca38ca322b7c571c5e96c0657ec32ac9eed21e612a6f89cfe3b5e779`.
- visual sweep 3쪽 1개 대조, flagged=0/1, pixel match=96.93023%, ink/visual proxy=40.41923%. 로고는 전체 표시되며 제목 폰트·선 색상 차이는 남는다. 전체 페이지 fidelity 승인으로 확대하지 않는다.
- 시각 바이너리: `target/pr-review/debug/rhwp`, SHA-256 `0e99a5cf23c96d8858ef84cac8740ddc6c4bc54c6dfa5011827a3590c1568def`.
