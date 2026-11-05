//! Explicit exact conversion for a timeline whose admitted frame tick is constant.
//! This never infers a media-source frame rate, drop-frame timecode or playback.
use timeline_editing::{FrameRate,TICKS_PER_SECOND};

pub(super) fn frame_ticks(num:u64,den:u64)->Result<u64,String> {
    let num=i64::try_from(num).map_err(|_|"Frame-rate numerator exceeds i64")?;
    let den=i64::try_from(den).map_err(|_|"Frame-rate denominator exceeds i64")?;
    if num<=0 || den<=0 {return Err("Frame rate must be positive".into());}
    let rate=FrameRate::new(num,den);
    let numerator=i128::from(TICKS_PER_SECOND).checked_mul(i128::from(rate.den)).ok_or("Frame duration overflow")?;
    if numerator % i128::from(rate.num)!=0 {return Err("This rate does not have an exact constant tick duration; choose an exact rate or enter ticks".into());}
    let exact=numerator/i128::from(rate.num);
    if exact<=0 || exact>i128::from(i64::MAX) {return Err("Frame duration outside timeline range".into());}
    let shared=rate.frame_duration().0;
    if i128::from(shared)!=exact {return Err("Shared frame duration does not exactly match the requested rate".into());}
    Ok(shared as u64)
}

pub(super) fn frames_to_ticks(frames:u64,frame_ticks:u64)->Result<u64,String> {
    if frame_ticks==0 || frame_ticks>i64::MAX as u64 {return Err("Invalid admitted frame duration".into());}
    frames.checked_mul(frame_ticks).filter(|n|*n<=i64::MAX as u64).ok_or("Frame count exceeds timeline range".into())
}
pub(super) fn ticks_to_frames_exact(ticks:u64,frame_ticks:u64)->Result<u64,String> {
    if frame_ticks==0 || frame_ticks>i64::MAX as u64 || ticks>i64::MAX as u64 {return Err("Invalid admitted frame duration or tick value".into());}
    if ticks%frame_ticks!=0 {return Err("Tick value is between frame boundaries; no rounding applied".into());}
    Ok(ticks/frame_ticks)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn ntsc_and_integer_rates_use_exact_shared_durations() {
        for (num,den) in [(24,1),(25,1),(30,1),(24000,1001),(30000,1001),(60000,1001)] {
            let frame=frame_ticks(num,den).unwrap();
            assert_eq!(frame as i64,FrameRate::new(num as i64,den as i64).frame_duration().0);
            assert_eq!(ticks_to_frames_exact(frames_to_ticks(120,frame).unwrap(),frame).unwrap(),120);
        }
    }
    #[test] fn refuses_inexact_or_out_of_range_conversion_without_rounding() {
        assert!(frame_ticks(0,1).is_err()); assert!(frame_ticks(1,0).is_err());
        assert!(frame_ticks(11,1).is_err());
        assert!(ticks_to_frames_exact(11,10).is_err());
        assert!(frames_to_ticks(u64::MAX,10).is_err());
    }
}
