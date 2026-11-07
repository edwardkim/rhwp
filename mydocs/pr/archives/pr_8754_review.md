# PR #8754 리뷰 — 셀 도형 편집·수식·연속 표·글꼴 프로그램 보존

## 최종 판정

**승인 — 지정한 해결 범위의 로컬 검증과 정확한 code head의 Full CI를 충족했다.**

완료 범위는 Native/fresh WASM 각각 20조건 562쪽이며 최저 98.31507%/98.32714%다.
고정 devel 및 리베이스 직전 대비 0.1%p 초과 하락·누락·쪽수 차이는 없다.
일부 글리프 가장자리·획 농도·미세 기준선·비트맵 보간 차이는 남으며 RGB 완전 동일을 주장하지 않는다.
CBTA #7727은 사용자 지정 제외 범위다. #7445와 #7765 전체를 닫지 않는다.

사용자가 준비 완료 후 PR 제출·CI·병합·후속 처리를 승인했다.
이 문서는 code head CI 성공 후 작성한다. 남은 조건은 이 문서-only trailing head의 실제 check와
현재 devel 병합 tree·기존 오늘할일 보존·정확한 최신 head의 MERGEABLE/CLEAN 확인이다.
아직 수행하지 않은 trailing CI·merge·issue 종료를 완료로 적지 않는다.

## 접수와 검토 경로

- PR: [#8754](https://github.com/edwardkim/rhwp/pull/8754), 작성자 jangster77, devel 대상 collaborator self PR.
- 원격 제출·CI code head: `0e1291a2ad50b78a8dc532c59de39f05b64eaaca`.
- 제품 Rust/Studio·Rust 회귀 검증 source: `5011277356ad04aac8b07df7e56cc695ba297943`.
- 고정 devel: `5077b216cbc51d2046da07350de562f6f521a7c1`.
- 정확한 리베이스 직전: `e944fff2f919f540c19e21caf4902b92f5a6b6da`.
- 최종 runtime 차이는 암호 E2E 준비 조건 1개이며 원 entry 연속 2회와 Studio 전체로 검증했다.
  생성 공개 선언도 같은 fresh pkg와 바이트 일치한다. 제품 코드·Rust 검사·입력·도구는 7,647개 파일 대조로 같다.
- 적용: collaborator_self_merge, intake_and_review, local_validation, visual_fixture_evidence,
  renderer_backend_verification, review_only_fast_pass, post_merge, review_template.
- reviewer는 지정하지 않았다. owner라는 이유로 자동 리뷰를 요청하지 않으며 branch protection을 우회하지 않는다.
- `.github` workflow/action·CI impact classifier·review-only/reuse 정책 변경은 없다.
  suite 생성기의 공유 helper 중복 mod 제거는 Rust harness 공급 보정이며 실행/재사용 정책 변경이 아니다.
  따라서 현행 self PR의 정상 문서-only trailing 경로를 사용하고 옵션 2를 시험하려는 추가 push는 하지 않는다.

## 변경과 조판 원칙 검토

| 구현 주장·경계 | 독립 근거·실제 검사와 판정 |
| --- | --- |
| 셀 도형 소유 경로·행렬 | 선택/이동/삭제/잘라내기/Undo·타입 불일치 반례와 저장 재열기를 확인했다. aaaaaa.hwp 27/28쪽과 전50쪽 Print를 대조했다. 곡률 파서/모델의 기존 upstream 지원과 새 편집 구현을 구분했다. |
| 수식 상자·글줄 점유 | 유효 저장 줄·현재 수식 프레임·본문 전진을 소비 지점까지 확인했다. 원본/재저장본 각21쪽·96수식3쪽·과학4쪽을 각각 대응 Print와 비교했다. 프라임/분수/장식/그래프·앞뒤 문항 소속을 직접 판독했다. |
| 표 높이·분할/이어받기 | 선언 잔여·내용 하한·종료 컷·요구/예약/배치·최종 paint가 같은 소유 계약을 소비한다. 전결103쪽·전기안전18쪽·LPG48쪽·거대표197쪽의 전쪽 TSV와 핵심 시작/이어받기/종료·다음 내용 회귀를 확인했다. |
| 글꼴 프로그램·문서별 선택 | 원본 TTF/OTF/TTC 탐색과 fallback, 실제 face 이름, 기존 메트릭 인증과 사용자 자연 전진을 구분한다. 프로그램/기록 초기화·다음 문서 로드·한글/영어/기타 순서와 실제 일반 화면의 옥수수 원본을 확인했다. |
| HFT 공급 전 출처 | 같은 최종 공개 CFF 검사에서 수정 전 2 PASS·1 FAIL→수정 후3PASS다. natural/무명원본/일반 프로그램/공백을 제외하고 원래 HFT 꾸밈만 보존했다. #8418 Native2쪽99.89494→99.99715로 복원했다. 새5%사양·특정 파일/쪽 예외를 추가하지 않았다. |
| 뒤배경표 가사 | 사용자 승인된 Haansoft Batang 선택·한겨레 공급 조건에서 반복 하이픈 원점과 악보/표·연락처 소속을 확인했다. 전4쪽 양 backend98% 이상이며, 원본/대체 글꼴을 혼동하지 않았다. |
| 반례·회귀 적합성 | 고정 px는 해당 완료 페이지 양쪽 raw98% 이상과 독립 Print ±2px 조건에서만 사용했다. 관계형은 누락/중복/내용 순서·소속·포함/겹침·실제 원본 크기 계약을 보호한다. 기대값 완화·출력 숨김으로 새 회귀를 없애지 않았다. |
| 기존 이슈와 원 해결 PR | 단계별 보정 기록에서 원 PR·메인터너 보정·독립 Print·base/head를 추적했다. 이미 upstream에 있는 기능을 중복 구현한 것으로 표시하지 않았다. |

세부 소비 경로·수정 전후 검사·개별 입력/Print는 [보정 보고서](../../working/task_m100_7375_rebase_20261107.md)와
[최종 결과](../assets/issue7375/final-20261108/validation.md)에 연결한다.

## 검증 입력과 실행 결과

실제 HWP/HWPX/PDF 99개의 commit 포함·바이트·SHA를 확인했다.
[20조건 입력·기준 Print·DPI/compat·해시](../assets/issue7375/final-20261108/README.md),
[TSV/PNG producer·SHA·bytes](../assets/issue7375/final-20261108/assets.tsv),
[전쪽 devel/리베이스 전 대조](../assets/issue7375/final-20261108/pagewise-preservation.tsv)를 보존한다.

- 전체 release-test nextest **12,962 PASS / 0 FAIL / 45 skip**, 810.366초.
- prepare·fmt·Native/WASM/all-target Clippy·workspace build·Skia·manifest·unit tiers 13 gate PASS.
- Native Skia root unit3,857 PASS/12ignore + library182PASS; PNG2PASS/PDF35PASS.
- 수동 집중36PASS·자동 집중163PASS·도구81PASS.
- Studio2,769PASS/2skip·font contract42PASS·최신 공개 선언을 소비한 build PASS.
- 최종 암호 E2E 전체 entry2회PASS. 원인인 빈 키 입력 경합을 준비 조건에서 검출하며 오답·정답·취소 기대는 유지했다.
- Native/fresh WASM 각각20조건562쪽 모두98% 이상, base/리베이스 전 대비0.1%p 초과하락0·누락0·쪽수차이0.
- 편람 Native383쪽 `--handbook-gate` PASS, 최저99.96465%(130쪽). 원값100%370쪽과 표시100.00%382쪽을 구분했다.
- Native Skia12/Studio Canvas2D12/지원 CanvasKit12 실제 소비PASS. 추가Canvas2D대조군6건도 확인했다.
  CanvasKit software의 실제 backend/profile을 확인했으며 미지원 fallback을 통과로 세지 않았다.
- 새 devel32조건88쪽×base/후보×양backend128조건PASS, #8738 Print7조건×4경로·UI8건PASS.
  [반영 PR 입력·producer 원장](../assets/issue7375/final-20261108/incoming-inputs.tsv)을 함께 보존했다.
- 기존 #7765의25~29쪽은 별도90% 보존 범위다. Native최저96.98466/WASM98.05371이며28/29쪽은양쪽98% 이상이다.
  본20조건·전쪽98% 완료로 합산하거나 이 이슈 전체를 닫지 않는다.

Mac은 jobs8·threads8·총capture8, Cargo순차·캡처배타lock으로 실행했다.
기본 작업장 고정 지시를 따라 /Users/tsjang/rhwp의 전용target·immutable baseline archive/binary를 사용했다.
개인 글꼴을 삭제/비활성화하지 않고 외부 Mac sandbox로 최소 글꼴 음성 대조를 확인했다.
Chrome 내부 sandbox 중첩 실패와 외부 격리 유지의 정정은 최종 결과에 구분했다.

Mac fresh WASM은 locked wrapper `--target web --out-dir pkg --no-opt` 로컬 대체 빌드다.
Docker 최적화 통과가 아니다. Native SHA는 `18c5e2e3a1b120fb0641e1ce010e62b58495cb0258b2f657dac5c4b04fd23675`,
WASM은 `2092cfe6d1b546ba24399f5d7db503ad12c49d096e47cbdce1bcd30b9605b78b`,
JS는 `e1c2a4402be1060a666dda07d51e280ac08b3682d3a229495a630394249614b4`다.
root pkg·Studio public·실제 browser 초기화 응답을 대조했다.

## 시각 증적과 직접 판독

PR 본문에 각 주장 페이지의 raw 링크 및 같은 쪽 Native/fresh WASM review·overlay를 inline으로 게시했다.
입력/PDF/TSV/PNG 등235개 exact-head rawURL(54,453,308bytes)을 다운로드해 원본 SHA·bytes와 대조했다.
실제 PR 화면에서 review·overlay44개와 actual2개가 모두 로드되고 naturalWidth>0임을 확인했다.

대표 경계는 번호 도형27·28쪽, 수식 원본/재저장18쪽·진단2쪽·과학4쪽,
전결26·27쪽, 전기안전6쪽, 분할90쪽, LPG2쪽, 뒤배경표1·3쪽, PII4쪽과 전체최저도형35쪽이다.
직접 판독과 전쪽 수치 판정을 구분하고 실제로 연 review/overlay만 최종 결과의 판독 표에 기록했다.
남은 차이는 작은 글리프 가장자리·농도·미세 기준선·점선/비트맵 보간이다.

## 정확한 code head GitHub Actions

제품/회귀 검증 head는 `0e1291a2ad50b78a8dc532c59de39f05b64eaaca`다. 아래 workflow는 모두 실제 완료 결과를 확인했다.
CI preflight는 fast_pass=false `unusable-file-list:bb2940b3c8f8f825147cc70bfa4ab190245da452`로 Full 실행됐다.
이를 재사용 또는 skipped 검사 실행으로 표시하지 않는다. GitHub 각 실행 범위와 로컬 전체 결과를 합산하지 않는다.

| workflow | 실제 run | event | 상태 |
| --- | --- | --- | --- |
| Render Diff | [42410045920](https://github.com/edwardkim/rhwp/actions/runs/42410045920) | pull_request | completed / success |
| CI Impact Policy Controller | [42410045991](https://github.com/edwardkim/rhwp/actions/runs/42410045991) | pull_request_target | completed / success |
| Proptest roundtrip | [42410045992](https://github.com/edwardkim/rhwp/actions/runs/42410045992) | pull_request | completed / success |
| CI | [42410046074](https://github.com/edwardkim/rhwp/actions/runs/42410046074) | pull_request | completed / success |
| CodeQL | [42410046126](https://github.com/edwardkim/rhwp/actions/runs/42410046126) | pull_request | completed / success |
| Adapter inter-diff | [42410046160](https://github.com/edwardkim/rhwp/actions/runs/42410046160) | pull_request | completed / success |
| CI Impact Policy Controller | [42412647640](https://github.com/edwardkim/rhwp/actions/runs/42412647640) | workflow_run | completed / success |

아직 실행 전인 trailing head는 이 code CI 성공으로 대신하지 않는다.
문서 commit의 source/test/fixture/workflow 불변·최신 base/merge tree·오늘할일 보존·상대 링크를
push 전에 확인하고 최신 head의 실제 fast-pass/required check·MERGEABLE/CLEAN을 다시 조회한다.

## 병합·후속 처리 계획

1. 정상 보호 조건을 만족한 exact trailing head로 squash merge하고 merge SHA를 확인한다.
2. 병합 뒤 duration 갱신 성공 또는 증거 부족의 보류 사유를 확인한다. 검증 CI를 재시작하지 않는다.
3. 완전 해결 #7375/#7640/#7642/#7700/#7728을 실제 범위·TSV/PNG 근거로 확인하고 후속 댓글·close한다.
   #7445는 완료한 하위 항목만 취소선 처리하고 전체 및 제외/보존 범위는 OPEN을 유지한다.
4. 같은 PR에 merge SHA·CI·남은 차이와 merge SHA 고정 raw/review/overlay를 한국어 존댓말로 게시하고 API/이미지 표시를 확인한다.
5. devel fast-forward·전용 local/remote branch/backup·실행 종료한 전용 target 정리와 output전체 삭제를 실제 사용/소유 확인 뒤 수행한다.
   사용자 Vite7700·Chrome·설치 글꼴·원본/PDF·다른 작업장·공유 target은 보존한다. 정리 로그는 output밖에 남긴다.
