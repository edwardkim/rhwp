//! #6916: Gym 안내는 제품 소유의 오프라인 리소스이며 URI 계약은 유지한다.
#![cfg(not(target_arch = "wasm32"))]

use serde_json::{json, Value};
use std::fs::{self, File};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

struct Server(Child);

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn gym_exchange() -> Vec<Value> {
    let binary =
        std::env::var_os("CARGO_BIN_EXE_rhwp").unwrap_or_else(|| env!("CARGO_BIN_EXE_rhwp").into());
    let binary = fs::canonicalize(binary).expect("rhwp binary");
    let dir = tempfile::tempdir().expect("isolated working directory");
    let input = dir.path().join("requests.jsonl");
    let output = dir.path().join("responses.jsonl");
    let stderr = dir.path().join("stderr.txt");
    let messages = [
        json!({"jsonrpc":"2.0", "id":1, "method":"initialize", "params":{
            "protocolVersion":"2025-06-18", "capabilities":{},
            "clientInfo":{"name":"gym-optional-contract", "version":"1"}
        }}),
        json!({"jsonrpc":"2.0", "method":"notifications/initialized"}),
        json!({"jsonrpc":"2.0", "id":2, "method":"resources/list", "params":{}}),
        json!({"jsonrpc":"2.0", "id":3, "method":"resources/read",
            "params":{"uri":"rhwp://docs/gym"}}),
        json!({"jsonrpc":"2.0", "id":4, "method":"resources/read",
            "params":{"uri":"rhwp://docs/no-such-6916-resource"}}),
    ];
    let input_text = messages
        .iter()
        .map(|message| format!("{message}\n"))
        .collect::<String>();
    fs::write(&input, input_text).expect("write requests");
    // File-backed streams cannot deadlock on a full pipe. EOF closes the session;
    // a finite deadline and Drop also reap the child on timeout/assertion failure.
    let mut server = Server(
        Command::new(binary)
            .arg("mcp-serve")
            .current_dir(dir.path())
            .stdin(Stdio::from(File::open(input).expect("open requests")))
            .stdout(Stdio::from(
                File::create(&output).expect("create responses"),
            ))
            .stderr(Stdio::from(File::create(&stderr).expect("create stderr")))
            .spawn()
            .expect("start MCP server"),
    );
    let deadline = Instant::now() + Duration::from_secs(20);
    let status = loop {
        if let Some(status) = server.0.try_wait().expect("poll MCP server") {
            break status;
        }
        assert!(
            Instant::now() < deadline,
            "MCP server exceeded 20s deadline"
        );
        std::thread::sleep(Duration::from_millis(10));
    };
    assert!(
        status.success(),
        "MCP server failed: {}",
        fs::read_to_string(stderr).unwrap_or_default()
    );
    fs::read_to_string(output)
        .expect("read responses")
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect("JSON-RPC response"))
        .collect()
}

#[test]
fn gym_resource_is_optional_guidance_with_stable_wire_contract() {
    let responses = gym_exchange();
    let response = |id: u64| {
        responses
            .iter()
            .find(|value| value["id"].as_u64() == Some(id))
            .unwrap_or_else(|| panic!("missing response {id}: {responses:?}"))
    };
    assert!(response(1)["result"]["capabilities"]["resources"].is_object());
    let resources = response(2)["result"]["resources"]
        .as_array()
        .expect("resource list");
    let gym = resources
        .iter()
        .filter(|resource| resource["uri"] == "rhwp://docs/gym")
        .collect::<Vec<_>>();
    assert_eq!(gym.len(), 1, "Gym URI must remain unique and listed");
    let listed = gym[0];
    assert_eq!(listed["name"], "gym-readme");
    assert_eq!(listed["title"], "rhwp 에이전트 운동장 (gym)");
    assert_eq!(listed["mimeType"], "text/markdown");
    assert!(response(3).get("error").is_none());
    let contents = response(3)["result"]["contents"]
        .as_array()
        .expect("resource contents");
    assert_eq!(contents.len(), 1);
    assert_eq!(contents[0]["uri"], "rhwp://docs/gym");
    assert_eq!(contents[0]["mimeType"], listed["mimeType"]);
    let text = contents[0]["text"].as_str().expect("Markdown text");
    assert!(
        text.contains("제품 실행에는 Gym 설치가 필요하지 않습니다."),
        "Gym resource must serve product-owned optional guidance, not the full Gym README"
    );
    assert!(text.contains("Gym → rhwp CLI/API"));
    assert!(text.contains("https://github.com/edwardkim/rhwp/blob/devel/gym/README.md"));
    assert!(text.contains(
        "https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/gym_benchmark_operations.md"
    ));
    assert_eq!(listed["size"].as_u64(), Some(text.len() as u64));
    assert_eq!(response(4)["error"]["code"], -32002);
}
