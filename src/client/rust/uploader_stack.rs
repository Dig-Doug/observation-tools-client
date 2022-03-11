use crate::base_artifact_uploader::BaseArtifactUploader;
use crate::generic_artifact_uploader::GenericArtifactUploader;
use itertools::Itertools;
use log::info;
use std::cell::RefCell;

thread_local! {
    static LOCAL_UPLOADER_STACK: RefCell<Option<UploaderStack >> = RefCell::new(None);
}

pub struct UploaderStack {
    group_stack: Vec<BaseArtifactUploader>,
}

impl UploaderStack {
    fn log_stack(&self) {
        info!(
            "Current stack: {:?}",
            self.group_stack.iter().map(|b| b.id()).join(",")
        )
    }
}

pub(crate) fn init_uploader_stack(base: &BaseArtifactUploader) {
    info!("Initializing context with: {:?}", base.id());
    LOCAL_UPLOADER_STACK.with(|f| {
        assert!(f.borrow().is_none(), "Context has already been initialized");

        *f.borrow_mut() = Some(UploaderStack {
            group_stack: vec![base.clone()],
        })
    });
}

pub(crate) fn push_uploader(base: &BaseArtifactUploader) {
    LOCAL_UPLOADER_STACK.with(|f| {
        let mut r = f.borrow_mut();
        assert!(r.is_some(), "Context has not been initialized");
        let context = r.as_mut().unwrap();
        info!("Pushing {:?}", base.id());
        context.group_stack.push(base.clone());
        context.log_stack();
    });
    info!("Finished pushing");
}

pub(crate) fn pop_uploader(base: &BaseArtifactUploader) {
    LOCAL_UPLOADER_STACK.with(|f| {
        let mut r = f.borrow_mut();
        assert!(r.is_some(), "Context has not been initialized");
        let context = r.as_mut().unwrap();
        info!("Popping {:?}", base.id(),);
        context.group_stack.pop();
        context.log_stack();
    });
    info!("Finished popping");
}

pub(crate) fn get_current_group() -> GenericArtifactUploader {
    LOCAL_UPLOADER_STACK.with(|f| {
        let r = f.borrow();
        assert!(r.is_some(), "Context has not been initialized");
        let context = r.as_ref();
        assert!(context.is_some(), "Context has not been initialized");
        GenericArtifactUploader {
            base: context.unwrap().group_stack.last().unwrap().clone(),
        }
    })
}

pub(crate) fn ffi_get_current_group() -> Box<GenericArtifactUploader> {
    Box::new(get_current_group())
}
