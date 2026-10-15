# PR #7936 리뷰 — PNG 투명 여백 판독 교정

## 최종 판정

**승인 — PNG alpha 판독의 메인터너 보정·로컬 검증·정확한 code head CI 완료.**

원 PR은 `0c2b60b410af00d26111dc5453477afec091c56e`로 병합됐다. 사용자 지시의 옵션2에 따라
이 review·오늘할일은 별도 문서 PR로 반영한다. #7902의 renderer19문서 보정 완료나 이슈 종료를
뜻하지 않는다. 후속 문서 PR의 CI·병합은 이 기록 작성 뒤 별도로 확인한다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| PR·작성자·base | [#7936](https://github.com/edwardkim/rhwp/pull/7936) / jangster77 / devel |
| 고정한 devel | `e4e6d0c8d83f688e376043c639e4fe5842169c2d` (#7935) |
| 검증·병합한 code head | `ceb50fd323d88f5c3c6810eda2e48660b07350d0` |
| 관련 이슈 | [#7902](https://github.com/edwardkim/rhwp/issues/7902), 참조만 하며 열어 둠 |
| 원격 권한·reviewer | collaborator write, self-review, reviewer 지정 없음 |
| 병합 직전 | 정확한 head의34개 check/status 완료, 실패·대기 없음, MERGEABLE/CLEAN |

## 변경과 조판 원칙 검토

PNG 열기/인메모리 입력 → `page_rgb`의 흰 배경 합성 → 같은 RGB를 사용하는 내용 마스크·
overlay·compare·review 경로를 확인했다. 기존의 alpha를 먼저 버리던 `.convert("RGB")` 경로도
교체했다. alpha0의 숨은 RGB는 종이색, 부분 투명은 실제 합성색, 불투명은 원래 픽셀을 유지한다.
위치·크기·DPI·2px 반경·색상 임계값32·실루엣 알고리즘은 바꾸지 않는다.

`image_decoding`에 helper SHA·배경 정책을 기록해 resume에서도 판독 구현을 고정한다.
기존 lint job에 동일 Python discovery를 연결했고 workflow의 trigger·권한·required check 이름은
유지했다. 제품 parser·layout·paint·Native/WASM·Studio·CLI 출력 변경은 비해당이다. 실제 문서의
새 배치 회귀/fixture/golden은 추가하지 않았으며 합성 PNG는 비교 도구의 alpha 계약을 검사한다.

## 검증 입력과 결과

| 검사 | 실제 결과 |
| --- | --- |
| 수정 전 최초 alpha 계약 | 기존 `e4e6d0c8d` 도구에서7개 중5개가 픽셀 판독으로 FAIL,2개 PASS |
| Visual Sweep Python | 독립 venv의 Pillow12.3.0/fonttools4.64.0에서152/152 PASS,28.098초 |
| 새 alpha 계약 | 11개: RGBA/LA/팔레트, 부분 투명, 불투명 보존, 이동·누락 검출, overlay/compare, CLI2작업자 |
| workflow 계약 | 250/250 PASS,30.002초; 누락된 sparse 입력 복구 후 최종 실행 |
| lint·문서 | actionlint, diff check, 변경 Markdown3개의 링크·canonical guide metadata PASS |
| 실제 보존 PNG 재판독 | 19문서 합계835쪽×Native/WASM×base/candidate =3,340쌍,76개 TSV 바이트 동일,406.167초 |
| Rust 전체 로컬 재실행 | Python/CI 명령 범위여서 반복하지 않음. 원격 Full CI의 실제 Rust 검증은 아래 성공 |

실제 PNG의 원래 renderer source는 `2efefc6a1f`와 `9c1de319b`다. 입력/PDF·producer·글꼴·
페이지·DPI를 보존한 기존 PNG 쌍의 **판독 보존** 검사이며 최신 devel의 새 렌더링 검증으로
승격하지 않는다. 0.01%p 이상 하락도0건이다. #7902의 최신 producer 전쪽 검증은 별도로 진행한다.
저장소 전체 메타데이터 검사의 기존 누락·sparse 경로 오류는 전체 통과로 보고하지 않는다.

원시 로그·TSV·JSON은 ignored `output/pr-review/issue7902-render-repair-20261014/`에 보존한다.
[구현·명령별 기록](../../working/task_m100_7902_stage6.md)과
[입력 SHA·수정 전후 PNG](../assets/visual_sweep_alpha_20261016/README.md)에 연결한다.

## 정확한 code head의 GitHub CI

- [CI](https://github.com/edwardkim/rhwp/actions/runs/38810612880): success.
  lint의 새 `Validate Visual Sweep PNG contracts`, 기존 workflow 계약, Native Skia,
  전체 default-feature archive A/B/C/D, frontend package, 집계가 모두 성공했다.
- [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/38810612974): success; GHAS 포함 실패 없음.
- [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/38810612895): success.
- [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/38810612977): success.
- [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/38810611415) 및 나머지 status 성공.

34개 check/status의 정확한 head를 다시 조회하고 `--match-head-commit`으로 일반 squash merge했다.
관리자 우회는 사용하지 않았다. 원문 PR 본문의 한글·SHA 고정 실제 Markdown 이미지 일치와
대표 raw PNG의 HTTP200을 확인했다.
병합 뒤 [duration 갱신](https://github.com/edwardkim/rhwp/actions/runs/38813249132)이 성공했고,
`ready=true/successful-pr-worker-measurements`, 원 PR7936·code SHA·B/C/D attempt1의 연결을
확인했다. 병합 뒤 제품 검증 CI를 새로 실행하지 않았다.

## 직접 판독과 남은 범위

384×224 합성 입력의 표·문구는 동일하고, 아래16행만 alpha0인 검정 RGB다. 기존 판독의 검은 띠와
파란 overlay가 수정 뒤 흰 종이로 바뀌는 것을 직접 확인했다. 실루엣39.55136→100.00000%,
오차6,144→0px다. 실제 내용 이동/누락의 음성 대조군은 계속 검출된다.

이는 실제 문서의 수식/미세 간격 완료를 주장하는 PNG가 아니다. #7902의 renderer·전체90%·
devel 대비0.01%p·완료 페이지98% 및 직접 확대 판독 기준은 유지한다.

## 후속 처리

문서 PR 병합 뒤 devel을 동기화하고 #7902·원 PR에 merge SHA 고정 asset 및 도구 범위만 안내한다.
원본 #7902는 닫지 않는다. 종료된 전용 branch/worktree를 정리하며, 현재 renderer 비교가 이
worktree의 도구를 사용하는 동안에는 그 작업 종료까지 보존한다. 공유 target·진행 중 증적은 보존한다.
