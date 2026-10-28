# 6쪽 평가 비율 표의 저장 축 회귀 증거

Issue: #7445. 제품 원점 보정은 `c4c5632a21d4a34e76b18fc88bccfaf939fff4ab`,
Native 장치 PNG 보정은 `6215ec12b1552a78eb9092dfa1530b7defd9dd43`다.
이후 `6cb3d68b95e40082a5f82154da8276da7bcc3d84`는 cfg(test) 검사와 보고서만 바꿨다.
이 증적은 전체 PR 검증 완료를 뜻하지 않는다. 원래 범위·incoming PR의 잔여 하락과 미검증이 남아 있다.

## 원본과 독립 기준

- 원본: `samples/hwp3-sample16-hwp5.hwp`, SHA-256 `6a3cdf2c148bf39f40ab06e847767e7f722543fce59e6ee4328ef44b835f3f45`.
- 사용자 제공 한컴2024 Print64쪽: `pdf/hwp3-sample16-hwp5-lossless-2024.pdf`, SHA-256 `9d0a504f5344f39b770d84eba613567feb7910eded9fbb125ad4e7b0bab6b589`.
- 양쪽 Print·compat2024·192DPI. 테스트 좌표는 PDF pt×96/72의96CSSpx다.
- canonical Visual Sweep의 색상 경계 보정 전2px raw 실루엣90% gate.
- Native64/64쪽 최저95.25071%, WASM64/64쪽 최저95.66182%, 미달·누락0쪽.
- 6쪽 Native98.16591%, WASM98.56406%. 두 review·overlay에서 표 외곽과 앞뒤 문단을 판독했다.
  수식/글꼴의 남은 미세 차이도 보존하며 완전한 픽셀 일치로 쓰지 않는다.
- 기존3쪽99%대 핀은 유지한다. 이번 원점 재시작을 실제로 통과하는 경계는6쪽의 평가 비율 표다.

## 독립 기대 좌표와 내용 소유

같은 PDF6쪽의 표 외곽 path stroke 중심을 사용한다. 왼쪽117.697337, 위603.340007,
오른쪽709.218669, 아래733.277344CSSpx이며 허용 오차는 각각±2CSSpx다.
표의 원본 내용으로 소유자를 선택하고 실제 쪽·문단·control 소유, 전64쪽 누락·중복,
앞 caption→표→후속 평점산식의 순서·비겹침·본문 포함도 함께 검사한다.
절대 px 영역으로 원본 문단/표를 찾지 않는다.

`tests/cases/verified_sample16_2024_print_boundaries.rs`의
`completed_page_six_keeps_the_evaluation_table_and_its_source_neighbors_in_the_print_frame`:
원점 보정 전 frozen repair3에서 위쪽595.093333으로1FAIL, 보정 뒤 frozen repair7에서1PASS다.
정식 manifest의 regression_suite_020에서도1실행1PASS/248skip, exit0을 확인했다.
테스트0.307초·전체118.012초이며 source/명령/로그는 stage8 C419에 연결한다.

## 생산자

- Native 새 빌드 CLI SHA-256: `f8224d24d54b6ba53c49792d2dec37579840b06ed69fdec0d18a6aea0ca13e9c`.
- WASM JS SHA-256: `23f085e36c78d952ddac0d91faed9752f33ee1124db78e290382687410917591`.
- WASM SHA-256: `4d27bd61e2a0a4feb2d610e9f7c8b19b3190af710f15be315b31330938dce570`.
- WASM은 Mac locked wrapper의 web/out-dir pkg/no-opt 로컬 대체 빌드다.
  현재 제품 변경은 WASM에서 제외되는 Native 모듈뿐이므로 source-equivalent fresh repair5
  package를 재사용해, 사용자 HancomEQN 등록 후64쪽 PNG/TSV는 전부 새로 산출했다.
  Docker 최적화 또는 실제 Studio 초기화 검증으로 보고하지 않는다.
- ignored `rebase-4a07d3ae-20261019/repair7-wasm-producer-binding.json`,
  `repair7-paired-sample16-qualification.json`, `repair9-page6-*-result.log`가 source/build를 연결한다.
  원본 PNG/TSV·검증 로그·글꼴 catalog는 ignored output에 보존한다. 글꼴 프로그램은 커밋하지 않는다.

| 제출 증거 | SHA-256 |
| --- | --- |
| native-whole64-silhouette.tsv | `a1efa09be33d093c9a72fbb65034ec5fbbd367ef313b929c835aedb11dd50322` |
| native-p006-review.png | `41af7ece2050c2e609564eb122f46e53bea60263472b8dcf3e72dd44f28dcc1a` |
| native-p006-overlay.png | `37c5d1f584b5c811d997d185c72b25419ce13a4a04b93acfe631f6f81c8e95bb` |
| wasm-whole64-silhouette.tsv | `067719caefd3260f1794c2411855c09c25cdff3707ca798475ff478e2a7422e6` |
| wasm-p006-review.png | `b6ea8a1988eb622ad344bb53ab35cda68bd3497577abd9d7386f62a2d8c6e457` |
| wasm-p006-overlay.png | `b3d615b2129ca2e9d1b8f9d9a1aa7382af71493dfa41a0c3b3fd938d9c76fc83` |
