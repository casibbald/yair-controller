use crate::controllers::lib::kubecontroller;
use kube::CustomResourceExt;

#[allow(dead_code)]
fn main() {
    print!(
        "{}",
        serde_yaml::to_string(&kubecontroller::Document::crd()).unwrap()
    );
}
