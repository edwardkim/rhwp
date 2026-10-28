# #3587 Stage 17 — D1 자원 준비와 경계 삽입

- 승인: 메인테이너 「승인합니다」. [D 계획](../plans/task_m100_3587_impl_d.md)의 후속 구현.
- 시작점: `aacda966b`, 제품·테스트 `1b3648aac`. 작업 브랜치 `task_m100_3587`.
- 범위: 도달 자원 매핑, detached 블록의 참조 재작성, 단일 경계 삽입과 native 계약 검증.
- 보호 조건: source 불변, 대상 기존 서식·용지·앞뒤 문단 보존, 누락/상한 오류의 부분 반영 금지,
  A 신원 할당기 재사용, 기존 대화형 paste 및 조판 규칙 변경 없음.
- #7065·#7084·#7090 수정, 공개 WASM/CLI/MCP 연결, Gym, remote push/PR은 이번 절편에서 수행하지 않는다.

## 구현 결과

native `import_paragraph_block_native`와 동일 준비 경로의
`preview_paragraph_block_import_native`를 추가했다. Stage 16의 `inspect_*`는 계속 구조 선검증이며
완료된 dry-run을 뜻하지 않는다. 새 preview는 자원 준비·복사본 ID·반환 경로까지 만든 뒤 버린다.

- B의 도달 자원 검사를 수집에도 재사용한다. 지원 범위·참조 종류를 별도로 추측하는 순회를 추가하지 않았다.
- target의 DocInfo 전체나 Document를 복제하지 않고, 추가할 항목만 detached delta에 준비한다.
  의존 참조를 먼저 매핑하고 의미가 같은 기존 항목은 재사용한다. 미사용 source 정의는 가져오지 않는다.
- `next_style_id`의 자기/상호 참조는 그래프 대응을 확인하거나 새 번호를 모두 예약한 뒤 연결한다.
  재사용한 target 스타일의 다음 스타일 번호를 수정하지 않는다.
- 그림 ordinal과 내장 글꼴 storage ID를 별도 참조로 처리한다. 바이너리는 제한 읽기만 사용하고,
  실제 바이트와 metadata가 맞아야 재사용한다. 같은 source 바이트는 한 번만 읽고 target 비교 읽기도
  합계 한도에 넣는다. 새 그림 참조는 renderer의 ordinal과 HWPX writer의 storage 해석이 일치해야 한다.
- 서식·그림 참조는 A와 같은 소유 트리 순회로 재작성하고, 개체 신원은 A 할당기로 한 번만 발급한다.
  빈 글자모양 목록의 source 0 fallback은 대상 0과 혼동하지 않도록 명시적인 대응 run으로 보존한다.
- B의 commit을 예약과 삽입으로 나눠 공통 사용한다. 자원·문단·응답 준비와 복구 가능한 예약 실패는
  삽입 전에 끝낸다. 실제 실행은 DocInfo dirty, 필요 시 BinData epoch, resolved styles, 구역 raw,
  기존 재조판·이벤트 경로를 사용한다. 실패와 preview는 source/target 의미와 이벤트를 바꾸지 않는다.
- ClickHere guide residue의 글자모양도 도달 자원 검사에 포함했다. 기존 대화형 paste는 변경하지 않았다.

## 검증과 정정 이력

- 제품 구현: `037303d3c`, 참조 경계 보강: `1f15f35b8`.
- 최종 검증 제품·테스트 SHA: **`cc9f49a8f34a9d47fd67a3f5ce1b6701ef41f528`**.
- 동일 SHA의 review worktree `/home/edward/mygithub/rhwp-review-3587`에서 파생 suite를 준비했다.
  target은 `/home/edward/mygithub/rhwp/target/pr-review`를 그대로 사용했다.
- 집중 실행 **169 PASS / 실패 0**. 새 native 가져오기 계약 11건과 기존 구조 선검증,
  A/B/C, 외부 paste, #4275 중첩 표, #5819 표 생성, raw 무효화 가드를 포함한다.
  선택식은 `test(issue_3587) | test(foreign_paste) | test(issue_4275) | test(issue_5819) | test(passthrough_invalidation)`이다.
- 수동 산출물 내보내기 ignored test를 명시 실행해 **1 PASS**. 테스트 배정 규칙의 Node 검사 **23 PASS**.
- `cargo fmt --all -- --check`, manifest `--check` PASS.
  manifest는 1,291 source / 5,558 static test attr / 48 integration target이다.
- `cargo clippy --locked --target-dir /home/edward/mygithub/rhwp/target/pr-review -- -D warnings`
  **PASS**(49.66초). WASM32·workspace all-target Clippy,
  전체 회귀, Native Skia, Docker WASM 및 한컴 시각 판정은 아직 수행하지 않았다.

처음 source checkout의 오래된 ignored generated suite 때문에 전체 fmt가 경로 오류를 냈다.
source checkout에 파생물을 추가하지 않고, 최신 SHA를 옮긴 review worktree에서 prepare 후 확인했다.
최초 새 테스트 빌드는 존재하지 않는 `load_document` 호출 때문에 실패했고 실제 공개 API `from_bytes`로
정정했다. 다음 실행은 167 PASS / 2 FAIL이었다. 하나는 새 테스트가 세 표짜리 원본 묶음의 복제 결과를
2개 표로 잘못 기대한 오류(실제 6개)였다. 원본 표 3개를 확인하고 각각의 행·열·셀 내용과 독립 ID를
검사하도록 고쳤다. 다른 하나는 공통 commit에 위임하는 새 mutation 함수의 가드 등록 누락이었다.
실제 무효화 경로와 저장·재열기 근거를 등록했다. 기존 회귀 baseline·허용치를 완화하지 않았다.

로그는 `output/3587/d1-import-{focused-r3,export,manifest-r3,fmt-r3,manifest-policy,clippy-native}.log`에 있다.
nextest 0.9.137/권장 0.9.140 및 JUnit unknown key 경고는 기존 환경 경고다.
`9572 skipped`는 필터 제외/ignored 수이며 전체 회귀 통과 수가 아니다.

## 메인테이너 확인용 산출물

입력은 `samples/rnote/labnote-001.hwp`
(`8401e778edc386f0a87a0df4120a2cf0733aee36c84070ddc864499bde73a1af`)이다.
별도의 새 빈 문서를 대상으로 원본 문단 `[12,13)`의 **표 세 개 묶음을 두 번** 가져왔다.
대상 삽입 문단 범위는 `[1,3)`, 표는 총 여섯 개다. C 내용 채우기나 Gym은 실행하지 않았다.

기본 확인 파일:

- `output/3587/d1-import/cc9f49a8f/hwp-source.hwp`
- `output/3587/d1-import/cc9f49a8f/hwp-source.hwpx`

HWPX 입력 경로도 같은 원본을 rhwp로 변환하여 `derived-hwpx-source.hwp`,
`derived-hwpx-source.hwpx`로 내보냈다. **파생 HWPX이며 독립 한컴 정답지가 아니다.**
각 `.json`에는 원본 상대→대상 경로와 자원 집계를 기록했다.
HWP 입력 결과는 공유 서식 21개 추가/8개 재사용, 바이너리 추가 0개다. 이 실물 블록에는 가져올
그림이 없어 그림 검증은 현재 합성 계약 테스트에 한정된다.

한컴에서 확인할 사항은 정상 열림, 여섯 표의 내용·서식 보존, 대상 용지 유지다.
이번 실행을 한컴 조판 일치 판정으로 기록하지 않는다. #7090 등의 기존 간격 문제를 수정했다고 하지 않는다.

## 명시적인 지원 제한과 다음 순서

다음은 값을 바꾸거나 자료를 버려 성공 처리하지 않고 **삽입 전 오류**로 반환한다.

1. 참조 재작성이 아직 없는 HWPX numbering/bullet의 원본 `paraHead` XML.
2. 새 image ordinal이 기존 storage ID와 충돌하거나 target BinData 목록의 append 정합성을 확보할 수 없는 경우.
3. source의 암묵적 기본 개요가 target의 다른 명시적 개요로 바뀌게 되는 경우.
4. 외부 링크·빈/누락 바이트·제한 읽기를 지원하지 않는 resolver·범위 초과 참조 및 예산 초과.

**D1 전체 완료는 아니다.** 기본 native 가져오기 경로는 구현·집중 검증했으나 위 지원 경계의
보존형 처리 검토, 실물 그림/중첩 개체 및 한컴 확인이 남아 있다. 이후 D2 공개 경로·레시피,
D3 전체 통합 검증·비용 계측, 선택적 Gym 순서를 유지한다. remote push·PR·WASM 교체는 수행하지 않았다.

## 2026-09-13 메인테이너 실패 판정 및 한컴 PDF 대조

메인테이너는 위 `hwp-source.hwp`·`hwp-source.hwpx`가 **한컴에서 모두 비정상 조판**된다고
판정했다. native 구조 테스트 169 PASS를 시각 수용으로 해석하지 않는다.
아래는 당시의 관측 기록이며, **가져오기 구현 결함의 확정 판정이 아니다.**
메인테이너의 후속 지적대로 용지·여백이 다르면 배치도 달라질 수 있다. 좁은 본문에 원본 크기의
표를 가져왔을 때의 넘침·분리와, 보존된 명시적 쪽나누기로 생긴 빈 쪽을 구현 회귀로 판정한
해석은 철회한다. D1은 원래 남아 있던 실물 확인을 계속하며, 동일 용지 조건으로 재검증한다.
`bug-hunter`의 독립 출력·provenance·자기 일관성 검사 구분에 따라 해당 두 파일과 원본만 대조했다.
제품 코드, 테스트 기대값, source와 기존 출력 파일은 변경하지 않았다.

### 독립 PDF와 관측

기준 디렉터리: `output/3587/d1-import/cc9f49a8f/hancom-check/`.

| 입력 | 한컴 PDF | 관측 |
| --- | --- | --- |
| `samples/rnote/labnote-001.hwp` | `labnote-001-original-2020.pdf` | 총 2쪽. 2쪽에 제목→기록→서명 표가 함께 배치됨 |
| `hwp-source.hwp` | `import-from-hwp-2020.pdf` | 총 5쪽. 1쪽 빈 쪽, 2·4쪽 제목/서명 표, 3·5쪽 기록 표. 오른쪽 잘림 |
| `hwp-source.hwpx` | `import-from-hwpx-2020.pdf` | HWP와 같은 5쪽 증상. 779×1100 raster의 각 대응 쪽 SHA-256도 동일 |

PDF SHA-256은 순서대로 다음과 같다.

- 원본: `64faf53a99b93ea923bf98fd6f81ffc596a4c4aa2bcfa7cef96964d3281d9ac7`
- HWP 출력: `bf2ea7dbf4a4a9f08eee40f5046a42cd6a0fa34ae91e36c053874e3b757514ea`
- HWPX 출력: `73d8dd73cf60dc1e8768c07c60ed7fd875bceb4a69f0f8cd827be0db0996e518`

기존 출력 두 파일의 SHA는 각각 `4687f1dc64763e1e7b18f58a6d0537ccd9a011831ae336b0da8ede3e85daaa16`,
`471db332842debdc3752fb298bd4a7f5c9dfbeb16a656fd7c61dc856b71d22f3`로 변환 전후 보존했다.

변환은 매뉴얼의 비동기 start→status→download로 실행했다. 입력의 `info --json` 제품이
2024가 아니어서 명시적으로 `engine=2020`을 사용했다. 실제 backend는
`hwp-managed-direct-dll-host`, `hancom_version=12.0.0.4605`, worker 32bit,
`frame_print_to_pdf_ex_one_up` / print method 0, preprocess none이다.
PDF Creator는 `Hwp 2022 0.0.0.0`, Producer는 `Hancom PDF 1.3.0.550`, A4 595×841pt다.
서버 글꼴 등록은 2/2 성공, PDF의 임베드 글꼴은 Gulim이다. 서버 전체 설치 글꼴 목록은 미확인이다.
요청·완료 상태는 `conversion-evidence.json`, PDF 래스터는 `original-N.png`, `hwp-N.png`,
`hwpx-N.png`에 보존했다. 엔드포인트·토큰은 기록하지 않았다.

### 확인된 조건 차이와 제한 실험

`rhwp dump <파일> --para <번호>`로 원본 pi12와 출력 pi1을 대조했다.

- 원본: 좌우/상하 여백 10mm, 머리말/꼬리말 0mm, 본문 폭 약 190mm.
- 생성 대상: 좌우 30mm, 위 20mm/아래 15mm, 머리말/꼬리말 각각 15mm, 본문 폭 약 150mm.
- 세 표의 폭 약 188.0/188.8/188.0mm와 외곽 여백 각 1mm는 그대로다.
  왼쪽 기준이 20mm 밀려 오른쪽 종이 경계를 초과하는 조건을 확인했다.
- 원본 pi12의 명시적 쪽나누기가 대상 pi1/pi2에도 보존된다. 앞에 빈 pi0을 둔 생성 방식이
  첫 빈 쪽의 원인이다. 사용자 의도 확인 없이 쪽나누기를 삭제하지 않는다.
- 원본/출력의 세 표 기준점·offset은 같다. 기록 표의 dump 값 `4294960279`는
  signed i32로 −7,017 HU이며 거대한 양수 위치로 해석하지 않는다.
- 호스트 LineSeg vpos는 76,450→1,600 HU지만 segment_width 53,860은 그대로다.
  서로 다른 본문 폭에서 저장 조판 정보의 유효성이 유지되는지는 추가 검토 대상이다.

직접 원인 경로는 `tests/cases/issue_3587_block_import.rs:477`의 기본 빈 문서 생성과
`:479`의 원본 pi12 가져오기다. import 준비 경로는 `original.to_vec()`로 원본 문단 속성을
유지하고 서식/신원만 재매핑한다. D 계획은 대상 용지 보존을 명시한다. 따라서 이번 확인용 생성
레시피가 원본 서식에 맞는 대상 본문 영역을 마련하지 않은 문제를, 곧바로 서식 ID 매핑 손상이나
HWP/HWPX 직렬화기 전체 결함으로 단정해서는 안 된다.

기존 CLI `edit set-page-def`로 **별도 진단용 사본**에만
`{"marginLeft":2834,"marginRight":2834,"marginTop":2834,"marginBottom":2834,"marginHeader":0,"marginFooter":0}`
를 지정했다. 입력은 위 실패 출력 두 파일이고 출력은 `diagnostic-source-margins.hwp`/`.hwpx`다.
이 명령은 여백뿐 아니라 기존 본문 reflow/저장도 수행하므로, 바이트 수준의 여백만 바꾼 실험은 아니다.
표 크기·offset·ID를 직접 수정하거나 제품 코드에 조건을 추가하지 않았다.

`diagnostic-hwp-source-margins-2020.pdf`, `diagnostic-hwpx-source-margins-2020.pdf`는 각각
3쪽이며 2·3쪽에 세 표가 한 묶음으로 배치된다. 오른쪽 잘림과 기록 표의 별도 쪽 분리가 해소됐다.
두 형식의 대응 raster SHA도 동일하다. **첫 빈 쪽은 유지되고 제목 표 시작 위치도 원본과 차이가
있으므로, 원본과 완전 동일 또는 메인테이너 시각 통과라고 판정하지 않는다.**
상태/해시는 `control-conversion-evidence.json`, 이미지는 `control-hwp-N.png`, `control-hwpx-N.png`다.

다음 작업은 정상 배치용 대상 문서 조건과 명시적 쪽나누기 보존 의도를 먼저 정리하는 것이다.
대상 용지를 몰래 바꾸거나 표를 자동 축소하는 구현은 승인된 D 계약이 아니다. 먼저 올바른 생성
레시피로 다시 한컴 판정을 받아야 하며, 별개의 좁은 대상 문서 시험은 호환성/넘침 검증으로 구분한다.
이번 턴은 PDF 진단과 기록 보완까지만 수행했다. 새 이슈·댓글·커밋·push·PR은 수행하지 않았다.

## 동일 용지 조건 재검증 — 2026-09-13 후속 승인

메인테이너가 「문서 용지 크기와 설정이 다르면 당연한 결과」라고 지적했고,
동일 조건으로 재검증하는 절차를 승인했다. 위의 배치 차이를 가져오기 결함으로 해석했던
판정은 정정했다. 제품 구현과 기존 계약 테스트의 기대값은 변경하지 않았다.

### 생성 절차와 검증 범위

기존 수동 산출 helper에 `materialize_labnote_foreign_import_matching_page` ignored test를
추가했다. 기존 기본 용지 내보내기는 그대로 남긴다. 이번에는 **가져오기 이전**의 별도 빈
대상 문서에 source의 PageDef를 공개 setter로 적용한다. 용지 너비·높이·좌우/상하/머리말/
꼬리말/제본 여백·방향·제책 방법을 가져오며 source DocInfo나 본문을 대상 문서로 복제하지 않는다.
PageDef 전체의 직렬화 값이 원본과 같음을 검사하고, import 전후 대상 PageDef도 동일함을 검사한다.
원본 크기 59,528×84,188 HU, 사방 여백 2,834 HU, 머리말/꼬리말 0 HU를 확인했다.

원본 문단 `[12,13)`을 대상 `[1,3)`에 두 번 가져온다. 원본 명시적 쪽나누기와 앞의 빈 pi0은
보존한다. 따라서 첫 빈 쪽은 예상 결과이며 임의로 삭제하지 않는다. 새 출력은 예전 출력에
여백을 사후 수정한 파일이 아니다. 대상 준비→가져오기→HWP/HWPX 저장을 처음부터 실행했다.
가져오기 전 빈 대상도 `hwp-source-blank-target.hwp`/`.hwpx`로 보존했다.

- 제품 기준 `cc9f49a8f34a9d47fd67a3f5ce1b6701ef41f528`은 그대로다.
- 변경한 test source SHA-256:
  `d4e6d7d8a415c53bc6c8a94a000ca94968d2ededca7b9ead663fb209f8bf044f`.
- review worktree에 같은 test 변경을 적용하고 prepare 후 실행했다.
  자동 배정이 suite008→suite019로 바뀌었다. 최초 과거 suite 번호 실행은 **0건/exit 4**였으며
  통과로 세지 않고 생성된 실제 배정을 확인해 suite019로 재실행했다.
- 새 수동 산출 test **1 PASS**, 기존 import 계약 **11 PASS**.
- `cargo fmt --all -- --check`, manifest `--check`, `git diff --check` PASS.
- 전체 회귀·전체 Clippy·WASM 빌드·원격 작업은 이번 제한 검증에 포함하지 않았다.

재현 명령(review worktree, prepare 후 현재 배정 기준):

```bash
RHWP_3587_IMPORT_OUTPUT=/home/edward/mygithub/rhwp/output/3587/d1-import/same-page-20260913 \
cargo nextest run --locked --cargo-profile release-test \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review \
  --test regression_suite_019 --run-ignored only \
  -E 'test(materialize_labnote_foreign_import_matching_page)'
```

### 한컴 확인용 파일과 관측

디렉터리: `output/3587/d1-import/same-page-20260913/`.

| 파일 | SHA-256 |
| --- | --- |
| `hwp-source.hwp` | `77981e5d199f31e5cdf4914d3d577413b3df7e413aae01fc9461749b94af69ab` |
| `hwp-source.hwpx` | `beef3fb11204b1897160bfcf1cffee5fde893f85257e4b67ee18c020491ebc92` |
| `same-page-hwp-2020.pdf` | `4c087a4b81ef89fe62bc2c84ecf169f458e47f514252392331e2011839a3b93f` |
| `same-page-hwpx-2020.pdf` | `76cc7b72f3290d8c9cefff992c12b1e4f3350720e6e142459118a2cacec726bf` |

MCP 요청은 명시적인 engine 2020, 실제 backend는 Hancom `12.0.0.4605`, 32bit,
`frame_print_to_pdf_ex_one_up`, preprocess none이다. PDF Creator는 `Hwp 2022 0.0.0.0`,
Producer `Hancom PDF 1.3.0.550`, A4 595×841pt, 임베드 글꼴 Gulim이다.
서버 글꼴 등록은 2/2 성공했고 전체 설치 목록은 미확인이다. 요청·완료 상태는
`conversion-evidence.json`에 보존했다.

두 PDF 모두 3쪽이며, 2·3쪽에는 각각 제목→기록→서명의 세 표가 함께 배치된다.
오른쪽 잘림과 기록 표의 별도 쪽 분리는 관측되지 않는다. 대응 raster SHA도 동일하다.
원본 PDF는 앞 절에서 확보한 것을 재사용한다. 원본의 제목 표 시작 위치와 차이가 있는 관측은
유지하되, 원본 문서 전체와 빈 대상은 선행 문단 문맥까지 같지는 않으므로 이를 곧바로 구현
결함으로 판정하지 않는다. 최종 시각 판정을 메인테이너에게 요청하며 전체 일치를 자동 선언하지 않는다.

메인테이너는 두 형식 모두 첫 쪽이 비고 2쪽부터 복사 표가 배치됨을 확인했다.
이는 명시적 쪽나누기 보존 관측으로 기록하며, 전체 시각 통과로 확대하지 않는다.
이후 「다음 절차를 진행하세요」에 따라 D1의 남은 실물 중첩·그림 자원 검증을 Stage 18에서
진행한다. 현재 증적과 확인용 helper 변경을 먼저 로컬 커밋하고, 공개 연결/D2·remote push는 하지 않는다.
