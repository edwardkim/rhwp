# Chrome Web Store — 0.9.1

상태: 제출 전 후보. 실제 패키지·source SHA·SHA-256과 배포 결과는 최종 포장 뒤 기록합니다.

- 0.9.1 엔진의 표·글꼴·저장 재열기·작은 화면 대화창 개선을 포함합니다.
- 문서는 브라우저 안의 WASM으로 처리합니다. HWP/HWPX 링크 감지·미리보기·컨텍스트 메뉴·로컬 파일 열기에 기존 match/host 권한을 사용합니다.
- v0.9.0과 실제 manifest를 대조했으며 버전 외 필드가 같습니다. 새 permissions·host_permissions·CSP 변경은 없습니다. 추가 외부 endpoint의 포장·실행 검증은 별도로 수행합니다.
- Chrome과 Edge는 같은 ZIP을 사용합니다. Firefox는 해당 ZIP과 재빌드 가능한 AMO source ZIP을 함께 제출합니다.
- PDF 가져오기는 CLI·WASM·Studio의 지원 계약을 따릅니다. 모든 웹 PDF를 자동으로 가로채는 확장이 아닙니다.
- 심사·공개 전환과 독립 설치 검증은 아직 완료되지 않았습니다.

[0.9.1 전체 변경과 기여자](../release/release_notes_v0.9.1.md)
