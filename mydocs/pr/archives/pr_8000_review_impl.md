# PR #8000 적용 기록

## 원 기여와 적용

- 원 [PR #8000](https://github.com/edwardkim/rhwp/pull/8000)의 head
  `01d775800d0473205899dd24f3adcd39b57c9707` → `5ea1b0d24755060b3bd7981c6c9e8ff4977b6cc0`.
- 기준 `4296fa045df2743a0eaa1b0cc71b2a3c1dd90bb6`, 작업 branch `review/donggyun112-20261018`.
- 원 author `donggyun112`와 `cherry-pick -x` 원 SHA를 보존했다. 이후 #8002 5개 commit을 순차 누적했고 충돌은 없었다.
- 원 검색 알고리즘의 추가 보정은 없다. API schema·인자는 그대로이고 비중복 주소 계약의 동작 변화는
  [개별 검토](pr_8000_review.md)에 명시했다.

## 코드와 소비 경로

[src/document_core/queries/search_query.rs](../../../src/document_core/queries/search_query.rs)의
`find_in_text`가 original scalar 시작·끝을 가진 비중복 span을 만든다. 대소문자 무시 경로는
소문자화 확장 인덱스에서 원문 경계로 환산한 뒤 같은 원문 span 중복을 거른다.
각 컨테이너의 `find_matches` 결과를 search·grep·nth/all replace가 소비해 검색 주소와 삭제 길이가 일치한다.
다른 컨테이너의 동일 모양·좌표를 같은 소유로 합치지 않는다.

독립 기대값은 `......`를 왼쪽부터 `...` 두 번으로 치환하는 원문 분할·뒤 단어 보존이며 구현 길이를 재인용하지 않는다.
기준 실제 CLI는 성공·4건에도 `검토…원문 끝.`으로 `완료`를 잃었다. 현재 실제 CLI/MCP는
2건·`검토……완료 원문 끝.`과 저장 후 suffix를 보존한다. dry-run은 원본을 변경하지 않고 같은 2건을 보고한다.

[정식 3검사](../../../tests/cases/issue_7999_nonoverlapping_replacement.rs)는 문장 부호·길이 증가,
search/grep/n번째 공통 주소, Unicode 확장과 본문/셀/글상자/수식 독립 범위를 검사한다.
실제 fresh WASM 검색·치환·저장 4개 대조군도 PASS다. 물리 배치·px golden은 추가하지 않았다.

## 공통 메인터너 보정의 분리

#8002의 CRLF 소비 지점인 Studio 커서·Undo 보정과 추가 검사는
[#8002 적용](pr_8002_review_impl.md)에 기록했다. 이 보정을 #8000 원 알고리즘 결함으로 합치지 않는다.
reviewer 선행 지정 규칙은 `3868f40b99090d921bfa8357f2c2cf8453e6b57f`에서
최소 분류 → 원 PR별 배정/API 확인 → 상세 조사·fetch·체리픽 순서로 구체화했다.

## 공통 필수 검증과 후속 조건

실제 전체 nextest **10,831 PASS / 0 FAIL / 기존 45 SKIP**, focused 15 PASS,
필수 fmt·Native/WASM·workspace Clippy와 build, Studio 2,328 PASS·타입·production build,
editor 36 PASS, 실제 CLI 8개·stdio MCP, fresh WASM Chrome 51 assertion을 확인했다.
정확한 code head·CI run과 초기 실패 이력·비해당/미검증 범위는 해당 개별 review에 연결한다.
검증한 제품 바이트는 이후 검사/문서 commit에서도 동일하다. 문서 trailing 때문에 전체 Rust·WASM을 반복하지 않는다.

통합 [PR #8004](https://github.com/edwardkim/rhwp/pull/8004)의 위 code head에서
[Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/39059657983) · [CI](https://github.com/edwardkim/rhwp/actions/runs/39059658110) · [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/39059658256) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/39059657994) · [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/39059658020) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/39059657909)가 모두 실제 완료·성공했다.
Native Skia·프런트엔드 패키지·lint·Archive A/B/C/D 회귀와 최종 Build & Test도 성공했다.
전체 check 35개 결과는 neutral 1개, skipped 4개, success 30개이며 미완료·실패 0개다.
해당 PR merge ref의 열린 CodeQL alert도 0건이었다. skipped를 실제 검사 실행 성공으로 바꾸어 적지 않았다.

다음 단계는 같은 PR의 문서 trailing 최신 CI·mergeability 확인 → 정상 merge → duration 결과·devel sync →
관련 이슈 상태/증거 댓글 → 원 PR의 개별 감사·통합 완료 close → 소유 임시 branch와 root output 전체 안전 정리다.
원격 후속 처리는 아직 완료로 단정하지 않는다. 영구 실행 영수증은 `/home/tsjang/rhwp-pr-evidence/pr-8004/`로 보존한다.
