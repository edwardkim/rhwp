# Microsoft Edge Add-ons — 0.9.1 reviewer notes

상태: 검증 완료. 메인테이너 수동 제출용입니다.

- 0.9.1 엔진의 표·글꼴·저장 재열기·작은 화면 대화창 개선을 포함합니다.
- 문서는 브라우저 안의 WASM으로 처리합니다. HWP/HWPX 링크 감지·미리보기·컨텍스트 메뉴·로컬 파일 열기에 기존 match/host 권한을 사용합니다.
- v0.9.0과 실제 manifest를 대조했으며 버전 외 필드가 같습니다. 새 permissions·host_permissions·CSP 변경은 없습니다. 검토한 소스 변경 범위에서 새 외부 네트워크 endpoint를 발견하지 않았습니다.
- Chrome과 Edge는 같은 ZIP을 사용합니다. Firefox는 별도 확장 ZIP과 그 확장을 재빌드할 수 있는 AMO source ZIP을 함께 제출합니다.
- PDF 가져오기는 CLI·WASM·Studio의 지원 계약을 따릅니다. 모든 웹 PDF를 자동으로 가로채는 확장이 아닙니다.
- 스토어 심사와 공개 설치는 제출 후 확인합니다.

[0.9.1 전체 변경과 기여자](../release/release_notes_v0.9.1.md)

No new permissions. v0.9.0 대비 manifest의 버전 외 필드는 동일합니다.

[최종 소스·파일별 SHA-256 기록](../pr/assets/release-0.9.1-execution/delivery-evidence.json)
