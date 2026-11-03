표 삽입 뒤 본문 흐름, 양식의 논리 편집과 줄 소유, 개체 변환·문단 간격·저장 metadata의 경계 오류를 메인터너 보정으로 해결합니다. semanticist21의 원 PR 10개 head와 기여 commit을 누적 merge history에 보존합니다.

| 원 PR | 최종 동작 |
| --- | --- |
| #8211 | 표 삽입·본문 흐름 |
| #8235 | 양식의 논리 편집·선택 |
| #8281 | 개체만 있는 마지막 줄의 끝 캐럿 |
| #8287 | 다각형 회전·대칭·크기 왕복 |
| #8290 | 문단 간격 편집·되돌리기 |
| #8292 | 중복 표 ID의 HWPX 격자 저장 |
| #8293 | 머리말·꼬리말 마지막 넘침 문단 |
| #8298 | 재조판한 표의 저장 높이 |
| #8299 | 저장 줄 없는 문단의 공간·위치 델타 |
| #8308 | 중첩·분할 셀 hit-test clip |

현행 논리 편집 API·Storage 예산·schema 35·#8517 및 선행 해결 PR의 동작을 유지했습니다. 원격 원 head를 그대로 승인한 결과와 보정된 통합 후보의 검증을 구분합니다. #8281의 기존 두 경계, #8290 뒤 간격, #8308의 기존 가시 배치처럼 이미 해결된 부분은 별도로 기록했습니다.

로컬 검증: stock release-test nextest 12,465 PASS/45 skip, 관련 19모듈 78 PASS, Native Skia 필수 3종·root/WASM/workspace Clippy·workspace build·release build/lib·최적화 WASM·Studio 2,716 PASS를 확인했습니다. 31개 입력·71쪽의 Native/fresh WASM 전쪽 raw 최저는 각각 92.88%이며 쪽수 일치와 확대 판독을 완료했습니다. 실제 53호출·양식/개체/간격/clip API와 Native/WASM 저장본 교차 재열기도 검증했습니다.

기준 devel은 `7d06d5c706f1d89f1662e739abf2b1b25d2a98e4`, 검증 구현 head는 `c6ad3eea59d1b7701efdc4fb582e2d9a44b4f775`입니다. 이후 정리는 증적·문서와 테스트 첫머리의 근거 주석에 한정하며 실행문 동일성을 확인합니다. 일반 release·제품 lint의 동일 source 재사용과 새 head의 전체 실행을 검토 문서에서 구분합니다.

[최종 검증](validation.md) · [원 PR별 검토](../../semanticist21_20261103_review_plan.md) · [원본/독립 Print 출처](../../../../pdf/semanticist21-20261103/print-provenance.md)

merge commit으로 통합해 원 기여 history를 보존합니다.
