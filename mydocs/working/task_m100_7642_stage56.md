# 이슈 #7642 Stage 56 — 한 줄 분할 후보와 명시적 문단 묶음

Issue: #7642

## 현재 source와 독립 출력

구현 source `f2fa41f79b9a4eb10476599f20dda2e3a1df5ddf`의 9월 미주 사이 20
HWP 16쪽은 98.29265%로 개선됐지만 17쪽은 69.67924%다. 전체 24쪽
Native TSV도 완료됐으며 최저는 37.48785%다. Print와 전체 쪽수는 같아도
모든 페이지 통과는 아니다. 같은 source의 9월 구분선 아래/위 20 HWP는
23쪽 전체에서 최저 90.77312%로 측정됐다. fresh WASM은 빌드 중이다.

17쪽에서 Print는 원본 본문 235, 미주 26 내부 5의 (iii) 제목 한 줄을
왼쪽 단 하단에 두고 나머지 설명·수식을 오른쪽 단에서 이어간다.
Native는 문단 전체를 오른쪽에서 시작해 뒤 문항까지 한 줄 밀린다.
앞 문단의 같은 부분은 Print와 대응한다.

원본 para shape 7의 attr1=10244이며 widow_orphan·keep_with_next·keep_lines
비트는 모두 0이다. 내부 5는 저장 줄 여섯 개와 명시적 개행을 가지며
ColumnBreakType=None이고 vpos는 증가한다. 원본 JSON과 attr1 관측은
ignored `stage56-sep-note26-source235.json` 및 진단 출력에 보존했다.

## 확인한 호출 경로와 아직 관측하지 않은 값

`initial_fit.rs`는 남은 높이로 split 후보를 만든 뒤
single_line_tail_split_at_bottom 조건으로 split=1을 제거한다. 이 조건은
비기본 미주 간격·미주 payload로 결정하며 명시적인 widow/keep 속성을
조회하지 않는다. 해당 입력은 이 묶음 조건에 해당한다. 그 뒤
`query_tail_fit → advance_for_fit → paragraph.rs의 단 전환 → emit`으로
이어진다. 실제 실행 로그에서도 내부 5가 단 전환을 일으킨다.

후반의 EN_FIRST_LINE_FIT는 이미 현재 단의 항목이 없는 상태에서
flow=None/occupied=None이다. 이를 앞 단에서 첫 줄이 넘쳤다는 증거로
사용할 수 없다. initial split의 값과 제거 조건의 실제 평가 값은 아직
이 실행에서 출력하지 않았으므로, 필터가 이 후보를 제거했다는 결론은
현재 코드와 조건에 근거한 추론이며 다음 진단으로 확인한다.

## 함께 확인할 수정 방향

자동 컷의 생산·유효성·예약과 단 전환 전에 실제 앞부분 점유를 확인한다.
한 줄을 남길 수 있는지는 문서별 간격·하단 비율로 임의 판정하지 않고
원본의 widow_orphan·keep_lines·명시적 경계와 같은 실제 배치 문맥으로
결정한다. 후행 간격과 글줄 점유를 구분하고, 이미 넘긴 단의 fit 결과를
앞 단의 판정으로 재사용하지 않는다.

다줄·수식·명시적 개행, 현재 들어가는 첫 줄, 실제 첫 줄도 넘치는 경우,
widow_orphan/keep_lines가 켜진 경계와 별도 float 계획을 대조한다.
원본이 정상인 다른 문서의 기존 컷을 보존해야 하는 범위도 확인한다.
현재 Native TSV의 뒤쪽 실패 입력·최초 차이와 같은 원인의 적용 범위를
모은 뒤 함께 수정하고 Native/fresh WASM에서 재검증한다.

미주 사이 20의 17·18·19·21·22·23쪽은 아직 90% 미만이다. 이 문서의
회귀 기대값·baseline은 변경하지 않는다. 전쪽 90% 이상과 확대 판독 후
소유·줄 범위·내용 보존·앞뒤 순서·포함 관계로 정식 회귀를 변경한다.
현재는 분석·계획이며 새 코드 보정이나 최종 판정은 아니다.

## 공통 수용 결과의 구현

미주 간격으로 한 줄 컷을 제거하던 판정을 원본 keep_lines·widow_orphan
계약으로 바꿨다. 연속된 일반 다줄 문단은 같은 단 배치의 전체 점유와
허용되는 prefix의 점유를 조회해 Whole / Split(줄 끝) / Overflow를 반환한다.
그 결과를 초기 컷, 자동 이월 취소, 최종 컷과 PartialParagraph 방출에
연결했다. 단 전환이나 선행 방출 뒤에는 이전 단·항목 수의 결과를 재사용하지
않는다. 별도 개체 계획, 명시적 경계, 다음 문단과의 keep 그룹, 저장 줄과
포맷 줄 수가 다른 경로는 기존 계약을 유지하고 이 조회의 통과로 보고하지 않는다.

빌드·진단에는 원래 스타일 검사 전 컷과 실제 수용 결과를 함께 기록한다.
특정 문단 번호·진단 높이·고정 하단 여유를 구현 조건으로 넣지 않았다.
새 구현의 빌드·시각 검증은 아직 미검증이다. 같은 코드의 전체 범위를
확인한 뒤에만 회귀를 관계형으로 바꾼다.

앞 source f2fa41f79의 fresh WASM은 1245.45초에 exit 0으로 완료됐고,
`rhwp.js`와 `rhwp_bg.wasm`의 package/public SHA-256이 각각 일치했다.
WASM SHA-256은 `4a9e7897de9d742d9d20920979f811cece3c995f387d1bec8623c51d34b561ad`다.
그 고정 package의 전쪽 WASM TSV는 진행 중이며 새 수정의 결과로 재사용하지 않는다.

## 새 source의 실행 증거

구현 source는 `fb627e5345607de8556fba1ef1c2acf28b484161`이다. Native 빌드는
751.03초에 exit 0으로 완료됐다. 같은 source의 `cargo fmt --all -- --check`는
64.12초, Native `cargo clippy --locked --target-dir target/pr-review -- -D warnings`는
249.88초에 각각 exit 0이었다. workspace·WASM Clippy·전체 nextest의 통과를
뜻하지 않는다.

고정 Native 실행 파일은 ignored `stage15-current-validation/rhwp-endnote-prefix-contract`이며
SHA-256은 `4508949d2e5096f41791f318fbb4cc55aaee0713a79a0fc9af9e2deab3c431af`다.
빌드 전후 source 파일 해시를 확인했고 `endnote-prefix-contract-native-provenance.json`에
명령·source·실행 결과를 보존했다.

새 진단은 실제 이전 컷과 최종 수용 결과를 확인했다. 본문 235, 미주 26 내부 5의
가상 문단 872에서 `before_style=Some(1)`, `actual=Some(Split(1))`, `cut=Some(1)`이다.
이때 단은 0, 앞 항목 수는 24, 기존 누계 높이는 975.680이었다. 이 높이는 관측값이며
구현의 고정 허용값이 아니다. 새 `dump-pages`에서는 같은 원본 소유자를 가진
PartialParagraph가 17쪽 왼쪽 단 `[0,1)`, 오른쪽 단 `[1,6)`으로 나타난다.
줄 범위의 합집합은 원본 여섯 줄이고 교집합은 없다. 앞 내부 4는 왼쪽 단,
뒤 내부 6·7은 오른쪽 단에 보존됐다.

`stage56-paragraph-cut-owner-proof.json`에 원본·독립 Print PDF·진단 파일의 SHA-256과
실제 항목을 함께 고정했다. 이 결과는 줄 소유·분할의 증거이며 시각 일치율이나
수식 간격의 통과 판정으로 대신하지 않는다. Native 집중 검증은 8개 문서의
37개 페이지를 진행 중이다. 같은 source의 fresh WASM은 루트 wrapper와 공용
`target/pr-review`로 새로 빌드 중이다. 기존 source f2fa41f79의 전쪽 큐는
별도의 변경 전 증거로 유지하며 최신 source의 통과 건수에 합산하지 않는다.

## 17쪽의 새 이미지와 확대 판독

새 Native 17쪽의 일치율은 96.65449%다. 앞 source f2fa41f79의 69.67924%와
같은 원본·Print PDF를 비교했다. 새 full review와 원본 소유자 872의 bbox로
선택한 왼쪽 제목·오른쪽 이어받기 영역의 3배 확대를 직접 표시했다.
제목 (iii)는 Print처럼 왼쪽 단 마지막 줄에 있고, 오른쪽 첫 줄은 A·B·C를
한 명으로 묶는 설명부터 시작한다. 4!와 2의 곱, 6! 분모, 마지막 1/15의
분수 구성과 설명의 줄 순서도 확대해 확인했다. 글꼴 획·래스터 차이는 남는다.
이 페이지의 개선을 뒤쪽 페이지나 fresh WASM의 통과로 확대하지 않는다.

full review는 ignored `endnote-prefix-contract-focused-native/all-11/all-11/review/review_017.png`,
확대 이미지는 `stage56-sep-note26-p017-split-detail-review.png`다.
`stage56-sep-note26-p017-split-detail-provenance.json`에 변경 전후 PNG·Print PNG의
해시, 원본 소유자, 관심 영역과 배율을 보존했다. 같은 좌표에서 잘라 확대했으며
별도 영역 점수를 만들거나 본문 일치율 대신 사용하지 않았다. Print PNG의 높이는
1122, Native는 1123이다. 관심 영역은 양쪽 범위 안에 있으며 재배율하지 않았다.
뒤쪽 집중 검증·전체 페이지·fresh WASM은 아직 미완료다.

fresh WASM 첫 시도의 실행 세션은 exit 143으로 종료됐다. 빌드 성공으로
기록하지 않았고 첫 시도 provenance·로그를 별도로 보존했다. 실제 Cargo·rustc·
wasm-pack 종료를 확인한 뒤 같은 source 파일 해시와 `HEAD:src` 일치 조건으로
공용 캐시에서 재시작했다. 문서만 추가된 HEAD와 구현 source SHA를 구분한다.
재시작은 실행 세션과 분리한 프로세스로 진행하며 성공 여부는 wrapper exit code와
package/public 해시 일치로 확인한다. 현재 fresh WASM 통과는 여전히 미검증이다.

9월 미주 사이 20 HWP의 선택한 12쪽 Native 실행은 1630.42초에 종료됐다.
누락 쪽은 없고 전체 쪽수는 독립 Print와 같은 24쪽이다. 17쪽은
69.67924→96.65449%, 22쪽은 58.20979→96.12768%로 개선됐다. 새 22쪽
review도 직접 표시했다. 최저 55.60179%이며 18·19·21·23쪽이 90% 미만으로
exit 1이다. 18쪽은 67.69353→79.55629%, 21쪽은 37.48785→55.60179%로
개선됐지만 통과하지 못했다. 19·23쪽은 각각 87.59770%, 66.43021%다.
이 결과는 8개 문서 집중 큐의 첫 문서 종료이며 큐 전체 종료가 아니다.
미달 컷과 간격의 후속 분석은 Stage 57에 이어 기록한다.
