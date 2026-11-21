# PR #9250 검토 — 한컴 Print 기록과 현재 원본·PDF의 대응

**승인.** postmelee의 자체 검토 대상은 devel `c198dc3e` 위의 `beb179e940f06d45d27362810aef25bb10b2e2f9`이다. 후보 수집은 기존 resolver에 맡기고, 선택형 CLI에서 명시된 원본·PDF 경로와 현재 파일 두 개의 SHA-256을 대조하는 경로를 확인했다. MCP Print의 엔진·출력 모드·방법과 수동 확인을 구별한다. 기본 resolver와 활성 회귀 수집의 출력 형식은 유지된다.

- [생성 기록](../assets/hyunmyungjanelee_20261112/mcp-print-reference/receipts.json)의 34쌍에서 두 파일의 해시가 모두 일치했다. 같은 원본에 기록된 엔진으로 기존 canonical 선택기를 적용해 34쌍 모두 같은 PDF 경로를 반환하는 것도 확인했다. 다른 경로의 동명 파일, 같은 경로의 바이트 변경, 증거 없는 후보와 누락 파일을 별도로 검사했다.
- Python 3.14에서 resolver·sweep runner·신규 검사를 포함한 46건, pypdf를 공급한 환경에서 이를 포함한 oracle_public 전체 테스트 126건이 통과했다. 별도 쪽수 재생성 도구의 기존 등록 경로 문제는 #9253에서 재현했고 #9254에 독립 보정을 제출했다.
- [코드 head의 CI](https://github.com/edwardkim/rhwp/actions/runs/44751415262)가 성공했다. 기존 샘플, PDF, 엔진 및 게이트 기대값을 바꾸지 않는 검증 도구 변경이다. 이 기록이 포함된 head의 CI와 병합 가능 상태를 확인한 뒤 통합한다.
