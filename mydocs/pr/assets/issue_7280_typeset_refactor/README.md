# #7280 대표 시각 증거

생성 source: `29130d5397da3dc3cd3d1d9416b153584e1a4f41`.
출력 보존 비교 base: `1966af77fa8046c844d654b157b5168baad8a30e`.
asset을 담는 후속 문서 commit과 실제 바이너리의 source SHA를 구별한다.
[실행·직접 판독 기록](../../../working/task_m100_7280_stage58.md),
[최종 결과보고](../../../report/task_m100_7280_report.md).

## 대표 입력과 판독

| 출력 | 입력 / 기준 PDF | 쪽·판독 | 보존 PNG |
| --- | --- | --- | --- |
| Native | [square-host 원본](../../../../samples/issue5809/156518601_p1_square_host.hwpx) / [PDF](../../../../pdf/issue5809/156518601_p1_square_host-hwpx-2020.pdf) | 1쪽. 표·본문 흐름 유사, 굵기/자형 차이 유지. proxy 9.77316% | [review](native-square-host-p001-review.png), [overlay](native-square-host-p001-overlay.png) |
| fresh Docker WASM | [chemical 원본](../../../../samples/issue6782/1480000-201900042-chemical-product-labeling-study.hwp) / [PDF](../../../../pdf/1480000-201900042-chemical-product-labeling-study-2020.pdf) | 14쪽. 표가 PDF보다 위에 놓이는 차이 유지. proxy 21.71057% | [review](wasm-chemical-rewind-p014-review.png), [overlay](wasm-chemical-rewind-p014-overlay.png) |

내용 픽셀 중심 자동 일치율 보조값은 높을수록 비슷하고 낮을수록 검토할 잉크 위치·형태 차이가 크다.
사람 판정 정확도가 아니다. review 패널은 rhwp/PDF/overlay 순서다. WASM review 상단의 긴
진단 라벨 일부가 오른쪽 끝에서 잘리지만 문서 영역과 위 표의 proxy 값은 보존되어 있다.
이미지를 수정하거나 문서 영역을 잘라 차이를 숨기지 않았다.
이 라벨 폭 초과는 [별도 도구 이슈 #7349](https://github.com/edwardkim/rhwp/issues/7349)로
추적한다. #7280 조판 구조 변경의 제품 결함과 구별하며 원래 증적과 수치를 보존한다.

원본 PNG 위치는 `output/7280/stage58-integration/` 아래 다음 경로다:

- `native/head/square-host/square-host/review/review_001.png`
- `native/head/square-host/square-host/overlay/overlay_001.png`
- `wasm/head/chemical-rewind/chemical-rewind/review/review_014.png`
- `wasm/head/chemical-rewind/chemical-rewind/overlay/overlay_014.png`

각 compare는 같은 문서 경로의 `compare/compare_NNN.png`에 있다. PNG는 원본 바이트 그대로
복사했다. 모든 입력/PDF는 base와 실행 head에 이미 존재하며 로컬 파일을 `git show SHA:path`의
blob 또는 LFS SHA-256과 대조했다(14원본 + 11PDF 모두 일치). 기존 파일을 중복 커밋하지 않는다.

| 대표 자료 | SHA-256 |
| --- | --- |
| square-host 원본 | `ed9a0589d9223c2750f4fb8240548551d4aa70fd35d6243f405183d525ff1f1f` |
| square-host PDF | `6be7d47ef90d0af026705e690bbf1aedd4b1254f17668c7d92e8120d4bcc5ec5` |
| chemical 원본 | `398d03a5d5e4d6e857086be532d6d9ed0cec9c8ad06f95c17bbb7f83056ae860` |
| chemical PDF | `f8e5c0408e221080ede9a9a67b153d02d792d22961c738e46749641f32a32e79` |
| Native 바이너리 | `aed84dd04813cb43d4c76baae43237d08e2f282f1e57984de7ca0504982612dc` |
| fresh WASM | `e28071e88db21df4a7d9ccaac9bdfe472c196f0b2af178f71a269fa6d8c3d3f4` |

## 검증 대상 재현 목록

다음 경로는 저장소 root 기준이다. PDF 없는 3개는 Native 데이터 대조군이며 한컴 피델리티
검증으로 세지 않는다. PDF 있는 11개는 Native/WASM 모두 전체 SVG/render tree를 비교하고
표시한 23쪽 PNG를 비교했다. 직접 판독한 쪽의 범위와 잔여 차이는 Stage58에 따로 적었다.

| key | 원본 | PDF | PNG 쪽 |
| --- | --- | --- | --- |
| square-host | `samples/issue5809/156518601_p1_square_host.hwpx` | `pdf/issue5809/156518601_p1_square_host-hwpx-2020.pdf` | 1 |
| square-body | `samples/issue6175/seed_expo_square_float_body.hwpx` | `pdf/pr_6314_issue6175_seed_expo_square_float_body-2020.pdf` | 1 |
| square-table | `samples/issue4090/156492236_규제샌드박스_min.hwpx` | `pdf/issue4090/156492236_규제샌드박스_min-hancom2020-production-verify.pdf` | 5–8 |
| night-guard | `samples/issue4599/36374873_night_guard_log.hwpx` | `pdf/36374873_night_guard_log-2020.pdf` | 1 |
| deferred-picture | `samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp` | `pdf/pr3740/hwp/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구)-2020.pdf` | 75,76,90,91 |
| tac-order | `samples/issue5807/coanchored_float_tac_order.hwp` | 없음 | 없음 |
| tac-tail | `samples/issue5701/1530000-200800002_slice_p139_tac_reset_tail.hwp` | 없음 | 없음 |
| square-tac | `samples/issue6797/156160455-social-pig-farm-income.hwp` | 없음 | 없음 |
| endnote-2022-09 | `samples/3-09월_교육_통합_2022.hwp` | `pdf/3-09월_교육_통합_2022.pdf` | 9,12 |
| endnote-between20 | `samples/3-09월_교육_통합_2024-미주사이20.hwp` | `pdf/3-09월_교육_통합_2024-미주사이20-2024.pdf` | 21,22 |
| endnote-zero | `samples/3-11월_실전_통합_2024-구분선위0미주사이0구분선아래0.hwp` | `pdf/3-11월_실전_통합_2024-구분선위0미주사이0구분선아래0.pdf` | 15,16 |
| endnote-no-separator | `samples/3-11월_실전_통합_2024-구분선없음구분선위20미주사이20구분선아래20.hwp` | `pdf/3-11월_실전_통합_2024-구분선없음구분선위20미주사이20구분선아래20.pdf` | 15,16 |
| market-rewind | `samples/task2070/1130000-201900011_D0150004-1-002_2017년기준 시장구조조사.hwp` | `pdf/task2070/1130000-201900011_D0150004-1-002_2017년기준 시장구조조사-2022.pdf` | 4,5 |
| chemical-rewind | `samples/issue6782/1480000-201900042-chemical-product-labeling-study.hwp` | `pdf/1480000-201900042-chemical-product-labeling-study-2020.pdf` | 13,14 |

## 원격 게시와 merge 후 사용 계획

원격 제출은 별도 승인 사항이다. PR 본문에는 최종 제출 head를 고정한
`https://raw.githubusercontent.com/edwardkim/rhwp/<head-sha>/mydocs/pr/assets/issue_7280_typeset_refactor/<PNG>`를
실제 Markdown image로 넣는다. 게시 후 API와 PR 화면에서 SHA/파일/본문/표시를 확인한다.
이 로컬 준비만으로 원격 이미지 접근 또는 CI 성공을 주장하지 않는다.

추후 merge 댓글을 승인받으면 이 4개 PNG의 merge commit SHA 고정 URL, 직접 판독한 위 2쪽과
proxy/한계, [Visual Sweep 정본](../../../manual/verification/visual_sweep_guide.md#github-merge-comment)을
함께 사용한다. `--body-file`로 게시하고 API로 본문을 재조회한다. 코드 변경이 생기면 오래된 캡처를
재사용하지 않는다. merge/댓글 승인이나 실행은 이번 준비에 포함하지 않는다.
