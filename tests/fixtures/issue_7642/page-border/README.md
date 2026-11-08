# 쪽 테두리의 정상 저장·독립 Print 대조군

종이/본문 기준 × 실선/이중선 × 굵기 index 1·6·10의 12개 입력이다. 기존 정상 저장
반례의 문단을 바탕으로 쪽 설정과 테두리만 바꾸고, 한컴으로 정상 저장한 뒤 해당 저장본을
Print 출력했다. source 생성기는 ignored output의 `page-border-counter.rs`에 보존한다.
원본 64쪽의 대체 파일이 아니다. 문구는 정상 저장 뒤 `테두리 검증`으로 확인했다.

engine 2020 / 한컴 11.0.0.9136 / PrintMethod 0 / one-up, 각 전체 한 쪽을 실행 결과에서
확인했다. Native/fresh WASM 전쪽 TSV는 12개 모두 100%다. 대표 확대 판독·정식 회귀와
최종 lint 상태는 [단계 기록](../../../../mydocs/working/task_m100_7642_stage6.md)을 따른다.
TSV의 숫자 통과를 자동 승인으로 보고하지 않는다.

독립 Print의 실선/이중선 각 stroke (pt)는 index 1: 0.36/0.12,
index 6: 1.079/0.24, index 10: 2.878/0.72다. 이중선 중심 사이 거리는 각 stroke의 세 배다.
종이/본문 경계와 저장 spacing이 선 중심을 정하고, 선 묶음 폭의 추가 원점 확장은 없다.
회귀는 문서 경계에 대한 비율과 이 독립 선 묶음의 관계를 검사한다. 실물 배치를 고정 px로
선택하거나 잠그지 않는다. 96dpi의 얇은 선 래스터 농도 차이는 물리 stroke 관측과 구분한다.

| stem | Save job | Print job | HWP SHA-256 | PDF SHA-256 |
| --- | --- | --- | --- | --- |
| body-double-w1 | 89a85b3b-4999-4fb5-86f8-6c23d4448cb5 | 12bc400d-7969-44ef-a04f-a1c53fc23056 | `c01fb540a22b09ad79b564d4639ab3da8a1b3453a643278db28e0e5265a0a885` | `de2353b086c5ac0aec9f782e3a7cee6f14f3157e44d270bae86b9b086e023cd0` |
| body-double-w10 | a450bf36-6894-447d-a829-3104b4cece54 | 5801ec7a-2466-42c9-ab6f-c717c5a093ff | `1577c1e2f3ed2a0a16e72b86cb2987ca7e5353885de67562f88af423927d23ba` | `6a66c19fc820378788bfc2c0510ae21c491ae6636a9f468483ef7840061af455` |
| body-double-w6 | 17364200-0923-4847-be27-6408cf0a890a | cc7521a1-ea64-41b7-b9f5-717957954c85 | `b6bff50c2138ed8c8508bbf4ef23a9e637fdeee9eb78bfdbf36f299a2db8dd7d` | `212aee4a2bb1ac9cf717af5cef43c45753bb6d0151977e97c70266ab75ad87e9` |
| body-solid-w1 | 6dc68ec2-cfe1-4a90-a7d1-844b19327df3 | 1bd5924f-eedf-4188-8c81-b8ab2d76420d | `d6ce062f582fcbd42449196f6919da2f377bc269a211d12a7cf0c994e6b35368` | `b389e85cf43a68695cc224dff0dcab9c5734a762ea716680d444dbc30936e34b` |
| body-solid-w10 | e1de1581-4e31-4de2-a714-05930691f170 | 69cdcc2d-0894-4dc3-8ea5-c61a1bc6fa80 | `a83f5c4765ddb6c18da6a35f5bfd163853256a91e16cd6f0a6200b0f9951ca41` | `fbf55d8825e2a8c7dc435c3e69275a8080dca4cf04c7f771d11da2ffbfbfcfa5` |
| body-solid-w6 | 429e47f8-4a79-42ee-897b-9c567d942586 | 963fc0bc-f66d-49d5-8467-0414d6771abf | `970666d8427bde54c8f92132766e18b9986a1310394a1c44e4ef385b58204b6b` | `91d480ec6e273f6a60eaaf8e4c381ce96bcafb5eb7c14f57e84d999d8c3b559d` |
| paper-double-w1 | f8c058c3-3fbf-421b-b797-694bb6c3511c | fc070745-0357-464a-a3bc-d7786fbe4add | `ad38eaf449ad798f8f612f1ea4d3583afcf6480bc82a76fc2178bc6f9b35564b` | `d82379a3c413d49a850262e6c53addef15d771d2ac4a41b6feec31a948cf314c` |
| paper-double-w10 | fee5d011-4b08-40b9-855d-03a8cb692acf | ff8db043-e8ea-43fb-97fa-0bd7c7747ec0 | `6828d02d17a944287dc9a1fa624c75101998afab084737f4c1969498a880f788` | `3209a1f25ed3a72dcaee7285023a8f815c951cb50e994d099351a7d8b2938a9d` |
| paper-double-w6 | 4502d620-8e53-40dd-81b3-455961e0a985 | 952cb791-9d71-4755-804e-61c6e8dc7b66 | `ca750066ff0e107fc80952709c312f56e4d0df7185226c6990ca3835d4b1df74` | `7b26296f60f8be8cc6f84e2c5b7aa6b703eb3cec67ed18e17d89b8d1b374f4bf` |
| paper-solid-w1 | 0e8da046-f1f9-459b-98fb-dc80b4dd0a26 | ba9e8900-1acc-433d-9512-8414a1232658 | `a47b38ab919d735410559dc8a3b1b9213171d6bed5908d6ee82af967d5abc92c` | `f5c2944d572d95987c5dd2bdbb0bdece11f97c810f1f90b4ae7b6af442f7a116` |
| paper-solid-w10 | 83313076-b269-40a0-acd3-f323b3a0a90d | db6fbec9-300c-4993-97ea-57d9ae83218c | `719c756c09616289ba265ee87b4abb3aca49888453f7d5b0374fb2f14f629bf8` | `2a4010a5d91970e2272f1c46c23119496a14f9dbc67f5f5555d8255dade5ebed` |
| paper-solid-w6 | 367b0eb3-0144-4383-bf21-6f4af05dba3d | c9d6d6bf-07a5-4169-ae01-e2dd67d2056f | `4a7ee3b26533931529aa396bbf84b98deb3b170546855fd163b91986943dc126` | `14ffe71508036bf0fdc1b498317a716750c4463a3f3b00c1b9ff8c2be43d5e93` |
