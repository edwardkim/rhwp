# 인라인 개체가 있는 저장 조각의 완결과 캡션 새 쪽 예산

Issue: #7445 / task1725 / #7379. 부모 HEAD `46fe448a0f`. 기존 문서와 정상·합성 대조군만 사용했다. 새 HWP 보정 대상을 추가하지 않았다.

## 입력과 생산자

- 원본: `samples/task1725/text_footnote_tail_overpagination.hwp`, SHA-256 `d335c26cbcd57b157c929ed36596d95a6676bea5624348baae3bff3b55a705d9`.
- 독립 Print: `pdf/task1725/text_footnote_tail_overpagination-hwp-2024.pdf`, SHA-256 `03ddf66494ebd1b25cfe49344039f0c6e324a0a837d3d4dce9f26bc14141e8af`. 원본과 PDF 모두 242쪽이다.
- before Native는 Stage69 producer이며 Stage70은 테스트만 고쳐 제품 소스가 동일하다. 이번 후보와 같은 글꼴·96 DPI·compat2024·Print·PDFium153·2px raw 기준을 적용했다.
- after Native: `713c3a3b8c53a45880a67b9c34a578c5d3c8d3e1c7a55999c4512ffe49ca939b`.
- fresh WASM: `10f0dfb40866664785ffe4d61031009e3ec71c29f1ea4ac3b55734ef3cf13cc6`. Mac locked wrapper의 web/pkg/no-opt 로컬 대체 빌드이며 Docker 최적화 검증이 아니다.
- 루트 pkg, rhwp-studio/public, 보존 package의 JS/WASM hash가 일치한다. 실제 Studio 초기화 응답·Actual36 검증은 아직 아니다.

## 전쪽 결과

| 출력 | 전체 쪽수 | 최저 raw | 90% 미만 |
| --- | ---: | ---: | --- |
| [before Native](before-native.tsv) | 242 | 13.12266% | 30·201쪽 |
| [after Native](after-native.tsv) | 242 | 71.82002% | 201쪽 |
| [after fresh WASM](after-wasm.tsv) | 242 | 71.95793% | 201쪽 |

Native 전후 raw는 29쪽 96.36000→99.87634%, 30쪽 13.12266→97.08818%이며 나머지 240쪽은 동일하다. fresh WASM의 29·30쪽은 99.87630/97.08815%다. 평균이나 29쪽 점수로 201쪽을 상쇄하지 않는다. devel 전쪽 비교의 완료 주장도 아니다.

29쪽에서 상단 두 행과 병합 셀의 인라인 수식이 함께 남고, 30쪽은 남은 두 행·뒤 설명·다음 절을 소유한다. 독립 PDF와 직접 판독했다. 표의 얇은 괘선·수식 자형/자간·일부 가장자리 차이는 남는다. 201쪽의 도형·후속 표가 위로 이동하는 기존 차이는 미완료다.

## 관계 검사와 공통 소비 경로

원 해결 task1725 `54c01b3420335d39507fd6252537e824118a7a91`(통합 `9853e8a1`)과 캡션 보정 `f72f1a68427`을 추적했다. 29쪽의 저장 common frame 6303HU 안에서 마지막 행의 내용·패딩 19.76px가 잔여 41.89333px 안에 든다. 원본 인라인 개체도 유효한 저장 줄과 실제 공통 CellUnit으로 완결을 확인한다. nested/floating/각주 개체·저장 재시작·편집된 프레임은 이 인증에 포함하지 않는다. 같은 완결 행의 물리 상자를 scanner의 cut/fit와 paint가 소비하며 임의의 높이 허용을 더하지 않는다.

마지막 캡션은 아직 소비하지 않은 병합 블록과 종료 공간이 새 쪽에 들어갈 때 같은 블록으로 재스캔한다. 앞 쪽의 본문 각주가 들어간 누적 예약을 새 쪽 각주로 재사용하지 않는다. 마지막 블록에 속한 실제 note의 내용·구분선·note 사이 간격을 기존/current와 fresh가 공유하는 계산으로 예약한다. 새 쪽보다 큰 유닛과 이미 소비한 블록의 내부 컷은 유지하며 TABLE raw2/CELL을 행 경계 전용으로 바꾸지 않았다.

기존 두 모듈의 최종 실제 실행은 **20 PASS**, 테스트 5.050초다. 삭제·skip·기대값 완화는 없다. 합성 셀 병합 후 주소 격자도 실제 편집 경로처럼 재구성했다. 별도 외부 하네스에서도 같은 조건을 통과했고 최종 정식 모듈을 재실행했다. source 수식/각주 소유·남은 행·캡션 1회·본문 경계를 기존 관계 검사로 보호한다. 전 242쪽이 아직 90% 이상이 아니므로 새 픽셀 핀이나 fixture는 추가하지 않았다.

## 직접 판독 PNG

- 이전 29쪽: [review](before-native-p029-review.png) / [overlay](before-native-p029-overlay.png).
- 이전 30쪽: [review](before-native-p030-review.png) / [overlay](before-native-p030-overlay.png).
- after-native 29쪽: [review](after-native-p029-review.png) / [overlay](after-native-p029-overlay.png).
- after-wasm 29쪽: [review](after-wasm-p029-review.png) / [overlay](after-wasm-p029-overlay.png).
- after-native 30쪽: [review](after-native-p030-review.png) / [overlay](after-native-p030-overlay.png).
- after-wasm 30쪽: [review](after-wasm-p030-review.png) / [overlay](after-wasm-p030-overlay.png).
- after-native 201쪽: [review](after-native-p201-review.png) / [overlay](after-native-p201-overlay.png).
- after-wasm 201쪽: [review](after-wasm-p201-review.png) / [overlay](after-wasm-p201-overlay.png).

원 로그·전체 PNG·render tree·producer/source/test hash는 ignored `output/pr-review/7445-completed-20261009/rebase-8155-20261022/stage71-remaining-contracts/`에 보존한다. WASM은 32쪽 묶음으로 전체 TSV를 다시 산출했으며 완료 SVG/font_policy/raw_svg만 SHA manifest를 남긴 뒤 정리했다. 삭제한 SVG를 재사용 가능 증거로 주장하지 않는다.

## PR 보류

201쪽, 기존 overlap partition12의 정상 대조군, C354의 devel 대비 4.28268%p 하락·C356 미달, 새 head 전체 nextest·범위 게이트·영향 문서와 backend의 전쪽 비교·실제 소비 경로 검증이 남았다. 이 단계는 로컬 체크포인트이며 PR 준비 완료가 아니다.
