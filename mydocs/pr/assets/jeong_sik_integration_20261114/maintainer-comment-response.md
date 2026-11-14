# 메인터너 댓글 반영 원장

원 PR별 메인터너 지적을 구현·독립 Print·회귀·실제 소비 경로로 대조했다. 최종 결과와 producer는 [검증 원장](validation.json) 및 [재사용 감사](rebase-reuse-audit.json)에 연결한다. 원격 통합·원 PR close는 정확한 최종 head CI 뒤에 수행한다.

| 원 PR의 지적 | 반영 방법 | 현재 상태 |
| --- | --- | --- |
| [#7668](https://github.com/edwardkim/rhwp/pull/7668#issuecomment-6068531985): 조판 프레임과 내부 그림 기하 분리 | 줄·셀 측정은 전체 common 프레임을 사용한다. paint에는 별도 흐름 프레임을 전달하고 current/orgSz·imgRect·crop·변환을 재작성하지 않는다. | 구현 반영, 33종 양쪽 시각·전체 회귀·실제 35출력 PASS |
| 같은 프레임·같은 픽셀이어도 두 번째 그림의 작은 내부 표시 영역 유지 | 공개 첨부의 `img_cmp_01.hwp/.hwpx`를 원본 바이트로 보존한다. 같은 입력을 한컴 2020 Desktop Print로 재출력해 대조한다. | 입력·독립 Print 확보, 양쪽 100%, 실제 Canvas2D/CanvasKit/SVG/Native 확인 |
| 겹침 금지 본문 그림의 개별 배치와 전체 높이 예약 | 저장 빈 호스트가 전체 프레임의 합을 닫고, 같은 앵커·겹침 금지·여백 조건을 만족하는 경우에 각 그림의 원점을 발행한다. 일반 float의 무조건 합산은 하지 않는다. | Native/fresh WASM에서 HWP/HWPX 각 100%, 2개체의 별도 표시 확인 |
| [#7675](https://github.com/edwardkim/rhwp/pull/7675#issuecomment-6068541361): 들여쓰기·가용 폭·같은 줄/너비 부족 줄바꿈 | 기존 필드·비흐름 앵커와 저장소 오류 전파를 유지하면서 줄별 가용 폭을 적용한다. 그림 내부 번호의 실제 표시도 직접 판독한다. | 원 공개 12종 Native/fresh WASM 각 100%, 줄 소속 관계 검사 PASS |
| [#7682](https://github.com/edwardkim/rhwp/pull/7682#issuecomment-6068546922): 문단별 advance·소속·뒤 간격 | 여러 문단의 그림 소속과 마지막 줄간격·문단 간격을 공유 프레임에서 소비한다. 빈 문단·저장 줄·분할 경로의 적용 여부는 따로 확인한다. | 원 공개 10종 Native/fresh WASM 각 100%, 기존 분할 경로 포함 전체 13,477건 PASS |
| 전체 점수만으로 독립 표시 누락을 면제하지 않음 | 원 공개 31종과 별도 메인터너 HWP/HWPX 대조군 2종을 분리한다. 숫자 비트맵·객체 소속·겹침·후속 내용의 관계를 review/overlay와 함께 검사한다. | 공개 31종 최저 99.97213%, 별도 대조군 각 100%, 신규 관계·파일 형식 검사 7건 PASS |
| 정확한 제출 head와 독립 Print 증거 | 원 PR의 WebHwp PNG를 Desktop Print 기준으로 사용하지 않는다. 한컴 2020 11.0.0.9136·Print method 0 PDF와 실제 입력 해시를 연결한다. | 31종 및 대조군 2종 Print 확보 |
| 기존 기능 보존 | 정본 보호 TSV 18개 문서·Native/fresh WASM 3,542행의 공식 두 자리 표시값을 대조하고 필수 편람 gate를 실행한다. | Native/fresh WASM 각 18문서·1,771쪽, 총 3,542행 하락 0; 필수 편람 gate PASS |

과거 댓글은 당시 누적 head `9f2556c092a76fc66eaa123480ffaeedaeaed22c`의 관측이다. 이를 각 원 PR의 신규 회귀나 현재 devel의 동일 상태로 단정하지 않고 이번 고정 base와 후보에서 재현한다. 당시 첨부 PDF의 호출 방법은 확인되지 않았으므로 최종 기준은 새 Desktop Print다. Mac fresh WASM은 승인된 locked wrapper의 `--no-opt` 로컬 대체 빌드를 사용하며 Docker 빌드 성공으로 표시하지 않는다.
