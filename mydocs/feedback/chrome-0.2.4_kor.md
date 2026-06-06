rhwp는 브라우저에서 HWP/HWPX 문서를 바로 열고, 편집하고, 인쇄할 수 있는 무료 오픈소스 확장 프로그램입니다. 별도 프로그램 설치가 필요 없습니다.

주요 기능:

웹에서 HWP/HWPX 파일 다운로드 시 자동으로 뷰어에서 열기
문서 편집: 텍스트 입력/수정, 표 편집, 서식 변경
인쇄: Ctrl+P로 인쇄 미리보기, PDF 저장 또는 프린터 출력
편집한 문서를 HWP 파일로 저장
드래그 & 드롭으로 파일 열기
웹페이지의 HWP 링크를 자동 감지하여 아이콘(배지) 표시
마우스 호버 시 문서 정보 미리보기 카드 표시
우클릭 메뉴: "rhwp로 열기"

개인정보 보호:

모든 처리는 브라우저 내에서 WebAssembly(WASM)로 수행됩니다
파일이 외부 서버로 전송되지 않습니다
광고 없음, 추적 없음, 회원가입 불필요
어떠한 개인정보도 수집하지 않습니다

[v0.2.4 변경 사항 / Changes — 2026-06-06]

▣ v0.2.4 (2026-06-06) 주요 변경

본 업데이트는 rhwp core v0.7.15 WASM 번들을 반영하고, 브라우저 확장 프로그램의 문서 fetch 경로 보안을 강화합니다.

[보안 강화]
• service worker의 HWP/HWPX 문서 fetch 요청에 sender 검증 추가
• localhost, loopback, private network, link-local, 내부 호스트명 URL 차단
• redirect 이후 최종 URL도 동일 정책으로 재검증
• extension-side fetch에 credentials: "omit" 적용
• 자동 thumbnail 데이터가 page DOM에 직접 노출되지 않도록 hover card 내부 처리 보강
• 새 권한 없음
• 새 외부 네트워크 엔드포인트 없음

[rhwp core v0.7.15 반영]
• 수식 TAC-only 라인 자동 줄넘김과 문단 들여쓰기 처리 보강
• 강제 줄넘김 뒤 수식 커서 이동과 미주 영역 커서 이동 개선
• HWPX 그림 직렬화 flip/rotation 및 isEmbeded 출력 정정
• HWPX 대각선 셀 테두리 slash/backSlash type 보존
• zero-length HWPX field ordering 보존

[알려진 한계]
• HWPX 원본 형식 직접 저장은 아직 베타 단계로 제한됩니다
• 일부 복잡한 HWPX roundtrip 시각 정합은 후속 버전에서 계속 개선 예정입니다

[전체 변경 이력]
https://github.com/edwardkim/rhwp/releases

[소스 코드]
https://github.com/edwardkim/rhwp
