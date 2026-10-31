from pathlib import Path
from PIL import Image
import hashlib, json, shutil

root=Path.cwd()
out=Path(__file__).resolve().parent
probe=out/'jpeg-probe'
observed=probe/'observed-zune'
observed.mkdir(parents=True,exist_ok=True)
source=root/'vendor/rhwp-zune-jpeg'
shutil.copytree(source/'src', observed/'src',dirs_exist_ok=True)
manifest=(source/'Cargo.toml').read_text().replace('name = "rhwp-zune-jpeg"','name = "issue8174-observed-zune"').replace('name = "zune_jpeg_limited"','name = "zune_jpeg_observed"')
(observed/'Cargo.toml').write_text(manifest)
worker=observed/'src/worker.rs'
text=worker.read_text()
needle='''            let count = allocation.product(&[component.raw_coeff.len(), 2])?;
            allocation.resize('''
replacement='''            let count = allocation.product(&[component.raw_coeff.len(), 2])?;
            // Audit-only observation; the original reservation and resize follow unchanged.
            crate::audit_observer::record(count, component.upsample_dest.len(), component.upsample_dest.capacity());
            allocation.resize('''
assert text.count(needle)==1
worker.write_text(text.replace(needle,replacement))
lib=observed/'src/lib.rs'
lib.write_text(lib.read_text()+'''
// Audit-only diagnostics. Not part of the product crate or its public API.
pub mod audit_observer {
    use core::sync::atomic::{AtomicUsize, Ordering};
    static CALLS: AtomicUsize = AtomicUsize::new(0);
    static LEN_GROWTH: AtomicUsize = AtomicUsize::new(0);
    static CAP_GROWTH: AtomicUsize = AtomicUsize::new(0);
    pub fn record(required: usize, len: usize, capacity: usize) {
        CALLS.fetch_add(1, Ordering::Relaxed);
        if required > len { LEN_GROWTH.fetch_add(1, Ordering::Relaxed); }
        if required > capacity { CAP_GROWTH.fetch_add(1, Ordering::Relaxed); }
    }
    pub fn reset() { for a in [&CALLS,&LEN_GROWTH,&CAP_GROWTH] { a.store(0,Ordering::Relaxed); } }
    pub fn snapshot() -> [usize;3] { [CALLS.load(Ordering::Relaxed),LEN_GROWTH.load(Ordering::Relaxed),CAP_GROWTH.load(Ordering::Relaxed)] }
}
''')
(probe/'src').mkdir(exist_ok=True)
(probe/'Cargo.toml').write_text(f'''[package]
name = "issue8174-jpeg-audit"
version = "0.0.0"
edition = "2021"
publish = false
[workspace]
exclude = ["observed-zune"]
[lib]
crate-type = ["cdylib", "rlib"]
[dependencies]
original = {{ package = "rhwp-zune-jpeg", path = "{source}" }}
observed = {{ package = "issue8174-observed-zune", path = "{observed}" }}
zune-core = "=0.5.3"
serde_json = "1"
wasm-bindgen = "0.2"
[profile.release]
lto = false
codegen-units = 16
[profile.release-test]
inherits = "release"
opt-level = 1
codegen-units = 64
''')
images=out/'jpeg-inputs'
images.mkdir(exist_ok=True)
cases=[]
for progressive in [False,True]:
    for width,height in [(1,1),(9,9),(17,17),(33,19),(65,33),(129,65)]:
        for subsampling in [0,1,2]:
            im=Image.new('RGB',(width,height))
            im.putdata([((x*31)%256,(y*19)%256,((x+y)*23)%256) for y in range(height) for x in range(width)])
            name=f'{"progressive" if progressive else "baseline"}-{width}x{height}-s{subsampling}.jpg'
            file=images/name
            im.save(file,quality=90,subsampling=subsampling,progressive=progressive)
            cases.append({'name':name,'width':width,'height':height,'subsampling':subsampling,'progressive':progressive,'bytes':file.stat().st_size,'sha256':hashlib.sha256(file.read_bytes()).hexdigest()})
(out/'jpeg-inputs.json').write_text(json.dumps(cases,indent=2)+'\n')
changes=[]
for original in sorted((source/'src').rglob('*.rs')):
    relative=original.relative_to(source)
    copy=observed/relative
    before=hashlib.sha256(original.read_bytes()).hexdigest()
    after=hashlib.sha256(copy.read_bytes()).hexdigest()
    if before!=after:changes.append({'path':str(relative),'original_sha256':before,'observed_sha256':after})
assert [c['path'] for c in changes]==['src/lib.rs','src/worker.rs']
(out/'jpeg-observer-source.json').write_text(json.dumps({'source':str(source),'original_tree_head':'695cea7b973fc54fc5ffc7f6d414f7ecf403b882','changes':changes,'manifest_change':'audit package/lib name only','scope':'diagnostic copy; records the real H branch before unchanged resize; pixel equality checked against unmodified source'},indent=2)+'\n')
print('prepared',len(cases),'JPEG inputs and an audit-only observer copy')
