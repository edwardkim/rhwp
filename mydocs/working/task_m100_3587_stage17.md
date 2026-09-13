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
