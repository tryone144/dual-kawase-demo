// This file is part auf the dual-kawase-demo package.
//
// (c) 2019 Bernd Busse
//
// For the full copyright and license information, please view the README.md file
// that was distributed with this source code.
//

use gl::types::GLuint;
use image::{DynamicImage, FilterType, GenericImageView};

pub struct ImgSurface {
    img: DynamicImage,
    img_scaled: DynamicImage,
    width: u32,
    height: u32,
    tex: GLuint,
}

impl ImgSurface {
    pub fn new_from_image(img: &DynamicImage, width: u32, height: u32) -> Self {
        // convert image file to internal format
        let img_internal = DynamicImage::ImageBgra8(img.to_bgra());

        // create new texture with scaled image
        let (scaled_width, scaled_height) =
            crate::utils::scale_keep_aspect(img.width(), img.height(), width, height);
        let img_scaled =
            img_internal.resize_exact(scaled_width, scaled_height, FilterType::CatmullRom);
        let tex = super::create_texture(scaled_width, scaled_height, Some(img_scaled.raw_pixels()));

        Self { img: img_internal,
               img_scaled,
               width: scaled_width,
               height: scaled_height,
               tex }
    }

    pub fn resize_image(&mut self, width: u32, height: u32) {
        let (scaled_width, scaled_height) =
            crate::utils::scale_keep_aspect(self.img.width(), self.img.height(), width, height);
        self.img_scaled = self.img
                              .resize_exact(scaled_width, scaled_height, FilterType::CatmullRom);

        self.width = scaled_width;
        self.height = scaled_height;
    }

    pub fn refresh_texture(&mut self) {
        super::resize_texture(self.tex,
                              self.width,
                              self.height,
                              Some(self.img_scaled.raw_pixels()));
    }

    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn texture(&self) -> GLuint {
        self.tex
    }
}
