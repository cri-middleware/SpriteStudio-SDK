//! [`OwningContext`] and [`OwningEffectSlot`]: a context or effect slot that holds its resource's `Arc`.

use ::core::ops::{Deref, DerefMut};
use alloc::sync::Arc;

use crate::{AnimationResource, Context, EffectSlot, SsabResource, error::SSRuntimeError};

/// `resource`'s parse, detached from the borrow of `resource` so that the `Arc` can move
/// into the holder that keeps it alive. Fails with `EmptyData` if the pack did not parse.
///
/// # Safety
///
/// Some `Arc` to `resource` must stay alive for as long as the returned reference is in use.
unsafe fn detach(resource: &SsabResource) -> Result<&'static AnimationResource<'static>, SSRuntimeError> {
    let parsed = resource.resource_static_ptr();
    if parsed.is_null() {
        return Err(SSRuntimeError::EmptyData);
    }
    // SAFETY: the caller's contract.
    Ok(unsafe { &*parsed })
}

/// A [`Context`] that holds a share of the resource it is bound to.
///
/// Rebind and reset it through [`Self::bind`] and [`Self::reset`] only; they are what keep
/// the context from outliving its resource.
pub(crate) struct OwningContext {
    // Declared first, so it is dropped before the resource it borrows.
    context: Context<'static>,
    resource: Option<Arc<SsabResource>>,
}

impl OwningContext {
    pub(crate) fn new() -> Self {
        Self { context: Context::new(), resource: None }
    }

    /// Binds the context to `resource`. On `EmptyData` the context is left as it was.
    pub(crate) fn bind(&mut self, resource: Arc<SsabResource>) -> Result<(), SSRuntimeError> {
        // SAFETY: `self.resource` holds the `Arc` until the context is rebound or reset.
        let parsed = unsafe { detach(&resource) }?;
        // The old share is released after the context stops borrowing it.
        let _old = self.resource.replace(resource);
        self.context.bind_resource(parsed);
        Ok(())
    }

    pub(crate) fn reset(&mut self) {
        self.context.reset();
        self.resource = None;
    }
}

impl Deref for OwningContext {
    type Target = Context<'static>;

    fn deref(&self) -> &Self::Target {
        &self.context
    }
}

impl DerefMut for OwningContext {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.context
    }
}

/// An [`EffectSlot`] that holds a share of the resource it plays from.
pub(crate) struct OwningEffectSlot {
    // Declared first, so it is dropped before the resource it borrows.
    slot: EffectSlot<'static>,
    _resource: Arc<SsabResource>,
}

impl OwningEffectSlot {
    pub(crate) fn new(resource: Arc<SsabResource>, effect_name_hash: u32, seed: u32) -> Result<Self, SSRuntimeError> {
        // SAFETY: the slot is stored beside the `Arc` and dropped before it.
        let parsed = unsafe { detach(&resource) }?;
        let slot = EffectSlot::new(parsed, effect_name_hash, seed)?;
        Ok(Self { slot, _resource: resource })
    }
}

impl Deref for OwningEffectSlot {
    type Target = EffectSlot<'static>;

    fn deref(&self) -> &Self::Target {
        &self.slot
    }
}

impl DerefMut for OwningEffectSlot {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.slot
    }
}
