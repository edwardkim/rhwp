# Native Metal 프로세스 admission

`rhwp_metal_presenter_create`는 thread별 2개 제한과 별도로 프로세스 전체에서 최대 4개의
생성 중·활성·격리 presenter를 허용합니다. 여러 thread를 만들거나 `abandon`으로 TLS handle을
제거해도 격리된 GPU 자원의 admission을 다시 얻을 수 없습니다. 이 제한은 Native FFI owner에
적용되며 직접 Rust `NativeMetalPresenter::new` 호출이나 전체 driver VRAM의 상한은 아닙니다.

## 생성과 반환

프로세스 slot을 먼저 예약하고 `NativeMetalPresenter::new`를 호출합니다. 이 constructor는
config·pointer·generation 검증, Metal backend/Ganesh context 생성과 resource cache 설정만
수행하며 render·flush·submit은 호출하지 않습니다. 생성 오류로 owner가 노출되기 전의 예약은
반환합니다. 성공 즉시 token을 live로 바꾸며 info 생성·TLS 삽입보다 먼저 처리합니다.

정상 `release`는 Native presenter의 GPU 완료 상태 검사와 resource release가 성공한 뒤에만
slot을 반환합니다. `abandon`, owner의 Drop, TLS 종료, 생성 이후 panic은 완료 증거가 아니므로
한 번만 quarantine하고 프로세스 수명 동안 slot을 유지합니다. render가 완료 불명확 상태를
보고하면 handle을 계속 보관하는 동안에도 quarantine 수에 반영합니다. 이미 격리한 token은
늦은 release·중복 abandon·Drop으로 환급하거나 다시 live로 바꿀 수 없습니다.

used와 quarantined 값은 하나의 atomic 상태에서 함께 조회합니다. 별도 thread의 생성·종료와
겹쳐도 관측값이 서로 다른 시점의 계수를 섞지 않습니다. 예약과 실제 surface 수명은 별개이며,
이 계수나 producer 완료가 외부 texture의 consumer 완료를 증명하지 않습니다.

## 관측과 호환성

기존 C ABI 함수·인자·ABI version 1은 유지합니다. capabilities에 `maxPresentersPerProcess: 4`와
`contextCredits: {inUse, quarantined, limit}`를 추가합니다. presenter `info`와 create 응답의
`info`에도 조회 시점의 같은 계수를 추가합니다. `inUse`는 생성 예약·live·quarantine을 합친 수며
`quarantined`는 그 부분 집합입니다. Apple/Metal 피처가 없는 build도 capabilities를 export하며
사용량은 0, 상한은 4입니다. 비활성 create/render/request는 기존대로 pointer를 읽지 않습니다.

P124 D3D의 context credit 4개 정책을 비교 근거로 삼았지만 D3D 구현은 변경하지 않았습니다.
Native Metal의 presenter별 scratch·tile 예산과 Flutter producer/frame 예산도 유지합니다.

## 검증

`tests/cases/issue_8701_metal_context_admission.rs`는 실제 private policy를 포함하는 기존 suite
검사입니다. `rustc --test`로도 실행할 수 있으며 생성 전 rollback, 정상 release, 중복 quarantine,
생성 이후 panic, TLS 종료, 여러 thread의 4개 상한과 반복 abandon 뒤 admission 거절을 다룹니다.
`bindings/Flutter/macos/tests/native_metal_disabled.py`는 새로 빌드한 비활성 Native ABI에서 추가
metadata와 잘못된 pointer/span의 비역참조를 검사합니다.

이 portable 정책·비활성 ABI 통과는 실제 Apple SDK/GPU 실행이 아닙니다. 실제 macOS에서는 정상
생성·render·release 반복과 producer 오류 주입·abandon·thread 종료 뒤의 admission을 별도로
확인해야 합니다. 한컴 전체 문서·물리 GPU 장시간 검증은 사용자 후속 범위입니다.
