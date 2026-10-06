# #3587 Stage 18 — D1 실물 중첩·그림 자원 검증

- 승인: 메인테이너 「다음 절차를 진행하세요」.
- 시작점: `adb696e95`, 제품 기준 `cc9f49a8f34a9d47fd67a3f5ce1b6701ef41f528`.
- 근거: [D 계획 §6](../plans/task_m100_3587_impl_d.md), [Stage 17](task_m100_3587_stage17.md).
- 목적: 연구노트 블록에 없던 실제 그림 자원과 중첩 소유 구조를 가져온 뒤 보존되는지 확인한다.
- 비범위: renderer 변경, 지원하지 않는 컨트롤 삭제/우회, D2 공개 연결, PR/push, 새 이슈.

## 입력과 순서

계획에 지정된 `samples/table-in-tbox.hwp`를 사용한다. CLI dump에서 본문 pi2의 표 셀 안에
그림이 있고, pi4에 글상자가 있음을 확인했다. pi0은 구역 컨트롤이 있으므로 완전한 문단을
지원하는 D 계약의 성공 사례로 사용하지 않는다. 임의로 구역 컨트롤을 삭제하지 않는다.

1. pi2·pi4 각각의 구조 선검증과 지원 경계를 확인한다.
2. 지원되는 범위는 별도 대상에 가져온다. 실물 시각 출력의 대상 용지·여백은 원본과 같게 먼저 준비한다.
3. source 불변, 목적지 기존 내용/용지 보존, 소유 구조·이미지 바이트·ID 독립성,
   저장/재열기 및 재가져오기 자원 재사용을 확인한다.
4. 지원 오류가 있으면 그 경로와 사유를 남긴다. 지원 범위를 조용히 축소하거나 테스트에서 무시하지 않는다.
5. 실제 출력이 생기면 한컴 확인용 HWP/HWPX와 필요한 PDF를 준비한다. 자동 구조 검사와 사람 판정은 구분한다.

## 실행 결과

제품 코드는 수정하지 않았다. `tests/cases/issue_3587_import_real_nested.rs`에 실물 계약 2건과
출력 보존용 ignored 진단 1건을 추가했다. 원본을 합성·축소하거나 컨트롤을 삭제하지 않았다.

| 원본 문단(0-based) | 결과 | 의미 |
| --- | --- | --- |
| pi0 | 구역/다단 경계로 사전 거부 | 구역 설정을 가져오지 않는 기존 계약 유지 |
| pi2 | 가져오기, HWP/HWPX 저장·재열기 성공 | 원본 2쪽 상단의 제목 표. 6행×4열, 병합 포함 15셀, 셀 안 그림 1개 |
| pi4 | `uninterpreted textbox LIST_HEADER tail`로 사전 거부 | 실제 글상자의 미해석 부가 데이터를 안전하게 이식하는 계약은 아직 없음 |

pi4 오류 경로는 선택 범위 기준 `[Paragraph(0), Control(0), Shape, TextBox]`다.
실제 원본에서는 pi4에 해당한다. 거부를 손상 문서 판정이나 가져오기 성공으로 바꾸지 않는다.
안전 거부 테스트 통과는 해당 글상자의 지원 완료가 아니다.

성공 사례에서는 다음을 검사했다.

- 행·열·셀 수, 셀 주소/병합/폭/높이, 셀 문단 수·텍스트·컨트롤 수, 표 나눔 속성 보존.
- 그림 폭/높이·crop, 제한 읽기로 얻은 원본 그림 바이트의 정확한 보존.
- 재호출 시 서식 추가 0·바이너리 추가 0·바이너리 재사용 1. 표와 그림의 복사본 ID는 서로 다름.
- 원본 문서와 이벤트 불변, 목적지 첫 문단과 용지 설정 보존.
- 두 복사본 모두 HWP/HWPX 저장 후 재열기에서도 셀 구조와 그림 바이트 보존.
- pi4의 preview/execute가 같은 사유로 거부되며 대상 문서와 이벤트에 부분 반영이 없음.

## 검증 명령과 코드 식별

review worktree: `/home/edward/mygithub/rhwp-review-3587`.
제품 SHA는 `cc9f49a8f34a9d47fd67a3f5ce1b6701ef41f528`이고, Stage 17의 동일 용지 출력 helper와
이번 신규 test source를 overlay했다. 제품 변경을 포함한 새 전체 SHA의 통합 검증은 아니다.

```bash
node scripts/rust-test-suite-manifest.mjs --prepare
rg -l 'cases/issue_3587_import_real_nested.rs' tests/generated/regression_suite_*.rs
cargo nextest run --locked --cargo-profile release-test \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review \
  --test regression_suite_001 -E 'test(issue_3587_import_real_nested)'
cargo fmt --all -- --check
node scripts/rust-test-suite-manifest.mjs --check
```

- 신규 계약 **2 PASS**, 실패 0. Nextest run `641fabb3-d5e8-4664-8127-55a27788a4e6`.
- 진단은 최초 probe source에서 **1 PASS**, run `ae8ca182-39f4-4b9a-8137-8ad157df23c2`.
  당시 suite023, 이후 test source 크기가 바뀌어 `--prepare`가 suite001로 재배치했다.
  재실행 시 위 `rg` 결과로 실제 suite를 선택해야 한다.
- 포맷·manifest·`git diff --check` PASS. 파생 suite/manifest는 review worktree에만 존재하며 stage하지 않는다.
- nextest 0.9.137/권장 0.9.140 차이는 경고이며 테스트 실패가 아니다.
- PDF raster 최초 실행은 review cwd의 상대 output 경로를 잘못 참조해 실패했다.
  주 작업 경로에서 재실행해 이미지 생성과 직접 확인을 마쳤다.
- 전체 회귀·세 Clippy 단계·WASM 재빌드·공개 API 호출은 이 절편에서 실행하지 않았다.

원본 SHA-256: `02b0ced8b96cec5b6e1cad6dd1926b701a2875ea9591c589e57edf7653daf4af`.
최종 test source SHA-256: `5566e3c288e4a5e29cd7c79100f9703598d3050aada531b7697f48249aa6283b`.

## 한컴 출력과 확인 경로

디렉터리: `/home/edward/mygithub/rhwp/output/3587/d1-real-nested/`.

| 파일 | SHA-256 |
| --- | --- |
| `pi2-import.hwp` | `3b8eb4b8d6ea2f569bcc02b23b65f77ac261c73152def54a998acab10607dcbe` |
| `pi2-import.hwpx` | `dd108e6495777f4c43fa131a09fe5145024a2837948ac5e9da45569089db8204` |
| `pi2-import-hwp-2020.pdf` | `6780fb4f13ac11a279389fe9d19d251ad23dd037016c0e5da534174f023677ba` |
| `pi2-import-hwpx-2020.pdf` | `e7099e713b8f80036838b320a89d1ebe91962ff9d79a376ca927c3582bc6b444` |

공식 client 0.9.0으로 `start → status(succeeded) → download` 수행.
입력 `info.lastSavedWith.product`는 두 형식 모두 `hancom-office-2020`이어서 engine2020 사용.
HWP job `d1f15410-9f7c-4fe3-8d36-cc987f225554`,
HWPX job `deb8be33-7918-45a5-8a63-857228235134`.
runtime `12.0.0.4605`, 32-bit, `frame_print_to_pdf_ex_one_up`, 전처리 없음,
font 등록 2/2 성공. 각각 PDF 1쪽, 17,401/17,399바이트. 다운로드 해시와 로컬 해시가 일치한다.

기존 정답지 `pdf/table-in-tbox-hwp-2020.pdf`를 재사용했다. 원본 **2쪽 상단 제목 표**와
새 PDF의 제목 영역을 직접 확인해 작은 그림, 기관명, 안내문 제목, 보라색 띠가 표시됨을 확인했다.
원본 아래 큰 글상자는 선택 범위가 아니며 출력에 없다는 것을 누락 버그로 판정하지 않는다.
원본과 대상의 앞 문단 구성이 다르므로 절대 y 위치 동일성을 주장하지 않는다.
HWPX는 rhwp가 HWP 원본에서 내보낸 결과이지 독립적인 한컴 HWPX 입력 자료는 아니다.

에이전트 확인 후 메인테이너가 앞쪽 빈 문단의 의도를 확인하고 **「성공입니다」로 시각 판정했다**.
이번 제목 표/그림 HWP·HWPX 출력 및 기존 첫 빈 문단 보존 사례는 통과다.
이 판정을 미지원 글상자나 Stage 17 전체 출력의 통과로 확대하지 않는다.

## 남은 순서

1. 이번 제목 표/그림 출력의 메인테이너 확인 — 완료.
2. pi4의 미해석 LIST_HEADER 부가 데이터와 내부 컨트롤을 조사하여 D1 지원 경계를 보완할지 판단.
   허용 검사 삭제나 raw 데이터 제거만으로 통과시키지 않는다.
3. D1 종료 검토 후 D2 공개 경로, D3 통합·비용 검증, 선택적 Gym 순서 유지.

이번 절편으로 D1 전체 완료를 선언하지 않는다. 원격 push·PR·댓글은 수행하지 않았다.
