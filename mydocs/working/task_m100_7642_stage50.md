# 이슈 #7642 Stage 50 — 미주 전진폭을 보존하는 그림 앵커 조회

Issue: #7642

## 독립 근거와 원인

시작 head는 `5d93d6e27`이며 실제 제품 source는 `c1cd0367f`다.
2024년 9월 구분선 아래/위 20 HWP·HWPX의 한컴 Print 8쪽은 구 그림이
본문의 앞 여섯 줄 뒤에 있다. 기존 Native는 문단 시작에 그림을 놓아 본문을
처음부터 좁힌다. 이전 source의 해당 쪽은 약 72.48%다.

`stored_picture_anchor_delta → 저장 배제 차선의 유일한 후보 조회 →
전폭 frame 조회`에서 배제 차선 후보가 유일하지 않고, 전폭 frame도 원본의
Endnote를 거절한다. 호출자는 `None`을 0으로 바꿔 문단 시작을 원점으로 쓴다.
저장 줄의 좁아짐만으로 그림의 소유를 추측해서는 안 된다.

## 구현 계약과 소비 경로

1. `endnote_prefix.rs`가 만든 실제 표시 run·언어별 글꼴·0.75 크기·전진폭을
   공통 결과로 발행한다. 기존 paint는 그 run과 폭을 사용하고, 앵커 조회의
   첫 줄 채움도 같은 전진폭을 예약한다. 별도 표시 노드도 본문 원점을 전진시킨다.
2. 일반 body frame과 그림 앵커 조회의 목적을 명시적으로 구분한다. 후자는
   원본 선두 Endnote prefix와 이미 지원하는 scalar/TAC 수식·그림만 수용한다.
   중간 Endnote·Footnote·표 및 다른 nested flow를 전역으로 수용하지 않는다.
3. 원본 control 목록·UTF-16 offset·문자 순서를 보존한다. 마커를 다른 control로
   치환하거나 원문 문자를 삭제하지 않는다. 앞 번호의 점유를 첫 scalar의
   기존 inline control 채움에 연결한다.
4. 공통 frame의 줄 구성 결과에서 원본 그림 control의 소유 줄을 찾고 물리
   원점을 배치에 전달한다. 커닝 실패 재시도에서도 조회 목적을 보존한다.

실제 연결은 `prefix geometry → frame의 inline 전진 → raw control 소유 줄 →
stored_picture_anchor_delta → 그림 배제·paint 원점`이다. 문서 ID·원래 실패
그림 번호·절대 px 위치로 분기를 만들지 않는다.

## 검증 범위와 현재 판정

같은 HWP/HWPX 8쪽을 독립 Print와 직접 비교하고, 해당 문서의 14/19쪽을
대조한다. 2022 HWPX 18/19쪽·2023 HWP/HWPX 19쪽·정상 한컴 저장 수식
진단은 정상 대조군으로 유지한다. 기존 normal body frame의 nested-flow
거절 검사는 기대값을 변경하지 않는다.

그림 문제의 보정과 뒤 페이지의 미주 이어받기 미달을 구분한다. 전체 원본의
Native/fresh WASM 90% 및 확대 판독 전에는 새 회귀나 px 검사 전환을 완료로
보고하지 않는다. 구현·검증 진행 중이다.

## 구현 중 타입·단위 점검

첫 컴파일은 공통 run의 글자 모양 ID를 u16으로 선언해 E0308로 실패했다.
기존 model/paint와 같은 u32로 수정했고 실패 로그·provenance는 ignored output에
보존한다. 폭 소비 지점에서도 기존 `/75`가 96dpi를 고정하고 있어 스타일의
`resolved_dpi`와 공통 `hwpunit_to_px`를 사용하도록 연결했다. 토큰화·토큰 내부
분할·커닝/수평 투영이 같은 폭 helper를 소비한다. 기본 96dpi 계약을 임의 비율로
바꾸지 않으며 다른 배율은 별도 대조한다.

## Native 재검증 결과

release-test 빌드는 472.91초, exit 0이며 source 1454개 파일(새 모듈 포함)의
해시가 빌드 전후 일치한다. frozen binary SHA-256은
`4aaa46dbe7c7c64f7191b674b8103174baba846ff18259f2026d85b869e754d9`다.
포맷 검사·Native Clippy `-- -D warnings`는 각각 20.39초·114.07초, exit 0이다.

| 원본·독립 Print | 변경 전 Native | 변경 후 Native | 판정 |
| --- | --- | --- | --- |
| 2024년 9월 아래/위 20 HWP 8쪽 | 72.47311% | 98.33020% | 해당 쪽 충족, 직접 판독 |
| 같은 HWPX 8쪽 | 72.46204% | 98.33234% | 해당 쪽 충족, 직접 판독 |
| 같은 HWP/HWPX 14쪽 | 약 35.83% | 35.79448% | 미충족, 이어받기 원인 보정 계속 |
| 같은 HWP/HWPX 19쪽 | 약 33.78% | 33.74395% | 미충족, 같은 후속 범위 |
| 2022 HWPX 18/19쪽 | 97.60151% / 97.76191% | 동일 | 정상 대조군 |
| 2023 HWP/HWPX 19쪽 | 95.94555% / 96.05956% | 동일 | 정상 대조군 |

2024년 11월 9/8/7 10·11쪽의 최저값은 97.83912%, 11월 구분선 없음
HWPX 11쪽은 97.87953%, 2022년 11월 HWPX 4쪽은 98.73016%다. 이들은
선택 페이지 대조군이며 전체 문서 통과로 확대하지 않는다.

실제 조회 로그는 prefix 전진 28.4800, 높이 9.0000, 기준선 7.6500을
같은 producer에서 소비했다. 원본 그림 control의 raw 위치를 전폭 결과의
`owner_start=313`, `owner_vpos=9302`, `first_vpos=0`에 연결한다. 그림의
실제 상단은 105.3에서 229.3으로 이동했고 독립 Print는 229.0293이다.
이 값은 진단 관측값이며 코드·회귀의 고정 px 기대값으로 쓰지 않는다.
새 8쪽 review를 사용자에게 표시하고 앞 여섯 줄·그림·뒤 문항을 직접 대조했다.
증적은 ignored `picture-anchor-prefix-before-native/`,
`picture-anchor-prefix-focused-native/`와 각 실행 로그에 보존한다.

## 전쪽 조사와 공간 정리

이전 제품 source `940c1802f`의 55개 원본·853쪽 Native 조사가 완료됐다.
30개 문서는 전쪽 수치 90% 이상, 25개 문서는 미달이다. 누락 쪽·쪽수 불일치는
없다. 이 결과는 변경 전의 분류이며 최신 head 통과나 확대 승인으로 바꾸지
않는다. 최신 보정으로 이미 개선한 페이지와 추가 미달 페이지를 구분해
`stage50-before-whole-native-summary.json`에 연결했다.

종료한 Stage49 fresh WASM의 재산출 가능한 SVG 132개, 약 10.50GiB를 정리했다.
원본·Print PDF·PNG·TSV·JSON·로그·frozen package와 파일별 해시 원장은
보존했다. 공유 Cargo target과 실행 중인 조사 산출물은 삭제하지 않았다.

fresh WASM 재검증과 다른 배율의 실행 대조는 아직 미검증이다. 미달 후속
페이지와 전체 원본의 보정·관계형 회귀 전환이 남아 있으므로 완료로 판정하지 않는다.

전체 조사와 압축 helper가 모두 종료됐음을 PID로 확인한 뒤, 이전 source의
SVG 압축 중간물 55개도 기존 검증 해시 원장을 보존하고 제거했다. 추가
32.85GiB를 확보했고 대응 853쪽 PNG·TSV·로그·원본·PDF·frozen binary를
보존했다. 삭제 원장은 ignored `stage50-completed-whole-native-archive-cleanup.json`이다.
