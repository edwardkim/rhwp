# #7902 7단계 — #7935 통합 후 전쪽 재검증과 도구 PR 종료

Issue: #7902. [6단계](task_m100_7902_stage6.md)의 PNG alpha 수정은 별도 PR로 병합했다.
renderer의 원래19문서·835쪽 범위는 유지하며 이 단계에서도 #7902 완료나 PR 준비 완료로 판정하지 않는다.

## upstream 확인과 실제 source

- 최초 fetch에서 devel은 `e4e6d0c8d83f688e376043c639e4fe5842169c2d`(#7935)였다.
  해당 변경을 반영한 renderer head `4b14bd1e11505c842caaa1e21126f0a87b51a575`에서 재검증했다.
- 도구 #7936(`0c2b60b410af00d26111dc5453477afec091c56e`)와 문서 #7937
  (`f5251e7de4e31ee0fb939f352c6ef144bb340bdb`)가 병합되어 devel SHA가 실제 변경된 뒤에만
  다시 리베이스했다. 결과는 `bf91a6dced72dc1ace519a9bc24b607134b428b2`다.
- `src`·`crates`·Cargo.toml/lock·build.rs·Studio src는 실제 producer source `4b14bd1e1`와
  리베이스 head가 동일하다. base의 같은 입력도 `e4e6d0c8d`와 `f5251e7de4`가 동일하다.
  새로 병합된 alpha 비교 도구는 아래 네 번의 전쪽 실행에서 이미 같은 바이트로 사용했다.
- 사용자 dirty `rhwp-studio/public/rhwp.js`는 autostash 적용 뒤 SHA-256
  `fd499c9eb514018d2063c8eef1e58ec892c734d512cf5e88ab97bd9e5d47a0a3`를 유지했다.
  stage/commit하거나 덮어쓰지 않았다. 로컬 devel은 ancestry 확인 후 최신 upstream으로 fast-forward했다.

## 공유 target의 source 재사용 오류 확인·수정

base worktree 빌드 뒤 기본 작업공간 빌드가 `Fresh`로 끝나고, 서로 다른 src tree인데 Native/WASM
바이트가 같았다. 이 candidate 산출물은 즉시 검증에서 제외했다. 공유 target을 삭제하지 않고
기본 작업공간 src의 mtime만 갱신해 Native·WASM 순서로 실제 재컴파일했다. 소스 바이트는 그대로다.
이후 Native3분13초·WASM4분23초에 성공했고 base와 다른 해시를 확인했다.

| 역할 | 실제 source | Native SHA-256 | WASM SHA-256 |
| --- | --- | --- | --- |
| base | `e4e6d0c8d` | `d25b3b4d422bbf3872ce30956b140ab1017c58b43f4406c8d6830abc8ed75eed` | `f4b3f4c506b9a3e3b0f009f13f79f115874e204235920b097fc14e208d60bbd4` |
| candidate | `4b14bd1e1` | `1cb40b5b7ed3579c37b435158b5f22bb4b4e97c85a90072ff0f521d11ef77a3a` | `59911a5e3fb77a8568b75a832d981ac6d3562b5ea748e919c2b0ca992c202a5c` |

공유 `/home/tsjang/rhwp/target/pr-review`, Cargo jobs8을 유지했다. Cargo는 순차 실행했고
`CARGO_INCREMENTAL=0`이나 캐시 초기화는 사용하지 않았다. 잘못 재사용한 candidate는
통과 증거로 세지 않으며 별도 verified 산출물만 아래 실행에 사용했다.

## 전쪽 재검증 결과

원본·독립 Print·compat·96dpi·글꼴·PNG exporter·실루엣 판독을 고정했다. Native와 WASM은
동시에 비교하되 base와 candidate 단계는 순차 실행했고, 각 backend는 문서2개/페이지 작업자2개다.
같은 alpha-aware 도구로 최신 base와 candidate의 총3,340페이지 쌍을 새로 렌더링·측정했다.

| candidate | 문서·쪽 | 최저 일치율 | 90% 미만·누락 | 독립 PDF 쪽수 |
| --- | --- | --- | --- | --- |
| Native | 19·835 | 91.39728% | 0·0 | 전부 일치 |
| fresh WASM | 19·835 | 90.20061% | 0·0 | 전부 일치 |

Decimal로 devel 대비0.01%p 이상 하락을 확인한 결과는 다음 한 페이지의 두 backend다.
나머지 페이지의 해당 하락은 없다. 90%·98% 통과로 이 차이를 면제하지 않는다.

| 입력·쪽 | backend | devel | candidate | 하락 |
| --- | --- | ---: | ---: | ---: |
| nested-hwpx104 | Native | 99.95853% | 99.94062% | 0.01791%p |
| nested-hwpx104 | WASM | 99.94545% | 99.92986% | 0.01559%p |

실제 입력은 `samples/issue1949_giant_cell_nested_tables_perf.hwpx`, 독립 기준은
`pdf/issue1949_giant_cell_nested_tables_perf-hwpx-2020.pdf`다. 새 producer의104쪽 review PNG를
양쪽 모두 생성해 표→빈 줄/제목→그림 앞 문단→그림→마지막 문단을 직접 확인했다.
전쪽 확대 판독 완료로 승격하지 않는다. **renderer PR은 보류**한다.

## 추가 CFF 진단 — 제품 미적용

독립 표본에서 공급한 실제 CFF 윤곽을 같은 논리 원점·크기로 Canvas path에 재생하는 진단을
실행했다. PDF 픽셀·좌표에 맞춰 입력을 이동/스케일하거나 마스크를 바꾸지 않았다.
보통 글자 하나의 실제 첫 family·cmap가 일치하는 경우만 대상으로 했고 나머지는 원래 text paint다.

- 전체 공급 CFF를 먼저 펼친229MiB 요청은 Chromium target 종료로 실패했다. fidelity 증거가 아니다.
- 현재 입력의 render tree 문자로만 subset한 재실행은23.456초에 성공했다. 대상 PDF glyph를
  추출해 넣지 않았고 원본 공급 글꼴을 변경하지 않았다. 실제 path paint125회다.
- 같은104쪽 WASM의 별도 진단은99.93860%였다. 실제 제품 source의99.92986%와 구분한다.
  Native 적용, 일반/비적용 대조군, 호스트가 선택한 실제 글꼴 바이트와 painter의 소비 경로,
  전체 무회귀는 아직 검증하지 않아 이 진단을 제품 보정·PR 허용 근거로 채택하지 않는다.
- 다음 분석은 글꼴 API의 raster와 Type3의 실제 윤곽 재생 차이, 실제 face 바이트의 전달/소유,
  측정·배치·paint 계약을 대조한다. 문서 ID·점수·우연한 속성 조합으로 분기를 만들지 않는다.

## 도구 PR과 후속 처리

[#7936 개별 review](../pr/archives/pr_7936_review.md)·오늘할일은 Full CI 성공 뒤 옵션2의
#7937에 반영했다. 원 code head의34개 check/status 및 문서 PR의 실제
`all-review-only-no-code-impact`·required checks를 각각 확인한 뒤 일반 squash merge했다.
관리자 우회와 문서만 변경한 뒤의 로컬 Rust 전체 재실행은 사용하지 않았다.
#7936의 duration 갱신은 원 PR worker 증거로 성공했다. #7902는 열어 두고 renderer 검증을 계속한다.

ignored 증적 루트는 `output/pr-review/issue7902-render-repair-20261014/`다.
`rebase-7935-candidate-rebuild-result.json`, `rebase-7935-{base,candidate}-full-{native,wasm}-result.json`,
`rebase-7935-develop-comparison.json`, `rebase-after-pr7936-7937.json`,
`cff-path-replay-subset-result.json`에 SHA·명령·TSV·입력/PDF 해시·직접 생성한 PNG를 연결했다.
로그·TSV·중간 JSON·공급 글꼴은 커밋하지 않는다.
