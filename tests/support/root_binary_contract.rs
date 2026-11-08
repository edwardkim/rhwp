// #7952: 자동 발견을 끄고 승인된 네 실행 파일만 명시 등록한다.
// 과거 [[bin]] 개수 핀은 자동 발견한 도구를 놓치고 기존 도구의 명시 등록을 오인한다.
// 실행 파일 수뿐 아니라 승인된 이름과 진입 경로도 함께 보호한다.
fn assert_root_binary_contract(cargo: &str) {
    let mut autobins = None;
    let mut in_package = false;
    let mut bin = None::<std::collections::BTreeMap<String, String>>;
    let mut targets = Vec::new();
    // 현행 manifest의 name/path 문자열을 읽는다. 주석의 [[bin]]은 선언으로 세지 않는다.
    for line in cargo.lines().chain(std::iter::once("[end]")) {
        let line = line.trim();
        if line.starts_with('[') {
            if let Some(mut fields) = bin.take() {
                targets.push((
                    fields.remove("name").expect("bin name"),
                    fields.remove("path").expect("bin path"),
                ));
            }
            in_package = line == "[package]";
            bin = (line == "[[bin]]").then(std::collections::BTreeMap::new);
            continue;
        }
        if line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let value = value.split('#').next().unwrap().trim();
        if in_package && key == "autobins" {
            autobins = Some(value);
        }
        if let Some(fields) = bin.as_mut() {
            if matches!(key, "name" | "path") {
                fields.insert(
                    key.to_string(),
                    serde_json::from_str::<String>(value).expect("bin 문자열"),
                );
            }
        }
    }
    assert_eq!(
        autobins,
        Some("false"),
        "승인되지 않은 bin의 자동 발견 금지"
    );
    targets.sort();
    let expected = [
        ("font-metric-gen", "src/tools/font_metric_gen.rs"),
        ("rhwp", "src/main.rs"),
        ("rhwp-agent", "src/bin/rhwp-agent/main.rs"),
        ("rhwp-q", "src/bin/rhwp-q/main.rs"),
    ]
    .map(|(name, path)| (name.to_string(), path.to_string()));
    assert_eq!(
        targets, expected,
        "승인된 CLI 집합 밖의 이름·진입점 추가/교체 금지"
    );
}
