# Rust 진단 도구

여기는 여러 이슈와 반복 검증에서 재사용하는 개발자용 Rust 실행 도구를 둔다.
일반 사용자를 위한 API 예제가 아니며 제품 CLI 표면에도 포함하지 않는다.

실행 가능한 파일은 루트 `Cargo.toml`의 명시적 `[[example]]` target으로만 등록한다.
따라서 기존 명령인 `cargo run --example <이름> -- ...`를 유지하면서도, 이 디렉터리에
파일을 추가했다는 이유만으로 CI의 `--all-targets` 대상이 되지는 않는다.

특정 이슈에서만 의미가 있는 진단·재현 source는 이곳에 추가하지 않고
`mydocs/tech/investigations/issue-####/probes/`에 보존한다.

이슈에서 출발했더라도 현재 반복 계측의 실행 구성 요소인 도구는 여기에 둘 수 있다. 이 경우
해당 `mydocs/tech/investigations/issue-####/README.md`가 source를 직접 가리켜 소유 이슈와
실행 경계를 함께 관리한다.
