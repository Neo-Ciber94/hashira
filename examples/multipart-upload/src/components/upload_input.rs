use yew::function_component;


#[function_component]
pub fn UploadInput() ->yew::Html {
    yew::html! {
        <input type="file" />
    }
}