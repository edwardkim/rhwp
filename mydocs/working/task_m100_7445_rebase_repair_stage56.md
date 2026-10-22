# Stage 56: 생산 장치 PNG와 최종 래스터의 해상도 구분

Issue #7445. 기존 sample16 변형만 보정하며 새 HWP 대상을 추가하지 않는다. 편람 242·277쪽은 완료 보존 대상이다. 새 base는 `605ccfd92f2f12f18e14182548d729f9929fda00`, 보정 전 후보는 `e236f80262eb6521dced7f705b62b66d1486fc74`다. 리베이스의 354개 branch patch는 range-diff에서 모두 동일했고 Kitty Native 계약 8건은 통과했다. 새 base의 실제 Kitty 소비 대조는 별도로 남아 있다.

## 원인과 독립 근거

`paint/metafile_surface.rs`는 2024 Print WMF를 600DPI 생산 장치의 PNG로 만든다. `ResolvedImageKind::OutputDevice`는 출력 조건에 따라 바이트가 달라진다는 출처이며 최종 Native 화면 픽셀과 1:1이라는 뜻이 아니다. C417은 이 태그를 무조건 선형 재생 조건으로 사용했다. 최종 96DPI로 줄일 때 고주파 무늬를 건너뛰어 가는 문자와 회색 면이 거칠어졌다.

같은 PNG·bbox·CTM·글꼴의 태그만 바꾸는 공개 SDK 진단에서 원래 장치 경로의 픽셀은 실제 CLI와 동일했다. 기존 bitmap 축소 경로를 적용한 18/35쪽은 raw 86.59436/91.35103%에서 98.55817/98.09853%로 개선됐다. PNG를 고치거나 원본 도해·배치를 바꾼 결과가 아니다. 독립 기준은 #8064가 사용한 원본과 한컴 Print PDF다. 현재 PDFium 152/153으로 같은 쪽을 대조해도 기존 후보의 raw 값은 동일해 rasterizer 버전으로 설명하지 않았다. 이 두 쪽 진단은 전쪽 완료 증거가 아니다.

앞선 고정 base 출력에는 별도의 글꼴 공급 오류도 있었다. 로컬 전체 폴더를 순서대로 공급하면 Native catalog의 64MiB 상한에 도달해 휴먼명조가 직접 공급에서 빠졌다. 실제 로컬 `HMKMM.TTF`·`HBATANG.TTF`를 우선 공급하는 동일 조건에서 base의 18/35쪽은 99.09218/98.54763%로 복원됐고 Native 관측도 `system`에서 `custom`으로 바뀌었다. 상한을 늘리거나 guard를 우회하지 않는다. 깨진 이전 base 점수를 회귀 자격으로 쓰지 않으며 전체 페이지의 같은 공급 대조가 필요하다.

## 공통 소비와 검사

첫 후보(v1)는 PNG payload의 생산 장치 출처를 계속 보존하면서 Native의 축소 여부를 기존 최종 paint 사각형·실제 canvas matrix·source 픽셀 수가 결정하게 한다. 1:1·확대·명시 nearest·crop·SVG·binary 처리의 기존 계약을 유지한다. 출처 태그만으로 축소 필터를 차단하던 지역 분기를 제거했다. 이 첫 후보의 구현 변경은 WASM에서 제외된 Native Skia 파일에 한정된다. 이후 공통 투명 coverage 후보(v2)는 아래와 같이 WASM 소비 경로에도 영향을 주므로 새 WASM을 빌드한다. 두 후보 모두 공통 WMF producer·글꼴·원본 PNG 바이트를 바꾸지 않는다.

기존 합성 paint 검사의 “축소 후에도 검은 점이 남아야 한다”는 가정은 source의 흰 픽셀 coverage를 건너뛴 alias를 정상으로 고정했다. 같은 PNG·최종 장치에서 출처 태그만 다른 두 경로를 비교하고, 축소 구간의 coverage·1:1 원 픽셀·확대·cache 전환을 검증하도록 교정한다. 문서의 절대 px·새 golden·허용치 변경은 없다. 정확한 새 검사 prefix의 수정 전 Rust SDK 실행은 결과를 확인해 아래에 연결하고, 최종 manifest 검사도 별도로 실행한다.

작업 증거는 ignored `rebase-8155-20261022/rebase-605ccfd9/device-replay-control/`, `sample16-local-font-control-v2/`, `sample16-consumer-resolution-v1/`에 보존한다. 빌드·정식 회귀·Native/fresh WASM 전쪽·새 base와의 비교·직접 판독·필수 gate·실제 소비는 아직 완료하지 않았다. PR 준비 완료가 아니다.

사용자가 전달한 별도 PR의 사양 정정도 현재 분할 검토의 근거로 반영한다. TABLE 원시 값 2/HWPX `CELL`은 셀 내부 글줄도 나누는 정책이며 내부 `RowBreak`를 행 경계 전용으로 해석하지 않는다. 사양 설명본·정오표·모델 주석은 진행 중인 다른 PR의 소유이므로 이 단계에서 수정하지 않는다. 이 의미만으로 특정 높이 계산식이 입증됐다고 쓰지 않는다.

## Native 국소 복구와 투명 coverage의 공통 원인

Native 전용 분기 제거 후보의 기존 장치 계약 4건은 정식 suite에서 4 PASS, 0.281초다. Native 세 변형 전 192쪽도 미달 없이 64/64를 유지했다. 최저는 원본 HWP3 97.01457%, HWP5 97.09573%, HWPX 95.30909%다. 직전 후보와 원본 HWP3의 64 PNG가 모두 같고, HWP5/HWPX는 각각 18·35쪽만 달라졌다. 192개 render tree도 모두 byte 동일하다. 이는 새 base의 전쪽 보존이나 전체 PR 검증 완료가 아니다.

동일 소스의 기존 fresh WASM을 재사용한 집중 실행은 HWP5/HWPX 18쪽 86.59527/83.88164%로 미달했다. 실제 Canvas 호출을 읽기 전용으로 관찰한 PNG는 기존 PNG와 pixel 동일하며, 3918×2574 source를 608.48×411.8933 사각형에 `imageSmoothingQuality=high`, smoothing=true로 그렸다. 품질 hint만으로 필요한 저역 통과가 실행됐다고 판단하지 않는다.

공통 `bitmap_raster::prepare_bitmap`은 투명 픽셀이 있으면 반환을 생략해 Native와 Canvas가 다른 암묵 필터를 썼다. 투명 PNG를 새 정수 box에 늘리는 단순 구간 평균 진단은 18쪽 97.32833%, 연속 장치 사각형의 실제 겹침과 premultiplied alpha를 적분한 진단은 98.11824%였다. 후자는 배경을 흰색으로 고정하거나 숨은 RGB를 섞지 않는다. Native와 WebCanvas/CanvasKit이 같은 준비 PNG와 장치 box를 소비하도록 이 공통 경로를 보정한다. 불투명 Print의 검증된 정수 구간 평균, crop·nearest·회전, 다른 프로필의 기존 투명 합성은 유지한다. 표본의 소수 원점·크기도 cache identity에 포함한다.

새 합성 계약은 8×8 source에서 빨강의 coverage 3/4×3/4 및 1/4×3/4를 흰 종이에 합성한 독립 기대값, 투명 파랑의 비기여, 같은 정수 크기의 소수 원점 변경 및 cache 복귀를 확인한다. 공통 투명 처리가 없던 Native 후보의 동일 검사 prefix는 실제 1 FAIL/0 PASS, 0.05초였다. 초기 SDK의 누락 serde 인자·서로 다른 serde dependency 선택은 컴파일 실패로 보존했고 실제 FAIL 증거와 구분한다. 새 공통 후보의 빌드·정식 검사·fresh WASM과 전쪽 검증은 이어서 실행한다. 이전 Native 국소 복구와 재사용 WASM 수치를 새 공통 후보 통과로 승격하지 않는다.

종료된 이 작업의 전용 incremental 캐시 정리는 논리 13.27GiB였지만 실제 여유 증가가 약17MiB였다. 이후 과거 생성 `font_resources`의 같은 bytes를 SHA-256으로 확인하고 4,718개 중복을 hardlink로 정리해 실제 30.51GiB를 확보했다. PNG·TSV·로그·원본·공유 target은 보존했으며 경로 삭제·콘텐츠 변경은 0이다. 원 글꼴 바이너리를 변경하거나 저장소에 추가하지 않았다. 상세 plan/result는 ignored `owned-incremental-cleanup/`에 있다.

## 공통 후보 v2의 실제 결과

fmt·suite manifest·unit tiers·Native build와 정식 장치 계약 5건이 통과했다. 새 coverage 검사도 실제 소비자에서 PASS이며, 수정 전 같은 검사 prefix의 FAIL과 구분해 보존했다. Mac locked web/no-opt WASM은 234.376초에 종료 코드 0으로 완료했고 root pkg·Studio public·frozen package의 JS/WASM 해시는 각각 일치했다. 실제 Studio 초기화 응답의 검증은 아직 남아 있다.

기존 기준 PDF의 Native와 새 WASM 전쪽은 각 3조건·192쪽을 산출했다. Native 최저는 HWP3 97.01457%, HWP5 97.09573%, HWPX 95.13828%이고 WASM은 각각 97.00639%, 97.03480%, 95.14423%다. 모두 64/64쪽·90% 미만 0이다. Native v1과 비교하면 192개 render tree가 모두 동일하며 원본 HWP3 PNG 64개도 동일하다. HWP5/HWPX는 18·35쪽만 바뀌었다. HWPX 18쪽의 v1 대비 0.17081%p 감소는 별도로 기록했으며, v1을 accepted upstream 기준으로 취급하지 않는다. 새 고정 base와 accepted #8064 기준의 보존 판정은 별도로 계속한다.

Native/fresh WASM HWP5 18쪽의 review와 overlay를 직접 열어 도해의 거친 표본이 복구된 것을 확인했다. 제목의 굵은 한글 간격과 도형의 세부 차이는 남는다. 전쪽 수치만으로 완료로 판정하지 않는다. 제출용 중간 TSV·대표 PNG는 `mydocs/pr/assets/issue7445/completed-20261009/rebase-605ccfd9-20261022/consumer-resolution/`에 보존한다. 로그·JSON·진단 스크립트는 ignored `sample16-consumer-resolution-v2/`에만 있다.

사용자가 제공한 ubuntu-ted2의 새 PDF 3개도 SHA를 대조해 별도 `pdf/issue7445-sample16-ubuntu-ted2-20261022/`에 복사했다. 기존 동명 PDF는 덮어쓰지 않았다. 2022 HWP·2024 HWP·HWPX 각각의 원본 SHA는 양 호스트가 같으며, 2010 저장 HWP5를 2024 저장 HWP5와 같은 입력으로 바꾸지 않는다. 새 PDF 기준 검증과 현재 후보 전체 검증 완료를 혼동하지 않는다.
