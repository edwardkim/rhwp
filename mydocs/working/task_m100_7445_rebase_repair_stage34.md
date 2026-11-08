# #7445 보정34 — KoPubWorld 공급 조회와 실제 CLI 회귀 보호

Issue: #7445, 기능 보존 #8064/#7642.

설치한 KoPubWorld SFNT를 전체 family 이름으로 찾지 못해 문서 대체로 바꾸고46쪽을47쪽으로 밀던 오류를 같은 프로그램의 name6 연결로 보정했다. KoPub/World·다른 굵기·Pro는 구분한다. 원본/저장 줄/쪽수/메트릭 표와 임계값은 변경하지 않았다.

[원인·해결 계보·실제 소비 경로·수정 전후 검사·전쪽 TSV·review/overlay·잔여](../pr/assets/issue7445/completed-20261009/stage34-20261021/README.md).

실제 CLI의 두 원본은46쪽으로 복구됐다. Native HWP/HWPX 전46쪽 최저91.43430/90.48362%,fresh 각각94.45835/90.48362%이며 미달0이다.41쪽 네 출력은 모두99.61% 이상과 직접 판독 뒤 기존 ±2px/내용 소유 검사를 유지하며 공급 경로를 보완했다. 기본 None 경로가 통과하더라도 실제 자동 공급47쪽 오류를 놓치지 않게 최종 source에서1FAIL→1PASS를 확인했다. 같은 SFNT 이름 API도48조건과 정상 반례를 보완해1FAIL→1PASS다.

HWP는 base/#8064 대비0.1pp 하락0이지만 **HWPX는 각각11/13쪽의 하락이 남고46쪽 제목은 자간·글리프 잔여가 보인다.** 새 base 두 원본도 전46쪽 Native를 실제 재출력했다. 전체179조건 감사는 frozenV6의 이전 코드 증거이며 현재 소스의 전체 통과가 아니다. WMF·다른 문서·전체 nextest/필수 gate·실제36건을 계속하며 PR 준비 완료로 처리하지 않는다. 지침/remote는 변경하지 않았다.

정식 --prepare 뒤 실제 suite007/002의2검사는 모두PASS(실행0.484초/컴파일 포함213.705초,exit0)다. fmt·manifest/base809·unit-tier/base809·diff check도PASS다.471개는 필터 미선택이며 전체 회귀 결과로 합산하지 않는다. 이번 단계에서는 전체 strict Clippy/Skia·전체 nextest/실제36건을 완료했다고 주장하지 않는다.
