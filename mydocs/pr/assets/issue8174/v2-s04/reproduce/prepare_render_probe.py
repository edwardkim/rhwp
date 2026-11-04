from pathlib import Path
import json,subprocess
R=Path.cwd();O=R/'output/pr-review/issue8174/v2-s04-20261025';src=(R/'tests/cases/skia_native_glyph_replay.rs').read_text();src=src.replace('"../fixtures/','"'+str(R/'tests/fixtures')+'/').replace('"../../ttfs/','"'+str(R/'ttfs')+'/')
src+='''
fn main() {
    let output=std::path::PathBuf::from(std::env::args().nth(1).unwrap()); std::fs::create_dir_all(&output).unwrap();
    let input=output.parent().unwrap().join("runtime/svg-raster");
    let cases=["gradient","clip-opacity","internal-use","restricted-script","embedded-png","embedded-jpeg","foreground"];
    for name in cases {
      let fragment=std::fs::read_to_string(input.join(format!("{name}.svg"))).unwrap();
      let fragment=fragment.as_str();
      let mut tree=svg_tree(fragment); outline_mut(&mut tree).paint_style.color=0x008000; let pixels=assert_selected(&tree); pixels.save(output.join(format!("{name}.png"))).unwrap();
      std::fs::write(output.join(format!("{name}.json")),tree.to_json()).unwrap();
      // Source-based independent SVG reference uses the same OpenType viewport;
      // it contains original artwork, never a RHWP-generated raster.
      let inner=fragment.replacen("<svg", "<g",1).replace("</svg>","</g>");
      let reference=format!("<svg xmlns=\\"http://www.w3.org/2000/svg\\" width=\\"180\\" height=\\"80\\" color=\\"#008000\\"><svg x=\\"10\\" y=\\"10\\" width=\\"30\\" height=\\"30\\" viewBox=\\"10 20 30 30\\">{inner}</svg></svg>");
      std::fs::write(output.join(format!("{name}.svg")),reference).unwrap();
      println!("SVG {name} selected");
    }
    use rhwp::document_core::{DocumentCore,hyperlink::HyperlinkTarget};
    use rhwp::renderer::pdf::{svgs_to_pdf_with_annotations,PdfExportOptions};
    use rhwp::renderer::hyperlinks::{PdfLink,PdfNamedDestination};
    let targets=["https://example.com/한글", "mailto:user@example.com", "#목표", "../문서.pdf", "C:\\\\docs\\\\문서.hwp", "rhwp-app:open/document"];
    let links=targets.iter().enumerate().map(|(i,uri)|PdfLink{uri:uri.to_string(),rect:BoundingBox::new(5.0+i as f64*20.0,5.0,15.0,20.0)}).collect();
    let destinations=vec![vec![PdfNamedDestination{name:"목표".into(),x:40.0,y:60.0}]];
    let pdf=svgs_to_pdf_with_annotations(&["<svg xmlns=\\"http://www.w3.org/2000/svg\\" width=\\"160\\" height=\\"80\\"><rect x=\\"5\\" y=\\"5\\" width=\\"140\\" height=\\"20\\" fill=\\"blue\\"/></svg>".into()],&[links],&destinations,&PdfExportOptions::default()).unwrap();
    std::fs::write(output.join("link-actions.pdf"),pdf).unwrap();
    for uri in targets {
      let mut core=DocumentCore::new_empty();core.create_blank_document_native().unwrap();core.insert_text_native(0,0,0,"Link target").unwrap();
      core.insert_hyperlink_native(&HyperlinkTarget::body(0,0),0,4,uri).unwrap();
      assert_eq!(core.hyperlinks_native(&HyperlinkTarget::body(0,0)).unwrap()[0].uri,uri);
    }
    let mut core=DocumentCore::new_empty(); core.create_blank_document_native().unwrap();core.insert_text_native(0,0,0,"Link target").unwrap();
    core.split_paragraph_native(0,0,5,None).unwrap();
    println!("bookmark inserted {}",core.add_bookmark_native(0,1,1,"목표").unwrap());
    core.insert_hyperlink_native(&HyperlinkTarget::body(0,0),0,4,"#목표").unwrap();
    let tree=core.build_page_layer_tree_with_profile(0,rhwp::paint::RenderProfile::Print).unwrap();
    let anchors=core.bookmarks_in_layer_tree(&tree).unwrap();assert_eq!(anchors.len(),1); println!("Core bookmark {:?}",anchors);
    std::fs::write(output.join("core-bookmark.pdf"),core.render_document_pdf_native().unwrap()).unwrap();
    std::fs::write(output.join("core-bookmark-direct.pdf"),core.render_document_pdf_direct_native().unwrap()).unwrap();
    println!("links classify/create/PDF actions complete");
}
'''
p=O/'render_probe.rs';p.write_text(src)
def lib(name,feature=None):
 values=[]
 if name=='rhwp':return R/'target/pr-review/release-test/deps/librhwp.rlib'
 for f in (R/'target/pr-review/release-test/.fingerprint').glob(name+'-*/lib-*.json'):
  d=json.loads(f.read_text())
  if feature and feature not in d['features']:continue
  crate=f.stem[4:]; candidate=f.parents[1].parent/'deps'/('lib'+crate+'-'+f.parent.name.rsplit('-',1)[1]+'.rlib')
  if candidate.exists():values.append(candidate)
 return max(values,key=lambda x:x.stat().st_mtime)
libs={name:lib(name,'native-skia' if name=='rhwp' else None) for name in ['rhwp','image','ttf-parser']}
args=['rustc','--edition=2021','--cfg','feature="native-skia"','-A','dead_code',str(p),'-L','dependency='+str(R/'target/pr-review/release-test/deps')]
for p in (R/'target/pr-review/release-test/build').glob('skia-bindings-*/out/skia'):
 if (p/'libskia.a').exists():args+=['-L','native='+str(p)]
for name,lib in libs.items():args+=['--extern',f'{name.replace(chr(45),chr(95))}={lib}']
args+=['-o',str(O/'render-probe')]
print({k:v.name for k,v in libs.items()});subprocess.run(args,check=True)
subprocess.run([str(O/'render-probe'),str(O/'render-probes')],check=True)
