//! Validate derived organ/mesh correspondence before publishing or compiling.
use super::botanical::{read_recipe, validate_projection, BotanicalRecipe};
use crate::graphical::geometry_host::GeometryHost;
use vibe::session::SourceSession;

pub(crate) fn validate(
    session: &SourceSession,
    host: &GeometryHost,
) -> Result<Option<BotanicalRecipe>, String> {
    let Some(binding) = session.value("poet_botanical") else {
        return Ok(None);
    };
    let limits = host.botanical_limits()?;
    let recipe = read_recipe(Some(binding), limits)?.ok_or("Botanical recipe missing")?;
    let parts = session
        .value("poet_parts")
        .ok_or("Botanical source has no parts")?;
    validate_projection(parts, Some(&recipe), &recipe, limits)?;
    Ok(Some(recipe))
}
