# 저장된 빈 Square 호스트의 독립 출력

기존 p122의 정상화 대조군에서 그림을 종이 기준 부동 Square로 바꾸고, 빈 호스트 뒤에
빈 문단 또는 본문을 배치했다. 한컴 2020 MCP 정상 저장본을 그대로 PrintMethod 0 /
one-up으로 출력했다. 한컴 버전은 11.0.0.9136이다. 원본 64쪽의 재생성본이 아니다.

저장 첫 줄은 1000HU·간격 600HU다. 다음 저장 원점 1600HU가 빈 문자열에도 실제 줄
예약이 있음을 증언한다. 각 문서 1쪽의 최종 Native/fresh WASM 일치율은 모두 100%이며
review·overlay를 직접 확인했다. 검사는 소유·내용 보존·순서·포함·독립 저장 메트릭의
무차원 비율을 사용하며 절대 px 좌표를 고정하지 않는다.

| 파일 stem | Save job | Print job | HWP SHA-256 | PDF SHA-256 |
| --- | --- | --- | --- | --- |
| empty-square-empty-follower | 5631bd47-839a-4b98-8a93-875f97bf976a | ee055445-ee1f-4adb-b0ef-0d093e86c8af | `88a4f316f07828159096eca12e027ed474f19ebc6ae9b9ff05fe7cccadad72fe` | `b9e58de474c5e0146b7ba655a3f5efc98450096dbe53f6ca625a45243bb08ca3` |
| empty-square-text-follower | 89056f23-2853-4d19-a4a8-0e3aa6b0df4f | 21a3e489-67bb-4251-854b-2506f2185147 | `35cb16686393e21f987a91249400a4d92c265938cec746352b87c2671d92e788` | `3fd43cae9d9e637e0ac0af6739e0e0562fc56a509c723f6cba0e9434fca8c4d3` |

[단계 기록](../../../../mydocs/working/task_m100_7642_stage5.md). 전쪽 TSV·로그는 ignored output에 보존한다.
