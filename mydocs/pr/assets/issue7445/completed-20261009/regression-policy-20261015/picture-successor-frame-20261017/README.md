# 생물독소63쪽 그림 뒤 캡션의 원본 좌표계 보존

## 독립 근거와 원인·소비 경로

원본 `samples/issue5941/1480000-201900698-native-neartop-reset.hwp`205쪽의 SHA-256은
`18765ac22270a2cb4ece6a1e6539c2dea6ca6ccde2573b42fb9acce7c34fa83a`다.
독립 한컴2020 Print `pdf/issue5941-1480000-201900698-2020.pdf`의 SHA-256은
`fe01f1ddffc3153d015adab93178f0fa8a5e9624763ff529362d4410d50955dc`다.
compat2022·print·96DPI, 기존 공통 공급 폴더와 실제 `Library/Fonts`를 사용했다.

4구역 문단343의 그림은 문단 기준 어울림(Square)이며 호스트 글자 영역의 폭은0이다.
문단344의 저장 시작63554HU는 그림 전체 아래에 있다. 그림 위치 자체는 정상인데,
그림을 칠한 후 paint가 유효한 단 좌표계를 버리고 빈 호스트의 순차 커서에서
lazy base1900HU를 재계산했다. 캡션과 뒤 문단이1900/75=25.33333CSSpx 올라갔다.

수정은 `HeightCursor::vpos_adjust`의 저장 좌표계 복원이다. 원본 저장 줄·미편집 상태,
원래 단 좌표계, 빈 어울림 그림 호스트, 그림 전체 아래에 있는 후속 저장 줄을 확인한다.
이 근거가 있을 때 원래 단의base와origin을 유지한다. 캡션을 그림 끝으로clamp하거나,
문서 이름·그림 번호로 예외를 추가하거나, 그림 자체 좌표를 옮기지 않는다.

`원본 LINE_SEG/그림 프레임 → HeightCursor의 확정 단 원점 → typeset/section/vpos의 측정 →
layout의 같은 vpos_adjust → 최종 TextRun 원점`으로 연결된다. 두 소비 경로가 같은 결과를
사용한다. 편집·합성 줄·원점 불명·그림 옆의 후속 줄은 이 복원 근거가 없다.

## 전쪽 비교와 직접 판독

| 대상 | Native raw | fresh WASM raw |
| --- | ---: | ---: |
| 수정 전63쪽 |93.26163%|93.26176%|
| 수정 후63쪽 |99.95764%|99.95854%|
| 수정 후205쪽 최저 |95.69921%|95.70411%|

양쪽205/205쪽·90% 미달0쪽·누락0쪽이다. raw로 판단하며 색상 경계 조정값으로
90/98%를 대신하지 않았다. C80과C81의205쪽 PNG를 전수 hash 대조하면63쪽만 다르고,
양쪽 모두0.01%p 이상 하락한 쪽은 없다. 이 비교는직전C80 대비이며 최신devel 무회귀
전쪽 검증 전체의 대용이 아니다.

전후 review·overlay에서 그림을 유지하며 캡션이 그림 밖으로 내려온 것을 확인했다.
주석 두 줄·다음 소제목·쪽번호도 확인했다. 글자 주변AA와 그림 내부의 미세 raster 차이가
남으므로 엄격 RGB 픽셀 동일이라고 주장하지 않는다. 새 최저55쪽 표의 잔차는 별도 확인 대상이다.

## 회귀 검사와 생산 출처

`picture_successor_keeps_caption_and_following_body_below_the_picture`는205쪽, 그림의
단일 소유·원본BinData, 캡션/주석/후속 본문의 같은 쪽 소유와 전체 내용 순서·누락·중복,
그림→캡션→주석→본문의 겹침 금지를 검사한다. 같은 쪽의 양쪽 raw98% 이상과 직접 판독을
확인한 뒤 독립 Print의 캡션 상단699.79083pt(933.05444CSSpx), 그림 하단688.64697pt
(918.19596CSSpx)를±2CSSpx로 보호한다. 이 좌표로 검사 대상을 선택하지 않는다.
테스트 대상은 원본의 구역·문단·그림 소속으로 고른다.

실제 before는6건 중5PASS/1FAIL(0.833초), after는6PASS(0.839초)다.
before의 캡션 상단908.97333이 그림 하단919.2보다 위에 있어 겹침 관계에서 실패한다.
원본 쪽수·기존 표/그림 경계5건은 before/after 모두 통과한다.
정상 대조군 #6596·#6704·#6812의23건도 통과했다(0.302초).
그림 옆의 가용 lane·음수 오프셋·수평 프레임 밖 그림·이어받기 앵커의 기존 계약을 함께 확인했다.

Native binary SHA-256은 `58c9ffbe983777cb61325b2f6a642d689ca30692e1cfc15917a65346c3d722f6`,
WASM SHA-256은 `025ed6093b29ab4f7bbb0ab3d571b9354e4dcc57ffd8211785ed42f5cf657142`다.
수정된HeightCursor의 SHA-256은 `0d2c8a2132335a04b46bf684f108dd09904196ebe6903b92f09078aadc3fbfde`다.
부모58cc38bb92에 당시 전체 후보를 더한 frozen source inventory와 빌드 로그를 보존한다.
WASM은 Mac locked wrapper web/no-opt 로컬 대체 빌드이며 root pkg/Studio public 해시가 같다.
Docker 최적화 통과를 주장하지 않는다.

세부 TSV·producer·로그는 ignored
`output/pr-review/7445-completed-20261009/regression-policy-audit-20261015/rebase-552514f4-20261016/biotoxin63-c81/`
아래에 보존한다. 첫 회귀 실행은 포맷 이후 suite 배정 불일치로0건/exit4였으며,
`formal-v1-invalid.txt`로 분리했다. 이 실행을 의도한 결함 재현이나 통과로 집계하지 않는다.

새 전체 nextest·모든 변경 대상의raw 미달 보정·실제 Studio 소비 검증은 아직 완료 전이다.
이번 문서205쪽의 결과만으로 PR 전체를 준비 완료로 판정하지 않는다.

| 출력 | review | overlay |
| --- | --- | --- |
| 수정 전 Native |[review](before-native-review.png)|[overlay](before-native-overlay.png)|
| 수정 후 Native |[review](after-native-review.png)|[overlay](after-native-overlay.png)|
| 수정 전 WASM |[review](before-wasm-review.png)|[overlay](before-wasm-overlay.png)|
| 수정 후 WASM |[review](after-wasm-review.png)|[overlay](after-wasm-overlay.png)|
