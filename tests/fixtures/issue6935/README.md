# #6935 실제 RowBreak 컷 재현 문서

- 원본: `18179365_[별표 11의4] 고전원전기장치의 충돌시험기준(제91조의3 관련)(자동차 및 자동차부품의 성능과 기준에 관한 규칙).hwp`
- 출처: [이슈 #6935](https://github.com/edwardkim/rhwp/issues/6935)의 실제 문서. 로컬 법령 다운로드 원본을 내용 변경 없이 보존했다.
- HWP SHA-256: `a1c4ad6b4e1a4253869b7e79474527c120c62529404037f1fbdff4c6f1520ab6`.
- 기존 추적 HWP/HWPX 중 같은 크기·SHA-256 파일이 없음을 확인했다.
- 저장 메타데이터: `hancom-office-2020`, `11.0.0.5409`.
- 기준: `pdf/18179365_high_voltage_collision-2020.pdf`, 원본 그대로 engine 2020 변환, 3쪽.
- PDF SHA-256: `179353f2e83e35e913571796d3ef4033c9662d229c70b0f0b0bc8257c67b88b7`.
- MCP job: `b230e2be-6905-4f0d-8d1b-64ebcb3d4e2b`, 한컴 `11.0.0.9136`, queued → succeeded → download success.
- client/server 147622 bytes·SHA-256 일치, input_preprocess=none. PDF 1.4 / Hwp 2020 0.0.0.0 / Hancom PDF 1.3.0.550.

시작은 행 공간, 끝은 블록 공간인 2쪽과 앞뒤 조각의 내용 소유·표 기하를 검증한다.
