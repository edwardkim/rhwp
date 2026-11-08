# #5846 빈 앞 프레임 뒤 종료 행의 원본 컷 복구

Issue: #7445. 현재 개인정보 원본 범위의 보정이며 새 문서 대상을 추가하지 않았다.

## 생산 근거

- parent HEAD: `715c9d259126850da7de8d60126cd52d20743707`. 런타임 패치 SHA-256: `05d4e9cf3882e8a6bde54546f55e182a47e26402c133262df37d89224b523c7f`.
- 수정 파일: `src/renderer/layout/table_layout.rs`, SHA-256 `05ea939dc66fd59e1c95de00fc2490b79445622d98d61365709a563ef5226eaa`.
- frozen Native CLI: `145139c705e40cb6612eb30fa529bdf907093bf8c2945ea85343ae7076a875aa`. lib: `83650d792855e0b08ee42f4b860e5c4eb835bcc1460031cd03e031dda6f03e00`.
- 입력: `samples/task2097/75544_pii_bunseok.hwpx`, SHA-256 `f429768ec02e5a849d08c06678cbee029fdc8618ca9e2bdcdbba485087fe71f1`.
- 독립 한컴2020 Print PDF: `pdf/task2097/75544_pii_bunseok-hwpx-2020.pdf`, SHA-256 `0fef38f44b3825f1c97869c2720112536504eaf30cec2a2d2af6d4399ffe8770`.
- 선언 환경: `mydocs/pr/assets/issue7445/resolution-20261005/pii5846-reference-font-environment.json`, SHA-256 `21e5c5a0b081bf3101749e5b80c1b053fa3eeff4fcb1ba3f8aa33fd3e2a4d544`.
- 기존 713개 글꼴 ledger의 모든 hash를 재확인했다. Native Skia Print/compat2022/96DPI, PDFium152.0.7947.0 smooth RGB, raw 2px/90%, capture 8.
- 기존 base2e087a5b와 수정 전 P33의 같은 조건 66쪽 PNG/TSV는 고정 producer로 보존되어 있다. 후보는 전 66쪽을 새로 출력했다.

## 결과와 한계

| 범위 | 수정 전 | 후보 | 판정 |
| --- | ---: | ---: | --- |
| 60쪽 Native | 63.16676% | 98.32834% | 제목→첫 표→다음 제목→다음 표의 겹침 해소 |
| 전 66쪽 최저 | 46.20358% | 46.20358% | 59쪽, 전체 수용 보류 |
| 미달 쪽 | 57·59·60·66 | 57·59·66 | 세 쪽 보정 잔여 |

PNG 변경은 60쪽 하나, 나머지65쪽은 수정 전과 byte동일하고0.1%p 이상 하락 0쪽이다. 원본 소유 키 1828개 중 두 빈 문단의 빈 문자열 노드만 후보에서 제외됐으며 비공백 내용의 변경은 0이다. 이것은 전역 읽기 순서 증명과 별개다. 기존 관계 검사는 쪽·셀 소속, 내용 한 번 보존, 표/후속 문단 순서·포함을 별도로 확인한다.

기존 95개 집중 검사는94 PASS / 1 FAIL, test 18.418초다. #5846은 이전 P33의60쪽 겹침 실패를 지나 모든60쪽 소속·포함 검사를 통과한 뒤,65·66쪽 설명 표의 원본 칸(4,1) 소실에서 실패한다.57·58쪽 종료 프레임 차이도 같은 함수가 누적하므로 전체 함수 PASS가 아니다. #4138 원본/분할 및 #7752 보존 대조군은 PASS다.

60쪽 review와 overlay를 직접 열어 제목·표 원점·뒤 문단과 마지막 서명의 소속을 대조했다. 괘선 굵기·일부 글자 가장자리·외곽 종료의 미세 차이는 남는다. base 대비 41쪽99.23712→97.93657% 하락도 그대로 보류다. fresh WASM·최신 전체 nextest·최종 필수 게이트·actual36 결과로 재인용하지 않는다. 새 회귀/px 핀·기대값·원본·PDF·지침 변경은 없다.

첫 캡처 driver를 Python 3.14로 실행해 Native 66쪽 생성 뒤 PDFium 부재로 중단했다. 그 디렉터리를 `pii-closing-native-v36-invalid-python314`로 보존하고, 선언된 Python 3.12 venv로 같은 frozen producer의 전 66쪽을 새 디렉터리에 다시 실행해 종료 0을 확인했다. 실패 실행은 전쪽 TSV 증거가 아니다.

## 대표 이미지

| 조건 | review | overlay |
| --- | --- | --- |
| 수정 전60쪽 | [review](before-native-p060-review.png) | [overlay](before-native-p060-overlay.png) |
| 후보60쪽 | [review](candidate-native-p060-review.png) | [overlay](candidate-native-p060-overlay.png) |

전쪽 TSV: [base](base-native.tsv), [수정 전](before-native.tsv), [후보](candidate-native.tsv).

## 제출 증적 SHA-256

- `base-native.tsv`: `9c7fd36cbb4e8eb417cf142df62d1f6dfd886574069c399aae3844e71a74ceac`
- `before-native.tsv`: `12a3ab2046612f3b4fc292c43375217cb123cfb68316eb21951c7d952efe37b5`
- `before-native-p060-review.png`: `85cde25c62c4144bb50408d87f95364b9c103b1312d0a9d3ca992d9f7275d17b`
- `before-native-p060-overlay.png`: `cfe4769d04bff95177d2fd82f3567229443c3595f80ddedaa5ce80fbd56a0338`
- `candidate-native.tsv`: `63dedb9763bfdd4396c2e10cdc17a6f04252052cd78f80a48b7ed6c00c44b774`
- `candidate-native-p060-review.png`: `a96f8881d9bee4284b456e56113efd6b2f475425af05de5c8b309b673cb77144`
- `candidate-native-p060-overlay.png`: `e7918f1ac7a40489fe8d72e37a84a7c35af48c291b666e4a47c6caa45b7f0ec1`

실행 로그·JSON·임시 SDK는 ignored `output/pr-review/7445-completed-20261009/rebase-7752-20261022/`의 `repair-focused-v36`, `pii-closing-source-frame-v36`, `pii-closing-native-v36`에만 보존한다.
