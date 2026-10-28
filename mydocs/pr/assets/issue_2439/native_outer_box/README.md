# #2439: 실제 저장된 빈 host 표의 위여백과 후속 본문

## 입력 계약과 독립 기준

이 자료는 원본 `issue2439_repeat_table_overlap.hwp`의 전체 결함 해결을 뜻하지 않는다.
`issue1549_empty_host_float_clamp.hwpx`에서 앞 문단의 무관한 표를 제거한 진단 입력을
한컴 2020에서 **HWP로 다시 저장**한 두 작은 대조군이다. 두 번째 대조군은 위 바깥여백을
283 → 850 HU로 변경했다. 저장 뒤 provenance·합성 태그·vpos는 수정하지 않았다.
이 실제 저장 바이트를 다시 독립 Print했다. 재저장 전 합성 입력의 PDF를 재사용하지 않았다.

| 입력 (`samples/issue2439/`) | HWP SHA-256 | 독립 PDF (`pdf/issue2439/`) | PDF SHA-256 |
| --- | --- | --- | --- |
| isolated-base-native-2020.hwp | c5f8dd297317c3f82885f573ad13d8fe2aee9ff9379273cc58ea1f2becf549d9 | isolated-base-native-2020.pdf | 68bfaadabbc07775194a9f9faf6b0f24b766ccaa5d84ff4c3f6e0db534661fda |
| isolated-top-margin-native-2020.hwp | 27893011b672b23b71cd3823cca82b59afbcad4306e9026e69fd2974fdf0dd9e | isolated-top-margin-native-2020.pdf | fab9d7b502e38352fd4b453c0dfe5c47913edd6ee21ae05de2dc4901c7cca699 |

한컴 엔진은 `11.0.0.9136`, 각 PDF는 1쪽이다. PrintMethod=0,
`hancom2020_pdf_driver_one_up` 경로다. 재저장 작업은
`07f643d5-a968-43eb-b5fd-80b63fcd8089` / `443c07a2-5f84-42f1-abc6-14afb528b14d`,
그 저장본의 Print 작업은 `2238d748-964d-4b28-99bf-6700c1022d6a` /
`1c5ffa19-4dac-4e06-8532-644ae79e5fd5`다. 클라이언트의 `start --input <HWP>
--target hwp|pdf --engine 2020 --timeout-seconds 300`으로 실행했다.
인증은 로컬 env-file로만 전달했으며 이 자료에 포함하지 않는다.

## 실제 결함과 공통 결과

실제 저장 셀의 lineHeight=2600 HU는 textHeight=2318 HU와 안 여백=282 HU를 이미
포함한다. 측정이 안 여백을 재가산하여 닫힌 원본 프레임의 수용을 거부했다.
`stored_plain_closed_cell_ink_height`에서 실제 단일 저장 줄의 이 계약을 인정하고,
기존 측정과 배치가 같은 내용 높이를 소비한다. 빈 host lane도 block·중첩 표와 같은
`ParagraphFloatPlacement::from_empty_stored_ladder_host`를 소비한다. 최종 lane 이동은
표 원점·점유 끝·뒤 저장 원점을 함께 옮긴다. 좌우 여백의 대칭 여부로 위여백을 버리지 않는다.

관측은 다음과 같다. 좌표계는 96dpi 페이지 좌표다.

| 입력 | 독립 Print 표 윗변 px | 수정 전 Native px | 수정 후 Native px | Native raw | fresh WASM raw |
| --- | ---: | ---: | ---: | ---: | ---: |
| isolated-base | 165.2586 | 161.6 | 165.4 | 100.00000% | 100.00000% |
| isolated-top-margin | 172.7707 | 161.6 | 172.9 | 100.00000% | 100.00000% |

정식 검사는 표 윗변을 독립 Print의 ±2px로 보호하며, C11의 셀 내부 포함, 32개 후속
문단의 순서·누락·중복·표 뒤 배치·본문 내부 포함도 확인한다. 다른 세 검사와 내부의
접힌 저장 사다리 계약 검사는 유지한다. 과거 검사가 HWPX의 provenance와 일부 합성
표시·vpos만 조작하여 불완전한 저장 사다리를 만든 문제는 `issue_2439_before.rs.txt`에
원문 그대로 보존했다. 입력 교체를 기존 합성 입력의 출력 개선으로 보고하지 않는다.

## 실행과 직접 판독

실행 코드는 `ff4674e33d8ac3954e28348e6eb4f2ba87c9e71d` +
생산 패치 SHA-256 `8bc0ffe9a67a98b44b303db17aaa3a515522187bf5bcfb03dcf44952b9402e10`이며,
새 `stored_padded_line.rs` SHA-256은
`3299f0d97bc8e5c09ac935f98aeb0cc2f8316ff54717f7b61b85807acb863505`다.
패치에는 별도로 검토 중인 overlay 물리 컷 변경도 포함된다. 소스 대응과 정확한 명령은
ignored `output/pr-review/planet6897-20261018/closed-padded-float-frame-{build,wasm-build}-results.json`에 있다.
Native는 `release-test`, 진단 opt-level=1이다. WASM은 공용 `target/pr-review`, jobs=8,
codegen-units=16으로 `scripts/wasm-pack-locked.sh --target web --out-dir pkg`를
실행하여 wasm-opt까지 완료했다. WASM SHA-256은
`ce310bcf2be1f98aa2a8b5d345556e7a68713ca3f03a25c82431e8fd0fb6c7dc`다.

```bash
venv/bin/python scripts/visual_sweep.py --silhouette-only \
  --file-target isolated-base samples/issue2439/isolated-base-native-2020.hwp \
  pdf/issue2439/isolated-base-native-2020.pdf --compat 2022 --dpi 96 \
  --page-workers 1 --rhwp-bin output/pr-review/planet6897-20261018/closed-padded-float-frame/rhwp \
  --out output/pr-review/planet6897-20261018/issue2439-native-repeat
```

다른 입력도 같은 인자의 입력 이름을 `isolated-top-margin`으로 변경해 실행한다.
fresh WASM은 같은 인자에 `--wasm-pkg
output/pr-review/planet6897-20261018/closed-padded-float-frame-wasm/pkg`를 추가한다.
보존한 TSV는 실행 시 출력 원문이며 Native/fresh WASM 각각의 하위 폴더에 둔다.

Native/fresh WASM의 각 review·overlay를 직접 확인했다. 두 입력 모두 표 외곽, C11,
제목, 후속 32개 문단의 위치·순서·간격에서 배치 결함이나 겹침은 보이지 않는다.
가는 선과 글자의 농도·안티앨리어싱에는 차이가 남는다. raw 100%는 2px 실루엣 일치이며,
색·농도를 포함한 모든 픽셀의 동일함을 뜻하지 않는다. 정상 대조군 H1~H6도 fresh WASM
전체 6쪽 raw 100%다. 이 제한된 확인은 최종 통합 head의 전체 회귀·대상 문서 검증을 대신하지 않는다.

변경한 정식 검사 자체도 이전 `empty-line-owner` 라이브러리에서 Print 윗변 차이로
실패(exit 101)하고 현재 라이브러리에서 통과했다. 로컬 원장은
`qualified-issue2439-before-results.json`과 `qualified-four-failures-source-content-contracts-results.json`이다.
집중 47건 결과는 46건 통과·1건 실패(#7687 재활성화 검사)다.
TextRun bbox의 lineHeight는 저장 할당 상자이므로 잉크 높이로 오인하지 않는다.
셀 내부 포함 검사는 실제 배치 원점과 독립 원본 textHeight의 내용 프레임으로 확인한다.
