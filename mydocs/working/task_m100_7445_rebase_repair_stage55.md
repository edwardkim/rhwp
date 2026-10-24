# Stage 55: 개인정보 닫힌 블록 컷과 완전한 이어받기 정렬

Issue: #7445. 새 HWP 보정 대상을 추가하지 않고 기존 개인정보 원본의 41·42쪽 소비 경계를 보정했다. 편람 242·277쪽은 완료 범위의 보존 대상이다. 고정 base는 `f7abae4a8e70c062c378db06ad64d2d2eacb88cd`이며, 실행 소스는 parent `d4f0a28eceed68d9121d6940ef0946006e7dbdc8`와 보존한 Runtime patch로 고정했다.

행 오프셋의 예산과 같은 물리 컷 높이 사이 약 7.8e-14px 계산 오차가 한 줄 이월을 만들었다. 픽셀 여유를 추가하지 않고 누적 산술의 오차만 구분했다. 블록 컷의 이어받기 정렬도 실제 줄 선택과 같은 `cell_cut_window`/`is_split_start_row`를 소비하게 했다. 문서 ID·특정 쪽·화면 점수에 따른 Runtime 분기는 없다.

Native 41/42쪽은 99.99530/99.82579%, fresh WASM은 99.99527/99.82542%다. 양쪽 전 66쪽 최저 96.57165/96.56850%, 미달 0쪽이며 65·66쪽은 100%를 유지했다. 다른 64쪽 PNG는 각각 직전 결과와 byte 동일하고 Native 본문 소유별 연결 텍스트 1,826개도 동일하다. 전후 review/overlay에서 라벨 소속·Center 정렬·표 하단을 직접 확인했다. fixed base Native 대비 전 66쪽에 0.1%p 이상 하락은 없으며, fixed base fresh WASM 비교는 아직 미완료다.

같은 신규 검사 본문은 수정 전 P51의 Rust SDK test harness에서 1 FAIL / 0 PASS, 수정 후 정식 suite에서 기존 검사와 합계 2 PASS / 0 FAIL, 0.590초다. 41·42쪽 양쪽 raw 98% 이상·독립 PDF·직접 판독 후 라벨 원점 ±2px 세 개를 추가했다. 0건 수집 실행과 SDK 컴파일 오류는 최종 증거에서 제외하고, 포맷 후 manifest 재생성으로 실제 2건 수집을 확인했다. fmt·manifest·unit-tier 검사는 통과했다. 테스트 전용 import 순서도 정리했으며 정책·허용 기준선을 바꾸지 않았다.

[전후 TSV·review/overlay·원인·검사·빌드 신원과 잔여](../pr/assets/issue7445/completed-20261009/stage55-pii-closed-cut-20261022/verification.md). 로그·중간 JSON·바이너리는 ignored output에 보존했다. 전체 nextest·필수 범위 gate·기존 제출 문서의 최종 전쪽 비교·실제 소비 경로가 남아 있어 PR 준비 전체 완료가 아니다.
