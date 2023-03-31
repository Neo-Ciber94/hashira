use yew::BaseComponent;

/*
### Server adapter:

|req: Request| {
    let path = req.path();
    let service = req.data::<AppService>().unwrap();
    let page = service.router.recognize(path).unwrap();
    let ctx = service.create_context(req);
    let res = page.call(ctx).await;
    Ok(res)
}


### Page handle

|ctx: AppContext| {
    ctx.add_metadata(...);
    ctx.add_links(...);
    ctx.add_scripts(...);

    let req = ctx.request();
    let id = req.params.get::<u32>("id");
    let user = db.get_user_by_id(id).await.unwrap();
    let res = ctx.render_with_props<Component>(user).await;
    Ok(res)
}
*/