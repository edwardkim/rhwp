# #7728 — 부모 예약과 자식 행 컷의 물리 여백

완료 판정이 아닌 부분 보정이다. 사용자 지정 Native/fresh WASM 전 18쪽 각각 98% 목표는 미완료다.

- 원본 SHA-256: `a80b09754ce4f8e73398af9a9467f570c02be237815fec203ee8ff2db1bb8cbb`.
- 독립 한컴2020 Print PDF SHA-256: `6cec821ff5897714669602c8707d66f669cfcb1187362c69101a0255c8d17956`.
- 수정 전 source: `429eb75b893cfa52050909f603cdfca5ee2bd82f`.
- 수정 후: 같은 source에 `table_layout.rs`의 정확한 자식 행 컷 질의 보정. 해당 파일 SHA-256: `fa8d1e82527b13b4b5a312729af2eb14f96e990818159a7f0663f614e2e5ad16`.
- 수정 후 Native binary SHA-256: `83cd5cbfc798407b2b46fb2b26ea3dc12f70830386039688b38bef1b835a18e9`.
- 동일 원본·사용자 검증 글꼴 경로·Print/compat2022·96DPI/PDFium·2px raw, 전 18쪽을 다시 출력했다.
- 4쪽 97.90408→99.92664%, 5쪽 95.35356→95.41089%, 6쪽 94.62153→94.59369%, 7쪽 94.79966→96.06208%. 나머지 14쪽은 점수가 같다.
- 6쪽 하단 부모 괘선의 자식 포함 관계는 실제 1 FAIL→1 PASS이며 기존 두 경계 검사도 PASS다. 절대 좌표를 추가하지 않았다.
- 부모가 자식 아래에서 닫히는 부분은 복구했지만 6쪽 안쪽 표의 빈 종료 공간은 독립 Print와 아직 다르다. 새 후보 fresh WASM·전체 회귀·정상 대조군은 미검증이다.

실행 로그·진단 스크립트·중간 JSON은 ignored output에만 보존한다.

## 산출물 해시

- `before-native-silhouette.tsv`: `fa896633b70d53e472b93f2d6c9aca51ed5f2dd75a1fba8a2b516b67f7d986a2`.
- `after-native-silhouette.tsv`: `d029805d16c9068ff8e4408742f21f856c4c67b112fe15b9a2242d0f4fddf04c`.
- `before-native-p006-review.png`: `4b90adf1ee3a9266c6b6b71e5f47e1eb5c473a74e1affb3472e64d408b242f7c`.
- `before-native-p006-overlay.png`: `6adff737915c1982f56f70a924a1daca3e3b1584caf3cc7bb8ff8c2e6cdafdd4`.
- `after-native-p006-review.png`: `ef248c0ebf594c26e192f06a0f03e6b7a992c475c2cdde71aeb058c635971892`.
- `after-native-p006-overlay.png`: `e4426801163b51a1a7a55d3224220322ccb41db09a9ccc8413954adfd2ea0ea9`.
