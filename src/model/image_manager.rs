use log::{error, debug};
use image::{ImageBuffer, DynamicImage};
use iced::widget::image::Handle;

use crate::archive_reader::{ArchiveError, ArchiveResult};

pub struct ImageManager;

impl ImageManager {
    pub fn new() -> Self {
        Self
    }

    /// 画像データからIcedのハンドルを作成
    pub fn create_image_handle(
        data: &[u8], 
        rotate: bool
    ) -> ArchiveResult<Handle> {
        if rotate {
            let pimg = image::load_from_memory(data)?;
            let rotated = pimg.rotate180();
            let rgba_image = rotated.to_rgba8();
            Ok(Handle::from_rgba(
                rgba_image.width(),
                rgba_image.height(),
                rgba_image.into_raw(),
            ))
        } else {
            // データをコピーして所有権を移転
            Ok(Handle::from_bytes(data.to_vec()))
        }
    }

    /// エラー用の赤い画像を作成
    pub fn create_error_image() -> Handle {
        let pimg = ImageBuffer::from_pixel(64, 64, image::Rgba([255, 0, 0, 255]));
        Handle::from_rgba(
            pimg.width(),
            pimg.height(),
            pimg.into_vec(),
        )
    }

    /// 画像データの妥当性をチェック
    pub fn validate_image_data(data: &[u8]) -> bool {
        if data.is_empty() {
            return false;
        }

        // 基本的な画像フォーマットのシグネチャをチェック
        Self::is_supported_image_format(data)
    }

    /// サポートされている画像フォーマットかチェック
    fn is_supported_image_format(data: &[u8]) -> bool {
        if data.len() < 4 {
            return false;
        }

        // JPEG
        if data.len() >= 2 && data[0] == 0xFF && data[1] == 0xD8 {
            return true;
        }

        // PNG
        if data.len() >= 8 && &data[0..8] == [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A] {
            return true;
        }

        // GIF
        if data.len() >= 6 && (&data[0..6] == b"GIF87a" || &data[0..6] == b"GIF89a") {
            return true;
        }

        // BMP
        if data.len() >= 2 && &data[0..2] == b"BM" {
            return true;
        }

        // WebP
        if data.len() >= 12 && &data[0..4] == b"RIFF" && &data[8..12] == b"WEBP" {
            return true;
        }

        // TIFF (Little Endian)
        if data.len() >= 4 && &data[0..4] == [0x49, 0x49, 0x2A, 0x00] {
            return true;
        }

        // TIFF (Big Endian)
        if data.len() >= 4 && &data[0..4] == [0x4D, 0x4D, 0x00, 0x2A] {
            return true;
        }

        false
    }

    /// 画像の回転処理
    pub fn rotate_image(image: DynamicImage, degrees: f32) -> DynamicImage {
        match degrees as i32 {
            90 => image.rotate90(),
            180 => image.rotate180(),
            270 => image.rotate270(),
            _ => image, // その他の角度はサポートしない
        }
    }

    /// 画像のリサイズ処理
    pub fn resize_image(
        image: DynamicImage, 
        width: u32, 
        height: u32
    ) -> DynamicImage {
        image.resize(width, height, image::imageops::FilterType::Lanczos3)
    }

    /// 画像の品質情報を取得
    pub fn get_image_info(data: &[u8]) -> Option<ImageInfo> {
        match image::load_from_memory(data) {
            Ok(img) => Some(ImageInfo {
                width: img.width(),
                height: img.height(),
                format: Self::detect_format_from_data(data),
            }),
            Err(_) => None,
        }
    }

    /// データから画像フォーマットを検出
    fn detect_format_from_data(data: &[u8]) -> ImageFormat {
        if data.len() < 4 {
            return ImageFormat::Unknown;
        }

        // JPEG
        if data.len() >= 2 && data[0] == 0xFF && data[1] == 0xD8 {
            return ImageFormat::Jpeg;
        }

        // PNG
        if data.len() >= 8 && &data[0..8] == [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A] {
            return ImageFormat::Png;
        }

        // GIF
        if data.len() >= 6 && (&data[0..6] == b"GIF87a" || &data[0..6] == b"GIF89a") {
            return ImageFormat::Gif;
        }

        // BMP
        if data.len() >= 2 && &data[0..2] == b"BM" {
            return ImageFormat::Bmp;
        }

        // WebP
        if data.len() >= 12 && &data[0..4] == b"RIFF" && &data[8..12] == b"WEBP" {
            return ImageFormat::WebP;
        }

        ImageFormat::Unknown
    }
}

impl Default for ImageManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct ImageInfo {
    pub width: u32,
    pub height: u32,
    pub format: ImageFormat,
}

#[derive(Debug, Clone)]
pub enum ImageFormat {
    Jpeg,
    Png,
    Gif,
    Bmp,
    WebP,
    Tiff,
    Unknown,
}

impl std::fmt::Display for ImageFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImageFormat::Jpeg => write!(f, "JPEG"),
            ImageFormat::Png => write!(f, "PNG"),
            ImageFormat::Gif => write!(f, "GIF"),
            ImageFormat::Bmp => write!(f, "BMP"),
            ImageFormat::WebP => write!(f, "WebP"),
            ImageFormat::Tiff => write!(f, "TIFF"),
            ImageFormat::Unknown => write!(f, "Unknown"),
        }
    }
}
