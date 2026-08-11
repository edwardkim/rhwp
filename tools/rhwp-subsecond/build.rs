//! `dx --hot-patch` 가 찾는 rlib 별칭을 만든다.
//!
//! 이 스크립트가 조용히 실패하면 그 결과는 "핫패치가 아무 말 없이 안 먹는다"로만 나타난다 —
//! `dx serve` 도, 브라우저 소켓도, 데브서버의 `HotReload` 도 전부 정상으로 보인다. 그래서
//! 실패하는 가지는 전부 `cargo:warning` 으로 무엇이 막혔는지와 다음에 볼 곳을 남긴다.
//! 플랫폼 제약과 층별 신호는 `mydocs/manual/dev_environment_guide.md` 의
//! "Subsecond 핫패치" 절에 있다.

#[cfg(unix)]
use std::path::PathBuf;

/// `dx` 가 찾는 이름.
const ALIAS_NAME: &str = "librhwp-dioxus.rlib";
/// 별칭이 가리키는 실제 rlib. 이 스크립트가 끝난 뒤 rustc 가 만들므로 복사로 대체할 수 없다.
#[cfg(unix)]
const TARGET_NAME: &str = "librhwp.rlib";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    if std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() != Ok("wasm32") {
        return;
    }

    #[cfg(unix)]
    create_dioxus_rlib_alias();

    #[cfg(not(unix))]
    println!(
        "cargo:warning=rhwp-subsecond: 이 호스트는 unix 가 아니라 `{ALIAS_NAME}` 별칭을 만들지 않는다. \
         별칭 대상은 이 빌드 스크립트가 끝난 뒤에 생기므로 복사로 대체할 수 없고 심링크가 필요하다. \
         핫패치는 동작하지 않으며, dx serve·소켓·HotReload 는 모두 정상으로 보이고 화면만 바뀌지 않는다. \
         WSL 안에서 빌드하거나 일반 wasm-pack 경로를 쓴다 — mydocs/manual/dev_environment_guide.md 의 'Subsecond 핫패치' 절."
    );
}

#[cfg(unix)]
fn create_dioxus_rlib_alias() {
    use std::os::unix::fs::symlink;

    let Some(out_dir) = std::env::var_os("OUT_DIR").map(PathBuf::from) else {
        println!(
            "cargo:warning=rhwp-subsecond: OUT_DIR 이 없어 `{ALIAS_NAME}` 별칭을 만들지 못했다. \
             cargo 없이 build.rs 를 직접 실행했다면 무시해도 되고, 아니면 핫패치는 동작하지 않는다."
        );
        return;
    };

    // OUT_DIR 은 `<profile>/build/<pkg>-<hash>/out` 이므로 프로파일 디렉터리는 위로 3단계다.
    let Some(profile_dir) = out_dir.ancestors().nth(3) else {
        println!(
            "cargo:warning=rhwp-subsecond: OUT_DIR `{}` 에서 프로파일 디렉터리를 찾지 못했다(위로 3단계 필요). \
             cargo 의 target 레이아웃이 바뀌었을 수 있다. `{ALIAS_NAME}` 별칭 없이 진행하므로 핫패치는 동작하지 않는다.",
            out_dir.display()
        );
        return;
    };

    let deps_dir = profile_dir.join("deps");
    let alias = deps_dir.join(ALIAS_NAME);

    // 별칭 자체를 추적한다. build.rs 만 추적하면 `deps/` 가 정리돼 별칭이 사라져도 이 스크립트가
    // 다시 돌지 않아 별칭이 영영 복구되지 않는다. 없는 경로는 cargo 가 "변경됨"으로 보므로
    // 별칭이 사라진 다음 빌드에서 이 스크립트가 다시 돈다.
    println!("cargo:rerun-if-changed={}", alias.display());

    // `Path::exists()` 는 심링크를 따라가므로 끊긴 별칭을 "없음" 으로 읽는다. 그러면 아래
    // `symlink()` 가 EEXIST 로 실패하고, 그 실패마저 조용히 버려졌다. 링크 자체의 상태를 본다.
    //
    // 끊긴 별칭은 경고하지 않는다 — 대상 rlib 은 이 스크립트가 끝난 뒤에 생기고,
    // `cargo check` 만 돌면 아예 생기지 않으므로 "끊김" 은 정상 상태이기도 하다.
    match std::fs::symlink_metadata(&alias) {
        Ok(metadata) if metadata.is_symlink() => return,
        Ok(_) => {
            println!(
                "cargo:warning=rhwp-subsecond: `{}` 이 심링크가 아닌 실제 파일이라 손대지 않는다. \
                 핫패치가 낡은 rlib 을 물 수 있으니 이 파일을 지우고 다시 빌드한다.",
                alias.display()
            );
            return;
        }
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => {
            println!(
                "cargo:warning=rhwp-subsecond: `{}` 의 상태를 읽지 못했다: {error}. 핫패치는 동작하지 않는다.",
                alias.display()
            );
            return;
        }
        Err(_) => {}
    }

    if let Err(error) = std::fs::create_dir_all(&deps_dir) {
        println!(
            "cargo:warning=rhwp-subsecond: `{}` 를 만들지 못해 `{ALIAS_NAME}` 별칭을 건너뛴다: {error}. \
             핫패치는 동작하지 않는다.",
            deps_dir.display()
        );
        return;
    }

    if let Err(error) = symlink(TARGET_NAME, &alias) {
        println!(
            "cargo:warning=rhwp-subsecond: `{}` → `{TARGET_NAME}` 심링크를 만들지 못했다: {error}. \
             핫패치는 동작하지 않는다.",
            alias.display()
        );
    }
}
