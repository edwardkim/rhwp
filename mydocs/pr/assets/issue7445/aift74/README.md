# AIFT 74쪽의 기존 피델리티 미달 이관

[추가 등록 댓글](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-6090366139)의
전74쪽 진단입니다. 이전 댓글은 대표 4쪽만 확인한 결과였으며 전쪽 통과 근거가 아니었습니다.
[원본](aift.hwp), [새 한컴 한 쪽씩 Print PDF](aift-hwp-2020.pdf),
[이전 2022 참고 PDF](aift-2022.pdf), [제거 전 함수](issue_147_removed.rs.txt),
[종전 골든](aift-page3-before.svg)의 바이트와 해시를 보존합니다.

| 출력 | 전체 최저값 | 90% 미만 쪽 | 전체 쪽수·누락 |
| --- | ---: | --- | --- |
| 고정 devel Native/fresh WASM | 18.17911% | 20·21·22·24·25·26·44·45·72 | 74/74·0 |
| 첫 프레임 보정 Native/fresh WASM | 59.56157% | 21·22·24·25·26·44·45·72 | 74/74·0 |

현재 코드에서 새로 생겼던 19쪽 미달은 구현을 먼저 고쳤습니다. 최종 19쪽 92.75015%,
20쪽 93.86002%이며 새 미달 쪽은 없습니다. 첫 조각 높이·빈 이어받기 테두리와 뒤 본문을
직접 확인했습니다. 이 문서 전체가 정상 피델리티를 갖춘 것으로 보고하지 않습니다.
[전쪽 근거와 보정 출처](../completed-20261009/aift-first-frame-validation.json).

활성 검증에서 제거한 함수는 `svg_snapshot::issue_147_aift_page3` 한 개입니다.
전체 SVG 문자열의 fallback font-family 차이를 기대값 갱신으로 숨기지 않습니다.
이 원문의 body-overflow/render-page 원장 행·OVR 자동 프리셋·자동 renderer baseline
항목도 제거했습니다. 비렌더링 IR 왕복 두 행은 논리 이름 `aift.hwp`와 기존 건수를
유지하고 이슈 자산의 실제 입력을 읽습니다. 정상 북마크·쪽번호·목차·문서 경로·CLI 계약과
폰트 Native/WASM parity·gym 진단은 입력 경로만 연결하며 기대값과 기존 검사 수를 유지합니다.
공유 입력이므로 필요한 소비자까지 참조를 점검했습니다. HWPX 대응본은 이번 HWP 검증으로
측정하지 않았으며 `samples/hwpx/aift.hwpx`와 해당 PDF·기존 검사들은 그대로입니다.

[이전/새 경로·제거와 유지 범위](transfer-validation.json)에 검사 결과를 연결합니다.
전쪽 TSV·로그·중간 산출물은 ignored `output/pr-review/7445-completed-20261009/`에 있고
커밋에는 최종 요약·원문/Print·필요한 대표 PNG만 포함합니다.

## 복원 조건

같은 원문·독립 Print PDF로 Native/fresh WASM 전체 74쪽 최저값 90% 이상과 확대 판독을
충족한 뒤, 문단·개체의 소유·순서·누락·중복·포함 관계를 검사하는 회귀로 복원합니다.
이관은 피델리티 해결이 아닙니다. #7445는 열린 상태로 유지하며 이 문서를 현재 PR의
완료 범위 또는 closing issue로 포함하지 않습니다.

## 유지 검사 결과

기존 integration 계약 18건과 쪽번호·문서 경로 unit 계약 10건이 모두 통과했습니다.
폰트 원장 계약 10건도 통과했습니다. fmt 이후 canonical harness를 재동기화해
manifest와 unit-tier policy가 모두 통과했으며 파생 suite는 커밋하지 않습니다.

Gym은 메인터너 감사 모드에서 입력 경로의 재현성만 확인했습니다. 이전 KTX 이관에서
남아 있던 CR29 참조도 같은 자산으로 연결했습니다. security의 기존 공개 표본 계약은
이관 자산을 허용하되 실제 파일과 저장소 내부 경계를 추가 검사합니다. task/check/
allowExits/정답 값은 변경하지 않았습니다. 기존 단위 계약은 3,171건 실행·실패 0·
skip 1, 구조 감사와 oracle/selftest는 모두 성공했습니다. 벤치마크 전수 성적이나
제품 피델리티 통과의 근거로 사용하지 않습니다.
