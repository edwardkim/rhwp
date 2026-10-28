use std::{path::{Path,PathBuf},time::Instant};
use rhwp::{DocumentCore,paint::RenderProfile,native_host::NativeDocumentSession,renderer::{layer_renderer::RasterRenderOptions,raster_pixels::{RasterPixels,RasterRegion,RasterRegionPixels},skia::{SkiaLayerRenderer,SkiaRegionPlan,SkiaSurfaceRenderOptions}}};
use skia_safe::{surfaces,Color,ColorType,AlphaType,ImageInfo};
use serde_json::json;
fn crop(full:&RasterPixels,region:&RasterRegionPixels){
 for y in 0..region.pixels.height {for x in 0..region.pixels.width {
  let xx=region.origin_x+x;let yy=region.origin_y+y;let got=&region.pixels.bytes[y as usize*region.pixels.stride+x as usize*4..][..4];
  if xx>=0&&yy>=0&&xx<full.width&&yy<full.height {assert_eq!(got,&full.bytes[yy as usize*full.stride+xx as usize*4..][..4]);}else{assert_eq!(got,&[255;4]);}
 }}
}
fn save(path:&Path,p:&RasterPixels){image::save_buffer(path,&p.bytes,p.width as u32,p.height as u32,image::ColorType::Rgba8).unwrap();}
fn main(){
 let root=PathBuf::from(std::env::args().nth(1).unwrap());let cases:Vec<serde_json::Value>=serde_json::from_slice(&std::fs::read(root.join("cases.json")).unwrap()).unwrap();
 let fonts=PathBuf::from("output/pr-review/issue8109/stage2/fonts");let paths:Vec<_>=std::fs::read_dir(&fonts).unwrap().map(|x|x.unwrap().path()).filter(|p|p.file_name().unwrap().to_string_lossy().to_lowercase().contains("arial")).collect();
 let renderer=SkiaLayerRenderer::new().with_font_paths(&[fonts]);let mut records=Vec::new();
 for case in cases {
  let key=case["key"].as_str().unwrap();let bytes=std::fs::read(case["input"].as_str().unwrap()).unwrap();let core=DocumentCore::from_bytes(&bytes).unwrap();assert_eq!(core.page_count(),1);
  let tree=core.build_page_layer_tree_with_profile(0,RenderProfile::Print).unwrap();let mut scales=Vec::new();
  for scale in [0.75,1.0,1.25] {
   let options=RasterRenderOptions{scale,transparent:false,..Default::default()};let full=renderer.render_pixels_with_options(&tree,options).unwrap();
   let whole=RasterRegion{x:0.0,y:0.0,width:tree.page_width,height:tree.page_height};let plan=SkiaRegionPlan::new(tree.page_width,tree.page_height,whole,options).unwrap();assert_eq!(plan.output_bytes(),full.bytes.len());
   let (first,prepared)=renderer.prepare_page_for_region(&tree,whole,options).unwrap();assert_eq!(first.pixels.bytes,full.bytes);let prepared=prepared.unwrap();
   for x in [-5.75,20.25,tree.page_width-20.5,tree.page_width+20.0] {let r=prepared.render_region(RasterRegion{x,y:40.25,width:120.5,height:160.25}).unwrap();crop(&full,&r);}
   let mut target=surfaces::raster_n32_premul((full.width,full.height)).unwrap();target.canvas().clear(Color::WHITE);
   let before=(target.canvas().save_count(),target.canvas().local_to_device(),target.canvas().device_clip_bounds());
   renderer.render_to_surface_strict(&mut target,&tree,SkiaSurfaceRenderOptions{scale,..Default::default()}).unwrap();
   assert_eq!(before,(target.canvas().save_count(),target.canvas().local_to_device(),target.canvas().device_clip_bounds()));
   let info=ImageInfo::new((full.width,full.height),ColorType::RGBA8888,AlphaType::Premul,None);let mut output=vec![0;full.bytes.len()];assert!(target.read_pixels(&info,&mut output,full.stride,(0,0)));assert_eq!(output,full.bytes,"borrowed surface {key} scale {scale}");
   if scale==1.0 {for (mode,data) in [("full",full.bytes.as_slice()),("prepared",first.pixels.bytes.as_slice()),("host",output.as_slice())] {let dir=root.join("consumer").join(mode).join(key);std::fs::create_dir_all(&dir).unwrap();image::save_buffer(dir.join("rhwp_001.png"),data,full.width as u32,full.height as u32,image::ColorType::Rgba8).unwrap();}}
   scales.push(json!({"scale":scale,"dimensions":[full.width,full.height],"preparedAndHostEqualFull":true,"warmWindows":4,"retainedBytes":prepared.retained_bytes()}));
  }
  let mut session=NativeDocumentSession::open_bytes(&bytes,None).unwrap();session.set_font_sources(&paths,None).unwrap();let full=session.render_page(0,1.0).unwrap();let request=RasterRegion{x:20.25,y:30.5,width:192.0,height:160.0};
  let mut timings=Vec::new();
  for round in 0..10 {for enabled in if round%2==0 {[false,true]}else{[true,false]} {
   session.set_region_cache_limit(if enabled {8*1024*1024}else{0}).unwrap();session.clear_caches();let start=session.region_cache_stats();session.render_region(0,request,1.0).unwrap();let clock=Instant::now();
   for i in 0..20 {let r=RasterRegion{x:20.25+(i%5)as f64,..request};let output=session.render_region(0,r,1.0).unwrap();if round==0 {crop(&full.pixels,&RasterRegionPixels{pixels:output.pixels,origin_x:output.origin_x,origin_y:output.origin_y});}}
   let micros=clock.elapsed().as_secs_f64()*1_000_000.0;let end=session.region_cache_stats();let replays=end.replays-start.replays;assert_eq!(replays,if enabled {1}else{21});assert_eq!(end.hits-start.hits,if enabled {20}else{0});if round>0 {timings.push(json!({"round":round,"cache":enabled,"requests":20,"microseconds":micros,"replays":replays,"retainedBytes":end.retained_bytes}));}
  }}
  records.push(json!({"key":key,"scales":scales,"timings":timings}));std::fs::write(root.join("consumer-results.json"),serde_json::to_vec_pretty(&records).unwrap()).unwrap();println!("PASS real document {key}: preflight/prepared/host at 3 scales, moving-window cold/warm parity and replay counters");
 }
}
