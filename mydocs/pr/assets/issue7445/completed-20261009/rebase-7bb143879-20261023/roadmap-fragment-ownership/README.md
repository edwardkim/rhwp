# 로드맵 저장 조각·각주·그림 뒤 본문 프레임의 중간 증거

**진행 중이며 PR 수용 증거가 아니다.** 후보 Native/fresh WASM은 302쪽으로 일치하지만
153쪽 raw 83.55427% 미달과 기존 회귀 4건이 남아 있다. 새 테스트·기대값은 추가/완화하지 않았다.

## 입력과 생산 출처

- 원본: `samples/issue5941/1490000-201600081_roadmap_research.hwp` / SHA-256 `a06f46ec3f175c7cfa84eb3178b8b3fbdf78e94f71b31d7d87f3417a2617dae9`
- 독립 기준: `pdf/issue7382-regression-review/1490000-201600081_roadmap_research-2020.pdf` / SHA-256 `133f79cad7b756ebef777d253befb14f9407faaf7fe04eeea66036f3e049bbe7`
- base: `7bb143879dbde80a02fb4a63692556ac28fe46ac`
- 보정 전 code head: `41c06540d992e0cb15c9650829ec133ad6a85b26`
- 후보: 위 parent의 68단계 V6b 생산 소스. 같은 커밋의 구현 파일 hash는 아래 표와 같다.
- Native 봉인 바이너리 SHA-256: `7f9e7a1333aca90774ac1ef11aafbc26324ba4dcf934da8669515cc756e637cf`
- fresh WASM SHA-256: `b08570b54de61fe280e41653f15e4f329fefd63f4c8ed356b694f70e130633c1`
- 96 DPI, Print, compat 2022, PDFium 153, pdfium_smooth_rgb_v1, raw 2px/90%를 유지했다.
- WASM은 Mac locked wrapper의 `--target web --out-dir pkg --no-opt` 로컬 대체 빌드다.
  Docker 최적화 빌드가 아니다. root pkg/Studio public/봉인 파일의 JS·WASM hash는 같다.
  실제 Studio 초기화 응답 검증은 아직 완료하지 않았다.

## 전쪽 결과

| 출력 | 쪽수 | raw 최저 | 90% 미달 |
| --- | --- | --- | --- |
| before-native | 302 | 7.45335% | 117, 118, 119, 128, 129, 130, 131, 132, 133, 134, 135, 136, 137, 138, 139, 140, 141, 142, 153, 252, 297 |
| before-wasm | 302 | 7.45345% | 117, 118, 119, 128, 129, 130, 131, 132, 133, 134, 135, 136, 137, 138, 139, 140, 141, 142, 153, 252, 297 |
| after-native | 302 | 83.55427% | 153 |
| after-wasm | 302 | 83.55427% | 153 |

전쪽 TSV는 원본 바이트 그대로 복사했으며 CRLF를 정규화하지 않았다.

## 대표 직접 판독

127쪽은 각주 위 본문 두 줄의 소속과 각주 영역을 보존했고, 141쪽의 앞 그룹·본문과
142쪽의 뒤 그룹이 별도 쪽으로 복구됐다. 252쪽은 원본 표 전체가 닫는 저장 줄과 뒤 본문을
보존한다. 296~297쪽은 Estonia/Finland의 원본 행 소속을 복원했다.
153쪽은 원본 BMP와 독립 PDF의 래스터 출력 차이를 추적 중이며, 회색 격자·점의 차이를
보정된 진단 점수로 통과시키지 않는다. 기준 PDF는 교체하지 않았다.

- [before native 전쪽 TSV](before-native-silhouette.tsv)
- [before wasm 전쪽 TSV](before-wasm-silhouette.tsv)
- [after native 전쪽 TSV](after-native-silhouette.tsv)
- [after wasm 전쪽 TSV](after-wasm-silhouette.tsv)

- native 127쪽: [review](after-native-p127-review.png) / [overlay](after-native-p127-overlay.png)
- native 141쪽: [review](after-native-p141-review.png) / [overlay](after-native-p141-overlay.png)
- native 153쪽: [review](after-native-p153-review.png) / [overlay](after-native-p153-overlay.png)
- native 252쪽: [review](after-native-p252-review.png) / [overlay](after-native-p252-overlay.png)
- native 296쪽: [review](after-native-p296-review.png) / [overlay](after-native-p296-overlay.png)
- native 297쪽: [review](after-native-p297-review.png) / [overlay](after-native-p297-overlay.png)
- wasm 127쪽: [review](after-wasm-p127-review.png) / [overlay](after-wasm-p127-overlay.png)
- wasm 141쪽: [review](after-wasm-p141-review.png) / [overlay](after-wasm-p141-overlay.png)
- wasm 153쪽: [review](after-wasm-p153-review.png) / [overlay](after-wasm-p153-overlay.png)
- wasm 252쪽: [review](after-wasm-p252-review.png) / [overlay](after-wasm-p252-overlay.png)
- wasm 296쪽: [review](after-wasm-p296-review.png) / [overlay](after-wasm-p296-overlay.png)
- wasm 297쪽: [review](after-wasm-p297-review.png) / [overlay](after-wasm-p297-overlay.png)

## 검증의 범위와 잔여

- 기존 로드맵 3개 검사는 보정 전 2 PASS / 1 FAIL → 후보 3 PASS다.
- 확대 정상 대조군은 151 실행, 147 PASS / 기존 4 FAIL / 5,734 skip, 12.902초다.
  로드맵 겹침 회귀도 통과했고 새 실패는 없다. 전체 nextest 통과로 재인용하지 않는다.
- #7696의 저장/재조판 밴드, #6718의 저장 reset, #7702/#8105의 어절 꼬리와 정상 저장 줄,
  #7993 soft-hyphen, #7514 목록 caret 및 기존 표 분할/각주/그림 정상 대조를 포함한다.
- 새 실물·px 회귀는 전쪽 Native/fresh WASM 90% 선행 조건 충족 전 보류한다.
- 최신 devel 전쪽 비교, 남은 4건 보정·전체 회귀·필수 게이트·실제 소비 경로는 계속 진행한다.

## 후보 생산 소스 SHA-256

| 파일 | SHA-256 |
| --- | --- |
| `src/renderer/render_normalization/stored_word_tail.rs` | `12272a08cd704e5c4dfc85946384761d169937e7b30c050330ec71d84cb43576` |
| `src/renderer/composer/stored_cell_reflow.inc.rs` | `a98845e8a6f38885bdfd016043512370a9757282036a0d4601fa69ac6ca1cbf4` |
| `src/renderer/layout.rs` | `6fd854e54cdb252b16a34b9aaca4d4a12b0340038712fe04de56054f461ce388` |
| `src/renderer/float_placement.rs` | `15369e3db4a50d45f56a24a01bcf3e145b7801ee239c077b5d51c5852e8e5e71` |
| `src/renderer/typeset/section/flow.rs` | `17bd35867efaec68ec09d3b3e00d57570e22cfe851b2be5089752f8b96144e21` |
| `src/renderer/typeset/paragraph.rs` | `3377502241132256dcba72bd8d335da9a2287c183b7adc51d959918210bae52d` |
| `src/renderer/typeset/paragraph/flow.rs` | `7e81d25cc21681d952f440208b6a67e042764eb2c70a828bc5fba53eaea86a5a` |
| `src/renderer/typeset/paragraph/forced_boundary.rs` | `ed8d38c37d76f6f76363b7edf5747958617e702bb2a1dff47dddcb7efc0a9d0d` |
| `src/renderer/typeset/notes/footnotes/measure.rs` | `b3038d97c5540827643d777bae42c4569c93e7e3aaf2851e2936e78592dd307e` |
| `src/renderer/layout/table_layout.rs` | `afae38305b2d737b789fe6b1ee9a108b98558cf91dcf246cf5a22790914ab1c8` |
| `src/renderer/typeset/table/block/prepare.rs` | `d0e567f3be9d176803d03dbfb866d8ee80405622b11cbe61a9c582aa344770c5` |
| `src/renderer/typeset/table/continuation/fragment/budget.rs` | `9f08957b0e4b30eb4e63c2b2e959c48b1f452e18e71aaf8cfc8208b26e7fab61` |
| `src/renderer/layout/table_partial.rs` | `0df5fee6e29df9973b2f3496ebf0669f298d29cb791ee0b7ee98ea42b5b3a879` |

## TSV 원본 SHA-256

| 파일 | SHA-256 |
| --- | --- |
| `before-native-silhouette.tsv` | `dc0bb20fa1c72613a0d4e1f363e32f4212546cb0f00d91a174978b7d6b83b346` |
| `before-wasm-silhouette.tsv` | `81d41f157bc21b31f9263b672fc97c6880edccfad339d5338316bc718dd23169` |
| `after-native-silhouette.tsv` | `4fbdb3d3ec348a5b5d4e7b02bf15a6c94cd11762e498560d76b9e24ea7383ecc` |
| `after-wasm-silhouette.tsv` | `e4570b4981c98838c6fc8b99f4bffdf28c0b6d20d553b064c9f17dfac5edd84c` |
