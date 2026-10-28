# 표 뒤 호스트 줄의 실제 전진과 하단 여백 복구

- 고정 base: `605ccfd92f2f12f18e14182548d729f9929fda00`.
- 후보 출처: `b258cc9d1b628c26e6a29e0effa49f37f6ef388f` + 58단계 공통 흐름 보정.
- 입력: `tests/fixtures/issue6923/148738070_wrapper_table_stored_page_frame.hwp`.
- 독립 기준: 같은 경로의 `148738070_wrapper_table_stored_page_frame-2020.pdf`, 7쪽.
- 조건: Print, compat 2022, 96DPI, raw 2px 실루엣, 동일 로컬 글꼴 공급, 캡처 총8.
- Native SHA-256: `b2ff55e34828de823f7bf14ed1729b5e6113eb908cfc6d28ecdfc740e5bfd3fe`.
- fresh WASM SHA-256: `2168c36a2e5b19fbbf65039b5d2c060a6b11be93f664f6634e009bd953d38da7`.
- Mac locked web/no-opt 로컬 대체 빌드다. Docker 최적화 통과를 뜻하지 않는다.

## 결과

| 구분 | 전쪽 최저 | 3쪽 raw | 쪽수 | 미달 |
| --- | ---: | ---: | --- | ---: |
| 변경 전 Native | 84.62310% | 84.62310% | 7/7 | 1 |
| 변경 전 WASM | 84.62378% | 84.62378% | 7/7 | 1 |
| 변경 후 Native | 98.11680% | 98.68155% | 7/7 | 0 |
| 변경 후 fresh WASM | 98.22942% | 98.68225% | 7/7 | 0 |

Native는 3쪽 PNG만 바뀌고 다른6쪽의 SHA는 동일하다. 전7쪽 TextRun 원문과 순서도 동일하다.
Native/fresh WASM 3쪽 review와 overlay를 직접 판독해 중첩 표 뒤 본문·하단선의 복원을
확인했다. 남아 있는 글꼴 획 차이까지 해결했다는 주장은 아니다. 기존 하단선 검사를 포함한
#5585 세 검사가 통과하며, #4138 분할/자식 소유도 보존했다. 집중14건 중13 PASS·1 FAIL이고
실패는 별도 기존 #5966의143/142쪽 불일치다. 이 자산은 전체 PR 검증 완료가 아니다.

## 직접 판독 자료

| 출력 | review | overlay |
| --- | --- | --- |
| 변경 전 Native | [review](before-native-p003-review.png) | [overlay](before-native-p003-overlay.png) |
| 변경 후 Native | [review](after-native-p003-review.png) | [overlay](after-native-p003-overlay.png) |
| 변경 후 fresh WASM | [review](after-wasm-p003-review.png) | [overlay](after-wasm-p003-overlay.png) |

전쪽 TSV는 각 실행 원본을 그대로 복사했다. source·원본·PDF·빌드·글꼴 공급·페이지별 PNG 해시,
로그 및 변경 전후 비교는 ignored `rebase-8155-20261022/stage58-host-flow-fix/`와
`stage58-wrapper-frame/`에 보존한다. 글꼴 파일과 중간 실행 로그는 이 자산에 포함하지 않는다.
