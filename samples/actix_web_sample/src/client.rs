use crate::components::{HelloPage, HomePage};
use yew::{html::ChildrenProps, BaseComponent};

pub fn start_client<C>()
where
    C: BaseComponent<Properties = ChildrenProps>,
{
    let hashira = crate::hashira();
    let client_router = hashira.client_router().clone();
    wasm_logger::init(wasm_logger::Config::default());

    log::info!("Hydrating app...");
    hashira::client::mount_to::<C>(client_router);
}
