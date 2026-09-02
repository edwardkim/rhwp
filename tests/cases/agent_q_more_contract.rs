//! rhwp-q-more 계약. src 에 #[cfg(test)] 없음.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const SAMPLE: &str = "samples/form-01.hwp";

// 생성 shard를 제거하기 전의 `volume-probe` 결과다. 서로 다른 top-level Control과
// 표 셀 집계를 포함한 세 HWP에서 0..49 slot의 wrapping 산술을 고정한다.
const VOLUME_PROBE_GOLDENS: [(&str, [u64; 50]); 3] = [
    (
        "samples/form-01.hwp",
        [
            654289151198,
            658513173518,
            658515976156,
            654297558640,
            658521581196,
            654303163680,
            658527186236,
            654308768720,
            658532791276,
            654314373878,
            658538396198,
            658541198836,
            654322781320,
            658546803876,
            654328386360,
            658552408916,
            654333991400,
            658558013956,
            654339596558,
            658563618878,
            658566421516,
            654348004000,
            658572026556,
            654353609040,
            658577631596,
            654359214080,
            658583236636,
            654364819238,
            658588841558,
            658591644196,
            654373226680,
            658597249236,
            654378831720,
            658602854276,
            654384436760,
            658608459316,
            654390041918,
            658614064238,
            658616866876,
            654398449360,
            658622471916,
            654404054400,
            658628076956,
            654409659440,
            658633681996,
            654415264598,
            658639286918,
            658642089556,
            654423672040,
            658647694596,
        ],
    ),
    (
        "samples/pic-crop-01.hwp",
        [
            656983576820,
            652774946420,
            656989181860,
            656991984380,
            652783353980,
            656997589420,
            652788959020,
            657003194460,
            652794564060,
            657008799500,
            652800169100,
            657014404540,
            657017207060,
            652808576660,
            657022812100,
            652814181700,
            657028417140,
            652819786740,
            657034022180,
            652825391780,
            657039627220,
            657042429740,
            652833799340,
            657048034780,
            652839404380,
            657053639820,
            652845009420,
            657059244860,
            652850614460,
            657064849900,
            657067652420,
            652859022020,
            657073257460,
            652864627060,
            657078862500,
            652870232100,
            657084467540,
            652875837140,
            657090072580,
            657092875100,
            652884244700,
            657098480140,
            652889849740,
            657104085180,
            652895454780,
            657109690220,
            652901059820,
            657115295260,
            657118097780,
            652909467380,
        ],
    ),
    (
        "samples/equation-lim.hwp",
        [
            407964, 3208130, 6013004, 8815524, 11615690, 14420564, 17220730, 20025604, 22825770,
            25630644, 28430810, 31235684, 34038204, 36838370, 39643244, 42443410, 45248284,
            48048450, 50853324, 53653490, 56458364, 59260884, 62061050, 64865924, 67666090,
            70470964, 73271130, 76076004, 78876170, 81681044, 84483564, 87283730, 90088604,
            92888770, 95693644, 98493810, 101298684, 104098850, 106903724, 109706244, 112506410,
            115311284, 118111450, 120916324, 123716490, 126521364, 129321530, 132126404, 134928924,
            137729090,
        ],
    ),
];

fn bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp-q-more")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp-q-more").to_string())
}

fn sample() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

fn run(args: &[&str]) -> Output {
    Command::new(bin())
        .args(args)
        .output()
        .expect("rhwp-q-more")
}

#[test]
fn help_lists_pack_commands() {
    let out = run(&["--help"]);
    assert_eq!(out.status.code(), Some(0));
    let text = String::from_utf8_lossy(&out.stdout);
    for name in [
        "para-empty",
        "para-has-ctrl",
        "section-para-lens",
        "body-text-len",
        "ctrl-per-para",
        "table-border-fill",
        "table-spacing",
        "table-attr",
        "picture-border-width",
        "picture-opacity",
        "picture-href-set",
        "equation-baseline",
    ] {
        assert!(text.contains(name), "{name} missing from help:\n{text}");
    }
}

#[test]
fn unknown_command_is_usage() {
    let out = run(&["not-a-command"]);
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn para_has_ctrl_json() {
    let p = sample();
    let src = p.to_str().unwrap();
    let args = ["para-has-ctrl", "--json", src];
    let out = run(&args);
    assert_eq!(out.status.code(), Some(0), "{:?}", out);
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["tool"], "rhwp-q-more");
    assert_eq!(v["command"], "para-has-ctrl");
    assert!(v["count"].is_number());
}

#[test]
fn volume_probe_preserves_all_slot_goldens() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    for (relative_path, expected_values) in VOLUME_PROBE_GOLDENS {
        let path = root.join(relative_path);
        let src = path.to_str().unwrap();
        for (slot, expected_acc) in expected_values.into_iter().enumerate() {
            let slot_text = slot.to_string();
            let args = ["volume-probe", "--json", "--slot", &slot_text, src];
            let out = run(&args);
            assert_eq!(
                out.status.code(),
                Some(0),
                "{relative_path} slot {slot}: {out:?}"
            );
            let value: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
            assert_eq!(value["command"], "volume-probe");
            assert_eq!(value["slot"].as_u64(), Some(slot as u64));
            assert_eq!(
                value["acc"].as_u64(),
                Some(expected_acc),
                "{relative_path} slot {slot}"
            );
        }
    }
}

#[test]
fn volume_probe_rejects_slot_outside_legacy_range() {
    let p = sample();
    let src = p.to_str().unwrap();
    let out = run(&["volume-probe", "--slot", "50", src]);
    assert_eq!(out.status.code(), Some(2), "{out:?}");
}
