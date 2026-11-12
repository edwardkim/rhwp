# 메인터너 보정 코드 d0cc46573c의 최종 검증 자료

이 디렉터리는 최초 통합 코드 676e3900의 `canonical-visual/` 이력과 구분한 새 코드의 결과다.

Native 주장 22조건·380쪽, 보호 18조건·1,771쪽을 직접 다시 출력했다. 주장 전쪽은90% 이상이며,
보호 TSV의 공식 소수 둘째 자리 표시값 하락은0이다. 필수 `--handbook-gate` 옵션도 같은 frozen
Native에서 실행해383쪽 통과를 확인했다. 명령·프로듀서·쪽별 PNG hash·TSV는
[native-qualification.json](native-qualification.json)에 연결한다.

fresh WASM과 최종 backend 소비 검증은 진행 중이다. 이 Native 기록만으로 전체 준비 완료나
PR 승인을 뜻하지 않는다. 두 기존 악화 원본의 복구/기존 미달은 [보정 근거](../maintainer-repair/cause.md)와
구분하며, 원 주장 샘플 통과를 기존 #8997 전체 해결로 일반화하지 않는다.

기존 첨부 Print는 [실제 PDF 제품 정보](reference-pdf-products.json)의 경로·해시·버전으로 재사용한다.
파일명 `-2020`만으로 실제 출력 제품을 확정하지 않는다. 원 PR이 제공한 한컴2022 Print와
한컴2020 Print를 구분하며, 새 counterexample 두 개는 별도 영수증의 한컴2020 MCP Print다.

반복되는 글꼴 공급 목록은 `font-program-index.json`에 한 번만 보존한다. 각 결과의
`ordered_file_index_runs`는 `[시작 index, 개수]`이며 순서대로 펼치면 원래 `files` 목록과
정확히 같다. 파일 경로·SHA만 담고 글꼴 바이너리·윤곽·폭 테이블은 포함하지 않는다.
