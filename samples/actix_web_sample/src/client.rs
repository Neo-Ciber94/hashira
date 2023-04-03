use crate::components::hashira;
use yew::{html::ChildrenProps, BaseComponent};

pub fn start_client<C>()
where
    C: BaseComponent<Properties = ChildrenProps>,
{
    wasm_logger::init(wasm_logger::Config::default());
    log::info!("Hydrating app...");

    hashira::client::mount_to::<C>(hashira::<C>());
}
