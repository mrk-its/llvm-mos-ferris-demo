use std::{env, fs, path::Path};
fn main() {
    let decoder = png::Decoder::new(fs::File::open("assets/ferris.png").unwrap());
    let mut reader = decoder.read_info().unwrap();
    let mut image_data = vec![0; reader.output_buffer_size()];
    reader.next_frame(&mut image_data).unwrap();
    let info = reader.info();
    println!("width: {}, height: {}, palette: {:?}, color_type: {:?}, output_buffer_size: {:?}", info.width, info.height, info.palette, reader.output_color_type(), reader.output_buffer_size());
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("ferris.dat");
    fs::write(&dest_path, image_data).unwrap();
}
