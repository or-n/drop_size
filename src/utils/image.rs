use arrayref::{array_mut_ref, array_ref};
use num::point::{_2::*, _3::*};
use pixels;
use pixels::dimensions::Dimensions;

pub struct Image {
    pub pixels: Vec<f32>,
    pub dimensions: Dimensions,
}

impl Image {
    #[inline]
    pub fn index(&self, point: _2<i32>) -> Option<usize> {
        let [x, y] = point.0;
        if x >= 0
            && x < self.dimensions.width as i32
            && y >= 0
            && y < self.dimensions.height as i32
        {
            Some(((y * self.dimensions.width as i32 + x) * 4) as usize)
        } else {
            None
        }
    }

    #[inline]
    pub fn get_pixel(&self, point: _2<i32>) -> Option<_3<f32>> {
        self.index(point)
            .map(|index| _3(*array_ref![self.pixels, index, 3]))
    }

    #[inline]
    pub fn set_pixel(&mut self, point: _2<i32>, color: _3<f32>) {
        if let Some(index) = self.index(point) {
            *array_mut_ref![self.pixels, index, 3] = color.0;
        }
    }
}
