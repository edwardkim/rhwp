# 외부 생성 HWP3의 원본 줄 재구성과 표 뒤 흐름

- 원본: `samples/issue5699/20099369_yeongwol_forms.hwp`
- 독립 한컴 2020 Print PDF: `pdf/issue7642-failed-inputs-followup-20261011/20099369_yeongwol_forms-2020.pdf`
- Native 전4쪽: 99.26310 / 100.00000 / 99.19799 / 100.00000%.
- 실제 출력 쪽수: 원본 4 / 기준 PDF 4.
- 빌드: `stage146d-native-realign-build.json`, exit 0, 실행 전후 source 해시 일치.
- Native 바이너리 SHA-256: `dc3da30218ab08821c8cb95a96dccb2da9d9b891f8b8eb012053a710760206bd`.
- TSV·실행 증거: ignored `output/pr-review/issue7642/failed-inputs-20261008/stage15-current-validation/stage146d-realign-preflight/`.
- 대표 PNG는 이번 코드의 새 Native 출력이다. fresh WASM 및 전체 범위 검증은 미완료다.

직접 판독에서 첫 표 뒤 문단·서명 원점과 조건9·안전 안내의 두 줄 구성이 대응한다.
원본의 표 바깥 상자·서식별 쪽 소속을 보존했다. 괘선의 잉크 두께와 일부 글자 모양은
독립 Print와 차이가 남아 있으며 전체 시각 완료로 판정하지 않는다.
정상 저장 HWPX는 원인 진단에만 사용했고 원본 입력의 통과를 대체하지 않았다.


fresh WASM 전9쪽도 같은 source `fe5ad4db922e03f38665a6c1fb7ba34627e13596`에서
Native와 같은 페이지별 점수로 통과했다. Root package·고정 검증 복사본·Studio JS/WASM의
SHA-256 일치를 확인했다. `stage147-wasm-provenance.json`과
`stage147-fresh-wasm-preflight/receipt.json`이 실제 실행 근거다.

3쪽 확대 판독에서 마지막 행의 `월별/누계`가 아래로 밀려 셀 하단에 걸리는 결함을
확인했다. 99% 이상 점수로 이 결함을 면제하지 않는다. 재구성된 두 줄의 높이가 세로
정렬 전에 작은 저장 extent로 다시 덮이는 소비 경로를 다음 보정 대상으로 기록한다.
이 페이지의 신규 관계형 회귀 추가와 시각 완료 판정은 보정·재출력 뒤 진행한다.
