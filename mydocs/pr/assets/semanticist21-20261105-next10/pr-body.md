# semanticist21 PR 10건 통합: 테두리·문자 획·표 편집·각주·계산식

원 기여 commit을 그대로 보존해 다음 10건을 통합합니다. 최신 devel의 공통 편집·Storage 예산·기존 PDF TextPaint·글꼴 shaping·첫 줄 fit·3D 비표시 계약을 유지했습니다.

| 원 PR | 결과 |
| --- | --- |
| [#8368](https://github.com/edwardkim/rhwp/pull/8368) | 원형 점선·단일/이중 물결선 |
| [#8369](https://github.com/edwardkim/rhwp/pull/8369) | 글자 외곽선 여섯 종류 |
| [#8385](https://github.com/edwardkim/rhwp/pull/8385) | 셀 블록 높이·너비 균등화와 재격자 |
| [#8484](https://github.com/edwardkim/rhwp/pull/8484) | LOG/LN·각도·자릿수 ROUND 함수 |
| [#8488](https://github.com/edwardkim/rhwp/pull/8488) | 각주 모양 조회·적용·전파 |
| [#8490](https://github.com/edwardkim/rhwp/pull/8490) | 구역 개요 번호 모양 ID 조회·설정 |
| [#8492](https://github.com/edwardkim/rhwp/pull/8492) | 첫 글자 장식 메타데이터 저장 보존 |
| [#8494](https://github.com/edwardkim/rhwp/pull/8494) | 구역별 미주 번호와 편집 후 갱신 |
| [#8500](https://github.com/edwardkim/rhwp/pull/8500) | 문단 머리 첫 줄 플래그와 저장 |
| [#8504](https://github.com/edwardkim/rhwp/pull/8504) | FORMULA 필드 보존과 명시적 재계산 |

메인테이너 보정은 빈 셀 격자 오류의 원자적 거부, 본문 아래 각주 원점과 참조 메트릭, 바탕쪽 텍스트의 저장 본문 원점, 엄격한 개요 ID 검증, 공통 문자 획 descriptor의 CanvasKit 소비입니다. 계산식은 LOG/LN·ROUND를 실제 한컴 값에 맞추고, 일반 API가 비유한 값을 JSON·셀에 쓰기 전에 정상 오류로 반환하도록 공통 검사에 연결했습니다.

원 CI 실패 3건은 checkout 시간 초과·runner 디스크 부족·기존 WMF 글꼴/그림 회귀였습니다. 원 해결 이력을 추적해 최신 base에서 실제 재검증하며, 이 PR에서 workflow·timeout·검사 기준은 바꾸지 않습니다.

로컬 필수 검증은 최종 소스에서 완료했습니다. 전체 회귀 12,629 PASS·45 SKIP, release lib 3,983 PASS·12 IGNORE, Native 공식 3종 4,064 PASS·12 IGNORE입니다. fmt·Rust lint·workspace build·doc test·TypeScript·Studio 2,720건·build·실제 fresh WASM/Canvas2D/CanvasKit/계산식 대화창 검사도 통과했습니다. 아래 기록의 producer를 새로 출력하지 않은 head로 바꾸어 표시하지 않습니다.

- 이번 10건: 동일 입력·독립 한컴 Print·96 DPI 전 124쪽, Native 최저 96.49% / fresh WASM 97.10%. 90% 미만·누락·쪽수 불일치·2%p 초과 하락 없음.
- 최신 devel #8622의 추가 39 PR 보존: 80상태·406쪽, Native 최저 92.45% / WASM 92.99%, 경로별 403쪽 PNG 동일. 달라진 3쪽도 직접 확대 대조했고 점수 하락 없음.
- Native canonical 편람 383쪽 gate 통과: 최저 99.96%, 370쪽 100%.
- 위 시각 producer는 `d53115fabc92bd392044d9b906c599baddb4368d`. 마지막 계산 API 보정은 `584c9503baa651c62d0403dad9cb77c5cb9ffde6`이며 renderer/paint/parser/model/serializer/Studio/입력 tree 불변과 영향 계산식 전쪽·실제 API·대화창 검증을 별도로 기록합니다.

[개별 검토와 원 기여/보정 구분](../../semanticist21_20261105_next10_review.md), [실행 결과·전쪽 TSV·직접 판독·입력/Print/글꼴 해시](validation.md)를 연결합니다. 제출 시 링크는 제출 후보 SHA로 고정합니다.

FORMULA는 명시적 재계산 API 범위이며 기존 계산식 대화창의 일반 값 쓰기와 구분합니다. 첫 글자 장식은 메타데이터 저장 보존 범위입니다. 새 생성 UI·타이핑 자동 재계산·장식 자동 재배치를 구현한 것으로 설명하지 않습니다. 원 contributor fork branch는 변경하지 않았습니다.

현재 base `a95d1d5afad1bfd61998794fa78fb0db100d690a`와의 실제 병합 트리도 별도 검증했습니다. 원 코드 head는 유지했으며 Native/최적화 WASM의 이번 124쪽씩은 PNG 248개가 기존 후보와 같고, 새 글상자 편집의 3·4쪽은 양쪽 100%입니다. 집중 Rust 18건·실제 클릭/입력/Esc·API·Studio 2,721건이 성공했습니다. [검증한 source와 범위](current-base-compatibility.md)를 구분합니다.
