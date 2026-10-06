# Rust fixture 생성기

저작권에 안전하고 재현 가능한 HWP/HWPX 회귀 fixture를 만드는 Rust source를 둔다.
생성기가 계속 필요한 경우에만 루트 `Cargo.toml`의 명시적 `[[example]]` target으로 등록한다.

생성된 fixture가 테스트의 정답지인 경우, 해당 테스트나 fixture 설명에서 생성기 경로를
직접 연결해 입력·생성 절차·검증 계보가 분리되지 않게 한다.
