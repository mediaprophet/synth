//! Shared Etalon, Aether, and PortalGpu device and surface lifecycle contract.
//!
//! Provides capability intersection, deterministic state machine transitions,
//! canonical 16:9 (1280x720) viewport surface dimension validation, and safe
//! queue submission fencing across device loss and surface reconfiguration.

pub const CANONICAL_16X9_WIDTH: u32 = 1280;
pub const CANONICAL_16X9_HEIGHT: u32 = 720;
pub const DEFAULT_MAX_TEXTURE_DIMENSION_2D: u32 = 8192;

/// Typed errors for device and surface lease operations.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DeviceLeaseError {
    InvalidSurfaceDimensions { width: u32, height: u32, max_dimension: u32 },
    MissingRequiredCapability { capability: &'static str },
    DeviceLost { epoch: u64 },
    InvalidStateTransition { from: DeviceLeaseState, to: DeviceLeaseState },
    ResizingInProgress,
    Revoked,
    NotActive,
}

impl std::fmt::Display for DeviceLeaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidSurfaceDimensions { width, height, max_dimension } => {
                write!(f, "Invalid surface dimensions {width}x{height} (max {max_dimension})")
            }
            Self::MissingRequiredCapability { capability } => write!(f, "Missing capability: {capability}"),
            Self::DeviceLost { epoch } => write!(f, "GPU device lost during lease epoch {epoch}"),
            Self::InvalidStateTransition { from, to } => write!(f, "Invalid transition from {from:?} to {to:?}"),
            Self::ResizingInProgress => write!(f, "Surface resizing in progress; submission suspended"),
            Self::Revoked => write!(f, "Device lease permanently revoked"),
            Self::NotActive => write!(f, "Device lease is not in active state"),
        }
    }
}
impl std::error::Error for DeviceLeaseError {}

/// Surface dimensions with canonical 16:9 aspect ratio diagnostics.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct SurfaceExtent16x9 {
    pub width: u32,
    pub height: u32,
}

impl Default for SurfaceExtent16x9 {
    fn default() -> Self { Self::canonical() }
}

impl SurfaceExtent16x9 {
    pub const fn canonical() -> Self {
        Self { width: CANONICAL_16X9_WIDTH, height: CANONICAL_16X9_HEIGHT }
    }

    pub fn new(width: u32, height: u32, max_dimension: u32) -> Result<Self, DeviceLeaseError> {
        if width == 0 || height == 0 || width > max_dimension || height > max_dimension {
            return Err(DeviceLeaseError::InvalidSurfaceDimensions { width, height, max_dimension });
        }
        Ok(Self { width, height })
    }

    pub fn is_exact_16x9(&self) -> bool {
        (self.width as u64) * 9 == (self.height as u64) * 16
    }

    pub fn aspect_ratio(&self) -> f32 {
        if self.height == 0 { 0.0 } else { self.width as f32 / self.height as f32 }
    }

    pub fn aspect_ratio_deviation(&self) -> f32 {
        (self.aspect_ratio() - (16.0 / 9.0)).abs()
    }

    pub fn is_aspect_tolerated(&self, epsilon: f32) -> bool {
        self.aspect_ratio_deviation() <= epsilon
    }

    pub fn fit_width_16x9(width: u32, max_dimension: u32) -> Result<Self, DeviceLeaseError> {
        if width == 0 || width > max_dimension {
            return Err(DeviceLeaseError::InvalidSurfaceDimensions { width, height: 0, max_dimension });
        }
        let height = ((width as u64 * 9 + 8) / 16) as u32;
        Self::new(width, height, max_dimension)
    }
}

/// Capability report intersecting Etalon, Aether, and PortalGpu features.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct DeviceCapabilityReport {
    pub f16: bool,
    pub subgroups: bool,
    pub timestamp_query: bool,
    pub bgra8unorm_storage: bool,
    pub max_texture_dimension_2d: u32,
}

impl Default for DeviceCapabilityReport {
    fn default() -> Self { Self::baseline_compat() }
}

impl DeviceCapabilityReport {
    pub const fn baseline_compat() -> Self {
        Self { f16: false, subgroups: false, timestamp_query: false, bgra8unorm_storage: false, max_texture_dimension_2d: DEFAULT_MAX_TEXTURE_DIMENSION_2D }
    }

    pub const fn etalon_native_hardware() -> Self {
        Self { f16: true, subgroups: true, timestamp_query: true, bgra8unorm_storage: true, max_texture_dimension_2d: DEFAULT_MAX_TEXTURE_DIMENSION_2D }
    }

    pub const fn aether_window_surface() -> Self {
        Self { f16: false, subgroups: false, timestamp_query: false, bgra8unorm_storage: true, max_texture_dimension_2d: DEFAULT_MAX_TEXTURE_DIMENSION_2D }
    }

    pub const fn portal_gpu_viewport() -> Self {
        Self { f16: true, subgroups: true, timestamp_query: true, bgra8unorm_storage: false, max_texture_dimension_2d: DEFAULT_MAX_TEXTURE_DIMENSION_2D }
    }

    pub fn intersect(&self, other: &Self) -> Self {
        Self {
            f16: self.f16 && other.f16,
            subgroups: self.subgroups && other.subgroups,
            timestamp_query: self.timestamp_query && other.timestamp_query,
            bgra8unorm_storage: self.bgra8unorm_storage && other.bgra8unorm_storage,
            max_texture_dimension_2d: self.max_texture_dimension_2d.min(other.max_texture_dimension_2d),
        }
    }

    pub fn intersect_three(etalon: &Self, aether: &Self, portal_gpu: &Self) -> Self {
        etalon.intersect(aether).intersect(portal_gpu)
    }

    pub fn require_capabilities(&self, required: &Self) -> Result<(), DeviceLeaseError> {
        if required.f16 && !self.f16 { return Err(DeviceLeaseError::MissingRequiredCapability { capability: "f16" }); }
        if required.subgroups && !self.subgroups { return Err(DeviceLeaseError::MissingRequiredCapability { capability: "subgroups" }); }
        if required.timestamp_query && !self.timestamp_query { return Err(DeviceLeaseError::MissingRequiredCapability { capability: "timestamp_query" }); }
        if required.bgra8unorm_storage && !self.bgra8unorm_storage { return Err(DeviceLeaseError::MissingRequiredCapability { capability: "bgra8unorm_storage" }); }
        if self.max_texture_dimension_2d < required.max_texture_dimension_2d {
            return Err(DeviceLeaseError::InvalidSurfaceDimensions {
                width: required.max_texture_dimension_2d,
                height: required.max_texture_dimension_2d,
                max_dimension: self.max_texture_dimension_2d,
            });
        }
        Ok(())
    }
}

/// Lifecycle states of a shared device lease.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum DeviceLeaseState {
    Active,
    Resizing,
    DeviceLost,
    Recovered,
    Revoked,
}

impl DeviceLeaseState {
    pub fn allows_submission(&self) -> bool { matches!(self, Self::Active) }
    pub fn is_terminal(&self) -> bool { matches!(self, Self::Revoked) }
}

/// Leased device and surface contract shared across Etalon, Aether, and PortalGpu.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SharedDeviceLease {
    lease_id: u64,
    epoch: u64,
    state: DeviceLeaseState,
    extent: SurfaceExtent16x9,
    capabilities: DeviceCapabilityReport,
    submission_count: u64,
}

impl SharedDeviceLease {
    pub fn new(lease_id: u64, extent: SurfaceExtent16x9, capabilities: DeviceCapabilityReport) -> Result<Self, DeviceLeaseError> {
        SurfaceExtent16x9::new(extent.width, extent.height, capabilities.max_texture_dimension_2d)?;
        Ok(Self { lease_id, epoch: 1, state: DeviceLeaseState::Active, extent, capabilities, submission_count: 0 })
    }

    pub fn from_tripartite_capabilities(
        lease_id: u64, extent: SurfaceExtent16x9, etalon: &DeviceCapabilityReport, aether: &DeviceCapabilityReport, portal_gpu: &DeviceCapabilityReport,
    ) -> Result<Self, DeviceLeaseError> {
        let intersected = DeviceCapabilityReport::intersect_three(etalon, aether, portal_gpu);
        Self::new(lease_id, extent, intersected)
    }

    pub fn lease_id(&self) -> u64 { self.lease_id }
    pub fn epoch(&self) -> u64 { self.epoch }
    pub fn state(&self) -> DeviceLeaseState { self.state }
    pub fn extent(&self) -> SurfaceExtent16x9 { self.extent }
    pub fn capabilities(&self) -> &DeviceCapabilityReport { &self.capabilities }
    pub fn submission_count(&self) -> u64 { self.submission_count }

    pub fn begin_resize(&mut self) -> Result<(), DeviceLeaseError> {
        match self.state {
            DeviceLeaseState::Active => { self.state = DeviceLeaseState::Resizing; Ok(()) }
            DeviceLeaseState::DeviceLost => Err(DeviceLeaseError::DeviceLost { epoch: self.epoch }),
            DeviceLeaseState::Revoked => Err(DeviceLeaseError::Revoked),
            from => Err(DeviceLeaseError::InvalidStateTransition { from, to: DeviceLeaseState::Resizing }),
        }
    }

    pub fn complete_resize(&mut self, new_extent: SurfaceExtent16x9) -> Result<(), DeviceLeaseError> {
        if self.state != DeviceLeaseState::Resizing {
            return Err(DeviceLeaseError::InvalidStateTransition { from: self.state, to: DeviceLeaseState::Active });
        }
        SurfaceExtent16x9::new(new_extent.width, new_extent.height, self.capabilities.max_texture_dimension_2d)?;
        self.extent = new_extent;
        self.state = DeviceLeaseState::Active;
        Ok(())
    }

    pub fn resize(&mut self, new_extent: SurfaceExtent16x9) -> Result<(), DeviceLeaseError> {
        SurfaceExtent16x9::new(new_extent.width, new_extent.height, self.capabilities.max_texture_dimension_2d)?;
        self.begin_resize()?;
        self.complete_resize(new_extent)
    }

    pub fn mark_device_lost(&mut self) -> Result<u64, DeviceLeaseError> {
        if self.state == DeviceLeaseState::Revoked { return Err(DeviceLeaseError::Revoked); }
        self.state = DeviceLeaseState::DeviceLost;
        Ok(self.epoch)
    }

    pub fn recover(&mut self, new_capabilities: Option<DeviceCapabilityReport>) -> Result<u64, DeviceLeaseError> {
        if self.state != DeviceLeaseState::DeviceLost {
            return Err(DeviceLeaseError::InvalidStateTransition { from: self.state, to: DeviceLeaseState::Recovered });
        }
        if let Some(caps) = new_capabilities {
            SurfaceExtent16x9::new(self.extent.width, self.extent.height, caps.max_texture_dimension_2d)?;
            self.capabilities = caps;
        }
        self.epoch = self.epoch.checked_add(1).ok_or(DeviceLeaseError::Revoked)?;
        self.submission_count = 0;
        self.state = DeviceLeaseState::Recovered;
        Ok(self.epoch)
    }

    pub fn activate(&mut self) -> Result<(), DeviceLeaseError> {
        if self.state != DeviceLeaseState::Recovered {
            return Err(DeviceLeaseError::InvalidStateTransition { from: self.state, to: DeviceLeaseState::Active });
        }
        self.state = DeviceLeaseState::Active;
        Ok(())
    }

    pub fn revoke(&mut self) { self.state = DeviceLeaseState::Revoked; }

    pub fn acquire_submission(&mut self) -> Result<u64, DeviceLeaseError> {
        match self.state {
            DeviceLeaseState::Active => {
                self.submission_count = self.submission_count.checked_add(1).unwrap_or(u64::MAX);
                Ok(self.submission_count)
            }
            DeviceLeaseState::DeviceLost => Err(DeviceLeaseError::DeviceLost { epoch: self.epoch }),
            DeviceLeaseState::Resizing => Err(DeviceLeaseError::ResizingInProgress),
            DeviceLeaseState::Recovered => Err(DeviceLeaseError::NotActive),
            DeviceLeaseState::Revoked => Err(DeviceLeaseError::Revoked),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_1280x720_lease_and_capability_intersection() {
        let extent = SurfaceExtent16x9::canonical();
        assert_eq!((extent.width, extent.height), (1280, 720));
        assert!(extent.is_exact_16x9());
        assert!((extent.aspect_ratio() - (16.0 / 9.0)).abs() < f32::EPSILON);
        assert_eq!(extent.aspect_ratio_deviation(), 0.0);

        let etalon = DeviceCapabilityReport::etalon_native_hardware();
        let aether = DeviceCapabilityReport::aether_window_surface();
        let portal = DeviceCapabilityReport::portal_gpu_viewport();

        let intersected = DeviceCapabilityReport::intersect_three(&etalon, &aether, &portal);
        assert!(!intersected.f16 && !intersected.subgroups && !intersected.timestamp_query && !intersected.bgra8unorm_storage);
        assert_eq!(intersected.max_texture_dimension_2d, 8192);

        let etalon_portal = etalon.intersect(&portal);
        assert!(etalon_portal.f16 && etalon_portal.subgroups && etalon_portal.timestamp_query && !etalon_portal.bgra8unorm_storage);

        let mut lease = SharedDeviceLease::new(42, extent, etalon_portal).expect("valid lease creation");
        assert_eq!(lease.lease_id(), 42);
        assert_eq!(lease.epoch(), 1);
        assert_eq!(lease.state(), DeviceLeaseState::Active);
        assert_eq!(lease.extent(), extent);
        assert_eq!(lease.submission_count(), 0);

        let sub_id = lease.acquire_submission().expect("submission accepted");
        assert_eq!(sub_id, 1);
        assert_eq!(lease.submission_count(), 1);
    }

    #[test]
    fn test_negative_invalid_surface_dimensions() {
        let max_dim = 8192;
        assert_eq!(SurfaceExtent16x9::new(0, 0, max_dim), Err(DeviceLeaseError::InvalidSurfaceDimensions { width: 0, height: 0, max_dimension: max_dim }));
        assert_eq!(SurfaceExtent16x9::new(0, 720, max_dim), Err(DeviceLeaseError::InvalidSurfaceDimensions { width: 0, height: 720, max_dimension: max_dim }));
        assert_eq!(SurfaceExtent16x9::new(1280, 0, max_dim), Err(DeviceLeaseError::InvalidSurfaceDimensions { width: 1280, height: 0, max_dimension: max_dim }));
        assert_eq!(SurfaceExtent16x9::new(16384, 720, max_dim), Err(DeviceLeaseError::InvalidSurfaceDimensions { width: 16384, height: 720, max_dimension: max_dim }));
        assert_eq!(SurfaceExtent16x9::new(1280, 16384, max_dim), Err(DeviceLeaseError::InvalidSurfaceDimensions { width: 1280, height: 16384, max_dimension: max_dim }));

        let mut lease = SharedDeviceLease::new(1, SurfaceExtent16x9::canonical(), DeviceCapabilityReport::default()).unwrap();
        let bad_resize = lease.resize(SurfaceExtent16x9 { width: 0, height: 0 });
        assert_eq!(bad_resize, Err(DeviceLeaseError::InvalidSurfaceDimensions { width: 0, height: 0, max_dimension: max_dim }));
        assert_eq!(lease.state(), DeviceLeaseState::Active);

        let oversized_resize = lease.resize(SurfaceExtent16x9 { width: 9000, height: 5062 });
        assert_eq!(oversized_resize, Err(DeviceLeaseError::InvalidSurfaceDimensions { width: 9000, height: 5062, max_dimension: max_dim }));
        assert_eq!(lease.state(), DeviceLeaseState::Active);
    }

    #[test]
    fn test_negative_missing_required_capability() {
        let baseline = DeviceCapabilityReport::baseline_compat();

        let mut required_f16 = baseline;
        required_f16.f16 = true;
        assert_eq!(baseline.require_capabilities(&required_f16), Err(DeviceLeaseError::MissingRequiredCapability { capability: "f16" }));

        let mut required_subgroups = baseline;
        required_subgroups.subgroups = true;
        assert_eq!(baseline.require_capabilities(&required_subgroups), Err(DeviceLeaseError::MissingRequiredCapability { capability: "subgroups" }));

        let mut required_time = baseline;
        required_time.timestamp_query = true;
        assert_eq!(baseline.require_capabilities(&required_time), Err(DeviceLeaseError::MissingRequiredCapability { capability: "timestamp_query" }));

        let mut required_bgra = baseline;
        required_bgra.bgra8unorm_storage = true;
        assert_eq!(baseline.require_capabilities(&required_bgra), Err(DeviceLeaseError::MissingRequiredCapability { capability: "bgra8unorm_storage" }));
    }

    #[test]
    fn test_device_lost_and_recovery_epoch() {
        let mut lease = SharedDeviceLease::new(10, SurfaceExtent16x9::canonical(), DeviceCapabilityReport::default()).unwrap();
        assert_eq!(lease.acquire_submission(), Ok(1));
        assert_eq!(lease.acquire_submission(), Ok(2));

        let lost_epoch = lease.mark_device_lost().expect("marked lost");
        assert_eq!(lost_epoch, 1);
        assert_eq!(lease.state(), DeviceLeaseState::DeviceLost);

        let err = lease.acquire_submission().unwrap_err();
        assert_eq!(err, DeviceLeaseError::DeviceLost { epoch: 1 });

        let new_epoch = lease.recover(None).expect("recovered");
        assert_eq!(new_epoch, 2);
        assert_eq!(lease.epoch(), 2);
        assert_eq!(lease.state(), DeviceLeaseState::Recovered);
        assert_eq!(lease.submission_count(), 0);

        assert_eq!(lease.acquire_submission(), Err(DeviceLeaseError::NotActive));

        lease.activate().expect("activated");
        assert_eq!(lease.state(), DeviceLeaseState::Active);
        assert_eq!(lease.acquire_submission(), Ok(1));
    }

    #[test]
    fn test_resizing_lifecycle_and_aspect_diagnostics() {
        let mut lease = SharedDeviceLease::new(7, SurfaceExtent16x9::canonical(), DeviceCapabilityReport::default()).unwrap();
        lease.begin_resize().unwrap();
        assert_eq!(lease.state(), DeviceLeaseState::Resizing);
        assert_eq!(lease.acquire_submission(), Err(DeviceLeaseError::ResizingInProgress));

        let extent_1080p = SurfaceExtent16x9::new(1920, 1080, 8192).unwrap();
        assert!(extent_1080p.is_exact_16x9());
        lease.complete_resize(extent_1080p).unwrap();
        assert_eq!(lease.state(), DeviceLeaseState::Active);
        assert_eq!((lease.extent().width, lease.extent().height), (1920, 1080));
        assert_eq!(lease.acquire_submission(), Ok(1));

        let fitted = SurfaceExtent16x9::fit_width_16x9(2560, 8192).unwrap();
        assert_eq!((fitted.width, fitted.height), (2560, 1440));
        assert!(fitted.is_exact_16x9());

        lease.revoke();
        assert_eq!(lease.state(), DeviceLeaseState::Revoked);
        assert_eq!(lease.acquire_submission(), Err(DeviceLeaseError::Revoked));
        assert_eq!(lease.begin_resize(), Err(DeviceLeaseError::Revoked));
    }
}
