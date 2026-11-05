//! Canonical 16:9 viewport mount contract and aspect framing policy.
//!
//! Enforces the canonical 16:9 (1280x720 default) viewport surface dimensions
//! across Etalon, Aether, and PortalGpu rendering surfaces without silently
//! stretching or distorting camera frustums.

use crate::graphical::surface_host::device_lease::{
    DeviceLeaseError, SurfaceExtent16x9, CANONICAL_16X9_HEIGHT, CANONICAL_16X9_WIDTH,
    DEFAULT_MAX_TEXTURE_DIMENSION_2D,
};
use serde::{Deserialize, Serialize};

/// Policy governing non-16:9 host container extents.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AspectFramingPolicy {
    /// Strict 16:9 mode: refuses non-16:9 container extents with an explicit error.
    Exact16x9,
    /// Bounded letterboxing/pillarboxing: computes centered exact 16:9 active sub-rect.
    LetterboxTo16x9,
}

/// Host contract requesting a 16:9 viewport surface mount.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViewportMountContract {
    pub container_width: u32,
    pub container_height: u32,
    pub framing_policy: AspectFramingPolicy,
    pub max_dimension: u32,
    pub artifact_id: String,
}

impl Default for ViewportMountContract {
    fn default() -> Self {
        Self {
            container_width: CANONICAL_16X9_WIDTH,
            container_height: CANONICAL_16X9_HEIGHT,
            framing_policy: AspectFramingPolicy::Exact16x9,
            max_dimension: DEFAULT_MAX_TEXTURE_DIMENSION_2D,
            artifact_id: "default-viewport".into(),
        }
    }
}

/// Execution receipt returned by `mount_16x9_portal_viewport`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViewportRenderReceipt {
    pub extent: SurfaceExtent16x9,
    pub container_width: u32,
    pub container_height: u32,
    pub active_width: u32,
    pub active_height: u32,
    pub offset_x: u32,
    pub offset_y: u32,
    pub letterboxed: bool,
    pub aspect_warning: Option<String>,
    pub artifact_id: String,
}

/// Typed errors for viewport mount operations.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum ViewportMountError {
    ZeroDimension { width: u32, height: u32 },
    AspectMismatch { width: u32, height: u32 },
    ExceedsMaxDimension { width: u32, height: u32, max_dimension: u32 },
    InvalidArtifactId(String),
    DeviceLease(DeviceLeaseError),
}

impl std::fmt::Display for ViewportMountError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ZeroDimension { width, height } => {
                write!(f, "Viewport container zero-dimension rejection: {width}x{height}")
            }
            Self::AspectMismatch { width, height } => {
                write!(f, "Viewport container extent {width}x{height} violates 16:9 contract")
            }
            Self::ExceedsMaxDimension { width, height, max_dimension } => {
                write!(f, "Viewport extent {width}x{height} exceeds max texture dimension {max_dimension}")
            }
            Self::InvalidArtifactId(msg) => write!(f, "Invalid artifact id: {msg}"),
            Self::DeviceLease(err) => write!(f, "Device lease error: {err}"),
        }
    }
}
impl std::error::Error for ViewportMountError {}

/// Mount a canonical 16:9 interactive PortalGpu viewport within the host container.
///
/// When the container matches 16:9 (`w * 9 == h * 16`), mounts directly at full
/// extent with `letterboxed: false`. For non-16:9 extents, either refuses
/// (`Exact16x9`) or calculates an exact 16:9 sub-rectangle with `letterboxed: true`
/// and diagnostic warning. Never silently distorts aspect ratio.
pub fn mount_16x9_portal_viewport(
    contract: &ViewportMountContract,
) -> Result<ViewportRenderReceipt, ViewportMountError> {
    if contract.container_width == 0 || contract.container_height == 0 {
        return Err(ViewportMountError::ZeroDimension {
            width: contract.container_width,
            height: contract.container_height,
        });
    }
    if contract.container_width > contract.max_dimension || contract.container_height > contract.max_dimension {
        return Err(ViewportMountError::ExceedsMaxDimension {
            width: contract.container_width,
            height: contract.container_height,
            max_dimension: contract.max_dimension,
        });
    }
    if contract.artifact_id.is_empty()
        || contract.artifact_id.len() > 4096
        || contract.artifact_id.as_bytes().contains(&0)
    {
        return Err(ViewportMountError::InvalidArtifactId(
            "Viewport requires a bounded actual source component identifier".into(),
        ));
    }

    let is_exact = (contract.container_width as u64) * 9 == (contract.container_height as u64) * 16;

    if is_exact {
        let extent = SurfaceExtent16x9::new(
            contract.container_width,
            contract.container_height,
            contract.max_dimension,
        )
        .map_err(ViewportMountError::DeviceLease)?;

        Ok(ViewportRenderReceipt {
            extent,
            container_width: contract.container_width,
            container_height: contract.container_height,
            active_width: contract.container_width,
            active_height: contract.container_height,
            offset_x: 0,
            offset_y: 0,
            letterboxed: false,
            aspect_warning: None,
            artifact_id: contract.artifact_id.clone(),
        })
    } else {
        match contract.framing_policy {
            AspectFramingPolicy::Exact16x9 => Err(ViewportMountError::AspectMismatch {
                width: contract.container_width,
                height: contract.container_height,
            }),
            AspectFramingPolicy::LetterboxTo16x9 => {
                let k_w = contract.container_width / 16;
                let k_h = contract.container_height / 9;
                let k = k_w.min(k_h);
                if k == 0 {
                    return Err(ViewportMountError::ZeroDimension {
                        width: contract.container_width,
                        height: contract.container_height,
                    });
                }
                let active_width = k * 16;
                let active_height = k * 9;
                let offset_x = (contract.container_width - active_width) / 2;
                let offset_y = (contract.container_height - active_height) / 2;

                let extent = SurfaceExtent16x9::new(
                    active_width,
                    active_height,
                    contract.max_dimension,
                )
                .map_err(ViewportMountError::DeviceLease)?;

                let warning = format!(
                    "Non-16:9 container extent {}x{} letterboxed to 16:9 active sub-rect {}x{} at offset (+{}, +{})",
                    contract.container_width, contract.container_height, active_width, active_height, offset_x, offset_y
                );

                Ok(ViewportRenderReceipt {
                    extent,
                    container_width: contract.container_width,
                    container_height: contract.container_height,
                    active_width,
                    active_height,
                    offset_x,
                    offset_y,
                    letterboxed: true,
                    aspect_warning: Some(warning),
                    artifact_id: contract.artifact_id.clone(),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonical_1280x720_mount_positive() {
        let contract = ViewportMountContract::default();
        let receipt = mount_16x9_portal_viewport(&contract).expect("canonical mount succeeds");
        assert_eq!(receipt.container_width, 1280);
        assert_eq!(receipt.container_height, 720);
        assert_eq!(receipt.active_width, 1280);
        assert_eq!(receipt.active_height, 720);
        assert_eq!(receipt.offset_x, 0);
        assert_eq!(receipt.offset_y, 0);
        assert!(!receipt.letterboxed);
        assert!(receipt.aspect_warning.is_none());
        assert!(receipt.extent.is_exact_16x9());
    }

    #[test]
    fn test_1080p_1920x1080_mount_positive() {
        let contract = ViewportMountContract {
            container_width: 1920,
            container_height: 1080,
            framing_policy: AspectFramingPolicy::Exact16x9,
            max_dimension: 8192,
            artifact_id: "test-source-1080p".into(),
        };
        let receipt = mount_16x9_portal_viewport(&contract).expect("1080p mount succeeds");
        assert_eq!(receipt.active_width, 1920);
        assert_eq!(receipt.active_height, 1080);
        assert!(!receipt.letterboxed);
        assert!(receipt.extent.is_exact_16x9());
    }

    #[test]
    fn test_negative_1280x1280_exact_mode_refusal() {
        let contract = ViewportMountContract {
            container_width: 1280,
            container_height: 1280,
            framing_policy: AspectFramingPolicy::Exact16x9,
            max_dimension: 8192,
            artifact_id: "test-square".into(),
        };
        let err = mount_16x9_portal_viewport(&contract).unwrap_err();
        assert_eq!(
            err,
            ViewportMountError::AspectMismatch {
                width: 1280,
                height: 1280,
            }
        );
    }

    #[test]
    fn test_letterbox_1280x1280_square_container() {
        let contract = ViewportMountContract {
            container_width: 1280,
            container_height: 1280,
            framing_policy: AspectFramingPolicy::LetterboxTo16x9,
            max_dimension: 8192,
            artifact_id: "test-square-letterboxed".into(),
        };
        let receipt = mount_16x9_portal_viewport(&contract).expect("letterboxed mount succeeds");
        assert_eq!(receipt.container_width, 1280);
        assert_eq!(receipt.container_height, 1280);
        assert_eq!(receipt.active_width, 1280);
        assert_eq!(receipt.active_height, 720);
        assert_eq!(receipt.offset_x, 0);
        assert_eq!(receipt.offset_y, 280);
        assert!(receipt.letterboxed);
        assert!(receipt.aspect_warning.is_some());
        assert!(receipt.extent.is_exact_16x9());
    }

    #[test]
    fn test_letterbox_1024x768_4x3_container() {
        let contract = ViewportMountContract {
            container_width: 1024,
            container_height: 768,
            framing_policy: AspectFramingPolicy::LetterboxTo16x9,
            max_dimension: 8192,
            artifact_id: "test-4x3-letterboxed".into(),
        };
        let receipt = mount_16x9_portal_viewport(&contract).expect("4:3 letterbox succeeds");
        assert_eq!(receipt.active_width, 1024);
        assert_eq!(receipt.active_height, 576);
        assert_eq!(receipt.offset_x, 0);
        assert_eq!(receipt.offset_y, 96);
        assert!(receipt.letterboxed);
        assert!(receipt.extent.is_exact_16x9());
    }

    #[test]
    fn test_pillarbox_2560x1080_ultrawide_container() {
        let contract = ViewportMountContract {
            container_width: 2560,
            container_height: 1080,
            framing_policy: AspectFramingPolicy::LetterboxTo16x9,
            max_dimension: 8192,
            artifact_id: "test-ultrawide-pillarboxed".into(),
        };
        let receipt = mount_16x9_portal_viewport(&contract).expect("21:9 pillarbox succeeds");
        assert_eq!(receipt.active_width, 1920);
        assert_eq!(receipt.active_height, 1080);
        assert_eq!(receipt.offset_x, 320);
        assert_eq!(receipt.offset_y, 0);
        assert!(receipt.letterboxed);
        assert!(receipt.extent.is_exact_16x9());
    }

    #[test]
    fn test_negative_zero_dimension_rejection() {
        let bad_w = ViewportMountContract {
            container_width: 0,
            container_height: 720,
            ..Default::default()
        };
        assert_eq!(
            mount_16x9_portal_viewport(&bad_w),
            Err(ViewportMountError::ZeroDimension {
                width: 0,
                height: 720,
            })
        );

        let bad_h = ViewportMountContract {
            container_width: 1280,
            container_height: 0,
            ..Default::default()
        };
        assert_eq!(
            mount_16x9_portal_viewport(&bad_h),
            Err(ViewportMountError::ZeroDimension {
                width: 1280,
                height: 0,
            })
        );
    }

    #[test]
    fn test_negative_exceeds_max_dimension() {
        let contract = ViewportMountContract {
            container_width: 9000,
            container_height: 5062,
            max_dimension: 8192,
            ..Default::default()
        };
        assert_eq!(
            mount_16x9_portal_viewport(&contract),
            Err(ViewportMountError::ExceedsMaxDimension {
                width: 9000,
                height: 5062,
                max_dimension: 8192,
            })
        );
    }

    #[test]
    fn test_negative_invalid_artifact_id() {
        let contract = ViewportMountContract {
            artifact_id: String::new(),
            ..Default::default()
        };
        assert!(matches!(
            mount_16x9_portal_viewport(&contract),
            Err(ViewportMountError::InvalidArtifactId(_))
        ));
    }
}
