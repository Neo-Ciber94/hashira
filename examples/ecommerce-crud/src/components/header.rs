use yew::function_component;

#[function_component]
pub fn NavBar() -> yew::Html {
    yew::html! {
        <header class="flex items-center justify-between bg-slate-900 py-4 px-6">
        <div class="flex items-center">
            <h1 class="text-white text-xl font-bold mr-4">{"e-Commerce"}</h1>
            <nav class="text-white">
            <ul class="flex">
                <li class="mr-4"><a href="/products">{"Products"}</a></li>
                <li class="mr-4"><a href="/orders">{"Orders"}</a></li>
                <li class="mr-4"><a href="/users">{"Users"}</a></li>
            </ul>
            </nav>
        </div>
        <button class="bg-white text-gray-800 py-2 px-4 rounded">{"Login"}</button>
        </header>
    }
}
