# 과학 시험지의 저장 끝 간격 보정

원본과 기준 PDF는 evidence.json의 기존 정식 경로를 사용한다. 원본·PDF를 바꾸지 않았다.
보정 전5쪽에서 보정 후4쪽으로 독립 Print와 일치한다. Native/fresh WASM 전쪽 TSV를
바이트 그대로 보존했고 최저 raw는 각각 96.42394%/96.32059%다. compat2022·96DPI,
독립 Print 4쪽 전체를 비교했다. Native와 fresh의 source SHA는 증적 커밋만 다르며
생산 소스가 같은 것을 Git diff로 확인했다. 실제 source·입력·PDF·PNG/TSV SHA는
[evidence.json](evidence.json)에 있다.

2쪽의 review와 standalone overlay를 양쪽에서 직접 판독했다. 왼쪽 단 끝의 보기 표와
오른쪽 단의 문항 10~12, 수식 구분선과 후속 내용을 확인했다. raw 일치율을 엄격 RGB
일치율이나 글꼴·획 농도까지 완전히 같다는 뜻으로 보고하지 않는다. 전체 원본 내용·쪽수
검사와 기존 다단 관계/완료 페이지 검사는 기대값 변경 없이 통과했다.

실행 로그와 중간 원장은 ignored output에 보존한다. 이 증거는 전체 Rust·lint·다른 실패
원본의 검증 완료 또는 PR 제출 승인을 대신하지 않는다.
