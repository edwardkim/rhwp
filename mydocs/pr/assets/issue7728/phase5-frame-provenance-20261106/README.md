# #7728 다섯 번째 보정 — 물리 프레임의 성장·정렬·부모 소유

기존 9개 실패를 집중 검사에서 복구한 부분 검증이다. 최종 전체 회귀·fresh WASM·리베이스 기능
보존 검증은 남아 있어 PR과 이슈 종료는 보류한다. 고정 base는 `5716a226b562b7300193dd0b573ded9cd71ac890`이며,
수정 전은 그 위의 후보 `1a4b2bf37bac8e06c980cf5b79dc8fb818f2a9c6`이다. 수정 전 후보를 devel 자체로
표시하지 않는다. 수정 후 소스와 실행 파일은 validation.json (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue7728/phase5-frame-provenance-20261106/validation.json`)의 diff·파일 hash로 고정했다.

## 확인한 원인과 공통 소비

- 자식 문단의 저장 재시작과 완결 행으로 닫힌 개체 프레임은 소유 범위가 다르다. 새 부모 스캐너는
  측정으로 인증한 개체 프레임만 전달받고, 기존 자식 글줄 경계는 기존 소비 경로가 처리한다.
- 원시 행보다 작아진 정규화 상자에 성장한 행용 보완을 적용하면 거대 표 저장본의 첫 행 뒤에
  새 강제 경계가 생겼다. 측정 행 합이 선언 첫 프레임과 정확히 닫히고 원시 합보다 커진 경우에
  보완을 적용한다. 행 유닛 생산과 재귀 소속 조회가 같은 인증을 소비한다.
- `0→0` 저장 재시작의 표식만으로 첫 물리 상자를 인증하면 자식 행의 전진이 늘고 뒤 제목이
  약 6px 내려갔다. 첫 상자 증언과 부모 폭 변경 출처를 함께 조회하도록 했다. 상대 글줄의
  원점 재시작과 무효가 된 옛 개체 프레임을 구분한다.
- 측정 행 원점과 글줄 원점을 직접 대조하는 보완은 위쪽 정렬에서 사용한다. 가운데·아래 정렬은
  지역 원점0을 행 위쪽으로 해석할 수 없으므로 독립 원시 프레임·동료 소유 근거를 사용한다.
  이 차이가 개인정보 라벨 전체를 앞쪽에 넣거나 LPG 본문과 꼬리말을 겹치게 했다.

메타데이터 허용을 전체적으로 되돌린 후보와 저장 호스트 줄의 유무만으로 기각한 후보는
반례와 #7728 개선을 함께 보존하지 못해 채택하지 않았다. 원본·기존 검사·픽셀 기대값은 유지했다.

## 실제 검사

같은 Mac 최소 글꼴 sandbox에서 기존 #4138·#7114·#5846·글자 겹침 반례와 새 #7728 네 검사를
묶었다. 수정 전 **8 PASS / 9 FAIL**, 수정 후 **17 PASS / 0 FAIL**이다. 포맷 반영 소스에서도
다시 17 PASS를 확인했다. 실패 원인은 렌더 트리의 내용 소속·제목 기준선·재열기 쪽 구성·새
겹침이며, 컴파일 실패를 결함 재현으로 세지 않았다. 새 테스트 함수는 추가하지 않았다.
같은 수정 후 소스의 Native Skia 3종은 전체 lib 4,039 PASS/0 FAIL/12 ignore, PNG 2 PASS,
직접 PDF 22 PASS이며 모든 종료 코드는 0이다.

## Native 실제 출력

원본과 독립 Print PDF를 보존하고 canonical Visual Sweep의 Native PNG·PDFium·2px raw로
수정 전후를 각각 새로 출력했다. 전기안전은 전18쪽이며 나머지는 실패의 실제 출력 경계를
집중 대조했다. 각 입력의 전체 쪽수도 PDF와 18·197·66·48쪽으로 일치한다.

| 입력·쪽 | 수정 전 raw | 수정 후 raw | 비교 조건 |
| --- | ---: | ---: | --- |
| 전기안전 원본 전18쪽 최저 | 94.62153% | 94.62153% | 96DPI, compat2022 |
| 거대 표 분할 저장본177쪽 | 36.29070% | 93.17983% | 192DPI, compat2024 |
| 같은 저장본178쪽 | 22.53834% | 100% | 같은 조건 |
| 개인정보 HWPX41쪽 | 96.11992% | 100% | 96DPI, compat2022, 기존 기준 글꼴 환경 |
| 같은 원본42쪽 | 98.98370% | 100% | 같은 조건 |
| LPG 원본2쪽 | 92.44211% | 96.84314% | 96DPI, compat2024 |

전기안전의 모든 페이지 raw가 동일하고, 전후에 새로 출력한 Native PNG18개도 byte-identical이다.
거대 표·개인정보·LPG의 전쪽 수용 판정으로 집중 결과를 확대하지 않는다. 현재 소스의 fresh WASM은
아직 실행 전이다. 이전 단계의 WASM 수치를 새 소스 결과로 인용하지 않는다.

직접 확인한 실제 출력은 다음과 같다.

- 거대 표177쪽: [review](after-native-giant4138-p177-review.png) · [overlay](after-native-giant4138-p177-overlay.png).
  끝 괘선과 뒤 제목의 소속·순서를 복구했다. 세부 글자 위치 차이는 남아 있다.
- 거대 표178쪽: [review](after-native-giant4138-p178-review.png) · [overlay](after-native-giant4138-p178-overlay.png).
  원문 두 그림, 후속 본문과 다음 제목을 보존했다.
- 개인정보41쪽: [review](after-native-pii5846-p041-review.png) · [overlay](after-native-pii5846-p041-overlay.png).
  42쪽 [review](after-native-pii5846-p042-review.png) · [overlay](after-native-pii5846-p042-overlay.png)에서
  비용편익 라벨의 앞·뒤 소속과 이어받기 정렬을 확인했다.
- LPG2쪽: [review](after-native-lpg71352-p002-review.png) · [overlay](after-native-lpg71352-p002-overlay.png).
  새 본문·꼬리말 겹침을 복구했다. 잔여 글자 위치 차이까지 완전 일치로 표시하지 않는다.
- 전기안전6쪽 대조군: [review](after-native-electrical7728-p006-review.png) · [overlay](after-native-electrical7728-p006-overlay.png).

전후 TSV·원본/PDF/실행 파일 hash·실제 명령·종료 코드·Native Skia 3종은 `validation.json`에 기록한다.
로그와 중간 산출물은 ignored `output/open-issues-oldest-20261105/post-rebase-ece9/rebase-5716a226-20261106-115256/`
아래에 보존한다. 전체 nextest·범위 lint/build/manifest/unit-tier·fresh WASM 전쪽·실제 소비 경로·
들어온 PR의 기능 보존과 최종 편람 gate를 완료한 뒤 전체 완료 여부를 판단한다.
