use loco_rs::testing;
use serial_test::serial;
use yair::app::App;

#[tokio::test]
#[serial]
async fn can_get_health() {
    testing::request::<App, _, _>(|request, _ctx| async move {
        let res = request.get("/health/").await;
        assert_eq!(res.status_code(), 200);

        // you can assert content like this:
        // assert_eq!(res.text(), "content");
    })
    .await;
}
