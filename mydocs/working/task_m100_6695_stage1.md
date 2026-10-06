# #6695 Stage 1 — PDF LFS 전환 원장과 소비자 지도

## 1. 판정

**Stage 1 종료 게이트 통과**다.

- 기준선: `upstream/devel@2b2646f4051dd1ecefc296e2e032308bd0f25bd7`
- 작업 브랜치: `task_m100_6695_pdf_oracle_consolidation`
- 전환 대상: `pdf-2020/**` PDF 1개, `pdf-large/**` PDF 18개
- 실제 PDF 합계: 19개, 58,159,986 bytes
- Git index 상태: LFS pointer 17개, 일반 Git blob 2개
- LFS 검산: 실제 object 17/17 존재, 선언 OID 17/17 일치, 선언 크기 17/17 일치,
  `%PDF-` magic 17/17 정상
- 목적지: 서로 다른 경로 19개 중 신규 17개, 이미 존재하는 byte-identical 목적지 2개,
  hash 불일치 충돌 0개
- 예상 결과: 현재 `pdf/**` 1,161개 + 신규 17개 = 1,178개 PDF

따라서 복원 불가능한 pointer나 덮어쓰기 판단이 필요한 충돌 없이 Stage 2의 원자 전환을 수행할 수
있다. 이 문서는 조사 결과만 고정하며 PDF 이동, pointer 복원, `.gitattributes` 수정은 하지 않았다.

## 2. 측정 기준

세 상태를 섞지 않고 따로 판정했다.

1. **Git index**: `git show HEAD:<path>`로 커밋된 내용이 LFS pointer인지 실제 PDF blob인지 판정했다.
2. **working byte**: 현재 작업트리에 보이는 파일 크기를 기록했다. LFS 미수신 상태면 실제 PDF가
   아니라 작은 pointer일 수 있다.
3. **actual byte**: LFS pointer이면 `.git/lfs/objects/<OID>`의 실제 객체, 일반 blob이면 Git index
   bytes를 읽어 SHA-256·크기·magic을 계산했다.

`git lfs ls-files --long`의 17개 중 16개는 작업트리에 실제 bytes가 있는 `*`,
`pdf-large/hwpx/issue_1133.pdf`만 pointer가 남은 `-` 상태다. 후자도 로컬 object store에 선언과
일치하는 실제 PDF가 있으므로 Stage 2에서 네트워크 다운로드 없이 복원할 수 있다.

현재 `.gitattributes`는 `pdf-large/**/*.pdf` 18개 모두에 `filter=lfs`를 반환하지만,
`2026_oss_rst.pdf`는 그 속성과 달리 index에 실제 PDF blob으로 들어 있다. `pdf-2020`의 1개에는
LFS filter가 없다. Stage 2는 이 예외까지 일반 Git blob 단일 정책으로 정상화한다.

## 3. 전환 원장

`SHA-256 / LFS OID` 열은 LFS 대상에서는 pointer의 OID이자 실제 PDF bytes의 SHA-256이고,
일반 blob에서는 실제 PDF bytes의 SHA-256이다. `동일`은 목적지에 이미 같은 bytes가 있다는
뜻이며 해당 목적지 파일을 보존하고 기존 source만 제거한다.

| 기존 경로 | 확정 목적지 | index | working byte | actual byte | SHA-256 / LFS OID | PDF magic | 목적지 |
| --- | --- | --- | ---: | ---: | --- | --- | --- |
| `pdf-2020/pr-1674-2020.pdf` | `pdf/pr-1674-2020.pdf` | Git blob | 860,225 | 860,225 | `aead614b8e0e9dd40728124ba929074196b28ac950b09b7102188550af8efcfd` | 정상 | 신규 |
| `pdf-large/3-09월_교육_통합_2024-구분선아래20-2024.pdf` | `pdf/3-09월_교육_통합_2024-구분선아래20-2024.pdf` | LFS pointer | 1,472,249 | 1,472,249 | `d66b4c2071006986200f6f8ea91444353af84bb44ac83de5c8a58a6cedf9f643` | 정상 | 동일 |
| `pdf-large/3-09월_교육_통합_2024-미주사이20-2024.pdf` | `pdf/3-09월_교육_통합_2024-미주사이20-2024.pdf` | LFS pointer | 1,473,246 | 1,473,246 | `4aa311d6c3923d2c7cbea7e7ec1448c2e7b51d15cc376bad754d3a8cfcc78aa2` | 정상 | 동일 |
| `pdf-large/hwpx/143E433F503322BD33.pdf` | `pdf/hwpx/143E433F503322BD33.pdf` | LFS pointer | 147,003 | 147,003 | `6ea409794cedd1e75dabf36558ebd85429fc27e86f4d198f2f8eb2cb8097983d` | 정상 | 신규 |
| `pdf-large/hwpx/2026_oss_rst.pdf` | `pdf/hwpx/2026_oss_rst.pdf` | Git blob | 171,288 | 171,288 | `bec53a601cc7714a40ca340d26f971d1ab49eeb355682fc9db7b15cd5e04c86e` | 정상 | 신규 |
| `pdf-large/hwpx/[2027] 온새미로 1 본교재.pdf` | `pdf/hwpx/[2027] 온새미로 1 본교재.pdf` | LFS pointer | 1,255,445 | 1,255,445 | `72bc3cb076865b73cc021d9ad077bccd80cbc311085d49daace62073903b0b8d` | 정상 | 신규 |
| `pdf-large/hwpx/el-school-001.pdf` | `pdf/hwpx/el-school-001.pdf` | LFS pointer | 74,746 | 74,746 | `cc2e77044e31a99f0c611172b2959a757b8101977b93455ead521e800f372a72` | 정상 | 신규 |
| `pdf-large/hwpx/eq-002.pdf` | `pdf/hwpx/eq-002.pdf` | LFS pointer | 23,807 | 23,807 | `697aa5da4fbc6a240fb885bb0ac9b78f4301adf7a6560c4917e418018d244d8b` | 정상 | 신규 |
| `pdf-large/hwpx/footnote-tbox-01.pdf` | `pdf/hwpx/footnote-tbox-01.pdf` | LFS pointer | 14,537 | 14,537 | `7f10f3040fb3c18ddca21925b8a8ad074a886743478dd002d02ebfca5d1f6b97` | 정상 | 신규 |
| `pdf-large/hwpx/hcar-001.pdf` | `pdf/hwpx/hcar-001.pdf` | LFS pointer | 258,069 | 258,069 | `314ff5e27096103542ea00af74b48148478bb4852fb71550c57075c92acd8b3c` | 정상 | 신규 |
| `pdf-large/hwpx/hy-001.pdf` | `pdf/hwpx/hy-001.pdf` | LFS pointer | 126,209 | 126,209 | `5effd47b455c48399a40f1924b61df9828d1e5f57a5bdf6fb76b7b165c732ef7` | 정상 | 신규 |
| `pdf-large/hwpx/hy-002.pdf` | `pdf/hwpx/hy-002.pdf` | LFS pointer | 112,893 | 112,893 | `542e9b3a2ad3629ed0c400f6cd0c4ba1f819accebaebacc6228543b3372865b0` | 정상 | 신규 |
| `pdf-large/hwpx/issue_1133.pdf` | `pdf/hwpx/issue_1133.pdf` | LFS pointer | 131 | 110,523 | `ca066fdc5cda9fd8c47c9a29ba9a546e70c6acdaa96df4d6c66e0b8e02cdbc45` | 정상 | 신규 |
| `pdf-large/hwpx/k-water-rfp.pdf` | `pdf/hwpx/k-water-rfp.pdf` | LFS pointer | 1,451,239 | 1,451,239 | `16943d1ecdd167225d9f54bd96956a49649d358b22d20643a9e283eab49e4ee7` | 정상 | 신규 |
| `pdf-large/hwpx/math-001.pdf` | `pdf/hwpx/math-001.pdf` | LFS pointer | 101,699 | 101,699 | `3a5173e70db9b3d10a48a63ba830a47ccf28d0307b11a0fe2e2aa995a6e59d4a` | 정상 | 신규 |
| `pdf-large/hwpx/shape-001.pdf` | `pdf/hwpx/shape-001.pdf` | LFS pointer | 9,681 | 9,681 | `cfdba58aeab6e848dce48e350fd8122e6316c2edd8ef1cbabf102261fac0b930` | 정상 | 신규 |
| `pdf-large/hwpx/ta-pic-001-r.pdf` | `pdf/hwpx/ta-pic-001-r.pdf` | LFS pointer | 229,401 | 229,401 | `99b0f221559be71d55fb7ab82a35c6ad736c1fd85bbb7c5dd24cfb76b054aa4b` | 정상 | 신규 |
| `pdf-large/hwpx/tb-org-02.pdf` | `pdf/hwpx/tb-org-02.pdf` | LFS pointer | 38,942 | 38,942 | `494dc742b0b2820cbf2ea1a1d446a7a268f536a2bca84551b51ab4e55489b199` | 정상 | 신규 |
| `pdf-large/issue2006/1790387_prep_final_report-2022.pdf` | `pdf/issue2006/1790387_prep_final_report-2022.pdf` | LFS pointer | 50,228,784 | 50,228,784 | `226cd9b10e41394da09d96ce09eaa50f1b6c919952cecc9af87f2f18d6ce22d7` | 정상 | 신규 |

LFS pointer 17개의 선언 크기 합과 실제 object 크기 합은 모두 57,128,473 bytes다.
`pdf-large/`의 실제 PDF 합은 일반 blob 171,288 bytes를 더한 57,299,761 bytes이며,
`pdf-2020/`까지 합치면 58,159,986 bytes다.

## 4. 충돌 판정

### 4.1 상대경로와 목적지

- 19개 source가 만드는 목적지 경로는 19개로 서로 겹치지 않는다.
- 그중 2개 목적지만 기존 `pdf/`에 이미 있다.
- 두 파일 모두 기존 목적지와 크기·SHA-256이 같으므로 새 bytes를 쓰지 않고 source만 제거한다.
- 나머지 17개 목적지는 존재하지 않는다.

### 4.2 basename

`pdf/**` 전체에서 각 source basename을 재귀 검색했다. 위 동일 2건 외에는 같은 basename이 없고,
같은 basename인데 내용이 다른 경우도 0건이다. 따라서 경로 보존 이동으로 의미가 다른 오라클을
덮어쓸 위험은 발견되지 않았다.

### 4.3 상한

최대 파일은 `1790387_prep_final_report-2022.pdf` 50,228,784 bytes다. 계획의 엄격한 상한
52,428,800 bytes보다 2,200,016 bytes 작다. 19개 모두 상한 안이다.

## 5. 활성 소비자 지도

검색 기준은 실행 코드·workflow·test·현행 문서에서 `pdf-2020` 또는 `pdf-large`를 직접 읽거나
경로 정책으로 판정하는 파일이다. 생성기는 정본 source와 생성물을 함께 바꿔야 하고, 완료된 작업의
서술 이력은 당시 경로를 보존한다.

| 분류 | 수기 정본·실행 소비자 | 생성물·처리 원칙 |
| --- | --- | --- |
| GitHub workflows | `.github/workflows/{ci,codeql,deploy-pages,render-diff,adapter-diff,proptest-roundtrip,oracle-public-advisory}.yml` 7개 | 앞 6개는 PDF-only fast-pass 경로를 `pdf/**` 하나로 축소한다. advisory는 `include_large`, LFS checkout, 세 root 분기를 제거한다. |
| CI 정책과 mirror test | `scripts/ci-impact-classifier.cjs`, `scripts/ci-impact-policy.cjs`, `scripts/verify-trusted-postmerge-ci-reuse.mjs` 및 직접 참조 test 5개 | 코드와 mirror test를 같은 commit에서 고친다. 인접 workflow 계약 test도 함께 실행한다. |
| Oracle Public 정본 | `oracle_resolver.py`, `page_smoke.py`, `coverage_report.py`, `multiver_index.py`, `sweep_runner.py`, README·CI 안내와 관련 test/expected fixture | `oracle_pairs.json`, `fixtures/pairs/**`, `catalogs/**`, `reports/**`, `transcripts/**`, `drafts/examples/**`는 수기 치환하지 않고 `oracle_resolver.py`, `multiver_index.py`, `fatten_catalog.py`의 정식 명령으로 재생성한다. 직접 폐기 경로를 포함한 생성물은 최소 28개이며 동일 묶음의 sibling 산출물도 manifest와 함께 갱신한다. |
| LLM verifier 정본 | `oracle_vs_self/contracts.py`, `generate_corpus.py`, resolver/page-smoke 계약 fixture 2개 | `corpus/shard_0000.tsv`~`shard_0015.tsv`와 `corpus/manifest.json`은 generator로 전량 재생성하고 verifier를 실행한다. 세 root는 실제 파일 경로 계약이므로 `pdf` 하나로 축소한다. |
| 기타 실행·검증 | `tools/sparse_clone_hint.py`, `tools/layout_anomaly/advisory_samples.txt`, `tests/issue_1052_footnote_in_textbox.rs` | sparse preset, 설명, Rust test의 정답지 경로를 새 위치로 바꾼다. Rust test 변경이므로 최종 Rust 필수 gate 대상이다. |
| 살아 있는 자산 안내 | `pdf/README.md`, `pdf-large/README.md`, `samples/issue2006/README.md`, `pdf/issue5447/README.md`, `pdf/pr3740/README.md` | `pdf-large/README.md`의 유효 규칙을 `pdf/README.md`에 통합한 뒤 폐기하고, 실제 링크·명령을 새 경로로 바꾼다. |
| 현행 운영 문서 | `mydocs/manual/github_operations.md`, `pr_review/review_only_fast_pass.md`, `pr_review/visual_fixture_evidence.md`, `publish_guide.md` | 단일 경로·일반 Git blob·50 MiB 상한·PDF-only fast-pass를 새 정본으로 삼는다. |
| 현행 기술·장애 문서 | `mydocs/tech/investigations/issue-1251/hwp_ole_chart_visual_diff_against_hancom_pdf_1251.md`, `mydocs/tech/multicolumn_object_flow_backfill.md`, `mydocs/troubleshootings/hwpx_equation_save_eqedit_spec_errata.md` | 이동되는 실제 정답지 링크와 재현 명령만 새 경로로 고친다. 조사 판정 자체는 바꾸지 않는다. |
| 역사 기록 | 과거 CHANGELOG, PR archive, 완료 plan/working/report, orders, release feedback | 당시 경로 표기는 bulk rewrite하지 않는다. 현재 파일을 직접 가리키는 깨진 Markdown 링크만 링크 정합성 범위에서 교정하고, memory 문서는 새 정본을 가리키는 superseded 안내를 덧붙인다. |

Oracle Public의 source/test 중 폐기 경로 직접 참조는 14개, 생성물은 28개다. LLM verifier는
정본·계약 fixture 4개와 생성 shard 16개에 직접 참조가 있다. 숫자는 변경 범위의 하한이며,
정식 재생성으로 함께 바뀌는 manifest나 같은 bundle의 sibling 파일을 누락시키지 않는다.
`fatten_catalog.py`는 더 이상 생성되지 않는 과거 `by_root/pdf-2020.jsonl`·`pdf-large.jsonl`을
자동 삭제하지 않으므로 재생성 뒤 이 두 stale 산출물은 명시적으로 제거한다.

## 6. 보호 불변식 확인

- [x] 19개 old/new path와 actual SHA-256이 고정됐다.
- [x] 17개 LFS object의 OID·선언 크기·실제 크기·magic이 모두 일치한다.
- [x] 19개 목적지는 유일하며 hash 불일치 충돌이 없다.
- [x] 기존 동일 목적지 2개는 덮어쓰지 않고 보존한다.
- [x] 19개 모두 50 MiB 미만이다.
- [x] 실행 정본, 생성물, 현행 문서, 역사 기록의 처리 경계를 구분했다.
- [x] 공개 이력 rewrite나 원격 LFS purge는 수행하지 않았다.
- [x] PDF bytes와 `.gitattributes`는 변경하지 않았다.

## 7. Stage 2 진입 조건

메인테이너가 이 원장과 소비자 지도를 승인하면 다음 commit 절편에서만 아래를 수행한다.

1. `issue_1133.pdf`를 검산된 local LFS object에서 실제 PDF bytes로 복원한다.
2. 17개 신규 목적지를 만들고 byte-identical 2개 source는 기존 목적지를 보존한 채 제거한다.
3. 이동 직후 이 원장의 19개 SHA-256을 다시 전건 대조한다.
4. `pdf/README.md`, `.gitattributes`, `CONTRIBUTING.md`와 신규 PDF repository policy 검사기를
   같은 원자 전환에 포함한다.

Stage 2 전에는 원격 push나 PR을 만들지 않는다.
