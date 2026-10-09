# KTX 27쪽 문서의 기존 목차 미달 이관

[기존 #7445 등록](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-6063592661)에
동일 문서·증상으로 연결합니다. [원문](KTX.hwp), [새 한컴 Print PDF](KTX-hwp-2020.pdf),
[이전 2022 참고 PDF](KTX-2022.pdf), [전체 비교와 이관 범위](transfer-validation.json)를 보존합니다.

| 출력 | 2쪽 | 26쪽 | 전체 쪽수 |
| --- | ---: | ---: | --- |
| 고정 devel Native/fresh | 50.35805% | 71.16440% | 27/27 |
| 현재 Native | 50.35805% | 73.09399% | 27/27 |
| 현재 fresh WASM | 50.35805% | 72.83401% | 27/27 |

나머지 쪽은 모두 90% 이상이며 누락은 없습니다. [2쪽 review](current-p002-review.png)와
[26쪽 review](current-p026-review.png)를 직접 판독했습니다. 이관은 출력 개선이나 승인 완료가 아닙니다.

현재 실패한 `svg_snapshot::issue_267_ktx_toc_page` 한 함수와 해당 렌더 corpus 두 행만
제거했습니다. [제거 전 함수](issue_267_removed.rs.txt)와 [종전 골든](ktx-toc-page-before.svg)은
자동 회귀 수집 밖에 보존합니다. OVR 자동 프리셋의 해당 입력도 제거했습니다. 같은 원문의 기존 정상 탭·리더·page-hide·hit parity와
IR 왕복 계약은 이관 자산을 사용해 유지하며 기대값은 변경하지 않았습니다. 새 회귀 함수는 없습니다.

동명 `samples/basic/KTX.hwp`는 별도 1쪽 입력입니다. 새 PDF를 그 정본 경로에 저장한 오류는
기존 basic PDF 바이트를 복원해 정정했습니다. 이 문서의 새 정본은 위의 27쪽 파일입니다.

유지 검사 16건은 canonical harness에서 모두 통과했습니다(42.326초, exit 0).

## 복원 조건

같은 원문·독립 Print PDF로 Native/fresh 전체 27쪽 최저값이 90% 이상이고, 목차·외곽선과
표 뒤 본문의 위치·간격을 직접 확인한 뒤 관계형 회귀로 복원합니다. #7445는 열린 상태로 유지합니다.
