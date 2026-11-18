# 원본 글꼴 조건의 우선 보호 검사

현재 후보 `211253376d20aa40c4cf223fdaabd87467e88240`와 고정 base `197ce19e99438c2cba452c654ae1af504dab85ec`에서 nested 8쪽, 네 Helvetica 요청 스타일 대조군 1쪽, sample16 HWP2022 38·43쪽, 영월 서식 1쪽을 먼저 확인했다. Native/fresh WASM, base/후보 합계 20쪽 모두 100.00%이고 공식 보호값 하락이 없다. 평소 원본 글꼴 환경이며 최소 글꼴 회귀 실행과 구분한다.

[실제 TSV·생산자·대표 그림·직접 판독](validation.json)을 함께 보존한다. Native base 바이너리는 실제 `af541589` 빌드이며 제품 의존성이 같은 문서-only base `197ce19e`에 재사용했다. 검증 기준 SHA를 실제 빌드 SHA로 재명명하지 않는다. 2px raw 100%는 픽셀 동일이 아니다.

이 우선 검사는 18조건 1,771쪽의 Native/fresh WASM 전쪽 완료를 대신하지 않는다. 전체 보호 검증은 별도로 이어간다.
