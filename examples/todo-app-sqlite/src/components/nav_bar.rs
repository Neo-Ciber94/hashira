use yew::function_component;

#[function_component]
pub fn NavBar() -> yew::Html {
    yew::html! {
       <>
        <header class="bg-blue-500 text-white py-4 fixed w-full h-[60px]">
            <div class="container mx-auto flex justify-between items-center px-4">
                <a href="/" class="flex items-center">
                <span class="font-bold text-lg">{"Todo App"}</span>
                </a>
            </div>
        </header>
        <div class="pb-[60px]"></div>
       </>
    }
}
