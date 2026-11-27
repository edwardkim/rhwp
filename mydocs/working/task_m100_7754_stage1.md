# 9개 이관 이슈의 원본과 독립 Print 근거 복원

Issue: #7754, #7755, #7756, #7757, #7759, #7760, #7761, #7762, #7765.

작업 브랜치는 `task/7754-nine-fidelity-issues`이며 시작 base는
`d7b5286cbf6401678a70eed3d66236a68dd2a72c`다. 구현·PR·리베이스·병합 후 후속 처리는
사용자가 승인했다. [계획](../plans/task_m100_7754.md)의 전체 범위를 유지한다.

## 원본과 선행 변경 확인

9개 이슈의 본문과 코멘트를 읽었다. 과거 이관이 출력 해결을 뜻하지 않는다는 조건과,
각 원본·기준 PDF·삭제된 함수/핀 범위를 확인했다. 이슈 본문의 보정 SHA와 현재 이력의
SHA가 달라 Git에서 실제 반영된 대응 커밋을 추적했다. 아래 커밋 모두 시작 base의 조상이다.

| 이슈 | 현재 이력의 이관 커밋 | 확인한 계약 |
| --- | --- | --- |
| #7754 | `8603c88e0d` | 암호 HWPX의 과거 23쪽 고정과 정상 암호·보안 계약의 구분 |
| #7755 | `396662b157` | 시장구조조사의 전체 쪽수·행 분할과 저장 줄 되감김 보호 범위 |
| #7756 | `9da4787def` | 워터마크 layer 검사와 전체 SVG snapshot의 실제 삭제 범위 |
| #7757 | `dda9cab144` | 단위 문구와 oversized TAC 표의 줄 소속·선행 inset |
| #7759 | `8b0a0777ca` | 공개 HWP의 실제 렌더 런 수·문자 수·해시 핀 |
| #7760 | `a285e5f4be` | 수출입 HWP의 해시 핀과 보존한 WMF·프로필 계약 |
| #7761 | `d586cff463` | 온새미로 HWP의 해시·상대 크기 100% 계산 근거 |
| #7762 | `9e22208799` | 대응 HWPX의 별도 해시·substFont 계약 |
| #7765 | `8c64e94661` | 86712의 표 이어받기와 후속 표 경계 |

#7765는 이후 `3e02fe2749`와 `7f76cee60e` 등에서 일부 줄 구성·프레임 회귀가
복원됐다. 현재 `issue_7765_current_source_frames.rs`와 해당 전쪽 증적도 확인했으며,
그 과거 측정값을 이번 head의 전쪽 검증으로 재사용하지 않는다. 다른 이슈의 기존 검사도
각 원본 경로를 따라 추가 확인하고 최종 복원 범위를 정한다.

## 독립 Print 출처 보정

시장구조조사·복학원서·온새미로 HWP의 기존 PDF는 `Creator: Hwp 2022`였고,
배포용 문서의 기존 2024 PDF는 `Microsoft: Print To PDF`였다. 같은 원본을 MCP의
실제 한컴 2024에서 PrintMethod 0으로 새로 인쇄했다. 완료한 출력은 다음과 같다.

| 원본 | 새 독립 Print | 쪽수 |
| --- | --- | ---: |
| 시장구조조사 HWP | [2024 Print](../../pdf/issue7755/market-2017-original-print-2024.pdf) | 315 |
| 복학원서 HWP | [2024 Print](../../pdf/issue7756/bokhak-original-print-2024.pdf) | 1 |
| 온새미로 HWP | [2024 Print](../../pdf/issue7761/onsaemiro-hwp-original-print-2024.pdf) | 46 |
| 86712 구판 변환 HWPX | [2024 Print](../../pdf/issue7765/regulatory-86712-hwpx-original-print-2024.pdf) | 64 |

배포용 `distribution_doc.hwpx`는 실제로 2018 저장 HWP5 CFB다. 확장자만 보고 시작한
2024 요청은 `PrintToPDFEx action creation failed`로 종료됐다. 파일을 ZIP으로 여는
진단도 실패했다. 기존 [정상 Print 기록](../../pdf/issue7642-failed-regression-print-20261015/README.md)에서
같은 SHA의 `.hwp` 전송 별칭과 한컴 2020 Print 3쪽을 확인했다. 원본이나 저장 줄을
바꾸지 않고 이 [PDF](../../pdf/issue7642-failed-regression-print-20261015/distribution_doc-2020.pdf)를
사용하며 compat는 2022다. PDF와 입력 SHA도 실제 바이트에서 재확인했다.

복학원서는 한컴 2020에서도 [추가 Print](../../pdf/issue7756/bokhak-original-print-2020.pdf)를
확보했다. 새 2020·2024 Print에서 작은 PCX 로고가 보이지 않으며 Native 출력에는 보인다.
2024 Print의 영문 서체 차이도 직접 확인했다. 원본의 PCX, 원본 그림 속성 및 독립 출력의
코덱 계약을 조사하며, 그림을 임의로 숨기거나 이 차이를 글꼴 차이로만 분류하지 않는다.

실제 입력·PDF SHA, job, engine, Hancom runtime, PrintMethod·출력 모드와 base의
선행 커밋은 [출처 기록](../pr/assets/issues7754_7765/input-provenance.json)에 보존했다.
새 Print를 현재 렌더러와 일치시킨 자료나 회귀 기대값으로 주장하지 않는다.

## 현재 검증과 다음 단계

시작 base의 Native Skia CLI 빌드는 exit 0으로 완료했다. Native binary SHA-256은
`91dc23f0a85d17485765b6452b463908aecd43b27c771e0db689a80e8aa87052`다.
전쪽 Native 진단과 같은 소스의 최적화 fresh WASM 빌드는 진행 중이다. 초기 Native
결과만으로 두 경로의 검증 완료나 회귀 등록 자격을 선언하지 않는다.

추가 사용자 지시에 따라 새 회귀와 복원 회귀는 같은 원본의 검증 범위에서 Native와
fresh WASM 모두 98% 이상인 경우만 지정한다. 가능한 쪽은 100%로 개선한다.
소유·내용 순서·보존·겹침의 관계 검증과 수식·미세 간격 확대 판독도 계속 적용한다.
현재 단계는 자료·선행 변경의 확인이며 생산 코드와 테스트·기대값은 아직 바꾸지 않았다.

다음 단계에서 전쪽 TSV와 직접 판독을 기준으로 공통 원인을 보정한다. 암호 원본의
실제 출력은 평문 대응본의 점수와 구분해 실행한다. #7765의 HWP와 HWPX도 각각 검증한다.
필수 회귀·lint·최적화 WASM·브라우저·편람·보호 TSV와 최신 head CI·merge 후 확인을
모두 마치기 전에는 9개 이슈 해결이나 PR 제출 완료로 보고하지 않는다.
