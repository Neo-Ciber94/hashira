use crate::components::{hashira, HelloPage, HomePage};
use yew::{html::ChildrenProps, BaseComponent};

pub fn start_client<C>()
where
    C: BaseComponent<Properties = ChildrenProps>,
{
    wasm_logger::init(wasm_logger::Config::default());
    let hashira = hashira::<C>();
    let client_router = hashira.client_router().clone();

    log::info!("Hydrating app...");
    hashira::client::mount_to::<C>(client_router);
}
