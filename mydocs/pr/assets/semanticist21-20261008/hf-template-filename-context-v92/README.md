---
kind: evidence
status: active
canonical: mydocs/manual/pr_review/visual_fixture_evidence.md
last_verified: 2026-10-10
---

# #7638 머리말·꼬리말 20개 마당의 같은 파일명 문맥 검증

현재 공개 WASM API로 빈 문서에 `setFileName("보고서.hwpx")` 후 머리말/꼬리말1–10을 적용하고 HWPX를 캡처했다. 아래20개 입력 모두 과거 같은 파일명으로 실제 한컴2020 Print한 입력과 SHA가 정확히 같다. 합성/API 생성 입력이며 실제 한컴 작성 원본이나 새 정식 회귀 fixture로 분류하지 않는다. 기준은 각 링크의 같은 입력 PrintMethod0/1-up PDF1쪽이다. 파일명 문맥이 다른 과거 번호파일 Print는 이 판정에 사용하지 않았다.

Native v92는20개 전1쪽을 새 출력해 terminal exit0/858.564초/production source불변·각100%·미달0·누락0·쪽수1대1이다. 실제 fresh WASM v86 retry2 package를 재사용한 동일 조건 v92 sweep도 terminal exit0/958.237초/production불변·각1쪽100%·미달0·누락0이다. Native/WASM 전20개 raster가 모두 바이트 동일하다. 직접 판독한6개와확대2개는동일한두경로의관측을공유한다. 현재 code commit12d966의 production은9b53/v86 retry2와 같으며 Native binary SHAa41dbbebdf51f7c2fa928c6f2a7d5bd80a80a91bb9c904b3320fbd1afed8f2e2, WASM SHAab5e01324ad024dd34979f03a08d2da6844d26ff31c49069c17de0fe849779b6를 고정했다. 새 source build나 전체통합CI로 보고하지 않는다.

| 경로 / 마당 | 같은 입력 | 독립 Print | 입력 SHA-256 |
| --- | --- | --- | --- |
| header-01 | [HWPX](header-01/보고서.hwpx) | [Print](../../../../../pdf/semanticist21-20261008/hf_filename_context-보고서-1692d29dbc57-2020.pdf) | `1692d29dbc57e287ce0e81f965899dd8ebd311dfb0d69938fe0e3b05aa3c9d47` |
| header-02 | [HWPX](header-02/보고서.hwpx) | [Print](../../../../../pdf/semanticist21-20261008/hf_filename_context-보고서-865303b84642-2020.pdf) | `865303b846429f879fb93edaedff7367637e10e688eaaca1fcd2f4800834021e` |
| header-03 | [HWPX](header-03/보고서.hwpx) | [Print](../../../../../pdf/semanticist21-20261008/hf_filename_context-보고서-90aa0bccc412-2020.pdf) | `90aa0bccc412762cfb71d9fa7d9976c8a65168317d07944401474ed6de45d000` |
| header-04 | [HWPX](header-04/보고서.hwpx) | [Print](../../../../../pdf/semanticist21-20261008/hf_filename_context-보고서-cc2dae1277f5-2020.pdf) | `cc2dae1277f56de3c237cee97cfa914fe1ab9ca197f0c0872540762648575d2c` |
| header-05 | [HWPX](header-05/보고서.hwpx) | [Print](../../../../../pdf/semanticist21-20261008/hf_filename_context-보고서-f7b229c5adba-2020.pdf) | `f7b229c5adba3a0621fd62b2262d365de72072e5c8e5cc387fa968dab9ef1e38` |
| header-06 | [HWPX](header-06/보고서.hwpx) | [Print](../../../../../pdf/semanticist21-20261008/hf_filename_context-보고서-7a2d774a86da-2020.pdf) | `7a2d774a86dab82a2e82db0c7187001149f9abfc375466cee40d093f3b009825` |
| header-07 | [HWPX](header-07/보고서.hwpx) | [Print](../../../../../pdf/semanticist21-20261008/hf_filename_context-보고서-134444b85ce2-2020.pdf) | `134444b85ce2fe06675b615737eb87b31738430bdd715d4e69080ebea23c2294` |
| header-08 | [HWPX](header-08/보고서.hwpx) | [Print](../../../../../pdf/semanticist21-20261008/hf_filename_context-보고서-3430911b8858-2020.pdf) | `3430911b885845b652b0f1cb2f376ba4ec51e5ae7af578d6177884f1a6ecf21e` |
| header-09 | [HWPX](header-09/보고서.hwpx) | [Print](../../../../../pdf/semanticist21-20261008/hf_filename_context-보고서-6e6a419380a5-2020.pdf) | `6e6a419380a5a8d1f2d0a18338828c5194910d184b8c4809588bd88fb7c56908` |
| header-10 | [HWPX](header-10/보고서.hwpx) | [Print](../../../../../pdf/semanticist21-20261008/hf_filename_context-보고서-e25cd25c1dc6-2020.pdf) | `e25cd25c1dc6cc65750a7e65fedc42f5ff838433cc429c658f14e6c73ef36ef5` |
| footer-01 | [HWPX](footer-01/보고서.hwpx) | [Print](../../../../../pdf/semanticist21-20261008/hf_filename_context-보고서-083a1038beb8-2020.pdf) | `083a1038beb8e55b2e0babeefa5dcd427f65d7f5b9a3c0a4eb3b74dbfd7b125d` |
| footer-02 | [HWPX](footer-02/보고서.hwpx) | [Print](../../../../../pdf/semanticist21-20261008/hf_filename_context-보고서-c94aa52b0af3-2020.pdf) | `c94aa52b0af38650b4cea9999e91127259323283d96c030d5bd1b168226cbe95` |
| footer-03 | [HWPX](footer-03/보고서.hwpx) | [Print](../../../../../pdf/semanticist21-20261008/hf_filename_context-보고서-bd8776496fc5-2020.pdf) | `bd8776496fc5aad3a16bcfaca11cd9c5756cf40fc0d8fbd9df2d787969a29fae` |
| footer-04 | [HWPX](footer-04/보고서.hwpx) | [Print](../../../../../pdf/semanticist21-20261008/hf_filename_context-보고서-b2de25827113-2020.pdf) | `b2de25827113dad47eadc91409c7e5249ac37ab7c060ac45a23ade37e8a75f56` |
| footer-05 | [HWPX](footer-05/보고서.hwpx) | [Print](../../../../../pdf/semanticist21-20261008/hf_filename_context-보고서-fd84717fda61-2020.pdf) | `fd84717fda615545e33a1fe7e5f2ef80875dba3892f46450923187f1305b47da` |
| footer-06 | [HWPX](footer-06/보고서.hwpx) | [Print](../../../../../pdf/semanticist21-20261008/hf_filename_context-보고서-62a8dc1cb0a1-2020.pdf) | `62a8dc1cb0a1dfbf668a1269b81da2f1611f9016b8f9eebdbd26b96b5955fc25` |
| footer-07 | [HWPX](footer-07/보고서.hwpx) | [Print](../../../../../pdf/semanticist21-20261008/hf_filename_context-보고서-f3f646a8f3df-2020.pdf) | `f3f646a8f3dfc04e84e879720fac47e68f3911a3b1589c3aeb87713b5e9c9dce` |
| footer-08 | [HWPX](footer-08/보고서.hwpx) | [Print](../../../../../pdf/semanticist21-20261008/hf_filename_context-보고서-a95d4ad4a251-2020.pdf) | `a95d4ad4a25138da23351e5a9e806a99e481a617e21dd9f5d094cf02aa481cd1` |
| footer-09 | [HWPX](footer-09/보고서.hwpx) | [Print](../../../../../pdf/semanticist21-20261008/hf_filename_context-보고서-6e5f39ae606b-2020.pdf) | `6e5f39ae606baac47def371feddca6722ccd4bf052e1bb4dd7ea9b1413fc3678` |
| footer-10 | [HWPX](footer-10/보고서.hwpx) | [Print](../../../../../pdf/semanticist21-20261008/hf_filename_context-보고서-21910950ef25-2020.pdf) | `21910950ef2576f9cdf26a4335f8e1608458b7a6d31a7b2ddd21064fd8370a05` |

## 직접 판독한 대표 경계

머리말/꼬리말 각4(보통 파일명/번호),6(진하게·밑줄 번호),10(파일명·탭 구간 전체 밑줄·번호)의6개 review를 직접 읽었다.10번 두 경로는 원본 Native/Print raster의 내용 잉크 합집합 경계를4배 확대해 파일명 끝·번호 기준선/오른쪽 끝·탭 구간 밑줄의 연결을 확인했다. 같은 글자와 번호를 표시하고 밑줄이 탭 전체 구간을 잇는다. 이전처럼 번호/밑줄 끝이 크게 밀린 차이는 관측하지 않았다. 글자 굵기/edge·AA와 작은 밑줄 위상 차이는 남고 픽셀 완전일치를 주장하지 않는다. 수식은 이 입력에 없으며 별도의 수식 검증을 대신하지 않는다.

- header-04: [Native review](header-04/native-review_001.png), [overlay](header-04/native-overlay_001.png)
- header-06: [Native review](header-06/native-review_001.png), [overlay](header-06/native-overlay_001.png)
- header-10: [Native review](header-10/native-review_001.png), [overlay](header-10/native-overlay_001.png), [같은 영역 확대](header-10/native-review_zoom.png)
- footer-04: [Native review](footer-04/native-review_001.png), [overlay](footer-04/native-overlay_001.png)
- footer-06: [Native review](footer-06/native-review_001.png), [overlay](footer-06/native-overlay_001.png)
- footer-10: [Native review](footer-10/native-review_001.png), [overlay](footer-10/native-overlay_001.png), [같은 영역 확대](footer-10/native-review_zoom.png)

## 재현과 범위

현재 API 입력/SHA/기존 Print 쌍은 ignored `captured-current-public-hf-context-v92.json`, Native/WASM 명령은 `native-current-public-hf-context-20-actual-faces-v92-command.json` / `wasm-current-public-hf-context-20-actual-faces-v92-command.json`, 각 run_manifest·전쪽TSV·exit 기록은 같은 run에 있다. 직접 판독용 합성은 `hf-template-context-native-direct-review-v92/metrics.json`이다. Product Rust/Cargo와 renderer script를 고정해 test-only 이관과 병행했으며 그 작업을 product source 변경으로 섞지 않는다. TSV/로그/중간JSON/font 바이너리는 커밋하지 않는다. Studio Canvas/NativeSkia 직접 PNG나 초기18개 전체 gate 완료로 확대하지 않는다.

두경로의전체PNG SHA동일성은 ignored `hf-context-20-current-native-wasm-audit-v92.json`에있다. 대표Native review와같은raster의WASM이라는증거를고정했으며현재Package를새로빌드했다고보고하지않는다.
