# 보정25: 저장 원점·중첩 첫 프레임 증적

부모 code head `a36e45e80f2d370a24d6da108f84f084673e7965`, fixed base `2e3b4f880bd11d35398e6ea0100846611f13a72d` 위 보정이다. 아래는 이 두 입력의 검증이며 전체 브랜치 수용이나 PR 준비 완료가 아니다.

## 입력과 독립 기준

| 파일 | bytes | SHA-256 |
|---|---:|---|
| `samples/rowbreak-problem-pages.hwpx` | 3452994 | `4ccc321e7eb146f28b768654502a128ba1ae408238fd1ed37d51e963732eb5b0` |
| `samples/rowbreak-problem-pages.hwp` | 553984 | `10b6ab6548610e18c82ba78a1c844a00107fedbb28c195cb05e6fd20626d33ed` |
| `pdf/rowbreak-problem-pages-hwpx-2020.pdf` | 1122543 | `e3dc7bb21707f92e42e83808023bf155f62475fc4eb0e0498709d03986c5afe5` |
| `pdf/rowbreak-problem-pages-hwp-2024.pdf` | 1056768 | `2c49bda9cc21dc8b93b554d2607da241dd7e894657eb308ed823e7d44654e85c` |

HWPX는 Hancom2020, HWP는 Hancom2024의 동일 입력 Print18쪽을 각각 사용한다. 입력·PDF·문턱·기존 검사 기대값은 바꾸지 않았다. 현재 canonical `visual_sweep.py`, PDFium smooth_rgb_v1, Print/compat2022/96DPI/raw2px90을 사용했다.

## 전쪽 결과

| 형식 | Native 최저/쪽 | fresh WASM 최저/쪽 | 쪽수 | 미달/누락 |
|---|---|---|---|---|
| HWPX |92.05401% /15 |92.05433% /15 |각18/18 |0/0 |
| HWP |93.07060% /16 |92.96483% /16 |각18/18 |0/0 |

4개 전쪽 TSV로 총72개 쪽별 측정을 보존한다. 같은 형식의 입력을 같은 8쪽 묶음·capture8로 산출했다. `before-frame`은 CSAP 원점 보정을 이미 포함한930/932이며, 원본 실패 전체를 뜻하지 않는다. 원본908의10쪽 집중 결과는 별도 ignored925에 있다.

| 변경 쪽 | 보정 전 Native/WASM | 보정 후 Native/WASM |
|---|---|---|
| HWPX7 |87.01856 /86.86729 |99.60978 /99.61492 |
| HWPX10 |9.05432 /9.24732 |99.90903 /99.90809 |

930/932→960/961의 전18쪽 대응 비교는 HWPX7 PNG만 바뀌었고, HWP18쪽 및 나머지 HWPX17쪽의 PNG SHA가 각 backend에서 같다. 0.1%p 이상 하락0쪽이다. 고정 devel 전체 비교와 #8064 별도 하한 감사는 이 비교로 대체하지 않는다.

- [Native7 review](after-native-p007-review.png) · [overlay](after-native-p007-overlay.png)
- [fresh WASM7 review](after-wasm-p007-review.png) · [overlay](after-wasm-p007-overlay.png)
- [Native10 review](after-native-p010-review.png) · [overlay](after-native-p010-overlay.png)
- [fresh WASM10 review](after-wasm-p010-review.png) · [overlay](after-wasm-p010-overlay.png)
- [보정 전 Native7 review](before-native-p007-review.png) · [overlay](before-native-p007-overlay.png)
- [보정 전 Native10 review](before-native-p010-review.png) · [overlay](before-native-p010-overlay.png)

변경7/10쪽과7→8쪽을 확대 판독했다. 7쪽 제26조의 첫 글줄·중첩 인용상자,10쪽 CSAP 제목과 공공데이터 실무매뉴얼 표의 순서/간격이 복구됐다. 7→8쪽 자식 본문은 원본과 공백 제외 전체 문자열이 같고 누락/중복 없다. 기존15쪽 그림/본문과16쪽 미세 간격·가는 선 차이는 남아 있으며 완전 픽셀 일치라고 표현하지 않는다. 엄격RGB 일치는 raw 실루엣과 다르다.

## producer와 소비 경로

Native960 frozen CLI SHA `6cf280200468c670b8e06aa821c14d97a56c1562d3e5c1fc705860bdba7c6f05`.
Fresh961 JS SHA `63c587e4b072f0a65416e65a0e86bf00b2971cdfb0f46cfda7f150449dd30524`, WASM SHA `0bc3fc93aa2436da75e73a5ee2ff3a5edb9c6006104004a2a412b1620605e341`.

Mac locked wrapper `--target web --out-dir pkg --no-opt` 로컬 대체 빌드이며 Docker 최적화 통과가 아니다. 실제 root pkg/Studio public hash와 Studio7736 새 초기화 응답 JS의 Vite URL rewrite/source-map·WASM raw bytes를967에서 대조했다. 초기화 true/오류0이며 문서 Canvas/CanvasKit paint 또는 actual36 완료를 뜻하지 않는다.

일반 중첩 표42065의 Native17쪽은930/943 PNG SHA가 모두 같다(948). 이 대조군의 기존 PDF 미달을 수용했다고 쓰지 않는다.953은 두 형식 전18쪽의 Table/Cell/글줄/텍스트 순서·source pi/ci·row/col이 기하 외에는 그대로임을 확인했다.

## 정규 검사와 물리 예산

최종968의 새 관계/Native PNG2, 기존 RowBreak16, 캡션 대조3, 커서·기존 첫 프레임 단위56은 **77PASS/0FAIL**이다. manifest·unit-tier·fmt·diff check도 exit0이다. 원본 입력/쪽수/기대 컷을 완화하지 않았다. 최종 integration source는957에서 frozen908 0PASS/2FAIL→960 2PASS로 재현했다.

956에서 새 정확한 예산 검사가 ordinary 컷15/기대16으로 실패했다. source guard 뒤의 scalar 비교도 인증한 같은 물리 높이를 쓰도록 고쳐959의 새·기존2PASS를 확인했다. 새 source 단위 함수는 늘리지 않고, 기존 reset child 계약 함수 안에 연속 HWPX의 ordinary/row-offset 정확한 예산·9HU 부족 예산·예약/paint 높이 일치 assertion을 보존했다.968의 unit-tier도 통과했다.

960/961 재빌드 뒤962에서 전72쪽을 다시 산출했다.969에서72쪽 PNG 모두943/946과 byte 동일함을 확인했고, TSV는 최신962의 전쪽 재산출본으로 교체했다. 기존 직접 판독 재사용의 hash 근거이며, 물리 예산 assertion을 생략한 근거가 아니다.968의 cfg(test) 재배치 전후 runtime prefix SHA `df55638f7815cd92914881e86414f019655540a57d29e59bf1072304a7e5184f`는 동일하고 다른11개 제품 파일도 같다.

## 로컬 원장

로그·생성 JSON·전체 producer/hash 원장은 ignored `output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/`의924–970에 보존한다. 핵심 정본은960 Native,961 fresh WASM,962 전쪽,966 정상 wrapper,967 실제 초기화 응답,968 정규 검사,969 전후 품질이다. 생성 JSON·SVG·글꼴 바이너리는 커밋하지 않는다. 원본 source와 독립 Print PDF는 기존 추적 파일을 사용한다.
