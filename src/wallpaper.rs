use std::{
	error::Error,
	ops::Deref,
	path::Path,
	time::Instant
};

use image::{ImageReader, RgbImage};
use fast_image_resize::{images::Image, FilterType, ResizeAlg, ResizeOptions};

#[derive(Debug)]
pub struct Wallpaper {
	image: RgbImage,
}

impl Wallpaper {
	pub fn load(path: &Path) -> Result<Self, Box<dyn Error>> {
		let start = Instant::now();
		
		let image = ImageReader::open(path)?
			.decode()?;
		
		let duration = start.elapsed();
		
		println!("Loaded wallpaper with size {}x{}: {} ({duration:?})", image.width(), image.height(), path.to_string_lossy());
		
		Ok(Self {
			image: image.into(),
		})
	}
	
	pub fn resize_into(&mut self, width: u32, height: u32, buffer: &mut [u8]) {
		let pixel_count = (width * height) as usize;
		debug_assert_eq!(buffer.len(), pixel_count * 4);
		
		let mut resized_image;
		
		let src_buffer = if width == self.image.width() && height == self.image.height() {
			self.image.deref()
		} else {
			let start = Instant::now();
			
			let src_image = Image::from_slice_u8(self.image.width(), self.image.height(), self.image.as_mut(), fast_image_resize::PixelType::U8x3).unwrap();
			resized_image = Image::new(width, height, fast_image_resize::PixelType::U8x3);
			
			fast_image_resize::Resizer::new().resize(
				&src_image,
				&mut resized_image,
				&ResizeOptions::new()
					.resize_alg(ResizeAlg::Convolution(FilterType::Lanczos3))
					.fit_into_destination(None)
			).unwrap();
			
			let duration = start.elapsed();
			println!("Resized wallpaper to {width}x{height} ({duration:?})");
			
			resized_image.buffer()
		};
		
		debug_assert_eq!(src_buffer.len(), pixel_count * 3);
		
		#[allow(clippy::identity_op)]
		for i in 0..pixel_count {
			let src_index = i * 3;
			let dst_index = i * 4;
			buffer[dst_index + 0] = src_buffer[src_index + 2]; // B
			buffer[dst_index + 1] = src_buffer[src_index + 1]; // G
			buffer[dst_index + 2] = src_buffer[src_index + 0]; // R
		}
	}
}
