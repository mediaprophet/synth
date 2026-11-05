//! Bounded still-image preview of an already-authorized HMC media selection.
//! Never logs or serializes source media outside Poet's existing UI frame path.
use crate::graphical::checkpoints::SelectedMedia;
use qualia_core_db::render::{
    asset_package::HmcTextureResource,
    texture_decode::{decode_hmc_texture_rgba8_into, inspect_hmc_texture_requirements, TextureDecodeLimits},
};
use std::sync::atomic::{AtomicBool, Ordering};

const MAX_INPUT: usize = 16 * 1024 * 1024;
const MAX_SIDE: u32 = 1024;
const MAX_WORK: usize = 48 * 1024 * 1024;

/// This preview is padded for Poet's square native frame; original dimensions
/// and byte-exact resource remain unchanged. This is not colour qualification.
pub(super) struct PaddedImage { pub side: u32, pub width: u32, pub height: u32, pub rgba: Vec<u8> }

fn allocate(len: usize) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    out.try_reserve_exact(len).map_err(|_| "Image preview allocation refused")?;
    out.resize(len, 0);
    Ok(out)
}

pub(super) fn pad(width: u32, height: u32, rgba: &[u8], side: u32) -> Result<Vec<u8>, String> {
    let w=width as usize; let h=height as usize; let side=side as usize;
    if w == 0 || h == 0 || w > side || h > side || rgba.len() != w.checked_mul(h).and_then(|n|n.checked_mul(4)).ok_or("Image size overflow")? {
        return Err("Invalid image preview extent".into());
    }
    let mut out=allocate(side.checked_mul(side).and_then(|n|n.checked_mul(4)).ok_or("Image size overflow")?)?;
    let x=(side-w)/2; let y=(side-h)/2;
    for row in 0..h {
        let src=row*w*4; let dst=((y+row)*side+x)*4;
        out[dst..dst+w*4].copy_from_slice(&rgba[src..src+w*4]);
    }
    Ok(out)
}

pub(super) fn decode(selected: &SelectedMedia, digest: [u8;32], cancelled: &AtomicBool) -> Result<PaddedImage, String> {
    if cancelled.load(Ordering::Acquire) { return Err("Image preview cancelled".into()); }
    // Preview the formats actually admitted by the current Core decoder; a
    // compressed GPU/transcode asset is not silently displayed as decoded.
    if !matches!(selected.kind.as_str(), "image/png" | "image/jpeg" | "image/jpg") {
        return Err("Still preview supports admitted PNG and JPEG media".into());
    }
    let limits=TextureDecodeLimits { max_encoded_bytes:MAX_INPUT, max_width:MAX_SIDE, max_height:MAX_SIDE, max_decoded_bytes:(MAX_SIDE as usize)*(MAX_SIDE as usize)*4 };
    let resource=HmcTextureResource {digest,mime_type:&selected.kind,bytes:&selected.bytes};
    let (info,scratch_len)=inspect_hmc_texture_requirements(&resource,limits).map_err(|e|format!("Image admission refused: {e}"))?;
    let side=info.width.max(info.height).max(8);
    let square=(side as usize).checked_mul(side as usize).and_then(|n|n.checked_mul(4)).ok_or("Image size overflow")?;
    // Admit logical payload for input, decoder output/scratch, padded frame,
    // PNG buffer and final base64 UI string. This is not a measured heap peak
    // or a bound on codec-internal allocation. No source content is logged.
    let png=square.checked_add(side as usize).and_then(|n|n.checked_add(4096)).ok_or("Image output size overflow")?;
    let base64=png.checked_add(2).and_then(|n|n.checked_div(3)).and_then(|n|n.checked_mul(4)).and_then(|n|n.checked_add(64)).ok_or("Image URI size overflow")?;
    // Allow four URI-sized payloads for construction, reply, UI state and a
    // rendered-property copy; this is still not an observed framework peak.
    let work=selected.bytes.len().checked_add(info.rgba8_bytes).and_then(|n|n.checked_add(scratch_len)).and_then(|n|n.checked_add(square)).and_then(|n|n.checked_add(png)).and_then(|n|base64.checked_mul(4).and_then(|uri|n.checked_add(uri))).ok_or("Image working size overflow")?;
    if work>MAX_WORK { return Err("Image preview exceeds working-byte profile".into()); }
    let mut rgba=allocate(info.rgba8_bytes)?;
    let mut scratch=allocate(scratch_len)?;
    let actual=decode_hmc_texture_rgba8_into(&resource,limits,&mut rgba,&mut scratch).map_err(|e|format!("Image decode refused: {e}"))?;
    if actual!=info { return Err("Image metadata changed during decode".into()); }
    if cancelled.load(Ordering::Acquire) { return Err("Image preview cancelled; result withheld".into()); }
    let padded=pad(info.width,info.height,&rgba,side)?;
    Ok(PaddedImage {side,width:info.width,height:info.height,rgba:padded})
}

#[cfg(test)]
mod tests {
    use super::*;
    use qualia_core_db::render::surface_authoring::encode_surface_png_into;
    #[test]
    fn png_decode_and_rectangular_padding_preserve_pixels() {
        let rgba=vec![127;8*8*4]; let mut png=vec![0;rgba.len()+8+4096];
        let used=encode_surface_png_into(8,&rgba,&mut png).unwrap(); png.truncate(used);
        let image=decode(&SelectedMedia {kind:"image/png".into(),bytes:png},[1;32],&AtomicBool::new(false)).unwrap();
        assert_eq!((image.width,image.height,image.side),(8,8,8)); assert_eq!(image.rgba,rgba);
        let padded=pad(2,1,&[1,2,3,4,5,6,7,8],8).unwrap();
        let offset=((3*8)+3)*4; assert_eq!(&padded[offset..offset+8],&[1,2,3,4,5,6,7,8]);
        assert_eq!(&padded[0..4],&[0,0,0,0]);
    }
    #[test]
    fn invalid_or_cancelled_image_never_produces_frame() {
        let invalid=SelectedMedia {kind:"image/png".into(),bytes:b"not png".to_vec()};
        assert!(decode(&invalid,[0;32],&AtomicBool::new(false)).is_err());
        assert!(decode(&invalid,[0;32],&AtomicBool::new(true)).is_err());
        assert!(decode(&SelectedMedia {kind:"video/mp4".into(),bytes:vec![]},[0;32],&AtomicBool::new(false)).is_err());
        assert!(pad(2,2,&[0;3],8).is_err());
    }
    #[test]
    fn jpeg_fixture_is_decoded_through_existing_qualia_codec() {
        // Synthetic 8x8 solid-colour JPEG; lossy coding permits small error.
        let bytes=include_bytes!("timeline_image_preview_fixture.jpg").to_vec();
        let image=decode(&SelectedMedia {kind:"image/jpeg".into(),bytes},[1;32],&AtomicBool::new(false)).unwrap();
        assert_eq!((image.width,image.height,image.side),(8,8,8));
        for pixel in image.rgba.chunks_exact(4) {
            assert!((pixel[0] as i32-96).abs()<=8);
            assert!((pixel[1] as i32-128).abs()<=8);
            assert!((pixel[2] as i32-160).abs()<=8);
            assert_eq!(pixel[3],255);
        }
    }
}
