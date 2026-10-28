# #7998 보존 paint의 저장·재소비 독립 근거

B15에서 보류한 두 입력을 수정 전후 증거를 보존한 채 B16에서 등록한다.
`generate_native_border_sources.py --output <폴더>`의 PDF 두 개를 바이트 그대로 사용했다.
원본은 명시적인 PDF 연산으로 만들었으며 HWP/HWPX의 줄 정보를 수동 작성하지 않았다.

## 생성·저장과 독립 기대

- MediaBox는 400×360pt, 부모 격자는 `(20,130,240,180)`, 자식 격자는
  `(35,250,90,28)`이다. source stroke는 0.5pt다. `periodic-parent`의 부모 내부선과
  `occluded-parent` 자식 내부선은 실제 1pt 선분/1pt 틈을 가진다. occluded 입력에는
  `(18,260,4,18)`의 흰 가림이 뒤에서 그려진다. 이를 Solid로 채우지 않는다.
- 물리 면적 paint는 셀 경계에서 분할하고 표 바깥 조각은 원래 page owner에 남긴다.
  한 stroke quad 전체를 중심선 한쪽 셀에 넣으면 한컴의 셀 clip이 절반을 지운다.
  텍스트의 첫 lead는 문단 앞 간격과 저장 LineSeg에 속하며 작성자가 지정하지 않은
  Cell.padding.top으로 대신하지 않는다. 부모 180pt/자식 28pt는 텍스트 lead 때문에
  늘어나지 않는 완전한 원본 frame이다. 새 검사는 이 소스 크기와 실제 배치의 관계를 쓴다.
- `print-inputs/<key>.hwpx`는 이번 Native PDF import의 무편집 저장본이다.
  `*-edited.hwpx`는 실제 셀 첫 문장 앞에 X를 넣은 결과이며 font slot은 Arial로
  지정했다. default Helvetica의 한컴 대체 글꼴과 stroke/배치 문제를 혼동하지 않는다.
  X 삭제 복원은 대응 source Print와 비교한다. 실제 WASM도 Studio API로 저장·재열기했다.
- `hancom-occluded-parent.hwp`는 이 폴더의 `print-inputs/occluded-parent.hwpx`를
  한컴 2020에서 열어 HWP로 정상 재저장한 파일이다. 저장 LineSeg 첫 vpos=945 HU,
  부모 common.height=18000 HU, 자식=2800 HU다. 이 값은 수동 fixture 메타데이터가 아니다.
  같은 파일을 이전 core에서 열면 실제 부모/자식 높이가 265.2/43.7px로 부풀고,
  보정본은 원본 크기 240/37.3px를 유지한다(96 DPI, 값은 보고용 반올림).

인쇄·재저장은 2026-10-18 한컴 **2020 11.0.0.9136**, preprocess none으로 실행했다.
Print는 원본 용지·100%·한 쪽씩 `hancom2020_pdf_driver_one_up` 계약이다.
드라이버는 399×360pt로 반올림했으며 내용 오른쪽이 잘린 잘못된 세로 Print와 구분했다.
동일 작업의 완료된 변환을 재요청하지 않았다. jobs/status/download·명령·지문은 아래 로컬
증거 root에 남겼고 인증 환경 파일·서버 주소·공급 글꼴·실행 JSON은 커밋하지 않는다.

## 선행 시각 검증과 회귀

96 DPI, compat 2022에서 Native Skia / fresh Docker WASM을 전쪽 비교했다.
새 fixture와 정식 검사 등록은 아래 출력 및 직접 판독 뒤 수행했다.

| 범위 | 각 backend 쪽수 | 최저 Native / fresh WASM |
|---|---:|---:|
| 가림 source × HWP/HWPX, 이번 독립 source Print | 2 | 99.12616% / 99.23890% |
| 점선 source × HWP/HWPX, 이번 독립 source Print | 2 | 99.18575% / 99.29236% |
| Arial source·X 편집·삭제 복원, 4개 가족 × 양 저장 | 24 | 97.42375% / 96.78954% |
| 정상 한컴 재저장 HWP × 이번 source Print | 1 | 99.11651% / 99.25841% |

24쪽 중 solid/mixed 무편집·복원의 기준 Print는 B15의 동일 의도 자료를 재사용했다.
따라서 이 최저값은 새 독립 Print만의 최저값과 구분한다. 새 Arial X 편집 8쪽은
Native 100%이며 WASM 최저는 99.22952%다. 100%는 raw silhouette 점수이고 PNG 바이트 동일 주장이 아니다.
기존 Print에 남은 잘린 반쪽 선과 현재의 복원된 선을 직접 구분했다. fixture의 source/양 저장
전체 페이지 및 원문·가림·오른쪽 외곽·틈·문장 순서도 직접 읽었다.
메인테이너는 비교 HTML과 실제 Studio에서 **“시각·편집 구조 통과”**로 판정했다.

`issue_7998_pdf_cell_paint_frame.rs`는 실제 부모/자식 Table, source 크기 대비 최종
frame, 물리 셀 포함, 틈/가림의 보수적인 BorderFill, 텍스트 삽입·삭제·순서 및 양 저장을
관계로 검사한다. 절대 screen px golden이나 전체 SVG 해시를 등록하지 않는다.
정상 한컴 파일의 동일 frame 검사는 이전 CLI의 265.2px에서 FAIL, 후보 240px에서 PASS다.
첫 source의 잘못된 padding/셀 밖 quad도 이전 CLI에서 FAIL, 후보에서 PASS로 확인했다.
새 Rust 테스트 전체를 이전 core에서 재컴파일했다고 주장하지 않는다.

제품 source·실행 명령·전쪽 TSV·PNG와 남은 일반화 범위는
[stage6](../../mydocs/working/task_m100_7998_stage6.md)를 따른다.
로컬 증거 root:
`/home/edward/mydev/rhwp-review-evidence/pdf-table-feedback-20261017/semantic-stage3/p4-paint-save`.
이전 미달 자료는 형제 `p4-native-borders`에 그대로 남아 있다.
비표준 스타일 교체·새 행열의 paint 상속·첫 inline 개체 lead 일반화는 이 자료만으로
완료 판정하지 않는다.

## 등록 파일 SHA-256

| 파일 | SHA-256 |
|---|---|
| `hancom-occluded-parent.hwp` | `3fe1b5c18f95295f9e080d4ea81033d5cf2264d1e96b87f322a609314ae59d2f` |
| `occluded-parent.pdf` | `6bf14e9f180cc935e01a7b058086be0adf1a0cf6ef260ee8031a4cc0b33370c1` |
| `periodic-parent.pdf` | `f184ab9be12ff321465ac7604b05eaa8d352099d1765e274d6ebb4a8190c1899` |
| `print/occluded-parent-edited-2020.pdf` | `f1e91db1ef8b56e737adb75540867b4f744272ce3fd025fb11ea775604f80332` |
| `print/occluded-parent-source-2020.pdf` | `d462ebc5dcd018ccb38de4625189363d784c0ef1f6ae3b922fd37be8fa30b0e6` |
| `print/periodic-parent-edited-2020.pdf` | `cc518c619edae3bbd981fbb2f7beb487510103e2ca62ef263c1705fdb81df7cb` |
| `print/periodic-parent-source-2020.pdf` | `1b5191b4b50b371f77062481a6157329f239ba565185ab967fab756c82f1ff46` |
| `print-inputs/occluded-parent-edited.hwpx` | `3d77210efe83c34b706f32781b03e1e25072eabb79b29a9acd270d4f75cd8e29` |
| `print-inputs/occluded-parent.hwpx` | `6449bce5d7cd99fed51a754a63642d57e12e58430b73e077209ea33c749e908b` |
| `print-inputs/periodic-parent-edited.hwpx` | `6b751540b6c953b79381d898c67309ff3e691ce8b4f01e1a9b813ee5adc0546a` |
| `print-inputs/periodic-parent.hwpx` | `a5b8cfe1bdeb753dbfe9b4ab9832ad981a1ad76dc41e24ebe4a3faf863095378` |
