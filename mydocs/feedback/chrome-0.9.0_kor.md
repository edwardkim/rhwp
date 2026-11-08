# Chrome Web Store 설명 — v0.9.0 준비

rhwp는 브라우저에서 HWP/HWPX 문서를 열고 편집하는 무료 확장 프로그램입니다.
웹 다운로드 자동 열기, 문서 링크 배지·우클릭 메뉴, 로컬 파일 열기, 편집·인쇄·저장을 지원합니다.
문서 파싱·렌더링은 포함된 JavaScript·WebAssembly로 실행합니다.

## 이번 버전

- 중첩 표·쪽을 넘는 표·글상자·셀 안 그림과 머리말/꼬리말 배치를 개선했습니다.
- 그림 이동·셀 그림 삽입, 연속 형광펜, 문단 서식과 저장 후 재열기 보존을 보강했습니다.
- 수식·글꼴 fallback과 큰 표/JPEG의 처리 경로를 개선했습니다.
- 공통 엔진의 PDF 가져오기와 개발자 안내를 추가했습니다. PDF를 편집 문서로 변환하는 CLI·Studio 사용법은 아래 안내를 참고하세요. 모든 웹 PDF를 자동 가로채는 확장은 아닙니다.
- v0.8.7 대비 Chrome manifest의 permissions·host_permissions·CSP는 동일합니다.

배포 전 최종 패키지의 열기·편집·저장 검증과 스토어 심사를 완료해야 합니다.
PDF 안내: https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/pdf_developer_onboarding.md
개인정보 처리방침: https://edwardkim.github.io/rhwp/privacy.html
소스와 변경·기여자: https://github.com/edwardkim/rhwp
