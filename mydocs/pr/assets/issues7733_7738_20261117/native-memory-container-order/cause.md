# macOS 메모리 글꼴 공급의 원본 컨테이너 순서 조사

리베이스로 반영한 #9157의 원 해결 커밋은 `fcf2e11b1727297743343446333132e1409737f1`이며, 검사 이관 커밋은 `699c0afbbc86db1252128b8dcc09630038a8d971`이다. 같은 family와 CSS style에서는 원본 glyf TTF → glyf TTC face → CFF OTF 순서를 유지한다. 이름·확장자만으로 원본 컨테이너를 추정하지 않는다.

기존 Linux Native CI의 24 PASS는 [원 PR 기록](../../issue_8371/essential-validation-20261117/native-ci-registration/native-ci-job-excerpt.json)에 있다. 현재 Mac의 최소 글꼴 sandbox에서는 다음 결과를 실제 확인했다. 별도 사용자 글꼴을 요구하지 않는 저장소의 공개 합성 TTF/TTC fixture를 사용했다.

| 생산 코드 | Native 검사 | 결과 |
| --- | --- | --- |
| base `af54158960ec91504d42faf842b0cb3229366236` | `skia_native_glyph_replay` 24개 | 23 PASS / 1 FAIL, 실행 2.058초 |
| 후보 `da4034a7d38b38c356f5e7386cff4674918ec15f` | 같은 24개 | 23 PASS / 1 FAIL, 실행 1.967초 |

실패는 두 실행 모두 `native_supplied_bytes_prefer_ttf_over_nonzero_ttc_face_in_both_orders`다. 파일 경로로 공급하는 자매 검사는 통과했다. 후보의 실패 RGBA는 TTF 단독 출력과 3,462픽셀이 다르며, AAAA의 전체 잉크 폭이 약 절반인 TTC 윤곽을 보인다. 단순 안티앨리어싱 차이로 판정하지 않는다.

후속 문서 PR #9171의 base `197ce19e99438c2cba452c654ae1af504dab85ec`와 후보 `874a2ddc68cc9c63544747059cb347018c06d8cc`는 각각 직전 생산 코드와 제품 소스가 동일하다. 기존 실행의 실제 producer를 바꾸어 적지 않고 동일 소스 대조 근거로 재사용한다.

## 확정 원인과 보정 경로

[원본 fixture 메타데이터](fixture-metadata.json)의 두 대상 face는 family·weight 400·width 5가 같고 A의 전진 폭만 608/304로 다르다. [Mac 실제 추적](macos-style-trace.txt)에서 같은 TTF를 재적재한 마지막 face만 Skia가 weight 58로 보고했다. 기존 CSS 스타일 정렬이 이 OS 값을 먼저 소비하여 TTC weight 400을 원본 TTF보다 앞세웠다. 컨테이너 우선순위 자체는 각각 0/1로 보존돼 있었다.

`TypefaceCatalog::insert_face`에서 정적 glyf 프로그램의 원본 OS/2 스타일을 보존한다. catalog 후보 정렬과 최종 family 체인의 정렬이 같은 보존 값을 소비한다. CFF와 가변 글꼴의 기존 OS 스타일·축 해석은 유지하며, OS/2가 없는 입력에는 임의의 Regular를 만들지 않는다. 문서명·실패 페이지·특정 weight 58에 대한 예외는 추가하지 않는다.

기대 RGBA 동등성은 유지한다. 실패 메시지만 전체 바이트 배열 대신 공급 순서·길이·첫 불일치 위치로 요약한다. 기존 두 컨테이너 검사와 나머지 Native 검사 22개를 같은 최소 글꼴 조건에서 다시 실행하여 **24 PASS / 0 FAIL, 1.391초**를 확인했다. 선택 밖 325개는 실행하지 않았으며 전체 회귀 완료로 세지 않는다. 최종 전쪽·fresh WASM·보호 검증은 별도로 진행한다.
