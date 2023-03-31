use yew::BaseComponent;

pub trait RenderContext<Res> {
    fn render<COMP>(self, component: COMP) -> Res
    where
        COMP: BaseComponent;

    fn render_with_props<COMP>(self, component: COMP, props: COMP::Properties) -> Res
    where
        COMP: BaseComponent;
}
