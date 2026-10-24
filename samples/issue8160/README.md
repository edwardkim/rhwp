# #8160 독립 조판·인쇄 계약 입력

공통 로직의 근거로 사용한 독립 HWPX/HWP 입력이다. 인쇄 전달·장치 격자·WMF 창과
글자 Dx·서로 다른 고정 pitch family·가상 굵기·색 채널·기울임·리더·원본 개체 순서를
진단하도록 생성했다. 특정 sample16 문구나 문서 ID는 입력 조건에 쓰지 않았다.

대응 MCP Print PDF는 `pdf/issue8160/`에 보존한다. 실제 파일·SHA·기존 근거 JSON과
MCP 실행 신원은 `mydocs/pr/assets/issue_8160/independent-inputs.json`을 따른다.
이 입력들은 독립 규칙을 확인한 진단 증거이며 모든 쪽·모든 backend의 완전 일치를
기대하는 golden으로 등록하지 않는다. 예를 들어 두 family의 원본 TTC 공급 비교는
글자 영역 미대응 0을 확인했으나 기존 삼각형 stroke의 차이는 별도로 남아 있다.
설치본의 라이선스가 있는 글꼴 프로그램을 복사하지 않는다.

사용자 대상 세 원본은 이 폴더 밖의 기존 sample16 파일이다. 그 192쪽의 Native/fresh
표준 WASM 총 384쪽 raw 100% 선행 증거와 실제 전쪽 회귀는
`tests/fixtures/issue8160/README.md`를 따른다.
