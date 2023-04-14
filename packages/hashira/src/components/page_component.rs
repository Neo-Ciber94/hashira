use yew::BaseComponent;

/// Represents a page of a web app.
pub trait PageComponent: BaseComponent {
    /// Returns an unique identifier for this component.
    fn id() -> &'static str {
        std::any::type_name::<Self>()
    }
}

#[allow(dead_code)]
mod x {
    use crate::{
        app::{page_head::PageHead, RequestContext},
        error::Error,
    };
    use serde::{de::DeserializeOwned, Serialize};
    use std::future::Future;
    use yew::{BaseComponent, Properties};

    trait IntoHead {
        fn into_head(self) -> PageHead;
    }

    /// Loads data for the current page.
    trait PageLoader {
        /// The data to load.
        type Data: Serialize + DeserializeOwned;

        /// A future that resolves to the data.
        type Fut: Future<Output = Result<Self::Data, Error>>;

        /// Loads data for the current page.
        fn load(ctx: RequestContext) -> Self::Fut;
    }

    /// Provides the current page `<head>`.
    trait ComponentHead {
        /// A type that can be converted to a `<head>`.
        type Head: IntoHead;

        /// Returns the current page `<head>` elements.
        fn head(ctx: RequestContext) -> Self::Head;
    }

    #[derive(Debug, Default, Clone, PartialEq, Eq, Properties)]
    struct ServerProps<P>
    where
        P: PartialEq,
    {
        data: P,
    }

    /// Represents a page of a web app.
    trait PageComponent: BaseComponent<Properties = ServerProps<Self::Data>> {
        type Data: Serialize + DeserializeOwned + PartialEq;
        type Loader: PageLoader<Data = Self::Data>;
        type Head: ComponentHead;

        /// Returns the path of this page component.
        fn path(&self) -> Option<&'static str> {
            None
        }

        /// Returns an unique identifier of this component.
        fn id() -> &'static str {
            std::any::type_name::<Self>()
        }
    }

    /*
       #[loader]
       async fn GetUsers(ctx: RequestContext) -> Result<User, Error> {
           let pool = ctx.app_data::<DbPool>().unwrap();
           let users : Vec<User> = sqlx::query_as!("SELECT * FROM users")?;
           Ok(users)
       }

       #[head]
       async fn UserPageHead(ctx: RequestContext) -> PageHead {
           PageHead::new()
               .title("Application | Users")
               .description("Show all the users of the app")
               .meta(
                   Metadata::new()
               )
               .links(
                   PageLinks::new()
               )
               .scripts(
                   PageScripts::new()
               )
       }

       #[page_component("/users")]
       #[page_loader(GetUsers)]
       #[page_head(UserPageHead)]
       fn UserPage(props: &ServerProps<Vec<User>>) -> yew::Html {
           let users = props.data.clone();

           yew::html! {
               // ...
           }
       }
    */
}
