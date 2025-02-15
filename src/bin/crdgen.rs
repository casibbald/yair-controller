use kube::CustomResourceExt;
use yair::controllers::kubecontroller;

#[allow(dead_code)]
fn main() {
    print!(
        "{}",
        serde_yaml::to_string(&kubecontroller::Document::crd()).unwrap()
    );
}
