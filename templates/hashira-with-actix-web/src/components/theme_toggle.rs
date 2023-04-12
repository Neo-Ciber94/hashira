use yew::function_component;

#[function_component]
pub fn ThemeToggle() -> yew::Html {
    yew::html! {
        <form>
            <input type="submit" name="dark" value="true"/>
        </form>
    }
}
